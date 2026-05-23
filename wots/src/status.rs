use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::DOTFILES_DIR;
use crate::discover::{build_win_path, list_syncable_files};
use crate::types::PkgType;
use crate::util::skip_size_limit;

// ---------------------------------------------------------------------------
// Index data model
// ---------------------------------------------------------------------------

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
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
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

    #[cfg(test)]
    pub fn keys_cloned(&self) -> HashSet<String> {
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

// ---------------------------------------------------------------------------
// Per-file sync status
// ---------------------------------------------------------------------------

/// Detailed sync status for a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSyncStatus {
    Synced,
    NeedsSync,
    NewerOnWin,
    MissingWin,
    MissingWsl,
    Skipped,
    Error,
}

impl FileSyncStatus {
    pub fn label(&self) -> &'static str {
        match self {
            FileSyncStatus::Synced => "synced",
            FileSyncStatus::NeedsSync => "needs-sync",
            FileSyncStatus::NewerOnWin => "newer-on-win",
            FileSyncStatus::MissingWin => "missing-win",
            FileSyncStatus::MissingWsl => "missing-wsl",
            FileSyncStatus::Skipped => "skipped",
            FileSyncStatus::Error => "error",
        }
    }
}

/// Per-file status entry returned by the detailed check.
#[derive(Debug, Clone)]
pub struct FileStatusEntry {
    pub relative_path: PathBuf,
    pub status: FileSyncStatus,
}

// ---------------------------------------------------------------------------
// Copy-status accumulator
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct CopyStatusCounts {
    pub synced: usize,
    pub outdated_local: usize,
    pub outdated_remote: usize,
    pub missing_remote: usize,
    pub missing_wsl: usize,
    pub skipped: usize,
    pub error: usize,
}

impl CopyStatusCounts {
    pub fn inc(&mut self, st: &FileSyncStatus) {
        match st {
            FileSyncStatus::Synced => self.synced += 1,
            FileSyncStatus::NeedsSync => self.outdated_local += 1,
            FileSyncStatus::NewerOnWin => self.outdated_remote += 1,
            FileSyncStatus::MissingWin => self.missing_remote += 1,
            FileSyncStatus::MissingWsl => self.missing_wsl += 1,
            FileSyncStatus::Skipped => self.skipped += 1,
            FileSyncStatus::Error => self.error += 1,
        }
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
    if counts.outdated_remote > 0 {
        parts.push(format!("{} newer-on-win", counts.outdated_remote));
    }
    if counts.missing_remote > 0 {
        parts.push(format!("{} missing-win", counts.missing_remote));
    }
    if counts.missing_wsl > 0 {
        parts.push(format!("{} missing-wsl", counts.missing_wsl));
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

// ---------------------------------------------------------------------------
// Symlink helpers
// ---------------------------------------------------------------------------

pub fn is_symlink_or_parent(path: &Path, root: &Path) -> bool {
    if is_symlink(path) {
        return true;
    }
    let mut current = path.to_path_buf();
    while let Some(p) = current.parent() {
        let parent = p.to_path_buf();
        if parent == current || parent == *root {
            break;
        }
        if is_symlink(&parent) {
            return true;
        }
        current = parent;
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

// ---------------------------------------------------------------------------
// Stow status
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Core file-status check (shared between copy-status and diff)
// ---------------------------------------------------------------------------

/// Result of `file_sync_status`: carries metadata alongside the status so
/// the caller can update the index without re-reading the file system.
struct FileSyncResult {
    key: String,
    status: FileSyncStatus,
    mtime_ns: u64,
    size: u64,
    win_mtime_ns: Option<u64>,
    win_size: Option<u64>,
    /// True when the index entry is already up-to-date (fast-path Synced).
    /// The caller should skip re-writing the same values to avoid a TOCTOU
    /// window where the Windows file could vanish between the existence
    /// check and the index-update metadata read.
    index_ok: bool,
}

impl FileSyncResult {
    fn new(key: String, status: FileSyncStatus) -> Self {
        Self {
            key,
            status,
            mtime_ns: 0,
            size: 0,
            win_mtime_ns: None,
            win_size: None,
            index_ok: false,
        }
    }
}

/// Check the sync status of a single WSL-local file against its Windows
/// counterpart.  Returns metadata values to avoid double I/O in the caller.
fn file_sync_status(
    wsl_path: &Path,
    pkg: &Path,
    pt: &PkgType,
    index: &SyncIndex,
) -> FileSyncResult {
    let key = index_key(pkg, wsl_path.strip_prefix(pkg).unwrap_or(wsl_path));

    let meta = match wsl_path.metadata() {
        Ok(m) => m,
        Err(_) => return FileSyncResult::new(key, FileSyncStatus::Error),
    };

    if skip_size_limit(wsl_path).unwrap_or(false) {
        return FileSyncResult::new(key, FileSyncStatus::Skipped);
    }

    let mtime_ns = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let size = meta.len();

    // Fast-path: index says this was synced before and WSL hasn't changed.
    // BUT we must verify the Windows file still exists — it could have been
    // deleted manually outside the tool.
    let win_path = build_win_path(wsl_path, pkg, pt);
    if let Some(entry) = index.get(&key)
        && entry.mtime_ns == mtime_ns
        && entry.size == size
        && entry.win_mtime_ns.is_some()
        && entry.win_size.is_some()
    {
        // Lightweight existence check — symlink_metadata is fast.
        if win_path.symlink_metadata().is_ok() {
            return FileSyncResult {
                key,
                status: FileSyncStatus::Synced,
                mtime_ns,
                size,
                win_mtime_ns: entry.win_mtime_ns,
                win_size: entry.win_size,
                index_ok: true, // index already correct — skip update
            };
        }
        // Windows file vanished since last sync.
        return FileSyncResult {
            key,
            status: FileSyncStatus::MissingWin,
            mtime_ns,
            size,
            win_mtime_ns: None,
            win_size: None,
            index_ok: false,
        };
    }

    // Slow-path: read Windows metadata and compare in detail.
    if win_path.symlink_metadata().is_err() {
        return FileSyncResult {
            key,
            status: FileSyncStatus::MissingWin,
            mtime_ns,
            size,
            win_mtime_ns: None,
            win_size: None,
            index_ok: false,
        };
    }

    let wn = match win_path.metadata() {
        Ok(m) => m,
        Err(_) => return FileSyncResult::new(key, FileSyncStatus::Error),
    };

    let ws_mtime_secs = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());
    let wn_mtime_secs = wn
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

    let status = match (ws_mtime_secs, wn_mtime_secs) {
        (Some(wsm), Some(wnm)) => {
            let mtime_diff = wsm.abs_diff(wnm);
            if mtime_diff < 1 && size == wn_size {
                FileSyncStatus::Synced
            } else if wsm > wnm {
                FileSyncStatus::NeedsSync
            } else {
                FileSyncStatus::NewerOnWin
            }
        }
        _ => FileSyncStatus::Error,
    };

    FileSyncResult {
        key,
        status,
        mtime_ns,
        size,
        win_mtime_ns: wn_mtime_ns,
        win_size: Some(wn_size),
        index_ok: false,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build the index-key prefix used to scope keys to a single package.
fn pkg_key_prefix(pkg: &Path) -> String {
    let n = pkg
        .file_name()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_string_lossy();
    format!("{}/", n)
}

// ---------------------------------------------------------------------------
// Reverse check: detect index entries whose WSL file no longer exists.
// Returns (reported-missing-wsl, keys-to-remove-from-index).
// ---------------------------------------------------------------------------

fn detect_missing_wsl(
    pkg: &Path,
    pt: &PkgType,
    index: &SyncIndex,
    seen_keys: &HashSet<String>,
) -> (Vec<(String, FileSyncStatus)>, Vec<String>) {
    let mut missing: Vec<(String, FileSyncStatus)> = Vec::new();
    let mut remove_keys: Vec<String> = Vec::new();

    let pkg_prefix = pkg_key_prefix(pkg);

    for key in index.entries.keys() {
        if seen_keys.contains(key) {
            continue;
        }
        if !key.starts_with(&pkg_prefix) {
            continue;
        }

        let rel_str = key[pkg_prefix.len()..].to_string();
        let wsl_path = pkg.join(&rel_str);

        // WSL file is gone. Check if Windows still has a copy.
        let win_path = build_win_path(&wsl_path, pkg, pt);
        if win_path.symlink_metadata().is_ok() {
            // Orphaned: Windows still has it, WSL doesn't.
            missing.push((key.clone(), FileSyncStatus::MissingWsl));
        } else {
            // Both sides gone — stale index entry, safe to remove.
            remove_keys.push(key.clone());
        }
    }

    (missing, remove_keys)
}

// ---------------------------------------------------------------------------
// Full copy-status check (counts + per-file details)
// ---------------------------------------------------------------------------

/// Returns both aggregate counts and per-file status entries, performing
/// forward (WSL → Win) and reverse (Win → WSL) checks.
pub fn check_copy_status_detailed(
    pkg: &Path,
    pt: &PkgType,
) -> (CopyStatusCounts, Vec<FileStatusEntry>) {
    let mut counts = CopyStatusCounts::default();
    let mut entries: Vec<FileStatusEntry> = Vec::new();

    if !pkg.is_dir() {
        return (counts, entries);
    }

    let mut index = SyncIndex::load();
    let files = list_syncable_files(pkg);
    let mut seen_keys: HashSet<String> = HashSet::new();

    // --- Forward check: each WSL file ----------------------------------
    for f in &files {
        let result = file_sync_status(f, pkg, pt, &index);
        seen_keys.insert(result.key.clone());
        counts.inc(&result.status);

        if let Ok(rel) = f.strip_prefix(pkg) {
            entries.push(FileStatusEntry {
                relative_path: rel.to_path_buf(),
                status: result.status,
            });
        }

        // Update index with the metadata already collected by file_sync_status.
        // Skip the update when index_ok is true (fast-path Synced) to avoid a
        // TOCTOU race where the Windows file could vanish between the existence
        // check and this second metadata read.
        if !result.index_ok {
            index.set(
                result.key,
                IndexEntry {
                    mtime_ns: result.mtime_ns,
                    size: result.size,
                    win_mtime_ns: result.win_mtime_ns,
                    win_size: result.win_size,
                },
            );
        }
    }

    // --- Reverse check: stale index entries ----------------------------
    let (reverse, remove_keys) = detect_missing_wsl(pkg, pt, &index, &seen_keys);

    for (key, status) in &reverse {
        counts.inc(status);
        let pkg_prefix = pkg_key_prefix(pkg);
        let rel_str = key[pkg_prefix.len()..].to_string();
        entries.push(FileStatusEntry {
            relative_path: PathBuf::from(rel_str),
            status: status.clone(),
        });
    }

    // Remove entries for files that have disappeared from both sides.
    for key in &remove_keys {
        index.entries.remove(key);
    }

    index.save();
    (counts, entries)
}

/// Simplified wrapper: counts-only, ignores per-file details.
pub fn check_copy_status(pkg: &Path, pt: &PkgType) -> CopyStatusCounts {
    check_copy_status_detailed(pkg, pt).0
}

/// Check copy status for multiple packages sequentially (avoids index
/// write conflicts from parallel access).
pub fn check_copy_status_batch(pkgs: &[PathBuf], pt: PkgType) -> CopyStatusCounts {
    let mut total = CopyStatusCounts::default();
    for pkg in pkgs {
        let c = check_copy_status(pkg, &pt);
        total.synced += c.synced;
        total.outdated_local += c.outdated_local;
        total.outdated_remote += c.outdated_remote;
        total.missing_remote += c.missing_remote;
        total.missing_wsl += c.missing_wsl;
        total.skipped += c.skipped;
        total.error += c.error;
    }
    total
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn index_key(pkg: &Path, rel: &Path) -> String {
    let pkg_name = pkg
        .file_name()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_string_lossy();
    format!("{}/{}", pkg_name, rel.display())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("wots_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn is_symlink_regular_file() {
        let dir = temp_dir();
        let f = dir.join("regular.txt");
        write_file(&f, "hello");
        assert!(!is_symlink(&f));
    }

    #[test]
    fn symlink_metadata_nonexistent() {
        assert!(!is_symlink(Path::new("/nonexistent/path/for/testing")));
    }

    #[test]
    fn file_sync_status_missing_win() {
        let dir = temp_dir();
        // Create a minimal package structure
        let pkg = dir.join("testapp.winuser");
        fs::create_dir_all(&pkg).unwrap();
        let file = pkg.join("test.txt");
        write_file(&file, "content");

        let index = SyncIndex::default();
        // build_win_path will produce a path that almost certainly doesn't exist
        let result = file_sync_status(&file, &pkg, &PkgType::WinUser, &index);
        assert_eq!(result.status, FileSyncStatus::MissingWin);
    }

    #[test]
    fn copy_status_counts_default_zero() {
        let c = CopyStatusCounts::default();
        assert_eq!(c.synced, 0);
        assert_eq!(c.outdated_local, 0);
        assert_eq!(c.outdated_remote, 0);
        assert_eq!(c.missing_remote, 0);
        assert_eq!(c.missing_wsl, 0);
        assert_eq!(c.skipped, 0);
        assert_eq!(c.error, 0);
    }

    #[test]
    fn copy_status_counts_inc_synced() {
        let mut c = CopyStatusCounts::default();
        c.inc(&FileSyncStatus::Synced);
        assert_eq!(c.synced, 1);
    }

    #[test]
    fn copy_status_counts_inc_needs_sync() {
        let mut c = CopyStatusCounts::default();
        c.inc(&FileSyncStatus::NeedsSync);
        assert_eq!(c.outdated_local, 1);
    }

    #[test]
    fn copy_status_counts_inc_newer_on_win() {
        let mut c = CopyStatusCounts::default();
        c.inc(&FileSyncStatus::NewerOnWin);
        assert_eq!(c.outdated_remote, 1);
    }

    #[test]
    fn copy_status_counts_inc_missing_win() {
        let mut c = CopyStatusCounts::default();
        c.inc(&FileSyncStatus::MissingWin);
        assert_eq!(c.missing_remote, 1);
    }

    #[test]
    fn copy_status_counts_inc_missing_wsl() {
        let mut c = CopyStatusCounts::default();
        c.inc(&FileSyncStatus::MissingWsl);
        assert_eq!(c.missing_wsl, 1);
    }

    #[test]
    fn status_text_empty() {
        let c = CopyStatusCounts::default();
        assert_eq!(status_text(&c), "empty");
    }

    #[test]
    fn status_text_synced_only() {
        let c = CopyStatusCounts {
            synced: 3,
            ..Default::default()
        };
        assert!(status_text(&c).contains("3 synced"));
    }

    #[test]
    fn status_text_mixed() {
        let c = CopyStatusCounts {
            synced: 2,
            outdated_local: 1,
            missing_remote: 1,
            ..Default::default()
        };
        let s = status_text(&c);
        assert!(s.contains("2 synced"));
        assert!(s.contains("1 needs-sync"));
        assert!(s.contains("1 missing-win"));
    }

    #[test]
    fn index_key_format() {
        let pkg = PathBuf::from("/tmp/mypkg.winuser");
        let rel = PathBuf::from("subdir/file.txt");
        let key = index_key(&pkg, &rel);
        assert_eq!(key, "mypkg.winuser/subdir/file.txt");
    }

    #[test]
    fn index_key_root_file() {
        let pkg = PathBuf::from("/tmp/mypkg.winuser");
        let rel = PathBuf::from("file.txt");
        let key = index_key(&pkg, &rel);
        assert_eq!(key, "mypkg.winuser/file.txt");
    }

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
            },
        );
        assert!(idx.get("test.file").is_some());
        assert_eq!(idx.get("test.file").unwrap().size, 50);
        assert!(idx.get("nonexistent").is_none());
    }

    #[test]
    fn sync_index_keys_cloned() {
        let mut idx = SyncIndex::default();
        idx.set("a".into(), IndexEntry { mtime_ns: 0, size: 0, win_mtime_ns: None, win_size: None });
        idx.set("b".into(), IndexEntry { mtime_ns: 0, size: 0, win_mtime_ns: None, win_size: None });
        let keys = idx.keys_cloned();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains("a"));
        assert!(keys.contains("b"));
    }

    #[test]
    fn file_sync_status_label() {
        assert_eq!(FileSyncStatus::Synced.label(), "synced");
        assert_eq!(FileSyncStatus::NeedsSync.label(), "needs-sync");
        assert_eq!(FileSyncStatus::NewerOnWin.label(), "newer-on-win");
        assert_eq!(FileSyncStatus::MissingWin.label(), "missing-win");
        assert_eq!(FileSyncStatus::MissingWsl.label(), "missing-wsl");
        assert_eq!(FileSyncStatus::Skipped.label(), "skipped");
        assert_eq!(FileSyncStatus::Error.label(), "error");
    }

    #[test]
    fn copy_status_counts_all_fields() {
        let mut c = CopyStatusCounts::default();
        c.inc(&FileSyncStatus::Synced);
        c.inc(&FileSyncStatus::NeedsSync);
        c.inc(&FileSyncStatus::NewerOnWin);
        c.inc(&FileSyncStatus::MissingWin);
        c.inc(&FileSyncStatus::MissingWsl);
        c.inc(&FileSyncStatus::Skipped);
        c.inc(&FileSyncStatus::Error);
        assert_eq!(c.synced, 1);
        assert_eq!(c.outdated_local, 1);
        assert_eq!(c.outdated_remote, 1);
        assert_eq!(c.missing_remote, 1);
        assert_eq!(c.missing_wsl, 1);
        assert_eq!(c.skipped, 1);
        assert_eq!(c.error, 1);
    }
}
