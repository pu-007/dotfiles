use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;

pub static HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
});

pub static DOTFILES_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    env::var("DOTFILES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| HOME.join("dotfiles"))
});

pub static WSL_DISTRO_NAME: LazyLock<String> = LazyLock::new(|| {
    env::var("WSL_DISTRO").unwrap_or_else(|_| "archlinux".to_string())
});

pub static WIN_USERNAME: LazyLock<Option<String>> =
    LazyLock::new(|| env::var("WIN_USER").ok());

pub static USER_TARGET: LazyLock<PathBuf> = LazyLock::new(|| HOME.clone());

pub static CONFIG_TARGET: LazyLock<PathBuf> = LazyLock::new(|| USER_TARGET.join(".config"));

pub static ROOT_TARGET: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("/"));

/// Windows user profile, expressed as a `C:/Users/<name>` path. This is a
/// logical Windows path; use [`MNT_C`] to translate it into the WSL mount.
pub static WINUSER_TARGET: LazyLock<PathBuf> = LazyLock::new(|| {
    let user = WIN_USERNAME.as_deref().unwrap_or("user");
    PathBuf::from(format!("C:/Users/{}", user))
});

/// Windows drive root (`C:/`) for `.winroot` packages.
pub static WINROOT_TARGET: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("C:/"));

pub static MNT_C: LazyLock<PathBuf> = LazyLock::new(|| {
    env::var("WSL_MNT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/mnt/c"))
});

pub static SYNC_MAX_CONCURRENT: LazyLock<usize> = LazyLock::new(|| {
    env::var("WOTS_CONCURRENT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8)
});

pub static MAX_SYNC_SIZE_BYTES: LazyLock<u64> = LazyLock::new(|| {
    env::var("WOTS_MAX_SIZE_MB")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(50)
        * 1024
        * 1024
});

pub const EXCLUDE_PATTERNS: [&str; 10] = [
    ".git",
    ".pixi",
    "__pycache__",
    "node_modules",
    ".mypy_cache",
    ".ruff_cache",
    "*.pyc",
    ".DS_Store",
    "Thumbs.db",
    ".wots_index.json",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_is_set() {
        let h = HOME.to_string_lossy();
        assert!(!h.is_empty());
        assert!(h != "/", "HOME should be a real path, not /");
    }

    #[test]
    fn dotfiles_dir_defaults_to_home_dotfiles() {
        let expected = std::env::var("HOME")
            .map(|h| format!("{}/dotfiles", h))
            .unwrap_or_else(|_| "/dotfiles".into());
        assert_eq!(DOTFILES_DIR.to_string_lossy(), expected.as_str());
    }

    #[test]
    fn wsl_distro_has_default() {
        // defaults to "archlinux" when WSL_DISTRO is unset, or reads env
        let val = WSL_DISTRO_NAME.to_string();
        assert!(!val.is_empty());
    }

    #[test]
    fn mnt_c_default_is_mnt_c() {
        if std::env::var("WSL_MNT").is_err() {
            assert_eq!(MNT_C.to_string_lossy(), "/mnt/c");
        }
    }

    #[test]
    fn sync_max_concurrent_default() {
        if std::env::var("WOTS_CONCURRENT").is_err() {
            assert_eq!(*SYNC_MAX_CONCURRENT, 8);
        }
    }

    #[test]
    fn max_sync_size_default() {
        if std::env::var("WOTS_MAX_SIZE_MB").is_err() {
            assert_eq!(*MAX_SYNC_SIZE_BYTES, 50 * 1024 * 1024);
        }
    }

    #[test]
    fn exclude_patterns_has_expected_entries() {
        assert_eq!(EXCLUDE_PATTERNS.len(), 10);
        let patterns: Vec<&str> = EXCLUDE_PATTERNS.to_vec();
        for p in [
            ".git",
            ".pixi",
            "node_modules",
            "__pycache__",
            "*.pyc",
            ".DS_Store",
            "Thumbs.db",
            ".wots_index.json",
            ".mypy_cache",
            ".ruff_cache",
        ] {
            assert!(patterns.contains(&p), "missing pattern {p}");
        }
    }

    #[test]
    fn target_paths_are_absolute() {
        assert!(USER_TARGET.to_string_lossy().starts_with("/"));
        assert!(CONFIG_TARGET.to_string_lossy().starts_with("/"));
        assert_eq!(ROOT_TARGET.to_string_lossy(), "/");
    }

    #[test]
    fn win_targets_use_windows_paths() {
        assert!(WINUSER_TARGET.to_string_lossy().starts_with("C:/Users/"));
        assert_eq!(WINROOT_TARGET.to_string_lossy(), "C:/");
        // WinUser target must live under the WinRoot target.
        assert!(WINUSER_TARGET.starts_with(&*WINROOT_TARGET));
    }
}
