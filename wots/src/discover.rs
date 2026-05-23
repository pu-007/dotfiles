use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::config::{HOME, MNT_C, ROOT_TARGET, WIN_USERNAME};
use crate::types::{type_from_dir_name, PkgType, ALL_TYPES};
use crate::util::{is_excluded, is_quick_exclude_dir};

pub fn detect_type(path: &Path) -> PkgType {
    let rp = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    if let Ok(rel) = rp.strip_prefix(&*MNT_C) {
        let parts: Vec<&OsStr> = rel.iter().collect();
        if parts.len() >= 3 && parts[0] == "Users" {
            let sub = if parts.len() > 2 {
                PathBuf::from_iter(&parts[2..])
            } else {
                PathBuf::from(".")
            };

            for sub_path in sub.ancestors().skip(1) {
                if sub_path == Path::new("AppData/Roaming") {
                    return PkgType::WinRoaming;
                }
                if sub_path == Path::new("AppData/Local") {
                    return PkgType::WinLocal;
                }
            }

            if parts.len() >= 3 && parts[2] == ".config" {
                return PkgType::WinConfig;
            }

            return PkgType::WinUser;
        }
        return PkgType::Meta;
    }

    if let Ok(rel) = rp.strip_prefix(&*HOME) {
        if rp == *HOME {
            return PkgType::User;
        }
        let parts: Vec<&OsStr> = rel.iter().collect();
        if !parts.is_empty() {
            if parts[0] == ".config" {
                return PkgType::Config;
            }
            if parts[0] == ".local" {
                return PkgType::Local;
            }
        }
        return PkgType::User;
    }

    if rp.starts_with(&*ROOT_TARGET.join("etc")) || rp == *ROOT_TARGET {
        return PkgType::Root;
    }

    PkgType::Meta
}

pub fn find_packages(base: &Path) -> HashMap<PkgType, Vec<PathBuf>> {
    let mut result: HashMap<PkgType, Vec<PathBuf>> = HashMap::new();
    for pt in &ALL_TYPES {
        result.insert(*pt, Vec::new());
    }

    let dir = match fs::read_dir(base) {
        Ok(d) => d,
        Err(_) => return result,
    };

    let mut entries: Vec<_> = dir.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        if name_str.starts_with('.') {
            continue;
        }
        if let Some(pt) = type_from_dir_name(&name_str) {
            result.entry(pt).or_default().push(entry.path());
        }
    }

    result
}

pub fn pkg_basename(pkg_path: &Path) -> String {
    let name = pkg_path
        .file_name()
        .unwrap_or(OsStr::new(""))
        .to_string_lossy()
        .to_string();

    let pt = type_from_dir_name(&name);
    if let Some(pt) = pt {
        let suffix = pt.suffix();
        if name.ends_with(&suffix) && name.len() > suffix.len() {
            return name[..name.len() - suffix.len()].to_string();
        }
    }
    name
}

pub fn list_syncable_files(pkg: &Path) -> Vec<PathBuf> {
    if !pkg.is_dir() {
        return Vec::new();
    }

    WalkDir::new(pkg)
        .into_iter()
        .filter_entry(|e| is_quick_exclude_dir(e.file_name()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && !is_excluded(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn winuser_rel_path(pkg: &Path, file_path: &Path) -> PathBuf {
    let user_dir = pkg.join(
        WIN_USERNAME
            .as_deref()
            .unwrap_or("user"),
    );
    if user_dir.is_dir()
        && let Ok(rel) = file_path.strip_prefix(&user_dir) {
            return rel.to_path_buf();
        }
    file_path.strip_prefix(pkg).unwrap_or(file_path).to_path_buf()
}

pub fn build_win_path(file_path: &Path, pkg: &Path, pt: &PkgType) -> PathBuf {
    let rel = winuser_rel_path(pkg, file_path);
    let username = WIN_USERNAME.as_deref().unwrap_or("user");

    match pt {
        PkgType::WinUser => MNT_C.join("Users").join(username).join(rel),
        PkgType::WinConfig => MNT_C
            .join("Users")
            .join(username)
            .join(".config")
            .join(rel),
        PkgType::WinLocal => MNT_C
            .join("Users")
            .join(username)
            .join("AppData")
            .join("Local")
            .join(rel),
        PkgType::WinRoaming => MNT_C
            .join("Users")
            .join(username)
            .join("AppData")
            .join("Roaming")
            .join(rel),
        _ => MNT_C.join(rel),
    }
}

pub fn propose_name(sources: &[PathBuf]) -> String {
    if sources.is_empty() {
        return "unnamed".to_string();
    }

    let first = &sources[0];
    let file_name = first
        .file_name()
        .unwrap_or(OsStr::new("unnamed"))
        .to_string_lossy()
        .to_string();

    let init_names: &[&str] = &["init.lua", "init.vim", "config", "config.yaml", "settings.json"];

    if (init_names.contains(&file_name.as_str()) || sources.len() == 1)
        && let Some(parent) = first.parent()
            && let Some(parent_name) = parent.file_name() {
                let pn = parent_name.to_string_lossy();
                if !pn.is_empty() && pn != "." && pn != ".." && pn != "~" {
                    return pn.to_string();
                }
            }

    first
        .file_stem()
        .unwrap_or(first.file_name().unwrap_or(OsStr::new("unnamed")))
        .to_string_lossy()
        .to_string()
}

pub fn lcp_of(paths: &[PathBuf]) -> Option<PathBuf> {
    if paths.is_empty() {
        return None;
    }

    let canonical: Vec<PathBuf> = paths
        .iter()
        .map(|p| p.canonicalize().unwrap_or_else(|_| p.to_path_buf()))
        .collect();

    let mut common = canonical[0].clone();

    for p in &canonical[1..] {
        while !p.starts_with(&common) {
            if let Some(parent) = common.parent() {
                common = parent.to_path_buf();
            } else {
                return None;
            }
        }
    }

    Some(common)
}
