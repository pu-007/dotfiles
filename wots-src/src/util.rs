use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

use anyhow::Result;
use walkdir::WalkDir;

use crate::config::{EXCLUDE_PATTERNS, MAX_SYNC_SIZE_BYTES};

static QUICK_EXCLUDE_DIRS: LazyLock<std::collections::HashSet<&'static OsStr>> =
    LazyLock::new(|| {
        let mut s = std::collections::HashSet::new();
        let dirs: [&str; 7] = [
            ".git",
            "__pycache__",
            "node_modules",
            ".mypy_cache",
            ".ruff_cache",
            ".pixi",
            ".wots_index",
        ];
        for d in dirs {
            s.insert(OsStr::new(d));
        }
        s
    });

pub fn is_wsl() -> bool {
    Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists()
}

pub fn has_pwsh() -> bool {
    std::process::Command::new("pwsh.exe")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn has_stow() -> bool {
    std::process::Command::new("stow")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn has_robocopy() -> bool {
    Command::new("robocopy.exe")
        .arg("/?")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn count_and_size(dir: &Path) -> (usize, u64) {
    if !dir.is_dir() {
        return (0, 0);
    }
    WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| is_quick_exclude_dir(e.file_name()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && !is_excluded(e.path()))
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .fold((0usize, 0u64), |(count, sum), len| (count + 1, sum + len))
}

pub fn fmt_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn is_excluded(path: &Path) -> bool {
    for part in path.iter() {
        let s = part.to_string_lossy();
        for pat in &EXCLUDE_PATTERNS {
            if let Ok(matched) = glob_match(pat, &s)
                && matched {
                    return true;
                }
        }
    }
    false
}

pub fn is_quick_exclude_dir(name: &OsStr) -> bool {
    !QUICK_EXCLUDE_DIRS.contains(name)
}

fn glob_match(pattern: &str, name: &str) -> Result<bool> {
    if pattern == name {
        return Ok(true);
    }
    if !pattern.contains('*') && !pattern.contains('?') {
        return Ok(false);
    }
    let glob = glob::Pattern::new(pattern)?;
    Ok(glob.matches(name))
}

pub fn skip_size_limit(path: &Path) -> Result<bool> {
    Ok(*MAX_SYNC_SIZE_BYTES > 0 && fs::metadata(path)?.len() > *MAX_SYNC_SIZE_BYTES)
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(src)?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("wots_test_util_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(p) = path.parent() {
            let _ = fs::create_dir_all(p);
        }
        fs::write(path, content).unwrap();
    }

    // ------------------------------------------------------------------
    // fmt_size
    // ------------------------------------------------------------------

    #[test]
    fn fmt_bytes() {
        assert_eq!(fmt_size(0), "0 B");
        assert_eq!(fmt_size(512), "512 B");
        assert_eq!(fmt_size(1023), "1023 B");
    }

    #[test]
    fn fmt_kb() {
        let s = fmt_size(2048);
        assert!(s.contains("KB"));
    }

    #[test]
    fn fmt_mb() {
        let s = fmt_size(5 * 1024 * 1024);
        assert!(s.contains("MB"));
    }

    #[test]
    fn fmt_gb() {
        let s = fmt_size(3u64 * 1024 * 1024 * 1024);
        assert!(s.contains("GB"));
    }

    // ------------------------------------------------------------------
    // count_and_size
    // ------------------------------------------------------------------

    #[test]
    fn count_and_size_regular() {
        let dir = temp_dir();
        let pkg = dir.join("pkg.user");
        write_file(&pkg.join("a.txt"), "hello");
        write_file(&pkg.join("b.txt"), "world!!");
        let (count, size) = count_and_size(&pkg);
        assert_eq!(count, 2);
        assert!(size > 0);
    }

    #[test]
    fn count_and_size_excludes_dirs() {
        let dir = temp_dir();
        let pkg = dir.join("pkg2.user");
        write_file(&pkg.join("x.txt"), "x");
        write_file(
            &pkg.join("node_modules/ignored.js"),
            "console.log(1)",
        );
        let (count, _size) = count_and_size(&pkg);
        // node_modules should be excluded by is_quick_exclude_dir
        assert_eq!(count, 1);
    }

    #[test]
    fn count_and_size_nonexistent_is_zero() {
        assert_eq!(count_and_size(Path::new("/no/such/dir")), (0, 0));
    }

    // ------------------------------------------------------------------
    // is_excluded
    // ------------------------------------------------------------------

    #[test]
    fn excluded_git_dir() {
        assert!(is_excluded(Path::new("/some/pkg/.git/config")));
    }

    #[test]
    fn excluded_node_modules() {
        assert!(is_excluded(Path::new("/some/pkg/node_modules/x.js")));
    }

    #[test]
    fn excluded_pyc() {
        assert!(is_excluded(Path::new("/some/pkg/foo.pyc")));
    }

    #[test]
    fn not_excluded_normal() {
        assert!(!is_excluded(Path::new("/some/pkg/readme.md")));
        assert!(!is_excluded(Path::new("/some/pkg/src/main.rs")));
    }

    // ------------------------------------------------------------------
    // is_quick_exclude_dir
    // ------------------------------------------------------------------

    #[test]
    fn quick_exclude_known_dirs() {
        assert!(!is_quick_exclude_dir(OsStr::new(".git")));
        assert!(!is_quick_exclude_dir(OsStr::new("node_modules")));
        assert!(!is_quick_exclude_dir(OsStr::new("__pycache__")));
    }

    #[test]
    fn quick_exclude_normal_dir() {
        assert!(is_quick_exclude_dir(OsStr::new("src")));
        assert!(is_quick_exclude_dir(OsStr::new("config")));
    }

    // ------------------------------------------------------------------
    // copy_dir_all
    // ------------------------------------------------------------------

    #[test]
    fn copy_dir_all_preserves_structure() {
        let dir = temp_dir();
        let src = dir.join("src_dir");
        let dst = dir.join("dst_dir");
        write_file(&src.join("a.txt"), "a");
        write_file(&src.join("sub/b.txt"), "b");

        copy_dir_all(&src, &dst).unwrap();

        assert!(dst.join("a.txt").exists());
        assert!(dst.join("sub/b.txt").exists());
        assert_eq!(
            fs::read_to_string(dst.join("a.txt")).unwrap(),
            "a"
        );
        assert_eq!(
            fs::read_to_string(dst.join("sub/b.txt")).unwrap(),
            "b"
        );
    }
}
