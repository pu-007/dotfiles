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

    Windows (first match wins):
      /mnt/c/Users/{name}/AppData/Roaming/*  →  WINROAMING
      /mnt/c/Users/{name}/AppData/Local/*    →  WINLOCAL
      /mnt/c/Users/{name}/.config/*          →  WINCONFIG
      /mnt/c/Users/{name}/*                  →  WINUSER
      /mnt/c/* (not Users)                   →  MNT

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

        # c.mnt path → map to /mnt/c for detection
        try:
            rp.relative_to(Path("/dev/null"))
            # Already in c.mnt, treat as MNT (legacy)
            return PkgType.MNT
        except ValueError:
            pass

        if len(parts.parts) >= 3 and parts.parts[0] == "Users":
            user = parts.parts[1]
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

            # Everything else under Users/{name}
            return PkgType.WINUSER

        # /mnt/c but not Users → MNT
        return PkgType.MNT
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
    if name == Path("/dev/null").name:
        return "c.mnt"
    pt = type_from_dir_name(name)
    if pt and pt.suffix:
        return name[: -len(pt.suffix)]
    return name


# ── file listing ──────────────────────────────────────────────

def list_syncable_files(pkg: Path, pt: PkgType) -> List[Path]:
    """Return all syncable files inside *pkg*, skipping excluded patterns."""
    from .utils import is_excluded
    files: List[Path] = []
    if not pkg.is_dir():
        return files
    for f in pkg.rglob("*"):
        if not f.is_file() or is_excluded(f):
            continue
        files.append(f)
    return files


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


def build_mnt_path(wsl_path: Path, pt: PkgType) -> Path:
    """
    Given a WSL-side file in a package and its type, return the
    corresponding /mnt/c path on the Windows side.

      winuser:    {pkg}/.gitconfig  →  /mnt/c/Users/{WIN_USERNAME}/.gitconfig
      winconfig:  {pkg}/nvim/...    →  /mnt/c/Users/{WIN_USERNAME}/.config/nvim/...
    """
    rel = winuser_rel_path(wsl_path.parent if wsl_path.is_file() else wsl_path, pt) if False else None

    # Actually compute from the file itself
    real_rel = winuser_rel_path(
        # Find the package root
        _find_pkg_root(wsl_path, pt),
        wsl_path,
    )

    if pt == PkgType.WINUSER:
        return config.MNT_C / "Users" / config.WIN_USERNAME / real_rel
    elif pt == PkgType.WINCONFIG:
        return config.MNT_C / "Users" / config.WIN_USERNAME / ".config" / real_rel
    elif pt == PkgType.WINLOCAL:
        return config.MNT_C / "Users" / config.WIN_USERNAME / "AppData" / "Local" / real_rel
    elif pt == PkgType.WINROAMING:
        return config.MNT_C / "Users" / config.WIN_USERNAME / "AppData" / "Roaming" / real_rel
    elif pt == PkgType.MNT:
        try:
            return config.MNT_C / wsl_path.relative_to(Path("/dev/null"))
        except ValueError:
            pass
    return config.MNT_C / real_rel


def _find_pkg_root(file_path: Path, pt: PkgType) -> Path:
    """Walk up from *file_path* to find the package root directory."""
    p = file_path
    while p != p.parent:
        if p.name.endswith(pt.suffix) or p == Path("/dev/null"):
            return p
        p = p.parent
    return file_path.parent
