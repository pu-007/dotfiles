use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{DOTFILES_DIR, MAX_SYNC_SIZE_BYTES};
use crate::discover::{build_win_path, list_syncable_files};
use crate::types::PkgType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub mtime_ns: u64,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_mtime_ns: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncIndex {
    pub version: u32,
    pub entries: HashMap<String, IndexEntry>,
}

impl SyncIndex {
    pub fn load() -> Self {
        let path = DOTFILES_DIR.join(".wots_index.json");
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|_| Self::default()),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = DOTFILES_DIR.join(".wots_index.json");
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }

    pub fn get(&self, key: &str) -> Option<&IndexEntry> {
        self.entries.get(key)
    }

    pub fn set(&mut self, key: String, entry: IndexEntry) {
        self.entries.insert(key, entry);
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

pub fn is_symlink_or_parent(path: &Path, root: &Path) -> bool {
    if is_symlink(path) {
        return true;
    }

    let mut current = path.to_path_buf();
    while current != *root && current != current.parent().unwrap_or(&current) {
        if let Some(parent) = current.parent() {
            if is_symlink(parent) {
                return true;
            }
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    false
}

pub fn is_symlink(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(m) => m.file_type().is_symlink(),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            std::process::Command::new("sudo")
                .args(["test", "-L", &path.to_string_lossy()])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        Err(_) => false,
    }
}

#[derive(Debug, Default, Clone)]
pub struct CopyStatusCounts {
    pub synced: usize,
    pub outdated_local: usize,
    pub missing_remote: usize,
    pub skipped: usize,
    pub error: usize,
}

impl CopyStatusCounts {
    pub fn total(&self) -> usize {
        self.synced + self.outdated_local + self.missing_remote + self.skipped + self.error
    }
}

pub fn status_text(counts: &CopyStatusCounts) -> String {
    let mut parts: Vec<String> = Vec::new();
    if counts.synced > 0 {
        parts.push(format!("{} synced", counts.synced));
    }
    if counts.outdated_local > 0 {
        parts.push(format!("{} needs-sync", counts.outdated_local));
    }
    if counts.missing_remote > 0 {
        parts.push(format!("{} missing-win", counts.missing_remote));
    }
    if counts.skipped > 0 {
        parts.push(format!("{} skipped", counts.skipped));
    }
    if parts.is_empty() {
        "empty".to_string()
    } else {
        parts.join(", ")
    }
}

pub fn check_stow_status(pkg: &Path, pt: &PkgType) -> (usize, usize) {
    let target = match pt.sync_target() {
        Some(t) => t,
        None => return (0, 0),
    };

    if !pt.uses_stow() || !pkg.is_dir() {
        return (0, 0);
    }

    let mut stowed = 0usize;
    let mut total = 0usize;

    for f in list_syncable_files(pkg) {
        total += 1;
        let dest = match f.strip_prefix(pkg) {
            Ok(rel) => target.join(rel),
            Err(_) => continue,
        };

        if is_symlink_or_parent(&dest, &target) {
            stowed += 1;
        }
    }

    (stowed, total)
}

pub fn check_stow_status_batch(pkgs: &[PathBuf], pt: PkgType) -> (usize, usize) {
    let mut stowed = 0usize;
    let mut total = 0usize;
    for pkg in pkgs {
        let (s, t) = check_stow_status(pkg, &pt);
        stowed += s;
        total += t;
    }
    (stowed, total)
}

pub fn check_copy_status(pkg: &Path, pt: &PkgType) -> CopyStatusCounts {
    let mut counts = CopyStatusCounts::default();

    if !pkg.is_dir() {
        return counts;
    }

    let files = list_syncable_files(pkg);
    for f in &files {
        if let Ok(meta) = f.metadata() {
            if *MAX_SYNC_SIZE_BYTES > 0 && meta.len() > *MAX_SYNC_SIZE_BYTES {
                counts.skipped += 1;
                continue;
            }
        } else {
            counts.error += 1;
            continue;
        }

        let win_path = build_win_path(f, pkg, pt);

        let win_exists = win_path.symlink_metadata().is_ok();

        if !win_exists {
            counts.missing_remote += 1;
            continue;
        }

        let ws = match f.metadata() {
            Ok(m) => m,
            Err(_) => {
                counts.error += 1;
                continue;
            }
        };
        let wn = match win_path.metadata() {
            Ok(m) => m,
            Err(_) => {
                counts.error += 1;
                continue;
            }
        };

        let ws_mtime = ws
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        let wn_mtime = wn
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        if let (Some(wsm), Some(wnm)) = (ws_mtime, wn_mtime) {
            let mtime_diff = wsm.abs_diff(wnm);
            if mtime_diff < 1 && ws.len() == wn.len() {
                counts.synced += 1;
            } else if wsm > wnm {
                counts.outdated_local += 1;
            }
        } else {
            counts.error += 1;
        }
    }

    counts
}

pub fn check_copy_status_batch(pkgs: &[PathBuf], pt: PkgType) -> CopyStatusCounts {
    let mut total = CopyStatusCounts::default();
    for pkg in pkgs {
        let c = check_copy_status(pkg, &pt);
        total.synced += c.synced;
        total.outdated_local += c.outdated_local;
        total.missing_remote += c.missing_remote;
        total.skipped += c.skipped;
        total.error += c.error;
    }
    total
}

fn index_key(pkg: &Path, rel: &Path) -> String {
    let pkg_name = pkg
        .file_name()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_string_lossy();
    format!("{}/{}", pkg_name, rel.display())
}
