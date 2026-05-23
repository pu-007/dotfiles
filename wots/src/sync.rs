use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use rayon::prelude::*;

use crate::cli::SyncArgs;
use crate::config::{DOTFILES_DIR, MNT_C, SYNC_MAX_CONCURRENT, WSL_DISTRO_NAME};
use crate::discover::{build_win_path, find_packages, list_syncable_files, pkg_basename};
use crate::display;
use crate::types::{type_label, PkgType, SYNCABLE_TYPES};
use crate::util::{has_pwsh, has_robocopy, has_stow, is_excluded, is_wsl};

pub fn run(args: SyncArgs) -> Result<()> {
    let packages = find_packages(&DOTFILES_DIR);
    let types_to_sync: Vec<PkgType> = if let Some(pt) = &args.pkg_type {
        vec![*pt]
    } else {
        SYNCABLE_TYPES.to_vec()
    };

    let has_root = types_to_sync.contains(&PkgType::Root);
    if has_root && !args.bypass && !args.dry_run {
        display::info("\n⚠ Root sync requires sudo and will modify system files.");
        if !display::prompt::confirm("  Continue?", false) {
            display::info("Cancelled.");
            return Ok(());
        }
    }

    for pt in &types_to_sync {
        let pkgs = packages.get(pt).cloned().unwrap_or_default();
        let pkgs: Vec<PathBuf> = if let Some(ref app) = args.app {
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
    _max_concurrent: usize,
) -> HashMap<String, usize> {
    let use_robocopy = has_robocopy();

    let total = Arc::new(AtomicUsize::new(0));
    let copied = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(0));
    let missing_source = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(AtomicUsize::new(0));

    items
        .par_iter()
        .with_max_len(1)
        .try_for_each(|(wsl_path, win_path)| {
            total.fetch_add(1, Ordering::Relaxed);

            if is_excluded(wsl_path) {
                skipped.fetch_add(1, Ordering::Relaxed);
                return Ok::<_, anyhow::Error>(());
            }

            if !wsl_path.exists() {
                missing_source.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }

            if use_robocopy {
                match robocopy_sync(wsl_path, win_path, dry_run) {
                    Ok(_) => {
                        copied.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        errors.fetch_add(1, Ordering::Relaxed);
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
                    Err(_) => {
                        errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }

            Ok(())
        })
        .ok();

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

    let copy_cmd = if is_dir {
        format!("xcopy /E /I /Y \"{}\" \"{}\"", wsl_unc, win_str)
    } else {
        format!("copy /Y \"{}\" \"{}\"", wsl_unc, win_str)
    };

    if dry_run {
        display::dim(&format!(
            "    DRY-RUN  pwsh.exe -Command cmd /c {}",
            copy_cmd
        ));
        return Ok(true);
    }

    let output = Command::new("pwsh.exe")
        .args(["-NoProfile", "-Command", &format!("cmd /c {}", copy_cmd)])
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
