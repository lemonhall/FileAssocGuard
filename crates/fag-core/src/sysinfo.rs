#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sysinfo {
    pub sid: Option<String>,
    pub hash_version: Option<u32>,
    pub user_choice_latest_enabled: bool,
    pub ucpd_enabled: Option<bool>,
    pub ucpd_driver_present: Option<bool>,
    pub guidance: Vec<String>,
}

#[derive(Debug)]
pub enum SysinfoError {
    WindowsOnly,
    WindowsApiError { api: &'static str, code: u32 },
}

impl std::fmt::Display for SysinfoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WindowsOnly => write!(f, "windows-only"),
            Self::WindowsApiError { api, code } => write!(f, "{} failed with {}", api, code),
        }
    }
}

impl std::error::Error for SysinfoError {}

pub fn read_sysinfo() -> Result<Sysinfo, SysinfoError> {
    #[cfg(not(windows))]
    {
        return Err(SysinfoError::WindowsOnly);
    }

    #[cfg(windows)]
    unsafe {
        let sid = match crate::registry::current_user_sid() {
            Ok(s) => Some(s),
            Err(e) => {
                return Err(SysinfoError::WindowsApiError {
                    api: e.api,
                    code: e.code,
                })
            }
        };

        let hash_version = match sid.as_deref() {
            Some(s) => crate::registry::read_hash_version(s).map_err(|e| {
                SysinfoError::WindowsApiError {
                    api: e.api,
                    code: e.code,
                }
            })?,
            None => None,
        };
        let user_choice_latest_enabled = hash_version.unwrap_or(0) != 0;

        let (ucpd_enabled, ucpd_driver_present) = detect_ucpd();

        let mut guidance = Vec::new();
        if user_choice_latest_enabled {
            guidance.push(
                "HashVersion!=0 detected (UserChoiceLatest enabled): use capture-latest/apply-latest replay for restore."
                    .to_string(),
            );
            guidance.push(
                "Optional workaround (not required if replay works): ViveTool disable + reboot."
                    .to_string(),
            );
            guidance.push("  vivetool /disable /id:43229420".to_string());
            guidance.push("  vivetool /disable /id:27623730".to_string());
        }

        Ok(Sysinfo {
            sid,
            hash_version,
            user_choice_latest_enabled,
            ucpd_enabled,
            ucpd_driver_present,
            guidance,
        })
    }
}

#[cfg(windows)]
fn detect_ucpd() -> (Option<bool>, Option<bool>) {
    let driver_present = {
        let system_root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
        let path = std::path::PathBuf::from(system_root)
            .join("System32")
            .join("drivers")
            .join("UCPD.sys");
        Some(path.exists())
    };

    let enabled = unsafe { read_ucpd_service_start() };
    (enabled, driver_present)
}

#[cfg(windows)]
unsafe fn read_ucpd_service_start() -> Option<bool> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    type HKEY = isize;

    const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002_u32 as isize;
    const KEY_READ: u32 = 0x20019;
    const ERROR_SUCCESS: u32 = 0;
    const ERROR_FILE_NOT_FOUND: u32 = 2;
    const REG_DWORD: u32 = 4;

    #[link(name = "Advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            hKey: HKEY,
            lpSubKey: *const u16,
            ulOptions: u32,
            samDesired: u32,
            phkResult: *mut HKEY,
        ) -> u32;
        fn RegCloseKey(hKey: HKEY) -> u32;
        fn RegQueryValueExW(
            hKey: HKEY,
            lpValueName: *const u16,
            lpReserved: *mut u32,
            lpType: *mut u32,
            lpData: *mut u8,
            lpcbData: *mut u32,
        ) -> u32;
    }

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let subkey_w = to_wide("SYSTEM\\CurrentControlSet\\Services\\UCPD");
    let mut hkey: HKEY = 0;
    let rc = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        subkey_w.as_ptr(),
        0,
        KEY_READ,
        &mut hkey,
    );
    if rc == ERROR_FILE_NOT_FOUND {
        return None;
    }
    if rc != ERROR_SUCCESS {
        return None;
    }

    let value_w = to_wide("Start");
    let mut value_type: u32 = 0;
    let mut data: u32 = 0;
    let mut data_len: u32 = std::mem::size_of::<u32>() as u32;
    let rc = RegQueryValueExW(
        hkey,
        value_w.as_ptr(),
        std::ptr::null_mut(),
        &mut value_type,
        (&mut data as *mut u32) as *mut u8,
        &mut data_len,
    );
    let _ = RegCloseKey(hkey);
    if rc != ERROR_SUCCESS {
        return None;
    }
    if value_type != REG_DWORD {
        return None;
    }

    // Start=4 means disabled for services/drivers.
    Some(data != 4)
}
