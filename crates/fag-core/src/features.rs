#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FeatureConfigurationType {
    Boot,
    Runtime,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FeatureEnabledState {
    Default,
    Disabled,
    Enabled,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FeatureConfiguration {
    pub feature_id: u32,
    pub priority: u8,
    pub enabled_state: FeatureEnabledState,
    pub variant: u8,
    pub variant_payload_kind: u8,
    pub variant_payload: u32,
}

#[derive(Debug)]
pub enum FeatureError {
    WindowsOnly,
    WindowsApiError { api: &'static str, status: i32 },
    NotFound { feature_id: u32 },
}

impl std::fmt::Display for FeatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WindowsOnly => write!(f, "windows-only"),
            Self::WindowsApiError { api, status } => write!(f, "{} failed with NTSTATUS {}", api, status),
            Self::NotFound { feature_id } => write!(f, "feature id {} not found", feature_id),
        }
    }
}

impl std::error::Error for FeatureError {}

pub fn query_feature_configuration(
    feature_id: u32,
    config_type: FeatureConfigurationType,
) -> Result<FeatureConfiguration, FeatureError> {
    let all = query_all_feature_configurations(config_type)?;
    all.into_iter()
        .find(|c| c.feature_id == feature_id)
        .ok_or(FeatureError::NotFound { feature_id })
}

pub fn query_all_feature_configurations(
    config_type: FeatureConfigurationType,
) -> Result<Vec<FeatureConfiguration>, FeatureError> {
    #[cfg(not(windows))]
    {
        let _ = config_type;
        return Err(FeatureError::WindowsOnly);
    }

    #[cfg(windows)]
    unsafe {
        let mut count: usize = 0;
        let mut change_stamp: u64 = 0;
        let status = RtlQueryAllFeatureConfigurations(
            config_type_to_rtl(config_type),
            &mut change_stamp,
            std::ptr::null_mut(),
            &mut count,
        );
        // STATUS_BUFFER_TOO_SMALL (0xC0000023) and STATUS_BUFFER_OVERFLOW (0x80000005) are both
        // acceptable "tell me required count" style responses here.
        if status != 0 && status != STATUS_BUFFER_TOO_SMALL && status != STATUS_BUFFER_OVERFLOW {
            return Err(FeatureError::WindowsApiError {
                api: "RtlQueryAllFeatureConfigurations(size)",
                status,
            });
        }
        if count == 0 {
            return Ok(Vec::new());
        }

        // Add a little slack to avoid races while allocating.
        let mut buf = vec![RTL_FEATURE_CONFIGURATION::default(); count.saturating_add(64)];
        let mut count2 = buf.len();
        let status = RtlQueryAllFeatureConfigurations(
            config_type_to_rtl(config_type),
            &mut change_stamp,
            buf.as_mut_ptr(),
            &mut count2,
        );
        if status != 0 {
            return Err(FeatureError::WindowsApiError {
                api: "RtlQueryAllFeatureConfigurations(data)",
                status,
            });
        }

        buf.truncate(count2);
        Ok(buf.into_iter().map(decode_config).collect())
    }
}

pub fn set_feature_state(
    feature_id: u32,
    config_type: FeatureConfigurationType,
    enabled_state: FeatureEnabledState,
) -> Result<(), FeatureError> {
    #[cfg(not(windows))]
    {
        let _ = (feature_id, config_type, enabled_state);
        return Err(FeatureError::WindowsOnly);
    }

    #[cfg(windows)]
    unsafe {
        let mut update = RTL_FEATURE_CONFIGURATION_UPDATE::default();
        update.feature_id = feature_id;
        update.priority = FEATURE_CONFIGURATION_PRIORITY_USER;
        update.enabled_state = enabled_state_to_rtl(enabled_state);
        update.enabled_state_options = FEATURE_ENABLED_STATE_OPTIONS_NONE;
        update.variant_flags = 0;
        update.reserved = [0u8; 3];
        update.variant_payload_kind = FEATURE_VARIANT_PAYLOAD_KIND_NONE;
        update.variant_payload = 0;
        update.operation = FEATURE_CONFIGURATION_OPERATION_FEATURE_STATE;

        let status = RtlSetFeatureConfigurations(
            std::ptr::null(),
            config_type_to_rtl(config_type),
            &update,
            1,
        );
        if status != 0 {
            return Err(FeatureError::WindowsApiError {
                api: "RtlSetFeatureConfigurations",
                status,
            });
        }
        Ok(())
    }
}

#[cfg(windows)]
type NTSTATUS = i32;

#[cfg(windows)]
const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC000_0023u32 as i32;
#[cfg(windows)]
const STATUS_BUFFER_OVERFLOW: NTSTATUS = 0x8000_0005u32 as i32;

#[cfg(windows)]
const FEATURE_CONFIGURATION_PRIORITY_USER: u32 = 8;
#[cfg(windows)]
const FEATURE_ENABLED_STATE_OPTIONS_NONE: u32 = 0;
#[cfg(windows)]
const FEATURE_VARIANT_PAYLOAD_KIND_NONE: u8 = 0;
#[cfg(windows)]
const FEATURE_CONFIGURATION_OPERATION_FEATURE_STATE: u32 = 1;

#[cfg(windows)]
#[repr(C)]
#[derive(Copy, Clone, Default)]
struct RTL_FEATURE_CONFIGURATION {
    feature_id: u32,
    flags: u32,
    variant_payload: u32,
}

#[cfg(windows)]
#[repr(C)]
#[derive(Copy, Clone)]
struct RTL_FEATURE_CONFIGURATION_UPDATE {
    feature_id: u32,
    priority: u32,
    enabled_state: u32,
    enabled_state_options: u32,
    variant_flags: u32,
    reserved: [u8; 3],
    variant_payload_kind: u8,
    variant_payload: u32,
    operation: u32,
}

#[cfg(windows)]
impl Default for RTL_FEATURE_CONFIGURATION_UPDATE {
    fn default() -> Self {
        Self {
            feature_id: 0,
            priority: 0,
            enabled_state: 0,
            enabled_state_options: 0,
            variant_flags: 0,
            reserved: [0; 3],
            variant_payload_kind: 0,
            variant_payload: 0,
            operation: 0,
        }
    }
}

#[cfg(windows)]
fn decode_config(c: RTL_FEATURE_CONFIGURATION) -> FeatureConfiguration {
    let priority = (c.flags & 0x0F) as u8;
    let enabled_state_raw = ((c.flags >> 4) & 0x03) as u8;
    let variant = ((c.flags >> 8) & 0x3F) as u8;
    let variant_payload_kind = ((c.flags >> 14) & 0x03) as u8;

    let enabled_state = match enabled_state_raw {
        1 => FeatureEnabledState::Disabled,
        2 => FeatureEnabledState::Enabled,
        _ => FeatureEnabledState::Default,
    };

    FeatureConfiguration {
        feature_id: c.feature_id,
        priority,
        enabled_state,
        variant,
        variant_payload_kind,
        variant_payload: c.variant_payload,
    }
}

#[cfg(windows)]
fn enabled_state_to_rtl(state: FeatureEnabledState) -> u32 {
    match state {
        FeatureEnabledState::Default => 0,
        FeatureEnabledState::Disabled => 1,
        FeatureEnabledState::Enabled => 2,
    }
}

#[cfg(windows)]
fn config_type_to_rtl(ty: FeatureConfigurationType) -> u32 {
    match ty {
        FeatureConfigurationType::Boot => 0,
        FeatureConfigurationType::Runtime => 1,
    }
}

#[cfg(windows)]
#[link(name = "ntdll")]
extern "system" {
    fn RtlQueryAllFeatureConfigurations(
        configuration_type: u32,
        change_stamp: *mut u64,
        configurations: *mut RTL_FEATURE_CONFIGURATION,
        configuration_count: *mut usize,
    ) -> NTSTATUS;

    fn RtlSetFeatureConfigurations(
        previous_change_stamp: *const u64,
        configuration_type: u32,
        updates: *const RTL_FEATURE_CONFIGURATION_UPDATE,
        update_count: usize,
    ) -> NTSTATUS;
}

