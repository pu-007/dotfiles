use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::DOTFILES_DIR;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub mtime_ns: u64,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_mtime_ns: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blake3_wsl: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blake3_win: Option<String>,
    /// Whether this entry represents a successfully synced file.
    /// Only synced entries qualify for the fast-path shortcut;
    /// unsynced entries are tracked for deletion detection.
    #[serde(default)]
    pub synced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncIndex {
    pub version: u32,
    pub entries: HashMap<String, IndexEntry>,
}

impl SyncIndex {
    pub fn load() -> Self {
        Self::load_from(&DOTFILES_DIR)
    }

    pub fn load_from(base: &Path) -> Self {
        let path = base.join(".wots_index.json");
        match fs::read_to_string(&path) {
            Ok(contents) => {
                match serde_json::from_str(&contents) {
                    Ok(idx) => idx,
                    Err(e) => {
                        eprintln!(
                            "  wots: warning — failed to parse sync index, starting fresh: {e}"
                        );
                        Self::default()
                    }
                }
            }
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(&DOTFILES_DIR)
    }

    pub fn save_to(&self, base: &Path) -> std::io::Result<()> {
        let path = base.join(".wots_index.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, &json)?;
        fs::rename(&tmp, &path)
    }

    pub fn get(&self, key: &str) -> Option<&IndexEntry> {
        self.entries.get(key)
    }

    pub fn set(&mut self, key: String, entry: IndexEntry) {
        self.entries.insert(key, entry);
    }

    #[cfg(test)]
    pub fn keys_cloned(&self) -> std::collections::HashSet<String> {
        self.entries.keys().cloned().collect()
    }
}

impl Default for SyncIndex {
    fn default() -> Self {
        Self {
            version: 1,
            entries: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_index_default() {
        let idx = SyncIndex::default();
        assert_eq!(idx.version, 1);
        assert!(idx.entries.is_empty());
    }

    #[test]
    fn sync_index_set_get() {
        let mut idx = SyncIndex::default();
        idx.set(
            "test.file".into(),
            IndexEntry {
                mtime_ns: 100,
                size: 50,
                win_mtime_ns: Some(101),
                win_size: Some(50),
                blake3_wsl: None,
                blake3_win: None,
                synced: false,
            },
        );
        assert!(idx.get("test.file").is_some());
        assert_eq!(idx.get("test.file").unwrap().size, 50);
        assert!(idx.get("nonexistent").is_none());
    }

    #[test]
    fn sync_index_keys_cloned() {
        let mut idx = SyncIndex::default();
        idx.set("a".into(), IndexEntry { mtime_ns: 0, size: 0, win_mtime_ns: None, win_size: None, blake3_wsl: None, blake3_win: None, synced: false });
        idx.set("b".into(), IndexEntry { mtime_ns: 0, size: 0, win_mtime_ns: None, win_size: None, blake3_wsl: None, blake3_win: None, synced: false });
        let keys = idx.keys_cloned();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains("a"));
        assert!(keys.contains("b"));
    }

    #[test]
    fn index_entry_hash_serialization() {
        let mut idx = SyncIndex::default();
        idx.set(
            "h.txt".into(),
            IndexEntry {
                mtime_ns: 100,
                size: 50,
                win_mtime_ns: Some(101),
                win_size: Some(50),
                blake3_wsl: Some("abc123".into()),
                blake3_win: Some("def456".into()),
                synced: true,
            },
        );
        let json = serde_json::to_value(&idx).unwrap();
        let entry = &json["entries"]["h.txt"];
        assert_eq!(entry["blake3_wsl"], "abc123");
        assert_eq!(entry["blake3_win"], "def456");
        assert_eq!(entry["synced"], true);

        let restored: SyncIndex = serde_json::from_value(json).unwrap();
        let e = restored.get("h.txt").unwrap();
        assert_eq!(e.blake3_wsl.as_deref(), Some("abc123"));
        assert_eq!(e.blake3_win.as_deref(), Some("def456"));
    }

    #[test]
    fn index_entry_hash_none_omitted() {
        let entry = IndexEntry {
            mtime_ns: 100,
            size: 50,
            win_mtime_ns: None,
            win_size: None,
            blake3_wsl: None,
            blake3_win: None,
            synced: false,
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert!(json.get("blake3_wsl").is_none());
        assert!(json.get("blake3_win").is_none());
    }

    #[test]
    fn load_from_corrupted_json_warns_and_returns_default() {
        use std::fs;
        let dir = std::env::temp_dir().join(format!("wots_idx_corrupt_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(".wots_index.json"), "not json {{{").unwrap();
        let idx = SyncIndex::load_from(&dir);
        assert_eq!(idx.version, 1);
        assert!(idx.entries.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_from_missing_file_returns_default() {
        let idx = SyncIndex::load_from(Path::new("/nonexistent/path/for/index/test"));
        assert_eq!(idx.version, 1);
        assert!(idx.entries.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        use std::fs;
        let dir = std::env::temp_dir().join(format!("wots_idx_rt_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut idx = SyncIndex::default();
        idx.set(
            "p/x".into(),
            IndexEntry {
                mtime_ns: 100,
                size: 200,
                win_mtime_ns: Some(101),
                win_size: Some(200),
                blake3_wsl: None,
                blake3_win: None,
                synced: false,
            },
        );
        idx.save_to(&dir).unwrap();
        let ld = SyncIndex::load_from(&dir);
        assert_eq!(ld.get("p/x").unwrap().mtime_ns, 100);
        let _ = fs::remove_dir_all(&dir);
    }
}
