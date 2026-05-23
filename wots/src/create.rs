use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::cli::CreateArgs;
use crate::config::{DOTFILES_DIR, HOME, MNT_C, ROOT_TARGET};
use crate::discover::{detect_type, propose_name};
use crate::display;
use crate::sync::{do_stow, prepare_sync_items, sync_batch};
use crate::types::{type_label, PkgType};
use crate::util::copy_dir_all;

pub fn run(args: CreateArgs) -> Result<()> {
    let resolved: Vec<PathBuf> = args
        .sources
        .iter()
        .map(|s| {
            PathBuf::from(shellexpand::tilde(s).into_owned())
                .canonicalize()
                .with_context(|| format!("Source does not exist: {}", s))
        })
        .collect::<Result<Vec<_>>>()?;

    let pt = if let Some(pt_str) = args.pkg_type {
        pt_str
    } else {
        let detected: Vec<PkgType> = resolved.iter().map(|p| detect_type(p)).collect();
        let unique: Vec<PkgType> = {
            let mut u = detected.clone();
            u.sort_by_key(|t| t.value());
            u.dedup();
            u
        };

        if unique.len() > 1 {
            let labels: Vec<String> = unique.iter().map(|t| t.value().to_string()).collect();
            display::error(&format!(
                "Sources have mixed types — specify --type explicitly. Detected: {}",
                labels.join(", ")
            ));
            bail!("mixed source types");
        }

        let mut pt = unique[0];
        display::info(&format!(
            "Detected type: {} → {}",
            pt.value(),
            type_label(pt),
        ));

        if !args.yes {
            let resp = display::prompt::ask_custom(
                &format!("  Create as {}?", pt.value()),
                pt.value(),
                &[],
            );

            if resp == "n" {
                display::info("Cancelled.");
                return Ok(());
            }

            if resp != "y" && !resp.is_empty() && resp != pt.value() {
                match PkgType::from_str(&resp) {
                    Some(new_pt) => {
                        display::info(&format!("Using type: {}", new_pt.value()));
                        pt = new_pt;
                    }
                    None => {
                        display::error(&format!("Unknown type: {}", resp));
                        bail!("unknown type: {}", resp);
                    }
                }
            }
        }

        pt
    };

    let app_name = if let Some(ref name) = args.app_name {
        name.clone()
    } else {
        let default = propose_name(&resolved);
        if args.yes {
            default
        } else {
            display::prompt::ask("  App name", &default)
        }
    };

    let dest_root = DOTFILES_DIR.join(format!("{}{}", app_name, pt.suffix()));
    if dest_root.exists() {
        display::error(&format!(
            "Package already exists: {}",
            dest_root.display()
        ));
        bail!("package already exists: {}", dest_root.display());
    }

    validate_sources(&resolved, &pt)?;

    display::rule(&format!(
        "Creating {} package: {}",
        pt.value(),
        dest_root
            .file_name()
            .unwrap_or(std::ffi::OsStr::new(""))
            .to_string_lossy(),
    ));

    for src in &resolved {
        let dest = compute_dest(src, &pt, &dest_root)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        if args.dry_run {
            let action = if pt.is_linux_config() {
                "Move"
            } else {
                "Copy"
            };
            display::dim(&format!(
                "  DRY-RUN  {}  {}  →  {}",
                action,
                src.display(),
                dest.display()
            ));
        } else {
            create_atomic(src, &dest, pt.is_linux_config())?;
        }
    }

    let pkg_name = dest_root
        .file_name()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_string_lossy();
    display::success(&format!("Package '{}' created.", pkg_name));

    if pt.uses_stow() && !args.no_stow {
        do_stow(&dest_root, &pt, args.dry_run)?;
    } else if pt.uses_copy_sync() && !args.no_sync {
        let items = prepare_sync_items(&dest_root, &pt);
        for (wsl_p, win_p) in &items {
            sync_batch(&[(wsl_p.clone(), win_p.clone())], args.dry_run, 1);
        }
    }

    display::info("");
    display::success(&format!("Package '{}' ready.", pkg_name));

    Ok(())
}

fn validate_sources(sources: &[PathBuf], pt: &PkgType) -> Result<()> {
    for src in sources {
        if pt.is_linux_config() {
            if !src.starts_with(&*HOME) {
                display::warning(&format!(
                    "Source '{}' is outside HOME; only filename will be used for path mapping.",
                    src.display()
                ));
            }
            continue;
        }
        if *pt == PkgType::Root && src.starts_with(&*ROOT_TARGET) {
            continue;
        }
        if *pt == PkgType::Meta {
            continue;
        }
        if pt.is_windows() {
            if !src.starts_with(&*MNT_C) {
                display::error(&format!(
                    "Source must be under {} for {} type: {}",
                    MNT_C.display(),
                    pt.value(),
                    src.display()
                ));
                bail!("source not under /mnt/c: {}", src.display());
            }
            continue;
        }
        display::error(&format!(
            "Source '{}' does not match package type '{}'.",
            src.display(),
            pt.value()
        ));
        bail!(
            "source mismatch: {} for type {}",
            src.display(),
            pt.value()
        );
    }
    Ok(())
}

fn compute_dest(src: &Path, pt: &PkgType, pkg_root: &Path) -> Result<PathBuf> {
    if pt.is_linux_config() {
        if let Ok(rel) = src.strip_prefix(&*HOME) {
            return Ok(pkg_root.join(rel));
        }
        return Ok(pkg_root.join(
            src.file_name()
                .unwrap_or(std::ffi::OsStr::new("")),
        ));
    }
    if *pt == PkgType::Root {
        if let Ok(rel) = src.strip_prefix(&*ROOT_TARGET) {
            return Ok(pkg_root.join(rel));
        }
        return Ok(pkg_root.join(
            src.file_name()
                .unwrap_or(std::ffi::OsStr::new("")),
        ));
    }
    if pt.is_windows() {
        if let Some(target) = pt.sync_target() {
            let target_str = target.to_string_lossy();
            let without_drive = target_str.strip_prefix("C:").unwrap_or(&target_str);
            let target_mnt = MNT_C.join(without_drive.trim_start_matches('/').trim_start_matches('\\'));
            if let Ok(rel) = src.strip_prefix(&target_mnt) {
                return Ok(pkg_root.join(rel));
            }
        }
        if let Ok(rel) = src.strip_prefix(&*MNT_C) {
            return Ok(pkg_root.join(rel));
        }
        return Ok(pkg_root.join(
            src.file_name()
                .unwrap_or(std::ffi::OsStr::new("")),
        ));
    }
    if *pt == PkgType::Meta {
        return Ok(pkg_root.join(
            src.file_name()
                .unwrap_or(std::ffi::OsStr::new("")),
        ));
    }
    Ok(pkg_root.join(
        src.file_name()
            .unwrap_or(std::ffi::OsStr::new("")),
    ))
}

fn create_atomic(src: &Path, dest: &Path, is_move: bool) -> Result<()> {
    if is_move {
        let tmp = dest.with_extension(format!(
            ".wots_tmp_{}",
            std::process::id()
        ));

        if src.is_dir() {
            copy_dir_all(src, &tmp)?;
        } else {
            if let Some(parent) = tmp.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(src, &tmp)?;
        }

        validate_copy(src, &tmp)?;

        fs::rename(&tmp, dest)?;

        if src.is_dir() {
            fs::remove_dir_all(src)?;
        } else {
            fs::remove_file(src)?;
        }
    } else {
        if src.is_dir() {
            copy_dir_all(src, dest)?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(src, dest)?;
        }
    }

    Ok(())
}

fn validate_copy(src: &Path, dest: &Path) -> Result<()> {
    if src.is_dir() != dest.is_dir() {
        bail!("copy validation: type mismatch (dir vs file)");
    }

    if src.is_dir() {
        let src_count = walkdir::WalkDir::new(src).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()).count();
        let dest_count = walkdir::WalkDir::new(dest).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()).count();

        if src_count != dest_count {
            bail!(
                "copy validation: file count mismatch ({} vs {})",
                src_count,
                dest_count
            );
        }
    } else {
        let src_size = fs::metadata(src)?.len();
        let dest_size = fs::metadata(dest)?.len();
        if src_size != dest_size {
            bail!(
                "copy validation: size mismatch ({} vs {})",
                src_size,
                dest_size
            );
        }
    }

    Ok(())
}
