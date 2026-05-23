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

    if rp.starts_with(&*ROOT_TARGET) {
        if rp.starts_with("/proc") || rp.starts_with("/sys") || rp.starts_with("/dev") || rp.starts_with("/run") || rp.starts_with("/tmp") {
            return PkgType::Meta;
        }
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

    // When the source is a regular file (not a directory), the file's own
    // name is rarely a meaningful package name (e.g. config.yml, a.yml).
    // Prefer the parent directory name instead, unless the parent is HOME,
    // /, or some other generic top-level directory.
    if first.is_file()
        && let Some(parent) = first.parent() {
            let parent_str = parent.to_string_lossy();
            let home_str = HOME.to_string_lossy();
            if parent_str != home_str.as_ref()
                && parent_str != "/"
                && let Some(parent_name) = parent.file_name() {
                    let pn = parent_name.to_string_lossy();
                    if !pn.is_empty()
                        && pn != "."
                        && pn != ".."
                        && pn != "~"
                        && pn != ".config"
                        && pn != ".local" {
                            return pn.to_string();
                        }
                }
        }

    first
        .file_stem()
        .unwrap_or(first.file_name().unwrap_or(OsStr::new("unnamed")))
        .to_string_lossy()
        .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("wots_test_disc_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(p) = path.parent() {
            let _ = fs::create_dir_all(p);
        }
        fs::write(path, content).unwrap();
    }

    // ------------------------------------------------------------------
    // pkg_basename
    // ------------------------------------------------------------------

    #[test]
    fn basename_strips_suffix() {
        assert_eq!(pkg_basename(Path::new("/tmp/git.config")), "git");
        assert_eq!(pkg_basename(Path::new("/tmp/foo.user")), "foo");
        assert_eq!(pkg_basename(Path::new("/tmp/bar.winuser")), "bar");
        assert_eq!(
            pkg_basename(Path::new("/tmp/baz.winconfig")),
            "baz"
        );
        assert_eq!(
            pkg_basename(Path::new("/tmp/qux.winlocal")),
            "qux"
        );
        assert_eq!(
            pkg_basename(Path::new("/tmp/quux.winroaming")),
            "quux"
        );
        assert_eq!(pkg_basename(Path::new("/tmp/abc.root")), "abc");
        assert_eq!(pkg_basename(Path::new("/tmp/xyz.meta")), "xyz");
    }

    #[test]
    fn basename_no_suffix_returns_full() {
        assert_eq!(pkg_basename(Path::new("/tmp/plain_dir")), "plain_dir");
    }

    // ------------------------------------------------------------------
    // list_syncable_files
    // ------------------------------------------------------------------

    #[test]
    fn list_syncable_files_lists_correctly() {
        let dir = temp_dir();
        let pkg = dir.join("testapp.winuser");
        write_file(&pkg.join("a.txt"), "a");
        write_file(&pkg.join("sub/b.txt"), "b");
        // Excluded directory should be skipped
        write_file(&pkg.join("node_modules/pkg.json"), "{}");
        // Hidden file inside package should be included (not a dot-prefix dir)
        write_file(&pkg.join(".hidden"), "secret");

        let files = list_syncable_files(&pkg);
        let names: Vec<String> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(names.contains(&"a.txt".to_string()));
        assert!(names.contains(&"b.txt".to_string()));
        assert!(!names.contains(&"pkg.json".to_string()), "node_modules excluded");
        assert!(names.contains(&".hidden".to_string()));
    }

    #[test]
    fn list_syncable_files_nonexistent_is_empty() {
        let v = list_syncable_files(Path::new("/nonexistent/pkg"));
        assert!(v.is_empty());
    }

    // ------------------------------------------------------------------
    // winuser_rel_path
    // ------------------------------------------------------------------

    #[test]
    fn winuser_rel_path_fallback_when_no_username_subdir() {
        let dir = temp_dir();
        let pkg = dir.join("myapp.winuser");
        // File directly inside pkg (no matching username subdir)
        let file = pkg.join("Documents/notes.txt");
        write_file(&file, "notes");

        let rel = winuser_rel_path(&pkg, &file);
        assert_eq!(rel, PathBuf::from("Documents/notes.txt"));
    }

    #[test]
    fn winuser_rel_path_fallback_to_pkg_prefix() {
        let dir = temp_dir();
        let pkg = dir.join("myapp.winuser");
        let file = pkg.join("config.ini"); // no username subdir
        write_file(&file, "cfg");

        let rel = winuser_rel_path(&pkg, &file);
        assert_eq!(rel, PathBuf::from("config.ini"));
    }

    // ------------------------------------------------------------------
    // build_win_path
    // ------------------------------------------------------------------

    #[test]
    fn build_win_path_winuser() {
        let pkg = PathBuf::from("/tmp/myapp.winuser");
        let file = pkg.join("data.txt");
        let path = build_win_path(&file, &pkg, &PkgType::WinUser);
        // Should map to /mnt/c/Users/<user>/data.txt
        assert!(path.starts_with("/mnt/c/Users/"));
        assert!(path.ends_with("data.txt"));
    }

    #[test]
    fn build_win_path_winconfig() {
        let pkg = PathBuf::from("/tmp/myapp.winconfig");
        let file = pkg.join("settings.json");
        let path = build_win_path(&file, &pkg, &PkgType::WinConfig);
        assert!(path.starts_with("/mnt/c/Users/"));
        assert!(path.to_string_lossy().contains(".config"));
        assert!(path.ends_with("settings.json"));
    }

    #[test]
    fn build_win_path_winlocal() {
        let pkg = PathBuf::from("/tmp/myapp.winlocal");
        let file = pkg.join("app.cfg");
        let path = build_win_path(&file, &pkg, &PkgType::WinLocal);
        assert!(path.to_string_lossy().contains("AppData/Local"));
    }

    #[test]
    fn build_win_path_winroaming() {
        let pkg = PathBuf::from("/tmp/myapp.winroaming");
        let file = pkg.join("roam.dat");
        let path = build_win_path(&file, &pkg, &PkgType::WinRoaming);
        assert!(path.to_string_lossy().contains("AppData/Roaming"));
    }

    // ------------------------------------------------------------------
    // propose_name
    // ------------------------------------------------------------------

    #[test]
    fn propose_name_empty() {
        assert_eq!(propose_name(&[]), "unnamed");
    }

    #[test]
    fn propose_name_from_file_stem() {
        let name = propose_name(&[PathBuf::from("/home/user/.zshrc")]);
        assert_eq!(name, ".zshrc");
    }

    #[test]
    fn propose_name_file_uses_parent_dir() {
        let dir = temp_dir();
        let sub = dir.join("myapp");
        fs::create_dir_all(&sub).unwrap();
        write_file(&sub.join("config.yml"), "x");
        let name = propose_name(&[sub.join("config.yml")]);
        assert_eq!(name, "myapp");
    }

    #[test]
    fn propose_name_file_parent_is_home_falls_back_to_stem() {
        let home = std::env::var("HOME").unwrap();
        let name = propose_name(&[PathBuf::from(format!("{}/.zshrc", home))]);
        // ~/.zshrc is a file but parent is HOME → falls back to stem
        assert_eq!(name, ".zshrc");
    }

    #[test]
    fn propose_name_file_parent_is_dot_config_falls_back() {
        let dir = temp_dir();
        let dotconfig = dir.join(".config");
        let file = dotconfig.join("settings.yml");
        write_file(&file, "{}");
        // Override HOME to point to `dir` so the path looks like ~/.config/settings.yml
        // Can't override HOME per-test, so just test that .config parent falls back
        let name = propose_name(&[file]);
        // Parent is ".config" which is excluded → falls back to file_stem
        assert_eq!(name, "settings");
    }

    #[test]
    fn propose_name_file_parent_is_dot_local_falls_back() {
        let dir = temp_dir();
        let dotlocal = dir.join(".local");
        let file = dotlocal.join("state.json");
        write_file(&file, "{}");
        let name = propose_name(&[file]);
        assert_eq!(name, "state");
    }

    #[test]
    fn propose_name_deeply_nested_file_uses_immediate_parent() {
        let dir = temp_dir();
        let deep = dir.join("foo/bar/baz");
        fs::create_dir_all(&deep).unwrap();
        write_file(&deep.join("data.txt"), "x");
        let name = propose_name(&[deep.join("data.txt")]);
        assert_eq!(name, "baz");
    }

    // ------------------------------------------------------------------
    // detect_type
    // ------------------------------------------------------------------

    #[test]
    fn detect_type_config() {
        let t = detect_type(Path::new(
            &format!("{}/.config/alacritty", std::env::var("HOME").unwrap()),
        ));
        assert_eq!(t, PkgType::Config);
    }

    #[test]
    fn detect_type_user() {
        let t = detect_type(
            Path::new(&format!("{}/.bashrc", std::env::var("HOME").unwrap())),
        );
        assert_eq!(t, PkgType::User);
    }

    #[test]
    fn detect_type_local() {
        let t = detect_type(
            Path::new(&format!("{}/.local/share/app", std::env::var("HOME").unwrap())),
        );
        assert_eq!(t, PkgType::Local);
    }

    #[test]
    fn detect_type_root() {
        let t = detect_type(Path::new("/etc/hosts"));
        assert_eq!(t, PkgType::Root);
    }

    #[test]
    fn detect_type_meta_for_proc() {
        let t = detect_type(Path::new("/proc/cpuinfo"));
        assert_eq!(t, PkgType::Meta);
    }

    #[test]
    fn detect_type_winuser() {
        let t = detect_type(Path::new("/mnt/c/Users/john/Documents/notes.txt"));
        assert_eq!(t, PkgType::WinUser);
    }

    #[test]
    fn detect_type_winconfig() {
        let t = detect_type(Path::new("/mnt/c/Users/john/.config/pwsh/profile.ps1"));
        assert_eq!(t, PkgType::WinConfig);
    }

    #[test]
    fn detect_type_winroaming() {
        let t = detect_type(Path::new(
            "/mnt/c/Users/john/AppData/Roaming/Code/User/settings.json",
        ));
        assert_eq!(t, PkgType::WinRoaming);
    }
}
