use std::path::PathBuf;

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

impl PkgType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "user" => Some(PkgType::User),
            "config" => Some(PkgType::Config),
            "local" => Some(PkgType::Local),
            "root" => Some(PkgType::Root),
            "meta" => Some(PkgType::Meta),
            "winuser" => Some(PkgType::WinUser),
            "winconfig" => Some(PkgType::WinConfig),
            "winlocal" => Some(PkgType::WinLocal),
            "winroaming" => Some(PkgType::WinRoaming),
            _ => None,
        }
    }

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
