use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::config::DOTFILES_DIR;
use crate::discover::{build_win_path, list_syncable_files};
use crate::types::PkgType;
use crate::util::skip_size_limit;

pub use crate::index::{IndexEntry, SyncIndex};

// ===========================================================================
// Per-file sync status
// ===========================================================================

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
    ContentChanged,
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
            FileSyncStatus::ContentChanged => "content-changed",
        }
    }
}

/// Per-file status entry returned by the detailed check.
#[derive(Debug, Clone)]
pub struct FileStatusEntry {
    pub relative_path: PathBuf,
    pub status: FileSyncStatus,
}

// ===========================================================================
// Copy-status accumulator
// ===========================================================================

#[derive(Debug, Default, Clone)]
pub struct CopyStatusCounts {
    pub synced: usize,
    pub outdated_local: usize,
    pub outdated_remote: usize,
    pub missing_remote: usize,
    pub missing_wsl: usize,
    pub skipped: usize,
    pub error: usize,
    pub content_mat_mismatch: usize,
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
            FileSyncStatus::ContentChanged => self.content_mat_mismatch += 1,
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
    if counts.content_mat_mismatch > 0 {
        parts.push(format!("{} content-mismatch", counts.content_mat_mismatch));
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

// ===========================================================================
// Symlink helpers
// ===========================================================================

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

// ===========================================================================
// Stow status
// ===========================================================================

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

// ===========================================================================
// Core file-status check (shared between copy-status and diff)
// ===========================================================================

/// Result of `file_sync_status`: carries metadata alongside the status so
/// the caller can update the index without re-reading the file system.
struct FileSyncResult {
    key: String,
    status: FileSyncStatus,
    mtime_ns: u64,
    size: u64,
    win_mtime_ns: Option<u64>,
    win_size: Option<u64>,
    blake3_wsl: Option<String>,
    blake3_win: Option<String>,
    /// True when the index entry is already up-to-date (fast-path Synced).
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
            blake3_wsl: None,
            blake3_win: None,
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

    let win_path = build_win_path(wsl_path, pkg, pt);

    // Fast-path: index says both sides unchanged AND the file was synced.
    if let Some(entry) = index.get(&key)
        && entry.synced
        && entry.mtime_ns == mtime_ns
        && entry.size == size
        && entry.win_mtime_ns.is_some()
        && entry.win_size.is_some()
    {
        match win_path.metadata() {
            Ok(wn) => {
                let actual_wn_mtime_ns = wn
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos() as u64);
                if actual_wn_mtime_ns == entry.win_mtime_ns
                    && Some(wn.len()) == entry.win_size
                {
                    return FileSyncResult {
                        key,
                        status: FileSyncStatus::Synced,
                        mtime_ns,
                        size,
                        win_mtime_ns: entry.win_mtime_ns,
                        win_size: entry.win_size,
                        blake3_wsl: entry.blake3_wsl.clone(),
                        blake3_win: entry.blake3_win.clone(),
                        index_ok: true,
                    };
                }
            }
            Err(_) => {
                return FileSyncResult {
                    key,
                    status: FileSyncStatus::MissingWin,
                    mtime_ns,
                    size,
                    win_mtime_ns: None,
                    win_size: None,
                    blake3_wsl: entry.blake3_wsl.clone(),
                    blake3_win: None,
                    index_ok: false,
                };
            }
        }
    }

    // Slow-path: read Windows metadata.
    if win_path.symlink_metadata().is_err() {
        return FileSyncResult {
            key,
            status: FileSyncStatus::MissingWin,
            mtime_ns,
            size,
            win_mtime_ns: None,
            win_size: None,
            blake3_wsl: None,
            blake3_win: None,
            index_ok: false,
        };
    }

    let wn = match win_path.metadata() {
        Ok(m) => m,
        Err(_) => return FileSyncResult::new(key, FileSyncStatus::Error),
    };

    let ws_mtime_ns = mtime_ns;
    let wn_mtime_ns = wn
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as u64);
    let wn_size = wn.len();

    // Compare WSL and Windows metadata with nanosecond-level precision.
    // 2 ms tolerance to handle cross-filesystem timestamp rounding.
    let (wn_mtime_ns_val, status, blake3_wsl, blake3_win) = match (ws_mtime_ns, wn_mtime_ns) {
        (wsm, Some(wnm)) => {
            let mtime_diff = wsm.abs_diff(wnm);
            if mtime_diff < 2_000_000 && size == wn_size {
                let (w_hash, n_hash, content_status) =
                    hash_compare(wsl_path, &win_path, index.get(&key));
                match content_status {
                    Some(FileSyncStatus::ContentChanged) => {
                        (Some(wnm), FileSyncStatus::ContentChanged, w_hash, n_hash)
                    }
                    Some(FileSyncStatus::Error) => {
                        (Some(wnm), FileSyncStatus::Error, w_hash, n_hash)
                    }
                    Some(other) => {
                        (Some(wnm), other, w_hash, n_hash)
                    }
                    None => {
                        (Some(wnm), FileSyncStatus::Synced, w_hash, n_hash)
                    }
                }
            } else if wsm > wnm {
                (Some(wnm), FileSyncStatus::NeedsSync, None, None)
            } else {
                (Some(wnm), FileSyncStatus::NewerOnWin, None, None)
            }
        }
        _ => (wn_mtime_ns, FileSyncStatus::Error, None, None),
    };

    FileSyncResult {
        key,
        status: status.clone(),
        mtime_ns,
        size,
        win_mtime_ns: wn_mtime_ns_val,
        win_size: Some(wn_size),
        blake3_wsl,
        blake3_win,
        index_ok: false,
    }
}

/// Compare content hashes of WSL and Windows copies.
fn hash_compare(
    wsl_path: &Path,
    win_path: &Path,
    idx_entry: Option<&IndexEntry>,
) -> (Option<String>, Option<String>, Option<FileSyncStatus>) {
    let h_wsl = hash_file(wsl_path);
    let h_win = hash_file(win_path);

    match (&h_wsl, &h_win) {
        (Some(wsl_hash), Some(win_hash)) => {
            if wsl_hash == win_hash {
                (h_wsl, h_win, None)
            } else {
                (h_wsl, h_win, Some(FileSyncStatus::ContentChanged))
            }
        }
        (Some(_), None) | (None, Some(_)) => {
            (h_wsl, h_win, Some(FileSyncStatus::Error))
        }
        (None, None) => {
            let w = idx_entry.and_then(|e| e.blake3_wsl.clone());
            let n = idx_entry.and_then(|e| e.blake3_win.clone());
            (w, n, None)
        }
    }
}

/// Public test wrapper for hash_file.  Only used by integration tests.
#[doc(hidden)]
pub fn hash_file_test(path: &Path) -> Option<String> {
    hash_file(path)
}

fn hash_file(path: &Path) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    Some(blake3::hash(&data).to_hex().to_string())
}

// ===========================================================================
// Helpers
// ===========================================================================

fn pkg_key_prefix(pkg: &Path) -> String {
    let n = pkg
        .file_name()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_string_lossy();
    format!("{}/", n)
}

fn index_key(pkg: &Path, rel: &Path) -> String {
    let pkg_name = pkg
        .file_name()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_string_lossy();
    format!("{}/{}", pkg_name, rel.display())
}

// ===========================================================================
// Reverse check: detect index entries whose WSL file no longer exists.
// ===========================================================================

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

        let win_path = build_win_path(&wsl_path, pkg, pt);
        if win_path.symlink_metadata().is_ok() {
            missing.push((key.clone(), FileSyncStatus::MissingWsl));
        } else {
            remove_keys.push(key.clone());
        }
    }

    (missing, remove_keys)
}

/// Public test wrapper for `detect_missing_wsl`.
#[doc(hidden)]
pub fn detect_missing_wsl_test(
    pkg: &Path,
    pt: &PkgType,
    index: &SyncIndex,
    seen_keys: &HashSet<String>,
) -> (Vec<(String, FileSyncStatus)>, Vec<String>) {
    detect_missing_wsl(pkg, pt, index, seen_keys)
}

// ===========================================================================
// Direct Windows-side scan — does NOT depend on the index.
// ===========================================================================

#[allow(dead_code)]
fn scan_windows_for_orphans(
    pkg: &Path,
    pt: &PkgType,
) -> Vec<FileStatusEntry> {
    let mut orphans: Vec<FileStatusEntry> = Vec::new();

    let win_base = match pt {
        PkgType::WinUser => {
            let user = crate::config::WIN_USERNAME.as_deref().unwrap_or("user");
            crate::config::MNT_C.join("Users").join(user)
        }
        PkgType::WinConfig => {
            let user = crate::config::WIN_USERNAME.as_deref().unwrap_or("user");
            crate::config::MNT_C.join("Users").join(user).join(".config")
        }
        PkgType::WinLocal => {
            let user = crate::config::WIN_USERNAME.as_deref().unwrap_or("user");
            crate::config::MNT_C.join("Users").join(user).join("AppData").join("Local")
        }
        PkgType::WinRoaming => {
            let user = crate::config::WIN_USERNAME.as_deref().unwrap_or("user");
            crate::config::MNT_C.join("Users").join(user).join("AppData").join("Roaming")
        }
        _ => return orphans,
    };

    if !win_base.is_dir() {
        return orphans;
    }

    let wsl_set: HashSet<PathBuf> = list_syncable_files(pkg)
        .into_iter()
        .filter_map(|f| f.strip_prefix(pkg).ok().map(Path::to_path_buf))
        .collect();

    let max_depth: usize = 8;

    for entry in WalkDir::new(&win_base)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| crate::util::is_quick_exclude_dir(e.file_name()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && !crate::util::is_excluded(e.path()))
    {
        let win_file = entry.path();
        if let Ok(rel) = win_file.strip_prefix(&win_base)
            && !wsl_set.contains(rel)
        {
            orphans.push(FileStatusEntry {
                relative_path: rel.to_path_buf(),
                status: FileSyncStatus::MissingWsl,
            });
        }
    }

    orphans
}

// ===========================================================================
// Full copy-status check (counts + per-file details)
// ===========================================================================

pub fn check_copy_status_detailed(
    pkg: &Path,
    pt: &PkgType,
) -> (CopyStatusCounts, Vec<FileStatusEntry>, Option<std::io::Error>) {
    check_copy_status_detailed_at(pkg, pt, &DOTFILES_DIR)
}

#[doc(hidden)]
pub fn check_copy_status_detailed_at(
    pkg: &Path,
    pt: &PkgType,
    index_base: &Path,
) -> (CopyStatusCounts, Vec<FileStatusEntry>, Option<std::io::Error>) {
    let mut counts = CopyStatusCounts::default();
    let mut entries: Vec<FileStatusEntry> = Vec::new();

    if !pkg.is_dir() {
        return (counts, entries, None);
    }

    let mut index = SyncIndex::load_from(index_base);
    let files = list_syncable_files(pkg);
    let mut seen_keys: HashSet<String> = HashSet::new();

    for f in &files {
        let result = file_sync_status(f, pkg, pt, &index);
        seen_keys.insert(result.key.clone());
        counts.inc(&result.status);

        if let Ok(rel) = f.strip_prefix(pkg) {
            entries.push(FileStatusEntry {
                relative_path: rel.to_path_buf(),
                status: result.status.clone(),
            });
        }

        if !result.index_ok {
            index.set(
                result.key,
                IndexEntry {
                    mtime_ns: result.mtime_ns,
                    size: result.size,
                    win_mtime_ns: result.win_mtime_ns,
                    win_size: result.win_size,
                    blake3_wsl: result.blake3_wsl,
                    blake3_win: result.blake3_win,
                    synced: result.status == FileSyncStatus::Synced,
                },
            );
        }
    }

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

    for key in &remove_keys {
        index.entries.remove(key);
    }

    let save_err = index.save_to(index_base).err();
    if let Some(ref e) = save_err {
        eprintln!("  wots: warning — failed to save sync index: {e}");
    }
    (counts, entries, save_err)
}

pub fn check_copy_status(pkg: &Path, pt: &PkgType) -> CopyStatusCounts {
    let (counts, _, save_err) = check_copy_status_detailed(pkg, pt);
    if let Some(e) = save_err {
        eprintln!("  wots: warning — failed to save sync index: {e}");
    }
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
        total.missing_wsl += c.missing_wsl;
        total.skipped += c.skipped;
        total.error += c.error;
        total.content_mat_mismatch += c.content_mat_mismatch;
    }
    total
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "wots_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
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
        let pkg = dir.join("testapp.winuser");
        fs::create_dir_all(&pkg).unwrap();
        let file = pkg.join("test.txt");
        write_file(&file, "content");

        let index = SyncIndex::default();
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
        assert_eq!(c.content_mat_mismatch, 0);
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
    fn file_sync_status_label() {
        assert_eq!(FileSyncStatus::Synced.label(), "synced");
        assert_eq!(FileSyncStatus::NeedsSync.label(), "needs-sync");
        assert_eq!(FileSyncStatus::NewerOnWin.label(), "newer-on-win");
        assert_eq!(FileSyncStatus::MissingWin.label(), "missing-win");
        assert_eq!(FileSyncStatus::MissingWsl.label(), "missing-wsl");
        assert_eq!(FileSyncStatus::Skipped.label(), "skipped");
        assert_eq!(FileSyncStatus::Error.label(), "error");
        assert_eq!(FileSyncStatus::ContentChanged.label(), "content-changed");
    }

    #[test]
    fn content_changed_status_inc() {
        let mut c = CopyStatusCounts::default();
        c.inc(&FileSyncStatus::ContentChanged);
        assert_eq!(c.content_mat_mismatch, 1);
    }

    #[test]
    fn hash_file_same_content_same_hash() {
        let dir = temp_dir();
        let fa = dir.join("a.txt");
        let fb = dir.join("b.txt");
        write_file(&fa, "identical");
        write_file(&fb, "identical");
        assert_eq!(hash_file(&fa), hash_file(&fb));
    }

    #[test]
    fn hash_file_different_content_different_hash() {
        let dir = temp_dir();
        let fa = dir.join("a.txt");
        let fb = dir.join("b.txt");
        write_file(&fa, "one");
        write_file(&fb, "two");
        assert_ne!(hash_file(&fa), hash_file(&fb));
    }

    #[test]
    fn hash_file_nonexistent_is_none() {
        assert!(hash_file(Path::new("/nonexistent/hash_test")).is_none());
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
        c.inc(&FileSyncStatus::ContentChanged);
        assert_eq!(c.synced, 1);
        assert_eq!(c.outdated_local, 1);
        assert_eq!(c.outdated_remote, 1);
        assert_eq!(c.missing_remote, 1);
        assert_eq!(c.missing_wsl, 1);
        assert_eq!(c.skipped, 1);
        assert_eq!(c.error, 1);
        assert_eq!(c.content_mat_mismatch, 1);
    }
}
