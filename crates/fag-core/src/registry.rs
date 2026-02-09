#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserChoice {
    pub prog_id: Option<String>,
    pub hash: Option<String>,
    pub last_write_time: Option<FileTime>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FileTime {
    pub low_date_time: u32,
    pub high_date_time: u32,
}

impl FileTime {
    pub fn as_u64(self) -> u64 {
        (u64::from(self.high_date_time) << 32) | u64::from(self.low_date_time)
    }
}

pub fn read_user_choice(ext: &str) -> Result<Option<UserChoice>, ReadUserChoiceError> {
    let ext = normalize_ext(ext)?;
    let subkey = format!(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FileExts\\{}\\UserChoice",
        ext
    );

    unsafe { read_user_choice_from_subkey(&subkey) }
}

pub enum ReadUserChoiceError {
    InvalidExt,
    WindowsApiError { api: &'static str, code: u32 },
}

impl std::fmt::Debug for ReadUserChoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidExt => write!(f, "InvalidExt"),
            Self::WindowsApiError { api, code } => f
                .debug_struct("WindowsApiError")
                .field("api", api)
                .field("code", code)
                .finish(),
        }
    }
}

impl std::fmt::Display for ReadUserChoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidExt => write!(f, "invalid extension"),
            Self::WindowsApiError { api, code } => write!(f, "{} failed with {}", api, code),
        }
    }
}

impl std::error::Error for ReadUserChoiceError {}

fn normalize_ext(ext: &str) -> Result<String, ReadUserChoiceError> {
    let ext = ext.trim();
    if ext.is_empty() || ext == "." {
        return Err(ReadUserChoiceError::InvalidExt);
    }
    let ext = if let Some(stripped) = ext.strip_prefix('.') {
        stripped
    } else {
        ext
    };
    if ext.is_empty() || ext.contains(['\\', '/', '\0']) {
        return Err(ReadUserChoiceError::InvalidExt);
    }
    Ok(format!(".{}", ext))
}

#[cfg(windows)]
unsafe fn read_user_choice_from_subkey(
    subkey: &str,
) -> Result<Option<UserChoice>, ReadUserChoiceError> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    type HKEY = isize;

    const HKEY_CURRENT_USER: HKEY = 0x8000_0001_u32 as isize;
    const KEY_READ: u32 = 0x20019;
    const ERROR_FILE_NOT_FOUND: u32 = 2;
    const ERROR_SUCCESS: u32 = 0;
    const REG_SZ: u32 = 1;

    #[repr(C)]
    #[derive(Copy, Clone)]
    #[allow(non_snake_case)]
    struct FILETIME {
        dwLowDateTime: u32,
        dwHighDateTime: u32,
    }

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

        fn RegQueryInfoKeyW(
            hKey: HKEY,
            lpClass: *mut u16,
            lpcchClass: *mut u32,
            lpReserved: *mut u32,
            lpcSubKeys: *mut u32,
            lpcbMaxSubKeyLen: *mut u32,
            lpcbMaxClassLen: *mut u32,
            lpcValues: *mut u32,
            lpcbMaxValueNameLen: *mut u32,
            lpcbMaxValueLen: *mut u32,
            lpcbSecurityDescriptor: *mut u32,
            lpftLastWriteTime: *mut FILETIME,
        ) -> u32;
    }

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    unsafe fn read_reg_sz(hkey: HKEY, name: &str) -> Result<Option<String>, ReadUserChoiceError> {
        let name_w = to_wide(name);
        let mut value_type: u32 = 0;
        let mut data_len: u32 = 0;
        let rc = RegQueryValueExW(
            hkey,
            name_w.as_ptr(),
            std::ptr::null_mut(),
            &mut value_type,
            std::ptr::null_mut(),
            &mut data_len,
        );
        if rc == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if rc != ERROR_SUCCESS {
            return Err(ReadUserChoiceError::WindowsApiError {
                api: "RegQueryValueExW(size)",
                code: rc,
            });
        }
        if value_type != REG_SZ || data_len == 0 {
            return Ok(None);
        }

        let mut buf = vec![0u16; (data_len as usize + 1) / 2];
        let rc = RegQueryValueExW(
            hkey,
            name_w.as_ptr(),
            std::ptr::null_mut(),
            &mut value_type,
            buf.as_mut_ptr() as *mut u8,
            &mut data_len,
        );
        if rc != ERROR_SUCCESS {
            return Err(ReadUserChoiceError::WindowsApiError {
                api: "RegQueryValueExW(data)",
                code: rc,
            });
        }

        if let Some(pos) = buf.iter().position(|c| *c == 0) {
            buf.truncate(pos);
        }
        Ok(Some(String::from_utf16_lossy(&buf)))
    }

    let subkey_w = to_wide(subkey);
    let mut hkey: HKEY = 0;
    let rc = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        subkey_w.as_ptr(),
        0,
        KEY_READ,
        &mut hkey,
    );
    if rc == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    if rc != ERROR_SUCCESS {
        return Err(ReadUserChoiceError::WindowsApiError {
            api: "RegOpenKeyExW",
            code: rc,
        });
    }

    let mut last_write_time = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let rc = RegQueryInfoKeyW(
        hkey,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut last_write_time,
    );
    let ft = if rc == ERROR_SUCCESS {
        Some(FileTime {
            low_date_time: last_write_time.dwLowDateTime,
            high_date_time: last_write_time.dwHighDateTime,
        })
    } else {
        None
    };

    let prog_id = read_reg_sz(hkey, "ProgId")?;
    let hash = read_reg_sz(hkey, "Hash")?;
    let _ = RegCloseKey(hkey);

    Ok(Some(UserChoice {
        prog_id,
        hash,
        last_write_time: ft,
    }))
}

#[cfg(not(windows))]
unsafe fn read_user_choice_from_subkey(
    _subkey: &str,
) -> Result<Option<UserChoice>, ReadUserChoiceError> {
    Err(ReadUserChoiceError::WindowsApiError {
        api: "windows-only",
        code: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_ext_accepts_dot_prefixed() {
        assert_eq!(normalize_ext(".mp4").unwrap(), ".mp4");
    }

    #[test]
    fn normalize_ext_accepts_without_dot() {
        assert_eq!(normalize_ext("mp4").unwrap(), ".mp4");
    }

    #[test]
    fn normalize_ext_trims_whitespace() {
        assert_eq!(normalize_ext("  .mkv  ").unwrap(), ".mkv");
    }

    #[test]
    fn normalize_ext_rejects_empty() {
        assert!(matches!(normalize_ext(""), Err(ReadUserChoiceError::InvalidExt)));
        assert!(matches!(
            normalize_ext("   "),
            Err(ReadUserChoiceError::InvalidExt)
        ));
        assert!(matches!(normalize_ext("."), Err(ReadUserChoiceError::InvalidExt)));
    }

    #[test]
    fn normalize_ext_rejects_path_separators() {
        assert!(matches!(
            normalize_ext(".mp4\\x"),
            Err(ReadUserChoiceError::InvalidExt)
        ));
        assert!(matches!(
            normalize_ext("mp4/x"),
            Err(ReadUserChoiceError::InvalidExt)
        ));
    }

    #[test]
    fn read_user_choice_rejects_invalid_ext() {
        assert!(matches!(
            read_user_choice(""),
            Err(ReadUserChoiceError::InvalidExt)
        ));
    }
}
