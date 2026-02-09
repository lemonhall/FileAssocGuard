use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatestCapture {
    pub prog_id: String,
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_write_time_filetime: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prog_id_last_write_time_filetime: Option<u64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CaptureStore {
    version: u32,
    #[serde(default)]
    by_ext: BTreeMap<String, BTreeMap<String, LatestCapture>>,
}

pub fn default_store_path() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return PathBuf::from(appdata)
            .join("FileAssocGuard")
            .join("captures.json");
    }
    PathBuf::from("captures.json")
}

pub fn load_store(
    path: &Path,
) -> std::io::Result<BTreeMap<String, BTreeMap<String, LatestCapture>>> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(e) => return Err(e),
    };

    let store: CaptureStore = serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(store.by_ext)
}

pub fn save_store(
    path: &Path,
    by_ext: &BTreeMap<String, BTreeMap<String, LatestCapture>>,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let store = CaptureStore {
        version: 1,
        by_ext: by_ext.clone(),
    };
    let bytes = serde_json::to_vec_pretty(&store)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, bytes)
}

pub fn upsert_latest_capture(
    path: &Path,
    ext: &str,
    name: &str,
    cap: LatestCapture,
) -> std::io::Result<()> {
    let mut by_ext = load_store(path)?;
    by_ext
        .entry(ext.to_string())
        .or_default()
        .insert(name.to_string(), cap);
    save_store(path, &by_ext)
}

pub fn get_latest_capture(
    path: &Path,
    ext: &str,
    name: &str,
) -> std::io::Result<Option<LatestCapture>> {
    let by_ext = load_store(path)?;
    Ok(by_ext.get(ext).and_then(|m| m.get(name)).cloned())
}

pub fn list_capture_names(path: &Path, ext: &str) -> std::io::Result<Vec<String>> {
    let by_ext = load_store(path)?;
    let mut out = by_ext
        .get(ext)
        .map(|m| m.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        p.push(format!("fag-cli-{}-{}.json", name, nanos));
        p
    }

    #[test]
    fn store_roundtrip_upsert_get_list() {
        let path = temp_path("captures");

        let cap1 = LatestCapture {
            prog_id: "VLC.mp4".to_string(),
            hash: "abc=".to_string(),
            last_write_time_filetime: Some(123),
            prog_id_last_write_time_filetime: None,
        };
        upsert_latest_capture(&path, ".mp4", "vlc", cap1.clone()).unwrap();

        let cap2 = LatestCapture {
            prog_id: "PotPlayerMini64.mp4".to_string(),
            hash: "def=".to_string(),
            last_write_time_filetime: None,
            prog_id_last_write_time_filetime: Some(456),
        };
        upsert_latest_capture(&path, ".mp4", "potplayer", cap2.clone()).unwrap();

        assert_eq!(
            get_latest_capture(&path, ".mp4", "vlc").unwrap(),
            Some(cap1)
        );
        assert_eq!(
            get_latest_capture(&path, ".mp4", "potplayer").unwrap(),
            Some(cap2)
        );

        let names = list_capture_names(&path, ".mp4").unwrap();
        assert_eq!(names, vec!["potplayer".to_string(), "vlc".to_string()]);

        let _ = std::fs::remove_file(&path);
    }
}
