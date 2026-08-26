use std::path::{Path, PathBuf};

use clap::ValueEnum;

use crate::config::{
    CONFIG_TARGET, ROOT_TARGET, USER_TARGET, WINROOT_TARGET, WIN_USERNAME, WINUSER_TARGET,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum PkgType {
    #[value(name = "user")]
    User,
    #[value(name = "config")]
    Config,
    #[value(name = "root")]
    Root,
    #[value(name = "meta")]
    Meta,
    #[value(name = "winuser")]
    WinUser,
    #[value(name = "winroot")]
    WinRoot,
}

impl std::str::FromStr for PkgType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "user" => Ok(PkgType::User),
            "config" => Ok(PkgType::Config),
            "root" => Ok(PkgType::Root),
            "meta" => Ok(PkgType::Meta),
            "winuser" => Ok(PkgType::WinUser),
            "winroot" => Ok(PkgType::WinRoot),
            _ => Err(format!("unknown package type: {}", s)),
        }
    }
}

impl PkgType {
    pub fn suffix(&self) -> String {
        format!(".{}", self.value())
    }

    pub fn value(&self) -> &'static str {
        match self {
            PkgType::User => "user",
            PkgType::Config => "config",
            PkgType::Root => "root",
            PkgType::Meta => "meta",
            PkgType::WinUser => "winuser",
            PkgType::WinRoot => "winroot",
        }
    }

    pub fn sync_target(&self) -> Option<PathBuf> {
        match self {
            PkgType::User => Some(USER_TARGET.clone()),
            PkgType::Config => Some(CONFIG_TARGET.clone()),
            PkgType::Root => Some(ROOT_TARGET.clone()),
            PkgType::WinUser => Some(WINUSER_TARGET.clone()),
            PkgType::WinRoot => Some(WINROOT_TARGET.clone()),
            PkgType::Meta => None,
        }
    }

    pub fn needs_sudo(&self) -> bool {
        matches!(self, PkgType::Root)
    }

    pub fn uses_stow(&self) -> bool {
        matches!(self, PkgType::User | PkgType::Config | PkgType::Root)
    }

    pub fn uses_copy_sync(&self) -> bool {
        matches!(self, PkgType::WinUser | PkgType::WinRoot)
    }

    pub fn is_windows(&self) -> bool {
        matches!(self, PkgType::WinUser | PkgType::WinRoot)
    }

    pub fn is_linux_config(&self) -> bool {
        matches!(self, PkgType::User | PkgType::Config)
    }
}

pub fn type_from_dir_name(name: &str) -> Option<PkgType> {
    for pt in ALL_TYPES.iter() {
        let s = pt.suffix();
        if name.ends_with(&s) && name.len() > s.len() {
            return Some(*pt);
        }
    }
    None
}

pub fn parse_app_arg(raw: &str) -> (Option<PkgType>, String) {
    let name = Path::new(raw)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(raw);
    if name.is_empty() {
        return (None, raw.to_string());
    }
    if let Some(pt) = type_from_dir_name(name) {
        let suffix = pt.suffix();
        if name.len() > suffix.len() {
            return (Some(pt), name[..name.len() - suffix.len()].to_string());
        }
    }
    (None, name.to_string())
}

pub fn type_label(pt: PkgType) -> String {
    let name = format!("C:\\Users\\{}", WIN_USERNAME.as_deref().unwrap_or("user"));
    match pt {
        PkgType::User => "~".into(),
        PkgType::Config => "~/.config".into(),
        PkgType::Root => "/".into(),
        PkgType::Meta => "manual".into(),
        PkgType::WinUser => name,
        PkgType::WinRoot => "C:\\".into(),
    }
}

pub const ALL_TYPES: [PkgType; 6] = [
    PkgType::User,
    PkgType::Config,
    PkgType::Root,
    PkgType::Meta,
    PkgType::WinUser,
    PkgType::WinRoot,
];

pub const SYNCABLE_TYPES: [PkgType; 5] = [
    PkgType::User,
    PkgType::Config,
    PkgType::Root,
    PkgType::WinUser,
    PkgType::WinRoot,
];

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_valid() {
        assert_eq!("user".parse::<PkgType>().unwrap(), PkgType::User);
        assert_eq!("config".parse::<PkgType>().unwrap(), PkgType::Config);
        assert_eq!("root".parse::<PkgType>().unwrap(), PkgType::Root);
        assert_eq!("meta".parse::<PkgType>().unwrap(), PkgType::Meta);
        assert_eq!("winuser".parse::<PkgType>().unwrap(), PkgType::WinUser);
        assert_eq!("winroot".parse::<PkgType>().unwrap(), PkgType::WinRoot);
    }

    #[test]
    fn from_str_removed_types_are_invalid() {
        assert!("local".parse::<PkgType>().is_err());
        assert!("winlocal".parse::<PkgType>().is_err());
        assert!("winroaming".parse::<PkgType>().is_err());
    }

    #[test]
    fn from_str_invalid() {
        assert!("bogus".parse::<PkgType>().is_err());
        assert!("".parse::<PkgType>().is_err());
    }

    #[test]
    fn suffix() {
        for pt in ALL_TYPES {
            assert_eq!(pt.suffix(), format!(".{}", pt.value()));
        }
    }

    #[test]
    fn needs_sudo_only_root() {
        assert!(!PkgType::User.needs_sudo());
        assert!(!PkgType::Config.needs_sudo());
        assert!(PkgType::Root.needs_sudo());
        assert!(!PkgType::Meta.needs_sudo());
        assert!(!PkgType::WinUser.needs_sudo());
        // /mnt/c writes do not require sudo under the default WSL mount.
        assert!(!PkgType::WinRoot.needs_sudo());
    }

    #[test]
    fn uses_stow_linux_types() {
        assert!(PkgType::User.uses_stow());
        assert!(PkgType::Config.uses_stow());
        assert!(PkgType::Root.uses_stow());
        assert!(!PkgType::Meta.uses_stow());
        assert!(!PkgType::WinUser.uses_stow());
        assert!(!PkgType::WinRoot.uses_stow());
    }

    #[test]
    fn uses_copy_sync_windows_types() {
        assert!(!PkgType::User.uses_copy_sync());
        assert!(!PkgType::Config.uses_copy_sync());
        assert!(!PkgType::Root.uses_copy_sync());
        assert!(!PkgType::Meta.uses_copy_sync());
        assert!(PkgType::WinUser.uses_copy_sync());
        assert!(PkgType::WinRoot.uses_copy_sync());
    }

    #[test]
    fn is_windows() {
        assert!(PkgType::WinUser.is_windows());
        assert!(PkgType::WinRoot.is_windows());
        assert!(!PkgType::User.is_windows());
        assert!(!PkgType::Config.is_windows());
        assert!(!PkgType::Root.is_windows());
        assert!(!PkgType::Meta.is_windows());
    }

    #[test]
    fn is_linux_config() {
        assert!(PkgType::User.is_linux_config());
        assert!(PkgType::Config.is_linux_config());
        assert!(!PkgType::Root.is_linux_config());
        assert!(!PkgType::Meta.is_linux_config());
        assert!(!PkgType::WinUser.is_linux_config());
        assert!(!PkgType::WinRoot.is_linux_config());
    }

    #[test]
    fn sync_target_meta_is_none() {
        assert!(PkgType::Meta.sync_target().is_none());
        assert!(ALL_TYPES.iter().filter(|t| t.sync_target().is_none()).count() == 1);
    }

    #[test]
    fn sync_target_winroot_is_c_drive() {
        let t = PkgType::WinRoot.sync_target().unwrap();
        assert_eq!(t.to_string_lossy(), "C:/");
    }

    #[test]
    fn type_from_dir_name_recognizes_suffixes() {
        assert_eq!(type_from_dir_name("git.config"), Some(PkgType::Config));
        assert_eq!(type_from_dir_name("foo.user"), Some(PkgType::User));
        assert_eq!(type_from_dir_name("baz.root"), Some(PkgType::Root));
        assert_eq!(type_from_dir_name("qux.meta"), Some(PkgType::Meta));
        assert_eq!(type_from_dir_name("myapp.winuser"), Some(PkgType::WinUser));
        assert_eq!(type_from_dir_name("myapp.winroot"), Some(PkgType::WinRoot));
    }

    #[test]
    fn type_from_dir_name_removed_suffixes_are_unknown() {
        assert_eq!(type_from_dir_name("bar.local"), None);
        assert_eq!(type_from_dir_name("myapp.winlocal"), None);
        assert_eq!(type_from_dir_name("myapp.winroaming"), None);
    }

    #[test]
    fn type_from_dir_name_no_suffix() {
        assert_eq!(type_from_dir_name("justaname"), None);
        assert_eq!(type_from_dir_name(".hidden"), None);
    }

    #[test]
    fn type_label_contains_user_home() {
        assert_eq!(type_label(PkgType::User), "~");
        assert_eq!(type_label(PkgType::Config), "~/.config");
        assert_eq!(type_label(PkgType::Root), "/");
    }

    #[test]
    fn type_label_windows_contains_users() {
        assert!(type_label(PkgType::WinUser).contains("Users"));
        assert_eq!(type_label(PkgType::WinRoot), "C:\\");
    }

    #[test]
    fn all_types_has_six() {
        assert_eq!(ALL_TYPES.len(), 6);
    }

    #[test]
    fn syncable_types_excludes_meta() {
        assert_eq!(SYNCABLE_TYPES.len(), 5);
        assert!(!SYNCABLE_TYPES.contains(&PkgType::Meta));
    }

    #[test]
    fn value_roundtrip() {
        for pt in ALL_TYPES {
            assert_eq!(pt.value().parse::<PkgType>().unwrap(), pt);
        }
    }

    #[test]
    fn parse_app_arg_with_suffix() {
        assert_eq!(parse_app_arg("git.config"), (Some(PkgType::Config), "git".into()));
        assert_eq!(parse_app_arg("zsh.user"), (Some(PkgType::User), "zsh".into()));
        assert_eq!(parse_app_arg("wsl.root"), (Some(PkgType::Root), "wsl".into()));
        assert_eq!(parse_app_arg("pkg.meta"), (Some(PkgType::Meta), "pkg".into()));
        assert_eq!(parse_app_arg("pwsh.winuser"), (Some(PkgType::WinUser), "pwsh".into()));
        assert_eq!(parse_app_arg("boot.winroot"), (Some(PkgType::WinRoot), "boot".into()));
    }

    #[test]
    fn parse_app_arg_removed_suffix_falls_back_to_name() {
        // Removed suffixes are treated as plain names (no type detected).
        assert_eq!(parse_app_arg("nvim.local"), (None, "nvim.local".into()));
        assert_eq!(parse_app_arg("app.winlocal"), (None, "app.winlocal".into()));
    }

    #[test]
    fn parse_app_arg_without_suffix() {
        assert_eq!(parse_app_arg("zsh"), (None, "zsh".into()));
        assert_eq!(parse_app_arg(""), (None, "".into()));
    }

    #[test]
    fn parse_app_arg_trailing_slash() {
        assert_eq!(parse_app_arg("git.config/"), (Some(PkgType::Config), "git".into()));
        assert_eq!(parse_app_arg("pwsh.winuser/"), (Some(PkgType::WinUser), "pwsh".into()));
        assert_eq!(parse_app_arg("zsh/"), (None, "zsh".into()));
    }

    #[test]
    fn parse_app_arg_dot_slash_prefix() {
        assert_eq!(parse_app_arg("./im-select.winuser/"), (Some(PkgType::WinUser), "im-select".into()));
        assert_eq!(parse_app_arg("./git.config/"), (Some(PkgType::Config), "git".into()));
        assert_eq!(parse_app_arg("./git.config"), (Some(PkgType::Config), "git".into()));
    }

    #[test]
    fn parse_app_arg_parent_dir_prefix() {
        assert_eq!(parse_app_arg("../dotfiles/git.config"), (Some(PkgType::Config), "git".into()));
        assert_eq!(parse_app_arg("../../foo/pwsh.winuser/"), (Some(PkgType::WinUser), "pwsh".into()));
    }

    #[test]
    fn parse_app_arg_multiple_trailing_slashes() {
        assert_eq!(parse_app_arg("git.config//"), (Some(PkgType::Config), "git".into()));
        assert_eq!(parse_app_arg("zsh//"), (None, "zsh".into()));
    }

    #[test]
    fn parse_app_arg_only_path_separators() {
        assert_eq!(parse_app_arg("/"), (None, "/".into()));
        assert_eq!(parse_app_arg("///"), (None, "///".into()));
        assert_eq!(parse_app_arg("./"), (None, "./".into()));
    }
}
