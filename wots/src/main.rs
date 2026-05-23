use std::io;
use std::path::PathBuf;
use std::process;

use anyhow::Result;
use colored::Colorize;

fn main() {
    let result = run();

    match result {
        Ok(()) => process::exit(0),
        Err(e) => {
            if let Some(io_err) = e.downcast_ref::<io::Error>()
                && io_err.kind() == io::ErrorKind::BrokenPipe {
                    process::exit(0);
                }
            eprintln!("{} {}", "✗".red(), e);
            process::exit(1);
        }
    }
}

fn run() -> Result<()> {
    let cli = <wots::cli::Cli as clap::Parser>::parse();

    match cli.command {
        wots::cli::Command::Create(args) => wots::create::run(args),
        wots::cli::Command::Sync(args) => wots::sync::run(args),
        wots::cli::Command::Stats(args) => cmd_stats(&args),
        wots::cli::Command::List(args) => cmd_list(&args),
        wots::cli::Command::Diff(args) => cmd_diff(&args),
    }
}

fn cmd_stats(args: &wots::cli::StatsArgs) -> Result<()> {
    use rayon::prelude::*;

    let packages = wots::discover::find_packages(&wots::config::DOTFILES_DIR);
    let mut stats_data: std::collections::HashMap<&'static str, wots::display::TypeStats> =
        std::collections::HashMap::new();
    let mut total_pkgs = 0usize;
    let mut total_files = 0usize;
    let mut total_bytes = 0u64;

    for pt in &wots::types::SYNCABLE_TYPES {
        let pkgs = packages.get(pt).cloned().unwrap_or_default();
        let n_pkgs = pkgs.len();
        total_pkgs += n_pkgs;

        if n_pkgs == 0 {
            continue;
        }

        let counts: Vec<(usize, u64)> = pkgs
            .par_iter()
            .map(|p| wots::util::count_and_size(p))
            .collect();

        let n_files: usize = counts.iter().map(|(c, _)| c).sum();
        let n_bytes: u64 = counts.iter().map(|(_, s)| s).sum();
        total_files += n_files;
        total_bytes += n_bytes;

        let names: Vec<String> = pkgs.iter().map(|p| wots::discover::pkg_basename(p)).collect();
        let st = if pt.uses_stow() {
            let (stowed, stowable) = wots::status::check_stow_status_batch(&pkgs, *pt);
            if stowable > 0 {
                format!("{}/{} stowed", stowed, stowable)
            } else {
                "empty".to_string()
            }
        } else if pt.uses_copy_sync() {
            let counts = wots::status::check_copy_status_batch(&pkgs, *pt);
            wots::status::status_text(&counts)
        } else {
            "manual".to_string()
        };

        stats_data.insert(
            pt.value(),
            wots::display::TypeStats {
                packages: n_pkgs,
                files: n_files,
                size_bytes: n_bytes,
                size_human: wots::util::fmt_size(n_bytes),
                status_text: st,
                names,
            },
        );
    }

    if args.json_output {
        let json = serde_json::json!({
            "dotfiles": wots::config::DOTFILES_DIR.to_string_lossy(),
            "total_packages": total_pkgs,
            "total_files": total_files,
            "total_size_bytes": total_bytes,
            "total_size_human": wots::util::fmt_size(total_bytes),
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

    wots::display::render_stats(&stats_data, total_pkgs, total_files, total_bytes);
    Ok(())
}

fn cmd_list(args: &wots::cli::ListArgs) -> Result<()> {
    use rayon::prelude::*;

    let packages = wots::discover::find_packages(&wots::config::DOTFILES_DIR);
    let types_to_show: Vec<wots::types::PkgType> = if let Some(pt) = &args.pkg_type {
        vec![*pt]
    } else {
        wots::types::SYNCABLE_TYPES.to_vec()
    };

    let mut pkg_list: Vec<(PathBuf, wots::types::PkgType)> = Vec::new();
    for pt in &types_to_show {
        if let Some(pkgs) = packages.get(pt) {
            for pkg in pkgs {
                pkg_list.push((pkg.clone(), *pt));
            }
        }
    }

    if pkg_list.is_empty() {
        wots::display::warning("No packages found.");
        return Ok(());
    }

    let counts: Vec<(usize, u64)> = pkg_list
        .par_iter()
        .map(|(p, _)| wots::util::count_and_size(p))
        .collect();

    let mut rows: Vec<wots::display::ListRow> = Vec::new();
    for (idx, (pkg, pt)) in pkg_list.iter().enumerate() {
        let name = wots::discover::pkg_basename(pkg);
        let (files, size) = counts[idx];

        let st = if pt.uses_stow() {
            let (stowed, stowable) = wots::status::check_stow_status(pkg, pt);
            if stowed == stowable && stowable > 0 {
                "stowed".to_string()
            } else if stowable > 0 {
                format!("{}/{} stowed", stowed, stowable)
            } else {
                "empty".to_string()
            }
        } else if pt.uses_copy_sync() {
            let counts = wots::status::check_copy_status(pkg, pt);
            wots::status::status_text(&counts)
        } else {
            "manual".to_string()
        };

        rows.push(wots::display::ListRow {
            name,
            pkg_type: pt.value().to_string(),
            files,
            size_bytes: size,
            size_human: wots::util::fmt_size(size),
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

    if rows.is_empty() {
        wots::display::warning("No packages found.");
        return Ok(());
    }

    wots::display::render_list(&rows);
    Ok(())
}

fn cmd_diff(args: &wots::cli::DiffArgs) -> Result<()> {
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

    let packages = wots::discover::find_packages(&wots::config::DOTFILES_DIR);
    let types_to_show: Vec<wots::types::PkgType> = if let Some(pt) = &args.pkg_type {
        vec![*pt]
    } else {
        wots::types::SYNCABLE_TYPES.to_vec()
    };

    let mut entries: Vec<DiffEntry> = Vec::new();

    for pt in &types_to_show {
        let pkgs = match packages.get(pt) {
            Some(p) => p,
            None => continue,
        };

        for pkg in pkgs {
            if let Some(ref app) = args.app
                && wots::discover::pkg_basename(pkg) != *app {
                    continue;
                }

            if pt.uses_copy_sync() {
                let (counts, file_entries, save_err) = wots::status::check_copy_status_detailed(pkg, pt);
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
                    .filter(|e| e.status != wots::status::FileSyncStatus::Synced
                              && e.status != wots::status::FileSyncStatus::Skipped)
                    .map(|e| DiffFile {
                        status: e.status.label().to_string(),
                        path: e.relative_path.display().to_string(),
                    })
                    .collect();

                entries.push(DiffEntry {
                    package: wots::discover::pkg_basename(pkg),
                    pkg_type: pt.value().to_string(),
                    status: wots::status::status_text(&counts),
                    files,
                });
            } else if pt.uses_stow() {
                let (stowed, stowable) = wots::status::check_stow_status(pkg, pt);
                if stowed < stowable {
                    let mut files: Vec<DiffFile> = Vec::new();
                    let target = pt.sync_target();
                    if let Some(target) = target {
                        for f in wots::discover::list_syncable_files(pkg) {
                            if let Ok(rel) = f.strip_prefix(pkg) {
                                let dest = target.join(rel);
                                if !wots::status::is_symlink_or_parent(&dest, &target) {
                                    files.push(DiffFile {
                                        status: "not-stowed".into(),
                                        path: dest.display().to_string(),
                                    });
                                }
                            }
                        }
                    }
                    entries.push(DiffEntry {
                        package: wots::discover::pkg_basename(pkg),
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
            wots::display::success("All packages are in sync.");
        }
        return Ok(());
    }

    if args.json_output {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    for entry in &entries {
        wots::display::warning(&format!("  {} — {}", entry.package, entry.status));
        for f in &entry.files {
            wots::display::dim(&format!("    {}: {}", f.status, f.path));
        }
    }

    Ok(())
}
