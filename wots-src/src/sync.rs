use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use rayon::prelude::*;

use crate::cli::SyncArgs;
use crate::config::{DOTFILES_DIR, MNT_C, SYNC_MAX_CONCURRENT, WSL_DISTRO_NAME, WIN_USERNAME};
use crate::discover::{build_win_path, find_packages, list_syncable_files, pkg_basename};
use crate::display;
use crate::status;
use crate::types::{parse_app_arg, type_label, PkgType, SYNCABLE_TYPES};
use crate::util::{has_pwsh, has_robocopy, has_stow, is_excluded, is_wsl};

pub fn run(args: SyncArgs) -> Result<()> {
    let packages = find_packages(&DOTFILES_DIR);

    let (detected_type, effective_app): (Option<PkgType>, Option<String>) =
        match &args.app {
            Some(raw) => {
                let (dt, name) = parse_app_arg(raw);
                (dt, Some(name))
            }
            None => (None, None),
        };

    let types_to_sync: Vec<PkgType> = if let Some(pt) = &args.pkg_type {
        vec![*pt]
    } else if let Some(pt) = detected_type {
        vec![pt]
    } else {
        SYNCABLE_TYPES.to_vec()
    };

    let root_pkgs = packages.get(&PkgType::Root).cloned().unwrap_or_default();
    let has_actual_root = types_to_sync.contains(&PkgType::Root) && !root_pkgs.is_empty();
    if has_actual_root && !args.yes && !args.bypass && !args.dry_run {
        display::info("\n⚠ Root sync requires sudo and will modify system files.");
        if !display::prompt::confirm("  Continue?", false) {
            display::info("Cancelled.");
            return Ok(());
        }
    }

    for pt in &types_to_sync {
        let pkgs = packages.get(pt).cloned().unwrap_or_default();
        let pkgs: Vec<PathBuf> = if let Some(ref app) = effective_app {
            pkgs
                .into_iter()
                .filter(|p| pkg_basename(p) == *app)
                .collect()
        } else {
            pkgs
        };

        if pkgs.is_empty() {
            continue;
        }

        display::rule(&format!("Syncing {} packages", pt.value()));

        if pt.uses_stow() {
            sync_stow_batch(&pkgs, pt, args.dry_run, args.quiet)?;
        } else if pt.uses_copy_sync() {
            sync_copy_batch(&pkgs, pt, args.dry_run, args.quiet)?;
        } else {
            if !args.quiet {
                display::info("  (meta packages are manually managed)");
            }
        }
    }

    display::info("");
    display::success("Sync complete.");
    Ok(())
}

fn sync_stow_batch(pkgs: &[PathBuf], pt: &PkgType, dry_run: bool, quiet: bool) -> Result<()> {
    if !has_stow() {
        display::error("GNU Stow not installed — skipping.");
        return Ok(());
    }

    for pkg in pkgs {
        if !quiet {
            display::info(&format!(
                "  {}  →  {}",
                pkg.file_name()
                    .unwrap_or(std::ffi::OsStr::new(""))
                    .to_string_lossy(),
                type_label(*pt),
            ));
        }
        do_stow(pkg, pt, dry_run)?;
    }

    Ok(())
}

fn sync_copy_batch(pkgs: &[PathBuf], pt: &PkgType, dry_run: bool, quiet: bool) -> Result<()> {
    if !is_wsl() {
        display::warning("Not running in WSL — skipping Windows sync.");
        return Ok(());
    }

    if !has_pwsh() && !has_robocopy() {
        display::error("Neither pwsh.exe nor robocopy.exe found.");
        return Ok(());
    }

    if WIN_USERNAME.is_none() {
        display::error(
            "Windows username not set. Pass --win-user <NAME> or set WIN_USER env var.",
        );
        display::info("  Example: wots sync --win-user zion");
        bail!("missing --win-user");
    }

    if !quiet {
        let user = WIN_USERNAME.as_deref().unwrap();
        display::info(&format!(
            "Windows user = \"{}\"  →  C:\\Users\\{}",
            user, user,
        ));
    }

    for pkg in pkgs {
        let items = prepare_sync_items(pkg, pt);
        if items.is_empty() {
            if !quiet {
                display::info(&format!(
                    "  {}: no files",
                    pkg_basename(pkg),
                ));
            }
            continue;
        }

        if !quiet {
            display::info(&format!(
                "  {}: {} file(s)",
                pkg_basename(pkg),
                items.len(),
            ));
        }

        let counts = sync_batch(&items, dry_run, *SYNC_MAX_CONCURRENT);
        print_sync_summary(&counts, quiet);

        // Update the index so subsequent list/stats/diff can detect
        // deletions and out-of-sync states correctly.
        if !dry_run {
            let _ = status::check_copy_status(pkg, pt);
        }
    }

    Ok(())
}

pub fn do_stow(pkg: &Path, pt: &PkgType, dry_run: bool) -> Result<()> {
    let target = pt.sync_target().context("no sync target for type")?;
    let pkg_name = pkg
        .file_name()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_string_lossy();

    if pt.needs_sudo() {
        return stow_file_by_file(pkg, &target, true, dry_run);
    }

    let output = Command::new("stow")
        .args([
            "-v",
            "--adopt",
            "-t",
            &target.to_string_lossy(),
            &pkg_name,
        ])
        .current_dir(&*DOTFILES_DIR)
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        display::success(&format!("Stowed  {}  →  {}", pkg_name, target.display()));
        return Ok(());
    }

    if stderr.contains("existing target is not owned by stow") {
        display::warning(&format!("Stow conflict in {}", pkg_name));
        display::warning("  → Retrying file-by-file (ln -sf) ...");
        return stow_file_by_file(pkg, &target, false, dry_run);
    }

    if stderr.contains("Permission denied") || stderr.contains("cannot stow") {
        display::warning(&format!(
            "Stow permission error in {}: {}",
            pkg_name,
            &stderr[..stderr.len().min(200)]
        ));
    } else {
        return Err(anyhow::anyhow!(
            "Stow failed for {}: {}",
            pkg_name,
            &stderr[..stderr.len().min(300)]
        ));
    }

    Ok(())
}

pub fn stow_file_by_file(pkg: &Path, target: &Path, sudo: bool, dry_run: bool) -> Result<()> {
    for f in list_syncable_files(pkg) {
        let rel = f.strip_prefix(pkg)?;
        let dest = target.join(rel);

        if let Some(parent) = dest.parent() {
            if sudo {
                Command::new("sudo")
                    .args(["mkdir", "-p", &parent.to_string_lossy()])
                    .status()?;
            } else {
                fs::create_dir_all(parent)?;
            }
        }

        let src = f.canonicalize().unwrap_or_else(|_| f.to_path_buf());

        if dry_run {
            display::dim(&format!("      ln -sf {} {}", src.display(), dest.display()));
        } else if sudo {
            Command::new("sudo")
                .args(["ln", "-sf", &src.to_string_lossy(), &dest.to_string_lossy()])
                .status()?;
        } else {
            if dest.exists() || dest.is_symlink() {
                let _ = fs::remove_file(&dest);
            }
            std::os::unix::fs::symlink(&src, &dest)?;
        }
    }

    let pkg_name = pkg
        .file_name()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_string_lossy();
    display::success(&format!(
        "Linked (file-by-file)  {}  →  {}",
        pkg_name,
        target.display()
    ));

    Ok(())
}

pub fn prepare_sync_items(pkg: &Path, pt: &PkgType) -> Vec<(PathBuf, PathBuf)> {
    list_syncable_files(pkg)
        .into_iter()
        .map(|f| {
            let wp = build_win_path(&f, pkg, pt);
            (f, wp)
        })
        .collect()
}

pub fn sync_batch(
    items: &[(PathBuf, PathBuf)],
    dry_run: bool,
    max_concurrent: usize,
) -> HashMap<String, usize> {
    let use_robocopy = has_robocopy();

    let total = Arc::new(AtomicUsize::new(0));
    let copied = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(0));
    let missing_source = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(AtomicUsize::new(0));

    let n_threads = max_concurrent.clamp(1, 4);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(n_threads)
        .build()
        .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap());

    pool.install(|| {
        items
            .par_iter()
            .for_each(|(wsl_path, win_path)| {
            total.fetch_add(1, Ordering::Relaxed);

            if is_excluded(wsl_path) {
                skipped.fetch_add(1, Ordering::Relaxed);
                return;
            }

            if !wsl_path.exists() {
                missing_source.fetch_add(1, Ordering::Relaxed);
                return;
            }

            if use_robocopy {
                match robocopy_sync(wsl_path, win_path, dry_run) {
                    Ok(_) => {
                        copied.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        errors.fetch_add(1, Ordering::Relaxed);
                        display::error(&format!("robocopy failed for {}: {e}", wsl_path.display()));
                    }
                }
            } else {
                match pwsh_copy(wsl_path, win_path, wsl_path.is_dir(), dry_run) {
                    Ok(true) => {
                        copied.fetch_add(1, Ordering::Relaxed);
                    }
                    Ok(false) => {
                        errors.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        errors.fetch_add(1, Ordering::Relaxed);
                        display::error(&format!("pwsh copy failed for {}: {e}", wsl_path.display()));
                    }
                }
            }
        });
    });

    let mut counts = HashMap::new();
    counts.insert("copied_to_win".to_string(), copied.load(Ordering::Relaxed));
    counts.insert("missing_source".to_string(), missing_source.load(Ordering::Relaxed));
    counts.insert("skipped".to_string(), skipped.load(Ordering::Relaxed));
    counts.insert("error".to_string(), errors.load(Ordering::Relaxed));
    counts
}

fn robocopy_sync(wsl_src: &Path, win_dst: &Path, dry_run: bool) -> Result<()> {
    let wsl_unc = format!(
        "\\\\wsl$\\{}\\{}",
        *WSL_DISTRO_NAME,
        wsl_src.to_string_lossy().replace('/', "\\")
    );

    let win_str = format!(
        "C:\\{}",
        win_dst
            .strip_prefix(&*MNT_C)
            .unwrap_or(win_dst)
            .to_string_lossy()
            .replace('/', "\\")
    );

    if dry_run {
        display::dim(&format!(
            "    DRY-RUN  robocopy {} {} /MIR /MT:8",
            wsl_src.display(),
            win_str,
        ));
        return Ok(());
    }

    let mut cmd = if wsl_src.is_dir() {
        let mut c = Command::new("robocopy.exe");
        c.args([
            &wsl_unc,
            &win_str,
            "/MIR",
            "/MT:8",
            "/R:1",
            "/W:1",
            "/NJH",
            "/NJS",
            "/NP",
            "/XF",
            ".git",
        ]);
        c
    } else {
        let parent = wsl_unc
            .rsplit_once('\\')
            .map(|(p, _)| p.to_string())
            .unwrap_or(wsl_unc.clone());
        let file = wsl_src
            .file_name()
            .unwrap_or(std::ffi::OsStr::new(""))
            .to_string_lossy()
            .to_string();
        let mut c = Command::new("robocopy.exe");
        c.args([
            &parent,
            &win_str,
            &file,
            "/R:1",
            "/W:1",
            "/NJH",
            "/NJS",
            "/NP",
        ]);
        c
    };

    let status = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    let code = status.code().unwrap_or(8);
    if code < 8 {
        Ok(())
    } else {
        bail!("robocopy failed with exit code {}", code)
    }
}

fn pwsh_copy(wsl_src: &Path, win_dst: &Path, is_dir: bool, dry_run: bool) -> Result<bool> {
    let wsl_unc = format!(
        "\\\\wsl$\\{}\\{}",
        *WSL_DISTRO_NAME,
        wsl_src.to_string_lossy().replace('/', "\\")
    );

    let win_str = format!(
        "C:\\{}",
        win_dst
            .strip_prefix(&*MNT_C)
            .unwrap_or(win_dst)
            .to_string_lossy()
            .replace('/', "\\")
    );

    // Use PowerShell's native Copy-Item instead of cmd /c xcopy/copy
    // because CMD does not support UNC paths (e.g. \\wsl$\...).
    let ps_cmd = if is_dir {
        format!(
            "$ErrorActionPreference='Stop'; Copy-Item -LiteralPath '{}' -Destination '{}' -Recurse -Force",
            wsl_unc, win_str
        )
    } else {
        format!(
            "$ErrorActionPreference='Stop'; New-Item -ItemType Directory -Force -Path (Split-Path '{}' -Parent) | Out-Null; Copy-Item -LiteralPath '{}' -Destination '{}' -Force",
            win_str, wsl_unc, win_str
        )
    };

    if dry_run {
        display::dim(&format!(
            "    DRY-RUN  pwsh.exe -Command {}",
            ps_cmd
        ));
        return Ok(true);
    }

    let output = Command::new("pwsh.exe")
        .args(["-NoProfile", "-Command", &ps_cmd])
        .output()?;

    if output.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        display::error(&format!("Windows copy failed: {}", &stderr[..stderr.len().min(200)]));
        Ok(false)
    }
}

fn print_sync_summary(counts: &HashMap<String, usize>, quiet: bool) {
    if quiet {
        return;
    }

    let mut parts: Vec<String> = Vec::new();
    if let Some(&c) = counts.get("copied_to_win")
        && c > 0 {
            parts.push(format!("{} copied", c));
        }
    if let Some(&s) = counts.get("skipped")
        && s > 0 {
            parts.push(format!("{} skipped", s));
        }
    if let Some(&m) = counts.get("missing_source")
        && m > 0 {
            parts.push(format!("{} missing", m));
        }
    if let Some(&e) = counts.get("error")
        && e > 0 {
            parts.push(format!("{} errors", e));
        }

    if !parts.is_empty() {
        display::info(&format!("    Result: {}", parts.join(", ")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("wots_test_sync_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(p) = path.parent() {
            let _ = fs::create_dir_all(p);
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn prepare_sync_items_winuser() {
        let dir = temp_dir();
        let pkg = dir.join("test.winuser");
        write_file(&pkg.join("data.txt"), "hello");
        write_file(&pkg.join("sub/cfg.json"), "{}");

        let items = prepare_sync_items(&pkg, &PkgType::WinUser);
        assert_eq!(items.len(), 2);

        for (wsl, win) in &items {
            assert!(wsl.exists());
            assert!(win.to_string_lossy().contains("Users"));
        }
    }

    #[test]
    fn prepare_sync_items_winconfig() {
        let dir = temp_dir();
        let pkg = dir.join("cfg.winconfig");
        write_file(&pkg.join("settings.json"), "{}");

        let items = prepare_sync_items(&pkg, &PkgType::WinConfig);
        assert_eq!(items.len(), 1);
        let (_wsl, win) = &items[0];
        assert!(win.to_string_lossy().contains(".config"));
    }

    #[test]
    fn prepare_sync_items_winlocal() {
        let dir = temp_dir();
        let pkg = dir.join("app.winlocal");
        write_file(&pkg.join("state.json"), "{}");

        let items = prepare_sync_items(&pkg, &PkgType::WinLocal);
        assert_eq!(items.len(), 1);
        let (_wsl, win) = &items[0];
        assert!(win.to_string_lossy().contains("AppData/Local"));
    }

    #[test]
    fn prepare_sync_items_winroaming() {
        let dir = temp_dir();
        let pkg = dir.join("roam.winroaming");
        write_file(&pkg.join("prefs.json"), "{}");

        let items = prepare_sync_items(&pkg, &PkgType::WinRoaming);
        assert_eq!(items.len(), 1);
        let (_wsl, win) = &items[0];
        assert!(win.to_string_lossy().contains("AppData/Roaming"));
    }

    #[test]
    fn prepare_sync_items_excludes_ignored_dirs() {
        let dir = temp_dir();
        let pkg = dir.join("app.winuser");
        write_file(&pkg.join("keep.txt"), "keep");
        write_file(&pkg.join("node_modules/ignored.js"), "ignored");

        let items = prepare_sync_items(&pkg, &PkgType::WinUser);
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn prepare_sync_items_empty_package() {
        let dir = temp_dir();
        let pkg = dir.join("empty.winuser");
        fs::create_dir_all(&pkg).unwrap();

        let items = prepare_sync_items(&pkg, &PkgType::WinUser);
        assert!(items.is_empty());
    }

    #[test]
    fn print_sync_summary_all_fields() {
        let mut counts = HashMap::new();
        counts.insert("copied_to_win".into(), 3);
        counts.insert("skipped".into(), 1);
        counts.insert("missing_source".into(), 2);
        counts.insert("error".into(), 1);
        // Should not panic
        print_sync_summary(&counts, false);
    }

    #[test]
    fn print_sync_summary_quiet() {
        let mut counts = HashMap::new();
        counts.insert("copied_to_win".into(), 10);
        // In quiet mode, should produce no output
        print_sync_summary(&counts, true);
    }

    #[test]
    fn print_sync_summary_empty() {
        let counts = HashMap::new();
        // Should not panic, produces no output
        print_sync_summary(&counts, false);
    }

    #[test]
    fn print_sync_summary_all_zero() {
        let mut counts = HashMap::new();
        counts.insert("copied_to_win".into(), 0);
        counts.insert("skipped".into(), 0);
        // Zeros are skipped, should not panic
        print_sync_summary(&counts, false);
    }

    #[test]
    fn sync_batch_no_pwsh_no_robocopy_is_wsl() {
        // If we're not in WSL, sync_batch won't call robocopy/pwsh.
        // Test that sync_batch handles empty items gracefully.
        let items: Vec<(PathBuf, PathBuf)> = Vec::new();
        let counts = sync_batch(&items, true, 1);
        assert_eq!(counts.get("copied_to_win").copied().unwrap_or(0), 0);
    }
}
