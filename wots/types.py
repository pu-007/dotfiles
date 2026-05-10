"""Package type enumeration and helpers."""
from __future__ import annotations

from enum import Enum
from pathlib import Path
from typing import Optional

from . import config


class PkgType(Enum):
    """Dotfile package type."""

    USER    = "user"     # *.user/    → stow to $HOME
    ROOT    = "root"     # *.root/    → stow to / (sudo)
    WINUSER = "winuser"  # *.winuser/ → copy-sync to C:\Users\{user}
    META    = "meta"     # *.meta/    → manual
    MNT     = "mnt"      # c.mnt/     → legacy Windows mirror

    # ── properties ──────────────────────────────────────────────

    @property
    def suffix(self) -> str:
        """File-system suffix: .user, .root, .winuser, .meta, or '' for mnt."""
        return "" if self == PkgType.MNT else f".{self.value}"

    @property
    def stow_target(self) -> Optional[Path]:
        """Target directory for stow; None for non-stow types."""
        return {
            PkgType.USER: config.USER_TARGET,
            PkgType.ROOT: config.ROOT_TARGET,
        }.get(self)

    @property
    def needs_sudo(self) -> bool:
        return self == PkgType.ROOT

    @property
    def uses_stow(self) -> bool:
        """True for types managed by GNU Stow."""
        return self in (PkgType.USER, PkgType.ROOT)

    @property
    def uses_copy_sync(self) -> bool:
        """True for types managed by copy-based bidirectional sync."""
        return self in (PkgType.WINUSER, PkgType.MNT)


# ── helpers ───────────────────────────────────────────────────

def suffix_to_type(suffix: str) -> Optional[PkgType]:
    """Map a directory suffix (e.g. '.user') to PkgType."""
    mapping = {
        ".user":    PkgType.USER,
        ".root":    PkgType.ROOT,
        ".winuser": PkgType.WINUSER,
        ".meta":    PkgType.META,
    }
    return mapping.get(suffix)

def type_from_dir_name(name: str) -> Optional[PkgType]:
    """
    Infer PkgType from a directory name.
    Returns None for unrecognised names.
    """
    if name == config.WSL_MNT_BASE.name:
        return PkgType.MNT
    for suffix, pt in {
        ".user": PkgType.USER,
        ".root": PkgType.ROOT,
        ".winuser": PkgType.WINUSER,
        ".meta": PkgType.META,
    }.items():
        if name.endswith(suffix) and len(name) > len(suffix):
            return pt
    return None
