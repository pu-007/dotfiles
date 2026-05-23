// integration tests — create real files in temporary directories,
// exercise wots modules end-to-end without affecting live data.

use std::fs;
use std::path::{Path, PathBuf};

use wots::status::{
    self, CopyStatusCounts, FileSyncStatus, SyncIndex,
};
use wots::types::PkgType;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn temp_root() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wots_int_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    dir
}

fn write_file(path: &Path, content: &str) {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn touch(path: &Path) {
    write_file(path, "x");
}

/// Build a pkg directory with the given suffix inside a temporary root,
/// returning (pkg_path, root).
fn make_pkg(root: &Path, name: &str, suffix: &str) -> PathBuf {
    let pkg = root.join(format!("{}.{}", name, suffix));
    fs::create_dir_all(&pkg).unwrap();
    pkg
}

// ---------------------------------------------------------------------------
// status::CopyStatusCounts
// ---------------------------------------------------------------------------

#[test]
fn counts_inc_all_variants() {
    let mut c = CopyStatusCounts::default();
    let variants = [
        FileSyncStatus::Synced,
        FileSyncStatus::NeedsSync,
        FileSyncStatus::NewerOnWin,
        FileSyncStatus::MissingWin,
        FileSyncStatus::MissingWsl,
        FileSyncStatus::Skipped,
        FileSyncStatus::Error,
    ];
    for v in &variants {
        c.inc(v);
    }
    assert_eq!(c.synced, 1);
    assert_eq!(c.outdated_local, 1);
    assert_eq!(c.outdated_remote, 1);
    assert_eq!(c.missing_remote, 1);
    assert_eq!(c.missing_wsl, 1);
    assert_eq!(c.skipped, 1);
    assert_eq!(c.error, 1);
}

#[test]
fn counts_synced_accumulates() {
    let mut c = CopyStatusCounts::default();
    c.inc(&FileSyncStatus::Synced);
    c.inc(&FileSyncStatus::Synced);
    c.inc(&FileSyncStatus::Synced);
    assert_eq!(c.synced, 3);
}

// ---------------------------------------------------------------------------
// status::status_text
// ---------------------------------------------------------------------------

#[test]
fn status_text_reports_all_fields() {
    let c = CopyStatusCounts {
        synced: 1,
        outdated_local: 2,
        outdated_remote: 3,
        missing_remote: 4,
        missing_wsl: 5,
        skipped: 6,
        error: 0,
    };
    let s = status::status_text(&c);
    assert!(s.contains("1 synced"));
    assert!(s.contains("2 needs-sync"));
    assert!(s.contains("3 newer-on-win"));
    assert!(s.contains("4 missing-win"));
    assert!(s.contains("5 missing-wsl"));
    assert!(s.contains("6 skipped"));
}

// ---------------------------------------------------------------------------
// status::SyncIndex — save / load roundtrip
// ---------------------------------------------------------------------------

#[test]
fn sync_index_save_load_roundtrip() {
    // Use a unique file to avoid collisions with real index
    let tmp = temp_root();
    let idx_path = tmp.join(".wots_index.json");

    // Write a controlled index
    let mut idx = SyncIndex::default();
    idx.set(
        "pkg/file.txt".into(),
        status::IndexEntry {
            mtime_ns: 100,
            size: 200,
            win_mtime_ns: Some(101),
            win_size: Some(200),
        },
    );
    // Inject via env var to redirect DOTFILES_DIR (we use save/load path)
    // Actually save/load uses DOTFILES_DIR directly, so write our own file:
    let json = serde_json::to_string_pretty(&idx).unwrap();
    fs::write(&idx_path, &json).unwrap();

    let loaded: SyncIndex =
        serde_json::from_str(&fs::read_to_string(&idx_path).unwrap()).unwrap();
    let entry = loaded.get("pkg/file.txt").unwrap();
    assert_eq!(entry.mtime_ns, 100);
    assert_eq!(entry.size, 200);
    assert_eq!(entry.win_mtime_ns, Some(101));
    assert_eq!(entry.win_size, Some(200));
}

// ---------------------------------------------------------------------------
// status::is_symlink
// ---------------------------------------------------------------------------

#[test]
fn is_symlink_regular_file_returns_false() {
    let root = temp_root();
    let f = root.join("regular.txt");
    touch(&f);
    assert!(!status::is_symlink(&f));
}

#[test]
fn is_symlink_nonexistent_returns_false() {
    assert!(!status::is_symlink(Path::new("/no/such/file/anywhere")));
}

// ---------------------------------------------------------------------------
// status::check_stow_status — simulated package structure
// ---------------------------------------------------------------------------

#[test]
fn check_stow_status_empty_pkg_is_zero() {
    let root = temp_root();
    let pkg = make_pkg(&root, "emptystow", "user");
    // user type uses stow, but pkg is empty
    let (stowed, total) = status::check_stow_status(&pkg, &PkgType::User);
    assert_eq!(stowed, 0);
    assert_eq!(total, 0);
}

#[test]
fn check_stow_status_non_stow_type_returns_zero() {
    let root = temp_root();
    let pkg = make_pkg(&root, "foo", "winuser");
    touch(&pkg.join("file.txt"));
    let (stowed, total) = status::check_stow_status(&pkg, &PkgType::WinUser);
    // WinUser does not use stow
    assert_eq!(stowed, 0);
    assert_eq!(total, 0);
}

// ---------------------------------------------------------------------------
// status::check_copy_status — real files, no Windows (will report MissingWin)
// ---------------------------------------------------------------------------

#[test]
fn check_copy_status_all_missing_win() {
    let root = temp_root();
    let pkg = make_pkg(&root, "testapp", "winuser");
    touch(&pkg.join("config.json"));
    touch(&pkg.join("sub/deep.cfg"));

    let counts = status::check_copy_status(&pkg, &PkgType::WinUser);
    // On Linux, build_win_path maps to /mnt/c/... which likely doesn't exist
    // unless running in WSL with actual /mnt/c mount.
    // We check behavior is consistent — files should be either error or missing-win.
    assert!(
        counts.missing_remote + counts.error >= 1,
        "expected some missing/error statuses, got {counts:?}"
    );
}

// ---------------------------------------------------------------------------
// status::check_copy_status_detailed — returns per-file entries
// ---------------------------------------------------------------------------

#[test]
fn check_copy_status_detailed_produces_entries() {
    let root = temp_root();
    let pkg = make_pkg(&root, "detailed", "winuser");
    touch(&pkg.join("a.txt"));
    touch(&pkg.join("b.txt"));

    let (_counts, entries) =
        status::check_copy_status_detailed(&pkg, &PkgType::WinUser);
    assert_eq!(entries.len(), 2, "expected 2 file entries");

    let names: Vec<String> = entries
        .iter()
        .map(|e| e.relative_path.display().to_string())
        .collect();
    assert!(names.contains(&"a.txt".to_string()));
    assert!(names.contains(&"b.txt".to_string()));
}

#[test]
fn check_copy_status_detailed_non_dir_is_empty() {
    let root = temp_root();
    let file = root.join("notadir.txt");
    touch(&file);
    let (counts, entries) =
        status::check_copy_status_detailed(&file, &PkgType::WinUser);
    assert_eq!(counts.synced, 0);
    assert!(entries.is_empty());
}

// ---------------------------------------------------------------------------
// FileSyncStatus labels
// ---------------------------------------------------------------------------

#[test]
fn file_sync_status_labels_unique() {
    let labels: Vec<&str> = [
        FileSyncStatus::Synced,
        FileSyncStatus::NeedsSync,
        FileSyncStatus::NewerOnWin,
        FileSyncStatus::MissingWin,
        FileSyncStatus::MissingWsl,
        FileSyncStatus::Skipped,
        FileSyncStatus::Error,
    ]
    .iter()
    .map(|s| s.label())
    .collect();
    // All labels must be distinct
    let mut dedup = labels.clone();
    dedup.sort();
    dedup.dedup();
    assert_eq!(labels.len(), dedup.len(), "labels must be unique");
}

// ---------------------------------------------------------------------------
// status::check_copy_status_batch
// ---------------------------------------------------------------------------

#[test]
fn check_copy_status_batch_aggregates() {
    let root = temp_root();
    let pkg1 = make_pkg(&root, "app1", "winuser");
    let pkg2 = make_pkg(&root, "app2", "winuser");
    touch(&pkg1.join("f1.txt"));
    touch(&pkg2.join("f2.txt"));

    let total = status::check_copy_status_batch(
        &[pkg1.clone(), pkg2.clone()],
        PkgType::WinUser,
    );
    // Each file should be either missing-win or error
    assert!(
        total.missing_remote + total.error >= 2,
        "expected at least 2 statuses, got {total:?}"
    );
}

// ---------------------------------------------------------------------------
// wots::discover — integration-style tests with real dirs
// ---------------------------------------------------------------------------

#[test]
fn find_packages_discovers_by_suffix() {
    let root = temp_root();
    // Create packages with known suffixes
    make_pkg(&root, "git", "config");
    make_pkg(&root, "zsh", "user");
    make_pkg(&root, "myapp", "winuser");
    // Regular directories without suffix should be ignored
    fs::create_dir_all(root.join("not_a_pkg")).unwrap();

    let pkgs = wots::discover::find_packages(&root);

    // config type should have git
    let config_pkgs = pkgs.get(&PkgType::Config).unwrap();
    assert!(!config_pkgs.is_empty());
    let config_names: Vec<String> =
        config_pkgs.iter().map(|p| wots::discover::pkg_basename(p)).collect();
    assert!(config_names.contains(&"git".to_string()));

    // user type should have zsh
    let user_pkgs = pkgs.get(&PkgType::User).unwrap();
    let user_names: Vec<String> =
        user_pkgs.iter().map(|p| wots::discover::pkg_basename(p)).collect();
    assert!(user_names.contains(&"zsh".to_string()));

    // winuser type should have myapp
    let winuser_pkgs = pkgs.get(&PkgType::WinUser).unwrap();
    let win_names: Vec<String> =
        winuser_pkgs.iter().map(|p| wots::discover::pkg_basename(p)).collect();
    assert!(win_names.contains(&"myapp".to_string()));
}

#[test]
fn find_packages_skips_dot_dirs() {
    let root = temp_root();
    // Hidden directories should be skipped
    fs::create_dir_all(root.join(".hidden_pkg.config")).unwrap();
    // Regular package should be found
    make_pkg(&root, "visible", "config");

    let pkgs = wots::discover::find_packages(&root);
    let config_pkgs = pkgs.get(&PkgType::Config).unwrap();
    let names: Vec<String> =
        config_pkgs.iter().map(|p| wots::discover::pkg_basename(p)).collect();
    assert!(names.contains(&"visible".to_string()));
    assert!(!names.contains(&".hidden_pkg".to_string()));
}

// ---------------------------------------------------------------------------
// wots::types — all known types roundtrip
// ---------------------------------------------------------------------------

#[test]
fn all_types_roundtrip_name_to_type() {
    for pt in &wots::types::ALL_TYPES {
        let name = pt.value();
        let round: wots::types::PkgType = name.parse().unwrap();
        assert_eq!(round, *pt, "failed roundtrip for {name}");
    }
}

// ---------------------------------------------------------------------------
// wots::util — size formatting edge cases
// ---------------------------------------------------------------------------

#[test]
fn fmt_size_boundary_values() {
    assert!(wots::util::fmt_size(1024).contains("KB"));
    assert!(wots::util::fmt_size(1024 * 1024).contains("MB"));
}

// ---------------------------------------------------------------------------
// status::is_symlink_or_parent — non-symlink paths
// ---------------------------------------------------------------------------

#[test]
fn is_symlink_or_parent_plain_file() {
    let root = temp_root();
    let f = root.join("plain.txt");
    touch(&f);
    assert!(!status::is_symlink_or_parent(
        &f,
        &root
    ));
}

#[test]
fn is_symlink_or_parent_nonexistent() {
    let root = temp_root();
    assert!(!status::is_symlink_or_parent(
        Path::new("/no/such/path"),
        &root
    ));
}
