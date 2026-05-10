"""Package discovery and type auto-detection."""
from __future__ import annotations

from pathlib import Path
from typing import Dict, List

from . import config
from .types import PkgType, type_from_dir_name


# ── type auto-detection ───────────────────────────────────────

def detect_type(path: Path) -> PkgType:
    """
    Auto-detect package type from a source path.

    Rules (first match wins):
      /mnt/c/Users/{WIN_USERNAME}  →  WINUSER
      /mnt/c/                      →  MNT
      $HOME                        →  USER
      /etc                         →  ROOT
      everything else              →  META
    """
    rp = path.resolve()

    # Windows user profile
    winuser_root = config.MNT_C / "Users" / config.WIN_USERNAME
    try:
        rp.relative_to(winuser_root)
        return PkgType.WINUSER
    except ValueError:
        pass

    # Generic /mnt/c
    try:
        rp.relative_to(config.MNT_C)
        return PkgType.MNT
    except ValueError:
        pass

    # $HOME
    home = Path.home()
    try:
        rp.relative_to(home)
        if rp != home:
            return PkgType.USER
    except ValueError:
        pass

    # /etc
    try:
        rp.relative_to(Path("/etc"))
        return PkgType.ROOT
    except ValueError:
        pass

    return PkgType.META


# ── package scanning ──────────────────────────────────────────

def find_packages(base: Path | None = None) -> Dict[PkgType, List[Path]]:
    """
    Scan *base* (default DOTFILES_DIR) and return packages grouped by type.
    """
    base = base or config.DOTFILES_DIR
    result: Dict[PkgType, List[Path]] = {t: [] for t in PkgType}

    if not base.is_dir():
        return result

    for entry in sorted(base.iterdir()):
        if not entry.is_dir() or entry.name.startswith("."):
            continue
        pt = type_from_dir_name(entry.name)
        if pt is not None:
            result[pt].append(entry)

    return result


def pkg_basename(pkg_path: Path) -> str:
    """Return the human name of a package (strip suffix)."""
    name = pkg_path.name
    # MNT is special — it's the whole c.mnt dir
    if name == config.WSL_MNT_BASE.name:
        return "c.mnt"
    pt = type_from_dir_name(name)
    if pt and pt.suffix:
        return name[: -len(pt.suffix)]
    return name


# ── file listing helpers (for sync targets) ───────────────────

def list_syncable_files(pkg: Path, pt: PkgType) -> List[Path]:
    """
    Return all files inside *pkg* that should be synced.
    Excludes hidden files, .git, and config.EXCLUDE_PATTERNS.
    """
    from .utils import is_excluded
    files: List[Path] = []
    if not pkg.is_dir():
        return files
    for f in pkg.rglob("*"):
        if not f.is_file():
            continue
        if f.name.startswith("."):
            # allow dotfiles like .gitconfig, .zshrc
            # but skip .git/ contents (already excluded)
            pass
        if is_excluded(f):
            continue
        files.append(f)
    return files


def winuser_rel_path(pkg: Path, file_path: Path) -> Path:
    """
    For a .winuser package, compute the path relative to
    C:\\Users\\{WIN_USERNAME}.

    Package structure:
      {name}.winuser/
        {WIN_USERNAME}/
          .gitconfig          → C:\\Users\\{WIN_USERNAME}\\.gitconfig
          AppData/
            Roaming/
              foo.ini         → C:\\Users\\{WIN_USERNAME}\\AppData\\Roaming\\foo.ini

    The first directory inside the package is expected to be {WIN_USERNAME}.
    """
    # Find the WIN_USERNAME root inside the package
    pkg_root = pkg
    user_dir = pkg_root / config.WIN_USERNAME
    if user_dir.is_dir():
        try:
            return file_path.relative_to(user_dir)
        except ValueError:
            pass
    # Fallback: relative to package root
    return file_path.relative_to(pkg)
