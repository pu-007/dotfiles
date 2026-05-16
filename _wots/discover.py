"""Package discovery and type auto-detection."""
from __future__ import annotations

import os
from pathlib import Path
from typing import Dict, List

from . import config
from .types import PkgType, type_from_dir_name


# ── type auto-detection ───────────────────────────────────────

def detect_type(path: Path) -> PkgType:
    """
    Auto-detect package type from a source path.

    Windows (first match wins):
      /mnt/c/Users/{name}/AppData/Roaming/*  →  WINROAMING
      /mnt/c/Users/{name}/AppData/Local/*    →  WINLOCAL
      /mnt/c/Users/{name}/.config/*          →  WINCONFIG
      /mnt/c/Users/{name}/*                  →  WINUSER
      /mnt/c/* (not Users)                   →  META (manual)

    Linux:
      ~/.config/*   →  CONFIG
      ~/.local/*    →  LOCAL
      ~/*           →  USER
      /etc/*        →  ROOT
      other         →  META
    """
    rp = path.resolve()

    # ── Windows ────────────────────────────────────────────────
    try:
        rp.relative_to(config.MNT_C)
        parts = Path(*rp.relative_to(config.MNT_C).parts)

        if len(parts.parts) >= 3 and parts.parts[0] == "Users":
            sub = Path(*parts.parts[2:]) if len(parts.parts) > 2 else Path(".")

            # AppData\Roaming
            roaming = Path("AppData/Roaming")
            try:
                sub.relative_to(roaming)
                return PkgType.WINROAMING
            except ValueError:
                pass

            # AppData\Local
            loc = Path("AppData/Local")
            try:
                sub.relative_to(loc)
                return PkgType.WINLOCAL
            except ValueError:
                pass

            # .config
            if len(parts.parts) >= 3 and parts.parts[2] == ".config":
                return PkgType.WINCONFIG

            return PkgType.WINUSER

        return PkgType.META
    except ValueError:
        pass

    # ── Linux ──────────────────────────────────────────────────
    home = Path.home()
    try:
        rel = rp.relative_to(home)
        if rp == home:
            return PkgType.USER
        parts = rel.parts
        if len(parts) >= 1:
            if parts[0] == ".config":
                return PkgType.CONFIG
            if parts[0] == ".local":
                return PkgType.LOCAL
        return PkgType.USER
    except ValueError:
        pass

    try:
        rp.relative_to(Path("/etc"))
        return PkgType.ROOT
    except ValueError:
        pass

    return PkgType.META


# ── package scanning ──────────────────────────────────────────

def find_packages(base: Path | None = None) -> Dict[PkgType, List[Path]]:
    """Scan *base* (default DOTFILES_DIR) → packages grouped by type."""
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
    """Human-readable package name (strip suffix)."""
    name = pkg_path.name
    pt = type_from_dir_name(name)
    if pt and pt.suffix:
        return name[: -len(pt.suffix)]
    return name


# ── file listing ──────────────────────────────────────────────

def list_syncable_files(pkg: Path) -> List[Path]:
    """Return all syncable files inside *pkg*, skipping excluded patterns.
    Uses os.scandir for performance."""
    from .utils import is_excluded
    files: List[Path] = []
    if not pkg.is_dir():
        return files
    for dirpath, dirnames, filenames in os.walk(pkg):
        # filter excluded dirs in-place to skip traversal
        dirnames[:] = [d for d in dirnames if not _quick_exclude(d)]
        dp = Path(dirpath)
        for fn in filenames:
            fp = dp / fn
            if is_excluded(fp):
                continue
            files.append(fp)
    return files


def _quick_exclude(dirname: str) -> bool:
    """Fast exclusion check for directory names (no fnmatch overhead)."""
    return dirname in {".git", "__pycache__", "node_modules", ".mypy_cache",
                       ".ruff_cache", ".pixi"}


# ── winuser relative path (no username nesting) ───────────────

def winuser_rel_path(pkg: Path, file_path: Path) -> Path:
    """
    For a .winuser / .winconfig / .winroaming / .winlocal package,
    compute the path relative to the package root.

    The file tree inside the package maps directly to the Windows
    target — no {WIN_USERNAME}/ nesting.

      git.winuser/
        .gitconfig        →  C:\\Users\\zion\\.gitconfig
        AppData/
          Roaming/
            pip/pip.ini   →  C:\\Users\\zion\\AppData\\Roaming\\pip\\pip.ini

    For legacy packages that still have {WIN_USERNAME}/ nesting,
    that layer is stripped automatically.
    """
    user_dir = pkg / config.WIN_USERNAME
    if user_dir.is_dir():
        try:
            return file_path.relative_to(user_dir)
        except ValueError:
            pass
    return file_path.relative_to(pkg)


# ── Windows path mapping ──────────────────────────────────────

def build_win_path(file_path: Path, pkg: Path, pt: PkgType) -> Path:
    """
    Map a file inside a Windows-type package to its /mnt/c target path.

      pkg=git.winuser, file=.gitconfig  →  /mnt/c/Users/{WIN_USERNAME}/.gitconfig
      pkg=nvim.winconfig, file=init.lua →  /mnt/c/Users/{WIN_USERNAME}/.config/nvim/init.lua
    """
    rel = winuser_rel_path(pkg, file_path)

    if pt == PkgType.WINUSER:
        return config.MNT_C / "Users" / config.WIN_USERNAME / rel
    elif pt == PkgType.WINCONFIG:
        return config.MNT_C / "Users" / config.WIN_USERNAME / ".config" / rel
    elif pt == PkgType.WINLOCAL:
        return config.MNT_C / "Users" / config.WIN_USERNAME / "AppData" / "Local" / rel
    elif pt == PkgType.WINROAMING:
        return config.MNT_C / "Users" / config.WIN_USERNAME / "AppData" / "Roaming" / rel
    return config.MNT_C / rel
