use std::path::{Path, PathBuf};

use clap::ValueEnum;

use crate::config::{
    CONFIG_TARGET, LOCAL_TARGET, ROOT_TARGET, USER_TARGET, WIN_USERNAME, WINCONFIG_TARGET,
    WINLOCAL_TARGET, WINROAMING_TARGET, WINUSER_TARGET,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum PkgType {
    #[value(name = "user")]
    User,
    #[value(name = "config")]
    Config,
    #[value(name = "local")]
    Local,
    #[value(name = "root")]
    Root,
    #[value(name = "meta")]
    Meta,
    #[value(name = "winuser")]
    WinUser,
    #[value(name = "winconfig")]
    WinConfig,
    #[value(name = "winlocal")]
    WinLocal,
    #[value(name = "winroaming")]
    WinRoaming,
}

impl std::str::FromStr for PkgType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "user" => Ok(PkgType::User),
            "config" => Ok(PkgType::Config),
            "local" => Ok(PkgType::Local),
            "root" => Ok(PkgType::Root),
            "meta" => Ok(PkgType::Meta),
            "winuser" => Ok(PkgType::WinUser),
            "winconfig" => Ok(PkgType::WinConfig),
            "winlocal" => Ok(PkgType::WinLocal),
            "winroaming" => Ok(PkgType::WinRoaming),
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
            PkgType::Local => "local",
            PkgType::Root => "root",
            PkgType::Meta => "meta",
            PkgType::WinUser => "winuser",
            PkgType::WinConfig => "winconfig",
            PkgType::WinLocal => "winlocal",
            PkgType::WinRoaming => "winroaming",
        }
    }

    pub fn sync_target(&self) -> Option<PathBuf> {
        match self {
            PkgType::User => Some(USER_TARGET.clone()),
            PkgType::Config => Some(CONFIG_TARGET.clone()),
            PkgType::Local => Some(LOCAL_TARGET.clone()),
            PkgType::Root => Some(ROOT_TARGET.clone()),
            PkgType::WinUser => Some(WINUSER_TARGET.clone()),
            PkgType::WinConfig => Some(WINCONFIG_TARGET.clone()),
            PkgType::WinLocal => Some(WINLOCAL_TARGET.clone()),
            PkgType::WinRoaming => Some(WINROAMING_TARGET.clone()),
            PkgType::Meta => None,
        }
    }

    pub fn needs_sudo(&self) -> bool {
        matches!(self, PkgType::Root)
    }

    pub fn uses_stow(&self) -> bool {
        matches!(self, PkgType::User | PkgType::Config | PkgType::Local | PkgType::Root)
    }

    pub fn uses_copy_sync(&self) -> bool {
        matches!(
            self,
            PkgType::WinUser | PkgType::WinConfig | PkgType::WinLocal | PkgType::WinRoaming
        )
    }

    pub fn is_windows(&self) -> bool {
        matches!(
            self,
            PkgType::WinUser | PkgType::WinConfig | PkgType::WinLocal | PkgType::WinRoaming
        )
    }

    pub fn is_linux_config(&self) -> bool {
        matches!(self, PkgType::User | PkgType::Config | PkgType::Local)
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
        PkgType::Local => "~/.local".into(),
        PkgType::Root => "/".into(),
        PkgType::Meta => "manual".into(),
        PkgType::WinUser => name.clone(),
        PkgType::WinConfig => format!("{name}\\.config"),
        PkgType::WinLocal => format!("{name}\\AppData\\Local"),
        PkgType::WinRoaming => format!("{name}\\AppData\\Roaming"),
    }
}

pub const ALL_TYPES: [PkgType; 9] = [
    PkgType::User,
    PkgType::Config,
    PkgType::Local,
    PkgType::Root,
    PkgType::Meta,
    PkgType::WinUser,
    PkgType::WinConfig,
    PkgType::WinLocal,
    PkgType::WinRoaming,
];

pub const SYNCABLE_TYPES: [PkgType; 8] = [
    PkgType::User,
    PkgType::Config,
    PkgType::Local,
    PkgType::Root,
    PkgType::WinUser,
    PkgType::WinConfig,
    PkgType::WinLocal,
    PkgType::WinRoaming,
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
        assert_eq!("local".parse::<PkgType>().unwrap(), PkgType::Local);
        assert_eq!("root".parse::<PkgType>().unwrap(), PkgType::Root);
        assert_eq!("meta".parse::<PkgType>().unwrap(), PkgType::Meta);
        assert_eq!("winuser".parse::<PkgType>().unwrap(), PkgType::WinUser);
        assert_eq!(
            "winconfig".parse::<PkgType>().unwrap(),
            PkgType::WinConfig
        );
        assert_eq!(
            "winlocal".parse::<PkgType>().unwrap(),
            PkgType::WinLocal
        );
        assert_eq!(
            "winroaming".parse::<PkgType>().unwrap(),
            PkgType::WinRoaming
        );
    }

    #[test]
    fn from_str_invalid() {
        assert!("bogus".parse::<PkgType>().is_err());
        assert!("".parse::<PkgType>().is_err());
    }

    #[test]
    fn suffix() {
        assert_eq!(PkgType::User.suffix(), ".user");
        assert_eq!(PkgType::Config.suffix(), ".config");
        assert_eq!(PkgType::Local.suffix(), ".local");
        assert_eq!(PkgType::Root.suffix(), ".root");
        assert_eq!(PkgType::Meta.suffix(), ".meta");
        assert_eq!(PkgType::WinUser.suffix(), ".winuser");
        assert_eq!(PkgType::WinConfig.suffix(), ".winconfig");
        assert_eq!(PkgType::WinLocal.suffix(), ".winlocal");
        assert_eq!(PkgType::WinRoaming.suffix(), ".winroaming");
    }

    #[test]
    fn value() {
        assert_eq!(PkgType::User.value(), "user");
        assert_eq!(PkgType::Config.value(), "config");
        assert_eq!(PkgType::Local.value(), "local");
        assert_eq!(PkgType::Root.value(), "root");
        assert_eq!(PkgType::Meta.value(), "meta");
        assert_eq!(PkgType::WinUser.value(), "winuser");
        assert_eq!(PkgType::WinConfig.value(), "winconfig");
        assert_eq!(PkgType::WinLocal.value(), "winlocal");
        assert_eq!(PkgType::WinRoaming.value(), "winroaming");
    }

    #[test]
    fn needs_sudo_only_root() {
        assert!(!PkgType::User.needs_sudo());
        assert!(PkgType::Root.needs_sudo());
        assert!(!PkgType::WinUser.needs_sudo());
        assert!(!PkgType::Meta.needs_sudo());
    }

    #[test]
    fn uses_stow_linux_types() {
        assert!(PkgType::User.uses_stow());
        assert!(PkgType::Config.uses_stow());
        assert!(PkgType::Local.uses_stow());
        assert!(PkgType::Root.uses_stow());
        assert!(!PkgType::Meta.uses_stow());
        assert!(!PkgType::WinUser.uses_stow());
    }

    #[test]
    fn uses_copy_sync_windows_types() {
        assert!(!PkgType::User.uses_copy_sync());
        assert!(PkgType::WinUser.uses_copy_sync());
        assert!(PkgType::WinConfig.uses_copy_sync());
        assert!(PkgType::WinLocal.uses_copy_sync());
        assert!(PkgType::WinRoaming.uses_copy_sync());
    }

    #[test]
    fn is_windows() {
        assert!(PkgType::WinUser.is_windows());
        assert!(PkgType::WinConfig.is_windows());
        assert!(!PkgType::User.is_windows());
        assert!(!PkgType::Config.is_windows());
    }

    #[test]
    fn is_linux_config() {
        assert!(PkgType::User.is_linux_config());
        assert!(PkgType::Config.is_linux_config());
        assert!(PkgType::Local.is_linux_config());
        assert!(!PkgType::Root.is_linux_config());
        assert!(!PkgType::WinUser.is_linux_config());
    }

    #[test]
    fn sync_target_user_is_home() {
        let t = PkgType::User.sync_target().unwrap();
        assert!(t.to_string_lossy().contains(std::env::var("HOME").unwrap().as_str()));
    }

    #[test]
    fn sync_target_meta_is_none() {
        assert!(PkgType::Meta.sync_target().is_none());
    }

    #[test]
    fn type_from_dir_name_recognizes_suffixes() {
        assert_eq!(
            type_from_dir_name("git.config"),
            Some(PkgType::Config)
        );
        assert_eq!(
            type_from_dir_name("foo.user"),
            Some(PkgType::User)
        );
        assert_eq!(
            type_from_dir_name("bar.local"),
            Some(PkgType::Local)
        );
        assert_eq!(
            type_from_dir_name("baz.root"),
            Some(PkgType::Root)
        );
        assert_eq!(
            type_from_dir_name("qux.meta"),
            Some(PkgType::Meta)
        );
        assert_eq!(
            type_from_dir_name("myapp.winuser"),
            Some(PkgType::WinUser)
        );
        assert_eq!(
            type_from_dir_name("myapp.winconfig"),
            Some(PkgType::WinConfig)
        );
        assert_eq!(
            type_from_dir_name("myapp.winlocal"),
            Some(PkgType::WinLocal)
        );
        assert_eq!(
            type_from_dir_name("myapp.winroaming"),
            Some(PkgType::WinRoaming)
        );
    }

    #[test]
    fn type_from_dir_name_no_suffix() {
        assert_eq!(type_from_dir_name("justaname"), None);
        assert_eq!(type_from_dir_name(".hidden"), None);
    }

    #[test]
    fn type_label_contains_user_home() {
        let u = PkgType::User;
        assert_eq!(type_label(u), "~");

        let c = PkgType::Config;
        assert_eq!(type_label(c), "~/.config");

        let l = PkgType::Local;
        assert_eq!(type_label(l), "~/.local");
    }

    #[test]
    fn type_label_windows_contains_users() {
        for pt in [
            PkgType::WinUser,
            PkgType::WinConfig,
            PkgType::WinLocal,
            PkgType::WinRoaming,
        ] {
            let label = type_label(pt);
            assert!(label.contains("Users"), "label for {pt:?} missing 'Users': {label}");
        }
    }

    #[test]
    fn all_types_has_nine() {
        assert_eq!(ALL_TYPES.len(), 9);
    }

    #[test]
    fn syncable_types_has_eight() {
        assert_eq!(SYNCABLE_TYPES.len(), 8);
        // Meta should not be syncable
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
        assert_eq!(parse_app_arg("nvim.local"), (Some(PkgType::Local), "nvim".into()));
        assert_eq!(parse_app_arg("wsl.root"), (Some(PkgType::Root), "wsl".into()));
        assert_eq!(parse_app_arg("pkg.meta"), (Some(PkgType::Meta), "pkg".into()));
        assert_eq!(parse_app_arg("pwsh.winuser"), (Some(PkgType::WinUser), "pwsh".into()));
        assert_eq!(parse_app_arg("nvim.winconfig"), (Some(PkgType::WinConfig), "nvim".into()));
        assert_eq!(parse_app_arg("app.winlocal"), (Some(PkgType::WinLocal), "app".into()));
        assert_eq!(parse_app_arg("code.winroaming"), (Some(PkgType::WinRoaming), "code".into()));
    }

    #[test]
    fn parse_app_arg_without_suffix() {
        assert_eq!(parse_app_arg("zsh"), (None, "zsh".into()));
        assert_eq!(parse_app_arg("git"), (None, "git".into()));
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
        assert_eq!(parse_app_arg("./zsh/"), (None, "zsh".into()));
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
