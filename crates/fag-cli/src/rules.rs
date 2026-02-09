use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct RulesStore {
    version: u32,
    #[serde(default)]
    by_ext: BTreeMap<String, String>,
}

pub fn default_rules_path() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return PathBuf::from(appdata)
            .join("FileAssocGuard")
            .join("rules.json");
    }
    PathBuf::from("rules.json")
}

pub fn load_rules(path: &Path) -> std::io::Result<BTreeMap<String, String>> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(e) => return Err(e),
    };

    let store: RulesStore = serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(store.by_ext)
}

pub fn save_rules(path: &Path, by_ext: &BTreeMap<String, String>) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let store = RulesStore {
        version: 1,
        by_ext: by_ext.clone(),
    };
    let bytes = serde_json::to_vec_pretty(&store)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, bytes)
}

pub fn upsert_rule(path: &Path, ext: &str, name: &str) -> std::io::Result<()> {
    let mut rules = load_rules(path)?;
    rules.insert(ext.to_string(), name.to_string());
    save_rules(path, &rules)
}

pub fn remove_rule(path: &Path, ext: &str) -> std::io::Result<bool> {
    let mut rules = load_rules(path)?;
    let removed = rules.remove(ext).is_some();
    save_rules(path, &rules)?;
    Ok(removed)
}

pub fn list_rules(path: &Path) -> std::io::Result<Vec<(String, String)>> {
    let rules = load_rules(path)?;
    Ok(rules.into_iter().collect())
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
    fn rules_roundtrip_upsert_remove_list() {
        let path = temp_path("rules");

        upsert_rule(&path, ".mp4", "vlc").unwrap();
        upsert_rule(&path, ".mkv", "potplayer").unwrap();

        let items = list_rules(&path).unwrap();
        assert_eq!(
            items,
            vec![
                (".mkv".to_string(), "potplayer".to_string()),
                (".mp4".to_string(), "vlc".to_string())
            ]
        );

        assert_eq!(remove_rule(&path, ".mp4").unwrap(), true);
        assert_eq!(remove_rule(&path, ".mp4").unwrap(), false);
        assert_eq!(
            list_rules(&path).unwrap(),
            vec![(".mkv".to_string(), "potplayer".to_string())]
        );

        let _ = std::fs::remove_file(&path);
    }
}
