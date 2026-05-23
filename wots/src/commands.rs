use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;

use crate::cli::{DiffArgs, ListArgs, StatsArgs};
use crate::config::DOTFILES_DIR;
use crate::discover;
use crate::display;
use crate::status;
use crate::types::{self, PkgType, SYNCABLE_TYPES};
use crate::util;

pub fn cmd_stats(args: &StatsArgs) -> Result<()> {
    use rayon::prelude::*;

    let packages = discover::find_packages(&DOTFILES_DIR);
    let mut stats_data: HashMap<&'static str, display::TypeStats> = HashMap::new();
    let mut total_pkgs = 0usize;
    let mut total_files = 0usize;
    let mut total_bytes = 0u64;

    for pt in &SYNCABLE_TYPES {
        let pkgs = packages.get(pt).cloned().unwrap_or_default();
        let n_pkgs = pkgs.len();
        total_pkgs += n_pkgs;

        if n_pkgs == 0 {
            continue;
        }

        let counts: Vec<(usize, u64)> = pkgs
            .par_iter()
            .map(|p| util::count_and_size(p))
            .collect();

        let n_files: usize = counts.iter().map(|(c, _)| c).sum();
        let n_bytes: u64 = counts.iter().map(|(_, s)| s).sum();
        total_files += n_files;
        total_bytes += n_bytes;

        let names: Vec<String> = pkgs.iter().map(|p| discover::pkg_basename(p)).collect();
        let st = if pt.uses_stow() {
            let (stowed, stowable) = status::check_stow_status_batch(&pkgs, *pt);
            if stowable > 0 {
                format!("{}/{} stowed", stowed, stowable)
            } else {
                "empty".to_string()
            }
        } else if pt.uses_copy_sync() {
            let counts = status::check_copy_status_batch(&pkgs, *pt);
            status::status_text(&counts)
        } else {
            "manual".to_string()
        };

        stats_data.insert(
            pt.value(),
            display::TypeStats {
                packages: n_pkgs,
                files: n_files,
                size_bytes: n_bytes,
                size_human: util::fmt_size(n_bytes),
                status_text: st,
                names,
            },
        );
    }

    if args.json_output {
        let json = serde_json::json!({
            "dotfiles": DOTFILES_DIR.to_string_lossy(),
            "total_packages": total_pkgs,
            "total_files": total_files,
            "total_size_bytes": total_bytes,
            "total_size_human": util::fmt_size(total_bytes),
            "by_type": stats_data.iter().map(|(k, v)| {
                ((*k).to_string(), serde_json::json!({
                    "packages": v.packages,
                    "files": v.files,
                    "size_bytes": v.size_bytes,
                    "size_human": v.size_human,
                    "status_text": v.status_text,
                }))
            }).collect::<serde_json::Map<_, _>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    display::render_stats(&stats_data, total_pkgs, total_files, total_bytes);
    Ok(())
}

pub fn cmd_list(args: &ListArgs) -> Result<()> {
    use rayon::prelude::*;

    let packages = discover::find_packages(&DOTFILES_DIR);
    let types_to_show: Vec<types::PkgType> = if let Some(pt) = &args.pkg_type {
        vec![*pt]
    } else {
        SYNCABLE_TYPES.to_vec()
    };

    let mut pkg_list: Vec<(PathBuf, PkgType)> = Vec::new();
    for pt in &types_to_show {
        if let Some(pkgs) = packages.get(pt) {
            for pkg in pkgs {
                pkg_list.push((pkg.clone(), *pt));
            }
        }
    }

    if pkg_list.is_empty() {
        display::warning("No packages found.");
        return Ok(());
    }

    let counts: Vec<(usize, u64)> = pkg_list
        .par_iter()
        .map(|(p, _)| util::count_and_size(p))
        .collect();

    let mut rows: Vec<display::ListRow> = Vec::new();
    for (idx, (pkg, pt)) in pkg_list.iter().enumerate() {
        let name = discover::pkg_basename(pkg);
        let (files, size) = counts[idx];

        let st = if pt.uses_stow() {
            let (stowed, stowable) = status::check_stow_status(pkg, pt);
            if stowed == stowable && stowable > 0 {
                "stowed".to_string()
            } else if stowable > 0 {
                format!("{}/{} stowed", stowed, stowable)
            } else {
                "empty".to_string()
            }
        } else if pt.uses_copy_sync() {
            let counts = status::check_copy_status(pkg, pt);
            status::status_text(&counts)
        } else {
            "manual".to_string()
        };

        rows.push(display::ListRow {
            name,
            pkg_type: pt.value().to_string(),
            files,
            size_bytes: size,
            size_human: util::fmt_size(size),
            status: st,
            path: pkg.to_string_lossy().to_string(),
        });
    }

    if args.json_output {
        let json_rows: Vec<serde_json::Value> = rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "name": r.name,
                    "type": r.pkg_type,
                    "files": r.files,
                    "size_bytes": r.size_bytes,
                    "size_human": r.size_human,
                    "status": r.status,
                    "path": r.path,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_rows)?);
        return Ok(());
    }

    display::render_list(&rows);
    Ok(())
}

pub fn cmd_diff(args: &DiffArgs) -> Result<()> {
    #[derive(serde::Serialize)]
    struct DiffEntry {
        package: String,
        pkg_type: String,
        status: String,
        files: Vec<DiffFile>,
    }

    #[derive(serde::Serialize)]
    struct DiffFile {
        status: String,
        path: String,
    }

    let packages = discover::find_packages(&DOTFILES_DIR);
    let types_to_show: Vec<PkgType> = if let Some(pt) = &args.pkg_type {
        vec![*pt]
    } else {
        SYNCABLE_TYPES.to_vec()
    };

    let mut entries: Vec<DiffEntry> = Vec::new();

    for pt in &types_to_show {
        let pkgs = match packages.get(pt) {
            Some(p) => p,
            None => continue,
        };

        for pkg in pkgs {
            if let Some(ref app) = args.app
                && discover::pkg_basename(pkg) != *app {
                    continue;
                }

            if pt.uses_copy_sync() {
                let (counts, file_entries, save_err) =
                    status::check_copy_status_detailed(pkg, pt);
                if let Some(e) = save_err {
                    eprintln!("  Warning: failed to save sync index: {e}");
                }

                let has_work = counts.outdated_local > 0
                    || counts.missing_remote > 0
                    || counts.outdated_remote > 0
                    || counts.missing_wsl > 0
                    || counts.content_mat_mismatch > 0;
                if !has_work {
                    continue;
                }

                let files: Vec<DiffFile> = file_entries
                    .iter()
                    .filter(|e| e.status != status::FileSyncStatus::Synced
                              && e.status != status::FileSyncStatus::Skipped)
                    .map(|e| DiffFile {
                        status: e.status.label().to_string(),
                        path: e.relative_path.display().to_string(),
                    })
                    .collect();

                entries.push(DiffEntry {
                    package: discover::pkg_basename(pkg),
                    pkg_type: pt.value().to_string(),
                    status: status::status_text(&counts),
                    files,
                });
            } else if pt.uses_stow() {
                let (stowed, stowable) = status::check_stow_status(pkg, pt);
                if stowed < stowable {
                    let mut files: Vec<DiffFile> = Vec::new();
                    let target = pt.sync_target();
                    if let Some(target) = target {
                        for f in discover::list_syncable_files(pkg) {
                            if let Ok(rel) = f.strip_prefix(pkg) {
                                let dest = target.join(rel);
                                if !status::is_symlink_or_parent(&dest, &target) {
                                    files.push(DiffFile {
                                        status: "not-stowed".into(),
                                        path: dest.display().to_string(),
                                    });
                                }
                            }
                        }
                    }
                    entries.push(DiffEntry {
                        package: discover::pkg_basename(pkg),
                        pkg_type: pt.value().to_string(),
                        status: format!("{}/{} stowed", stowed, stowable),
                        files,
                    });
                }
            }
        }
    }

    if entries.is_empty() {
        if args.json_output {
            println!("[]");
        } else {
            display::success("All packages are in sync.");
        }
        return Ok(());
    }

    if args.json_output {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    for entry in &entries {
        display::warning(&format!("  {} — {}", entry.package, entry.status));
        for f in &entry.files {
            display::dim(&format!("    {}: {}", f.status, f.path));
        }
    }

    Ok(())
}
