use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::DOTFILES_DIR;
use crate::discover::{build_win_path, list_syncable_files};
use crate::types::PkgType;
use crate::util::skip_size_limit;

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
    pub outdated_remote: usize,
    pub missing_remote: usize,
    pub skipped: usize,
    pub error: usize,
}

pub fn status_text(counts: &CopyStatusCounts) -> String {
    let mut parts: Vec<String> = Vec::new();
    if counts.synced > 0 {
        parts.push(format!("{} synced", counts.synced));
    }
    if counts.outdated_local > 0 {
        parts.push(format!("{} needs-sync", counts.outdated_local));
    }
    if counts.outdated_remote > 0 {
        parts.push(format!("{} newer-on-win", counts.outdated_remote));
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
    use rayon::prelude::*;
    let (s, t): (Vec<usize>, Vec<usize>) = pkgs
        .par_iter()
        .map(|pkg| check_stow_status(pkg, &pt))
        .unzip();
    (s.iter().sum(), t.iter().sum())
}

pub fn check_copy_status(pkg: &Path, pt: &PkgType) -> CopyStatusCounts {
    let mut counts = CopyStatusCounts::default();

    if !pkg.is_dir() {
        return counts;
    }

    let mut index = SyncIndex::load();
    let files = list_syncable_files(pkg);

    for f in &files {
        let key = index_key(pkg, f.strip_prefix(pkg).unwrap_or(f));

        if let Ok(meta) = f.metadata() {
            if skip_size_limit(f).unwrap_or(false) {
                counts.skipped += 1;
                continue;
            }
            let mtime_ns = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            let size = meta.len();

            if let Some(entry) = index.get(&key) {
                if entry.mtime_ns == mtime_ns && entry.size == size {
                    if entry.win_mtime_ns.is_some() && entry.win_size.is_some() {
                        counts.synced += 1;
                        continue;
                    }
                }
            }

            let win_path = build_win_path(f, pkg, pt);
            let win_exists = win_path.symlink_metadata().is_ok();

            if !win_exists {
                counts.missing_remote += 1;
                index.set(key, IndexEntry {
                    mtime_ns,
                    size,
                    win_mtime_ns: None,
                    win_size: None,
                });
                continue;
            }

            let wn = match win_path.metadata() {
                Ok(m) => m,
                Err(_) => {
                    counts.error += 1;
                    continue;
                }
            };

            let ws_mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());
            let wn_mtime = wn
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());

            let wn_mtime_ns = wn
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_nanos() as u64);
            let wn_size = wn.len();

            index.set(key, IndexEntry {
                mtime_ns,
                size,
                win_mtime_ns: wn_mtime_ns,
                win_size: Some(wn_size),
            });

            if let (Some(wsm), Some(wnm)) = (ws_mtime, wn_mtime) {
                let mtime_diff = wsm.abs_diff(wnm);
                if mtime_diff < 1 && size == wn_size {
                    counts.synced += 1;
                } else if wsm > wnm {
                    counts.outdated_local += 1;
                } else {
                    counts.outdated_remote += 1;
                }
            } else {
                counts.error += 1;
            }
        } else {
            counts.error += 1;
        }
    }

    index.save();
    counts
}

pub fn check_copy_status_batch(pkgs: &[PathBuf], pt: PkgType) -> CopyStatusCounts {
    let mut total = CopyStatusCounts::default();
    for pkg in pkgs {
        let c = check_copy_status(pkg, &pt);
        total.synced += c.synced;
        total.outdated_local += c.outdated_local;
        total.outdated_remote += c.outdated_remote;
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
