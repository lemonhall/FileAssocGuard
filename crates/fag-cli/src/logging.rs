use std::path::{Path, PathBuf};

pub fn default_log_path() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return PathBuf::from(appdata)
            .join("FileAssocGuard")
            .join("guard.log");
    }
    PathBuf::from("guard.log")
}

pub fn append_line(path: &Path, line: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut s = String::with_capacity(line.len() + 1);
    s.push_str(line);
    if !s.ends_with('\n') {
        s.push('\n');
    }
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    f.write_all(s.as_bytes())
}
