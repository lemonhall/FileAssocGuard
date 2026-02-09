use std::path::{Path, PathBuf};
use std::process::Command;

pub const ENV_SETUSERFTA_EXE: &str = "FAG_SETUSERFTA_EXE";

pub fn find_setuserfta_exe(cli_override: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = cli_override.and_then(non_empty) {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }

    if let Ok(p) = std::env::var(ENV_SETUSERFTA_EXE) {
        let p = p.trim();
        if !p.is_empty() {
            let pb = PathBuf::from(p);
            if pb.is_file() {
                return Some(pb);
            }
        }
    }

    let local = PathBuf::from("SetUserFTA.exe");
    if local.is_file() {
        return Some(local);
    }

    which_in_path("SetUserFTA.exe")
}

pub fn set_association(exe: &Path, ext: &str, prog_id: &str) -> Result<(), SetUserFtaError> {
    let ext = ext.trim();
    let prog_id = prog_id.trim();
    if ext.is_empty() || prog_id.is_empty() {
        return Err(SetUserFtaError::InvalidArgs);
    }

    let mut cmd = if is_cmd_script(exe) {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(exe);
        c
    } else {
        Command::new(exe)
    };

    let out =
        cmd.arg(ext).arg(prog_id).output().map_err(|e| {
            SetUserFtaError::SpawnFailed(format!("failed to spawn SetUserFTA: {}", e))
        })?;

    if out.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Err(SetUserFtaError::NonZeroExit {
        code: out.status.code(),
        stdout,
        stderr,
    })
}

#[derive(Debug)]
pub enum SetUserFtaError {
    InvalidArgs,
    SpawnFailed(String),
    NonZeroExit {
        code: Option<i32>,
        stdout: String,
        stderr: String,
    },
}

impl std::fmt::Display for SetUserFtaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgs => write!(f, "invalid args"),
            Self::SpawnFailed(msg) => write!(f, "{}", msg),
            Self::NonZeroExit {
                code,
                stdout,
                stderr,
            } => {
                write!(f, "SetUserFTA failed")?;
                if let Some(code) = code {
                    write!(f, " (exit={})", code)?;
                }
                if !stderr.is_empty() {
                    write!(f, ": {}", stderr)?;
                } else if !stdout.is_empty() {
                    write!(f, ": {}", stdout)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for SetUserFtaError {}

fn non_empty(s: &str) -> Option<&str> {
    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn is_cmd_script(p: &Path) -> bool {
    matches!(
        p.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase()),
        Some(ref e) if e == "cmd" || e == "bat"
    )
}

fn which_in_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(name);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_setuserfta_prefers_cli_override() {
        let td = tempfile::tempdir().unwrap();
        let fake = td.path().join("SetUserFTA.exe");
        std::fs::write(&fake, b"").unwrap();

        let found = find_setuserfta_exe(Some(fake.to_string_lossy().as_ref())).unwrap();
        assert_eq!(found, fake);
    }

    #[test]
    fn find_setuserfta_uses_env_var() {
        let td = tempfile::tempdir().unwrap();
        let fake = td.path().join("SetUserFTA.exe");
        std::fs::write(&fake, b"").unwrap();

        std::env::set_var(ENV_SETUSERFTA_EXE, &fake);
        let found = find_setuserfta_exe(None).unwrap();
        std::env::remove_var(ENV_SETUSERFTA_EXE);

        assert_eq!(found, fake);
    }
}
