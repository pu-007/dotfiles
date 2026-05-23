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
                match resp.parse::<PkgType>() {
                    Ok(new_pt) => {
                        display::info(&format!("Using type: {}", new_pt.value()));
                        pt = new_pt;
                    }
                    Err(_) => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("wots_test_create_{}", std::process::id()));
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
    fn compute_dest_user_type_strips_home() {
        let dir = temp_dir();
        let pkg_root = dir.join("test.user");
        let src = PathBuf::from(format!("{}/.bashrc", HOME.to_string_lossy()));

        let dest = compute_dest(&src, &PkgType::User, &pkg_root).unwrap();
        assert_eq!(dest, pkg_root.join(".bashrc"));
    }

    #[test]
    fn compute_dest_config_type_preserves_subdir() {
        let dir = temp_dir();
        let pkg_root = dir.join("nvim.config");
        let src = PathBuf::from(format!("{}/.config/nvim/init.lua", HOME.to_string_lossy()));

        let dest = compute_dest(&src, &PkgType::Config, &pkg_root).unwrap();
        assert!(dest.ends_with("init.lua"));
        assert!(dest.to_string_lossy().contains("nvim"));
    }

    #[test]
    fn compute_dest_root_type_strips_root() {
        let dir = temp_dir();
        let pkg_root = dir.join("wsl.root");
        let src = PathBuf::from("/etc/wsl.conf");

        let dest = compute_dest(&src, &PkgType::Root, &pkg_root).unwrap();
        assert_eq!(dest, pkg_root.join("etc/wsl.conf"));
    }

    #[test]
    fn compute_dest_meta_type_uses_filename() {
        let dir = temp_dir();
        let pkg_root = dir.join("scripts.meta");
        let src = PathBuf::from("/tmp/some-script.sh");

        let dest = compute_dest(&src, &PkgType::Meta, &pkg_root).unwrap();
        assert_eq!(dest.file_name().unwrap(), "some-script.sh");
    }

    #[test]
    fn compute_dest_winuser_under_mnt_c() {
        let dir = temp_dir();
        let pkg_root = dir.join("git.winuser");
        let src = PathBuf::from("/mnt/c/Users/testuser/.gitconfig");

        let dest = compute_dest(&src, &PkgType::WinUser, &pkg_root).unwrap();
        assert!(dest.to_string_lossy().contains(".gitconfig"));
    }

    #[test]
    fn create_atomic_copy_file() {
        let dir = temp_dir();
        let src = dir.join("original.txt");
        let dest = dir.join("copied.txt");
        write_file(&src, "hello world");

        create_atomic(&src, &dest, false).unwrap();

        assert!(dest.exists());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "hello world");
        assert!(src.exists());
    }

    #[test]
    fn create_atomic_move_file() {
        let dir = temp_dir();
        let src = dir.join("to_move.txt");
        let dest = dir.join("moved.txt");
        write_file(&src, "move me");

        create_atomic(&src, &dest, true).unwrap();

        assert!(dest.exists());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "move me");
        assert!(!src.exists());
    }

    #[test]
    fn create_atomic_copy_dir() {
        let dir = temp_dir();
        let src = dir.join("src_dir");
        let dest = dir.join("dst_dir");
        write_file(&src.join("a.txt"), "a");
        write_file(&src.join("sub/b.txt"), "b");

        create_atomic(&src, &dest, false).unwrap();

        assert!(dest.join("a.txt").exists());
        assert!(dest.join("sub/b.txt").exists());
        assert!(src.exists());
    }

    #[test]
    fn create_atomic_move_dir() {
        let dir = temp_dir();
        let src = dir.join("src_dir2");
        let dest = dir.join("dst_dir2");
        write_file(&src.join("x.txt"), "x");

        create_atomic(&src, &dest, true).unwrap();

        assert!(dest.join("x.txt").exists());
        assert!(!src.exists());
    }

    #[test]
    fn validate_copy_files_same_size_ok() {
        let dir = temp_dir();
        let a = dir.join("a.txt");
        let b = dir.join("b.txt");
        write_file(&a, "hello");
        fs::copy(&a, &b).unwrap();

        assert!(validate_copy(&a, &b).is_ok());
    }

    #[test]
    fn validate_copy_files_different_size_err() {
        let dir = temp_dir();
        let a = dir.join("big.txt");
        let b = dir.join("small.txt");
        write_file(&a, "hello world");
        write_file(&b, "hi");

        assert!(validate_copy(&a, &b).is_err());
    }

    #[test]
    fn validate_copy_dirs_same_file_count_ok() {
        let dir = temp_dir();
        let a = dir.join("a_dir");
        let b = dir.join("b_dir");
        write_file(&a.join("1.txt"), "1");
        write_file(&a.join("2.txt"), "2");
        copy_dir_all(&a, &b).unwrap();

        assert!(validate_copy(&a, &b).is_ok());
    }

    #[test]
    fn validate_copy_dirs_different_count_err() {
        let dir = temp_dir();
        let a = dir.join("a_dir2");
        let b = dir.join("b_dir2");
        write_file(&a.join("1.txt"), "1");
        write_file(&b.join("1.txt"), "1");
        write_file(&b.join("2.txt"), "2");

        assert!(validate_copy(&a, &b).is_err());
    }

    #[test]
    fn validate_copy_type_mismatch_err() {
        let dir = temp_dir();
        let f = dir.join("file.txt");
        let d = dir.join("subdir");
        write_file(&f, "x");
        fs::create_dir_all(&d).unwrap();

        assert!(validate_copy(&f, &d).is_err());
    }

    #[test]
    fn validate_sources_linux_config_inside_home_ok() {
        let src = vec![PathBuf::from(format!("{}/.config/test", HOME.to_string_lossy()))];
        assert!(validate_sources(&src, &PkgType::Config).is_ok());
    }

    #[test]
    fn validate_sources_root_under_root_ok() {
        let src = vec![PathBuf::from("/etc/hosts")];
        assert!(validate_sources(&src, &PkgType::Root).is_ok());
    }

    #[test]
    fn validate_sources_meta_always_ok() {
        let src = vec![PathBuf::from("/any/path")];
        assert!(validate_sources(&src, &PkgType::Meta).is_ok());
    }

    #[test]
    fn validate_sources_winuser_not_under_mnt_c_err() {
        let src = vec![PathBuf::from("/etc/hosts")];
        assert!(validate_sources(&src, &PkgType::WinUser).is_err());
    }

    #[test]
    fn validate_sources_winuser_under_mnt_c_ok() {
        let src = vec![PathBuf::from("/mnt/c/Users/test/.gitconfig")];
        assert!(validate_sources(&src, &PkgType::WinUser).is_ok());
    }
}
