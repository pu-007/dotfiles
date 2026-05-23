use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::LazyLock;

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::config::{EXCLUDE_PATTERNS, MAX_SYNC_SIZE_BYTES, WSL_DISTRO_NAME};

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

pub fn count_files(dir: &Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| is_quick_exclude_dir(e.file_name()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && !is_excluded(e.path()))
        .count()
}

pub fn dir_size(dir: &Path) -> u64 {
    if !dir.is_dir() {
        return 0;
    }
    WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| is_quick_exclude_dir(e.file_name()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && !is_excluded(e.path()))
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
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

pub fn win_path_str(win_path: &Path, mnt_c: &Path) -> String {
    let relative = win_path.strip_prefix(mnt_c).unwrap_or(win_path);
    format!(
        "C:\\{}",
        relative.to_string_lossy().replace('/', "\\")
    )
}

pub fn win_unc_path(wsl_src: &Path) -> String {
    let distro = &*WSL_DISTRO_NAME;
    format!(
        "\\\\wsl$\\{}\\{}",
        distro,
        wsl_src.to_string_lossy().replace('/', "\\")
    )
}

pub fn run_command(cmd: &[&str], cwd: Option<&Path>, dry_run: bool) -> Result<Output> {
    if dry_run {
        return Ok(Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });
    }

    let (prog, args) = cmd.split_first().context("empty command")?;
    let mut c = Command::new(prog);
    c.args(args);
    if let Some(dir) = cwd {
        c.current_dir(dir);
    }
    let output = c.output().context("failed to execute command")?;
    Ok(output)
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
