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

pub static WIN_USERNAME: LazyLock<Option<String>> = LazyLock::new(|| {
    if let Ok(u) = env::var("WIN_USER") {
        return Some(u);
    }
    let mnt_users = PathBuf::from("/mnt/c/Users");
    if !mnt_users.exists() {
        return env::var("USER").ok();
    }
    let skip: &[&str] = &["Public", "Default", "Default User", "All Users", "desktop.ini"];
    if let Ok(entries) = std::fs::read_dir(&mnt_users) {
        let mut dirs: Vec<String> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.is_ascii() && !skip.contains(&n.as_str()) && !n.starts_with('.'))
            .collect();
        dirs.sort();
        return dirs.into_iter().next();
    }
    env::var("USER").ok()
});

pub static USER_TARGET: LazyLock<PathBuf> = LazyLock::new(|| HOME.clone());

pub static CONFIG_TARGET: LazyLock<PathBuf> = LazyLock::new(|| USER_TARGET.join(".config"));

pub static LOCAL_TARGET: LazyLock<PathBuf> = LazyLock::new(|| USER_TARGET.join(".local"));

pub static ROOT_TARGET: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("/"));

pub static WINUSER_TARGET: LazyLock<PathBuf> = LazyLock::new(|| {
    let user = WIN_USERNAME.as_deref().unwrap_or("user");
    PathBuf::from(format!("C:/Users/{}", user))
});

pub static WINCONFIG_TARGET: LazyLock<PathBuf> =
    LazyLock::new(|| WINUSER_TARGET.join(".config"));

pub static WINLOCAL_TARGET: LazyLock<PathBuf> =
    LazyLock::new(|| WINUSER_TARGET.join("AppData").join("Local"));

pub static WINROAMING_TARGET: LazyLock<PathBuf> =
    LazyLock::new(|| WINUSER_TARGET.join("AppData").join("Roaming"));

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
