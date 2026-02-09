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

#[derive(Debug)]
pub struct SetUserChoiceResult {
    pub ext: String,
    pub prog_id: String,
    pub regdate_hex: String,
    pub hash: String,
    pub attempts: u32,
}

#[derive(Debug)]
pub enum SetUserChoiceError {
    InvalidExt,
    UserChoiceLatestEnabled { hash_version: u32 },
    ProgIdEmpty,
    WindowsApiError { api: &'static str, code: u32 },
}

impl std::fmt::Display for SetUserChoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidExt => write!(f, "invalid extension"),
            Self::UserChoiceLatestEnabled { hash_version } => write!(
                f,
                "UserChoiceLatest is enabled (HashVersion={}); legacy hash restore is not supported. Suggested workaround: install ViveTool, run `vivetool /disable /id:43229420` and `vivetool /disable /id:27623730`, then reboot.",
                hash_version
            ),
            Self::ProgIdEmpty => write!(f, "prog_id is empty"),
            Self::WindowsApiError { api, code } => write!(f, "{} failed with {}", api, code),
        }
    }
}

impl std::error::Error for SetUserChoiceError {}

pub fn list_open_with_progids(ext: &str) -> Result<Vec<String>, ReadUserChoiceError> {
    let ext = normalize_ext(ext)?;
    let mut out = Vec::new();

    #[cfg(windows)]
    unsafe {
        out.extend(read_open_with_progids_hkcu_fileexts(&ext)?);
        if out.is_empty() {
            out.extend(read_open_with_progids_hkcr(&ext)?);
        }
    }

    out.sort();
    out.dedup();
    Ok(out)
}

pub fn set_user_choice(
    ext: &str,
    prog_id: &str,
) -> Result<SetUserChoiceResult, SetUserChoiceError> {
    let ext = normalize_ext(ext).map_err(|_| SetUserChoiceError::InvalidExt)?;
    let prog_id = prog_id.trim();
    if prog_id.is_empty() {
        return Err(SetUserChoiceError::ProgIdEmpty);
    }

    #[cfg(not(windows))]
    {
        let _ = ext;
        let _ = prog_id;
        return Err(SetUserChoiceError::WindowsApiError {
            api: "windows-only",
            code: 0,
        });
    }

    #[cfg(windows)]
    unsafe {
        let sid = current_user_sid().map_err(|e| SetUserChoiceError::WindowsApiError {
            api: e.api,
            code: e.code,
        })?;

        if let Ok(Some(hash_version)) = read_hash_version(&sid) {
            if hash_version != 0 {
                return Err(SetUserChoiceError::UserChoiceLatestEnabled { hash_version });
            }
        }

        let user_choice_subkey = format!(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FileExts\\{}\\UserChoice",
            ext
        );

        const MAX_ATTEMPTS: u32 = 5;
        for attempt in 1..=MAX_ATTEMPTS {
            let _ = delete_subkey_hkcu(&user_choice_subkey);
            let (hkey, created) = create_or_open_hkcu(&user_choice_subkey)?;
            let _ = created;

            let ft1 = query_key_last_write_time(hkey)?;
            let ft1_clamped = clamp_filetime_to_minute(ft1.as_u64());
            let regdate_hex = filetime_to_regdate_hex(ft1_clamped);
            let hash = crate::hash::compute_user_choice_hash(&ext, &sid, prog_id, &regdate_hex);

            set_reg_sz(hkey, "ProgId", prog_id)?;
            set_reg_sz(hkey, "Hash", &hash)?;

            let ft2 = query_key_last_write_time(hkey)?;
            let _ = reg_close_key(hkey);

            let ft2_clamped = clamp_filetime_to_minute(ft2.as_u64());
            if ft1_clamped == ft2_clamped {
                return Ok(SetUserChoiceResult {
                    ext,
                    prog_id: prog_id.to_string(),
                    regdate_hex,
                    hash,
                    attempts: attempt,
                });
            }
        }

        Err(SetUserChoiceError::WindowsApiError {
            api: "set_user_choice(retry_exhausted)",
            code: 0,
        })
    }
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
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
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
    let rc = RegOpenKeyExW(HKEY_CURRENT_USER, subkey_w.as_ptr(), 0, KEY_READ, &mut hkey);
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

#[cfg(windows)]
#[derive(Debug, Copy, Clone)]
struct WinApiError {
    api: &'static str,
    code: u32,
}

#[cfg(windows)]
unsafe fn windows_last_error() -> u32 {
    #[link(name = "Kernel32")]
    extern "system" {
        fn GetLastError() -> u32;
    }
    GetLastError()
}

#[cfg(windows)]
unsafe fn current_user_sid() -> Result<String, WinApiError> {
    type HANDLE = isize;
    type BOOL = i32;
    type PSID = *mut core::ffi::c_void;

    const TOKEN_QUERY: u32 = 0x0008;
    const TOKEN_USER_CLASS: u32 = 1;
    const ERROR_INSUFFICIENT_BUFFER: u32 = 122;

    #[repr(C)]
    #[allow(non_snake_case)]
    struct SID_AND_ATTRIBUTES {
        Sid: PSID,
        Attributes: u32,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    struct TOKEN_USER {
        User: SID_AND_ATTRIBUTES,
    }

    #[link(name = "Advapi32")]
    extern "system" {
        fn OpenProcessToken(
            ProcessHandle: HANDLE,
            DesiredAccess: u32,
            TokenHandle: *mut HANDLE,
        ) -> BOOL;
        fn GetTokenInformation(
            TokenHandle: HANDLE,
            TokenInformationClass: u32,
            TokenInformation: *mut core::ffi::c_void,
            TokenInformationLength: u32,
            ReturnLength: *mut u32,
        ) -> BOOL;
        fn ConvertSidToStringSidW(Sid: PSID, StringSid: *mut *mut u16) -> BOOL;
    }

    #[link(name = "Kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> HANDLE;
        fn CloseHandle(hObject: HANDLE) -> BOOL;
        fn LocalFree(hMem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    }

    let mut token: HANDLE = 0;
    let ok = OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token);
    if ok == 0 {
        return Err(WinApiError {
            api: "OpenProcessToken",
            code: windows_last_error(),
        });
    }

    let mut needed: u32 = 0;
    let ok = GetTokenInformation(
        token,
        TOKEN_USER_CLASS,
        core::ptr::null_mut(),
        0,
        &mut needed,
    );
    let code = windows_last_error();
    if ok != 0 || code != ERROR_INSUFFICIENT_BUFFER || needed == 0 {
        let _ = CloseHandle(token);
        return Err(WinApiError {
            api: "GetTokenInformation(size)",
            code,
        });
    }

    let mut buf = vec![0u8; needed as usize];
    let ok = GetTokenInformation(
        token,
        TOKEN_USER_CLASS,
        buf.as_mut_ptr() as *mut core::ffi::c_void,
        needed,
        &mut needed,
    );
    if ok == 0 {
        let code = windows_last_error();
        let _ = CloseHandle(token);
        return Err(WinApiError {
            api: "GetTokenInformation(data)",
            code,
        });
    }

    let token_user = &*(buf.as_ptr() as *const TOKEN_USER);
    let mut sid_w: *mut u16 = core::ptr::null_mut();
    let ok = ConvertSidToStringSidW(token_user.User.Sid, &mut sid_w);
    if ok == 0 {
        let code = windows_last_error();
        let _ = CloseHandle(token);
        return Err(WinApiError {
            api: "ConvertSidToStringSidW",
            code,
        });
    }

    let mut len = 0usize;
    while *sid_w.add(len) != 0 {
        len += 1;
    }
    let slice = core::slice::from_raw_parts(sid_w, len);
    let sid = String::from_utf16_lossy(slice);
    let _ = LocalFree(sid_w as *mut core::ffi::c_void);
    let _ = CloseHandle(token);

    Ok(sid)
}

#[cfg(windows)]
unsafe fn read_hash_version(sid: &str) -> Result<Option<u32>, WinApiError> {
    type HKEY = isize;
    const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002_u32 as isize;
    const KEY_READ: u32 = 0x20019;
    const ERROR_FILE_NOT_FOUND: u32 = 2;
    const ERROR_SUCCESS: u32 = 0;
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
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let subkey = format!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\SystemProtectedUserData\\{}\\AnyoneRead\\AppDefaults", sid);
    let subkey_w = to_wide(&subkey);
    let mut hkey: HKEY = 0;
    let rc = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        subkey_w.as_ptr(),
        0,
        KEY_READ,
        &mut hkey,
    );
    if rc == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    if rc != ERROR_SUCCESS {
        return Err(WinApiError {
            api: "RegOpenKeyExW(HKLM HashVersion)",
            code: rc,
        });
    }

    let name_w = to_wide("HashVersion");
    let mut value_type: u32 = 0;
    let mut data: u32 = 0;
    let mut data_len: u32 = 4;
    let rc = RegQueryValueExW(
        hkey,
        name_w.as_ptr(),
        core::ptr::null_mut(),
        &mut value_type,
        (&mut data as *mut u32) as *mut u8,
        &mut data_len,
    );
    let _ = RegCloseKey(hkey);
    if rc == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    if rc != ERROR_SUCCESS {
        return Err(WinApiError {
            api: "RegQueryValueExW(HashVersion)",
            code: rc,
        });
    }
    if value_type != REG_DWORD {
        return Ok(None);
    }
    Ok(Some(data))
}

#[cfg(windows)]
unsafe fn delete_subkey_hkcu(subkey: &str) -> Result<(), WinApiError> {
    type HKEY = isize;
    const HKEY_CURRENT_USER: HKEY = 0x8000_0001_u32 as isize;
    const ERROR_FILE_NOT_FOUND: u32 = 2;
    const ERROR_SUCCESS: u32 = 0;

    #[link(name = "Advapi32")]
    extern "system" {
        fn RegDeleteKeyW(hKey: HKEY, lpSubKey: *const u16) -> u32;
    }

    fn to_wide(s: &str) -> Vec<u16> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let subkey_w = to_wide(subkey);
    let rc = RegDeleteKeyW(HKEY_CURRENT_USER, subkey_w.as_ptr());
    if rc == ERROR_SUCCESS || rc == ERROR_FILE_NOT_FOUND {
        Ok(())
    } else {
        Err(WinApiError {
            api: "RegDeleteKeyW",
            code: rc,
        })
    }
}

#[cfg(windows)]
unsafe fn create_or_open_hkcu(subkey: &str) -> Result<(isize, bool), SetUserChoiceError> {
    type HKEY = isize;
    const HKEY_CURRENT_USER: HKEY = 0x8000_0001_u32 as isize;
    const KEY_READ: u32 = 0x20019;
    const KEY_WRITE: u32 = 0x20006;
    const ERROR_SUCCESS: u32 = 0;
    const REG_OPTION_NON_VOLATILE: u32 = 0;
    const REG_CREATED_NEW_KEY: u32 = 1;

    #[link(name = "Advapi32")]
    extern "system" {
        fn RegCreateKeyExW(
            hKey: HKEY,
            lpSubKey: *const u16,
            Reserved: u32,
            lpClass: *mut u16,
            dwOptions: u32,
            samDesired: u32,
            lpSecurityAttributes: *mut core::ffi::c_void,
            phkResult: *mut HKEY,
            lpdwDisposition: *mut u32,
        ) -> u32;
    }

    fn to_wide(s: &str) -> Vec<u16> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let subkey_w = to_wide(subkey);
    let mut hkey: HKEY = 0;
    let mut disp: u32 = 0;
    let rc = RegCreateKeyExW(
        HKEY_CURRENT_USER,
        subkey_w.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        &mut hkey,
        &mut disp,
    );
    if rc != ERROR_SUCCESS {
        return Err(SetUserChoiceError::WindowsApiError {
            api: "RegCreateKeyExW",
            code: rc,
        });
    }
    Ok((hkey, disp == REG_CREATED_NEW_KEY))
}

#[cfg(windows)]
unsafe fn query_key_last_write_time(hkey: isize) -> Result<FileTime, SetUserChoiceError> {
    type HKEY = isize;
    const ERROR_SUCCESS: u32 = 0;

    #[repr(C)]
    #[derive(Copy, Clone)]
    #[allow(non_snake_case)]
    struct FILETIME {
        dwLowDateTime: u32,
        dwHighDateTime: u32,
    }

    #[link(name = "Advapi32")]
    extern "system" {
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

    let mut ft = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let rc = RegQueryInfoKeyW(
        hkey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut ft,
    );
    if rc != ERROR_SUCCESS {
        return Err(SetUserChoiceError::WindowsApiError {
            api: "RegQueryInfoKeyW(last_write_time)",
            code: rc,
        });
    }
    Ok(FileTime {
        low_date_time: ft.dwLowDateTime,
        high_date_time: ft.dwHighDateTime,
    })
}

#[cfg(windows)]
unsafe fn set_reg_sz(hkey: isize, name: &str, value: &str) -> Result<(), SetUserChoiceError> {
    type HKEY = isize;
    const REG_SZ: u32 = 1;
    const ERROR_SUCCESS: u32 = 0;

    #[link(name = "Advapi32")]
    extern "system" {
        fn RegSetValueExW(
            hKey: HKEY,
            lpValueName: *const u16,
            Reserved: u32,
            dwType: u32,
            lpData: *const u8,
            cbData: u32,
        ) -> u32;
    }

    fn to_wide(s: &str) -> Vec<u16> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let name_w = to_wide(name);
    let value_w = to_wide(value);
    let rc = RegSetValueExW(
        hkey,
        name_w.as_ptr(),
        0,
        REG_SZ,
        value_w.as_ptr() as *const u8,
        (value_w.len() * 2) as u32,
    );
    if rc != ERROR_SUCCESS {
        return Err(SetUserChoiceError::WindowsApiError {
            api: "RegSetValueExW",
            code: rc,
        });
    }
    Ok(())
}

#[cfg(windows)]
unsafe fn reg_close_key(hkey: isize) -> u32 {
    type HKEY = isize;
    #[link(name = "Advapi32")]
    extern "system" {
        fn RegCloseKey(hKey: HKEY) -> u32;
    }
    RegCloseKey(hkey)
}

#[cfg(windows)]
unsafe fn read_open_with_progids_hkcu_fileexts(
    ext: &str,
) -> Result<Vec<String>, ReadUserChoiceError> {
    let subkey = format!(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FileExts\\{}\\OpenWithProgids",
        ext
    );
    enum_value_names(0x8000_0001_u32 as isize, &subkey).map_err(|e| {
        ReadUserChoiceError::WindowsApiError {
            api: e.api,
            code: e.code,
        }
    })
}

#[cfg(windows)]
unsafe fn read_open_with_progids_hkcr(ext: &str) -> Result<Vec<String>, ReadUserChoiceError> {
    let subkey = format!("{}\\OpenWithProgids", ext);
    enum_value_names(0x8000_0000_u32 as isize, &subkey).map_err(|e| {
        ReadUserChoiceError::WindowsApiError {
            api: e.api,
            code: e.code,
        }
    })
}

#[cfg(windows)]
unsafe fn enum_value_names(root: isize, subkey: &str) -> Result<Vec<String>, WinApiError> {
    type HKEY = isize;
    const KEY_READ: u32 = 0x20019;
    const ERROR_SUCCESS: u32 = 0;
    const ERROR_NO_MORE_ITEMS: u32 = 259;
    const ERROR_FILE_NOT_FOUND: u32 = 2;

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
        fn RegEnumValueW(
            hKey: HKEY,
            dwIndex: u32,
            lpValueName: *mut u16,
            lpcchValueName: *mut u32,
            lpReserved: *mut u32,
            lpType: *mut u32,
            lpData: *mut u8,
            lpcbData: *mut u32,
        ) -> u32;
    }

    fn to_wide(s: &str) -> Vec<u16> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let subkey_w = to_wide(subkey);
    let mut hkey: HKEY = 0;
    let rc = RegOpenKeyExW(root, subkey_w.as_ptr(), 0, KEY_READ, &mut hkey);
    if rc == ERROR_FILE_NOT_FOUND {
        return Ok(Vec::new());
    }
    if rc != ERROR_SUCCESS {
        return Err(WinApiError {
            api: "RegOpenKeyExW(enum_value_names)",
            code: rc,
        });
    }

    let mut out = Vec::new();
    for i in 0u32.. {
        let mut buf = vec![0u16; 1024];
        let mut len: u32 = (buf.len() - 1) as u32;
        let rc = RegEnumValueW(
            hkey,
            i,
            buf.as_mut_ptr(),
            &mut len,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if rc == ERROR_NO_MORE_ITEMS {
            break;
        }
        if rc != ERROR_SUCCESS {
            let _ = RegCloseKey(hkey);
            return Err(WinApiError {
                api: "RegEnumValueW",
                code: rc,
            });
        }
        buf.truncate(len as usize);
        if !buf.is_empty() {
            out.push(String::from_utf16_lossy(&buf));
        }
    }

    let _ = RegCloseKey(hkey);
    Ok(out)
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
        assert!(matches!(
            normalize_ext(""),
            Err(ReadUserChoiceError::InvalidExt)
        ));
        assert!(matches!(
            normalize_ext("   "),
            Err(ReadUserChoiceError::InvalidExt)
        ));
        assert!(matches!(
            normalize_ext("."),
            Err(ReadUserChoiceError::InvalidExt)
        ));
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

    #[test]
    fn clamp_filetime_to_minute_is_multiple_of_minute() {
        let minute = 600_000_000u64;
        let ft = 123456789012345678u64;
        let clamped = clamp_filetime_to_minute(ft);
        assert_eq!(clamped % minute, 0);
        assert!(clamped <= ft);
    }

    #[test]
    fn filetime_hex_is_lowercase_16_chars() {
        assert_eq!(filetime_to_regdate_hex(0), "0000000000000000");
        assert_eq!(filetime_to_regdate_hex(0x1), "0000000000000001");
        assert_eq!(filetime_to_regdate_hex(0xabcdef), "0000000000abcdef");
    }
}

pub fn clamp_filetime_to_minute(filetime: u64) -> u64 {
    const MINUTE_100NS: u64 = 600_000_000;
    filetime - (filetime % MINUTE_100NS)
}

pub fn filetime_to_regdate_hex(filetime: u64) -> String {
    format!("{:016x}", filetime)
}
