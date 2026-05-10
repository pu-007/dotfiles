"""Package type enumeration and helpers."""
from __future__ import annotations

from enum import Enum
from pathlib import Path
from typing import Optional

from . import config


class PkgType(Enum):
    """Dotfile package type — suffix determines target directory."""

    # ── Linux ──────────────────────────────────────────────────
    USER   = "user"       # ~/
    CONFIG = "config"     # ~/.config/
    LOCAL  = "local"      # ~/.local/
    ROOT   = "root"       # / (sudo)
    META   = "meta"       # manual only

    # ── Windows ────────────────────────────────────────────────
    WINUSER    = "winuser"     # C:\Users\{name}\
    WINCONFIG  = "winconfig"   # C:\Users\{name}\.config\
    WINLOCAL   = "winlocal"    # C:\Users\{name}\AppData\Local\
    WINROAMING = "winroaming"  # C:\Users\{name}\AppData\Roaming\

    # ── Legacy ─────────────────────────────────────────────────
    MNT = "mnt"          # c.mnt/  →  C:\  (full drive mirror)

    # ── properties ──────────────────────────────────────────────

    @property
    def suffix(self) -> str:
        return "" if self == PkgType.MNT else f".{self.value}"

    @property
    def sync_target(self) -> Optional[Path]:
        """Target directory for this type (stow or copy-sync)."""
        return {
            PkgType.USER:       config.USER_TARGET,
            PkgType.CONFIG:     config.CONFIG_TARGET,
            PkgType.LOCAL:      config.LOCAL_TARGET,
            PkgType.ROOT:       config.ROOT_TARGET,
            PkgType.WINUSER:    config.WINUSER_TARGET,
            PkgType.WINCONFIG:  config.WINCONFIG_TARGET,
            PkgType.WINLOCAL:   config.WINLOCAL_TARGET,
            PkgType.WINROAMING: config.WINROAMING_TARGET,
            # MNT targets the whole C:\ drive
        }.get(self)

    @property
    def needs_sudo(self) -> bool:
        return self == PkgType.ROOT

    @property
    def uses_stow(self) -> bool:
        """Linux types managed by GNU Stow."""
        return self in (PkgType.USER, PkgType.CONFIG, PkgType.LOCAL, PkgType.ROOT)

    @property
    def uses_copy_sync(self) -> bool:
        """Windows types managed by copy-based bidirectional sync."""
        return self in (
            PkgType.WINUSER, PkgType.WINCONFIG,
            PkgType.WINLOCAL, PkgType.WINROAMING, PkgType.MNT,
        )

    @property
    def is_linux(self) -> bool:
        return self in (PkgType.USER, PkgType.CONFIG, PkgType.LOCAL, PkgType.ROOT, PkgType.META)

    @property
    def is_windows(self) -> bool:
        return self in (
            PkgType.WINUSER, PkgType.WINCONFIG,
            PkgType.WINLOCAL, PkgType.WINROAMING, PkgType.MNT,
        )

    @property
    def is_linux_config(self) -> bool:
        """True for types that target $HOME sub-paths (not /)."""
        return self in (PkgType.USER, PkgType.CONFIG, PkgType.LOCAL)


# ── helpers ───────────────────────────────────────────────────

def suffix_to_type(suffix: str) -> Optional[PkgType]:
    """Map a directory suffix ('.config') to PkgType."""
    for pt in PkgType:
        if pt.suffix == suffix and suffix:
            return pt
    return None

def type_from_dir_name(name: str) -> Optional[PkgType]:
    """Infer PkgType from a directory name, or None."""
    if name == config.WSL_MNT_BASE.name:
        return PkgType.MNT
    for pt in PkgType:
        s = pt.suffix
        if s and name.endswith(s) and len(name) > len(s):
            return pt
    return None

# ── human-readable labels ─────────────────────────────────────

_TYPE_LABELS: dict[PkgType, str] = {
    PkgType.USER:       "~",
    PkgType.CONFIG:     "~/.config",
    PkgType.LOCAL:      "~/.local",
    PkgType.ROOT:       "/",
    PkgType.META:       "manual",
    PkgType.WINUSER:    f"C:\\Users\\{config.WIN_USERNAME}",
    PkgType.WINCONFIG:  f"C:\\Users\\{config.WIN_USERNAME}\\.config",
    PkgType.WINLOCAL:   f"C:\\Users\\{config.WIN_USERNAME}\\AppData\\Local",
    PkgType.WINROAMING: f"C:\\Users\\{config.WIN_USERNAME}\\AppData\\Roaming",
    PkgType.MNT:        "C:\\",
}

def type_label(pt: PkgType) -> str:
    """Human-readable target for a package type."""
    return _TYPE_LABELS.get(pt, pt.value)
