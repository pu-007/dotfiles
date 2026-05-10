"""Status checks for stow and copy-sync packages."""
from __future__ import annotations

from pathlib import Path
from typing import Dict, List, Tuple

from . import config
from .types import PkgType


# ── stow status (user / root) ─────────────────────────────────

def check_stow_status(pkg: Path, pt: PkgType) -> Tuple[int, int]:
    """
    Return (stowed_count, total_stowable).

    A file is *stowed* when its target path exists via a symlink
    ancestor — i.e. the destination itself or a parent directory
    is a symlink pointing into the package.
    """
    if not pt.uses_stow or pt.stow_target is None or not pkg.is_dir():
        return (0, 0)

    target = pt.stow_target
    stowed = 0
    total = 0

    for f in pkg.rglob("*"):
        if not f.is_file() or ".git" in f.parts:
            continue
        total += 1
        rel = f.relative_to(pkg)
        dest = target / rel

        linked = dest.is_symlink()
        if not linked:
            p = dest.parent
            while p != target and p != p.parent:
                if p.is_symlink():
                    linked = True
                    break
                p = p.parent
        if linked:
            stowed += 1

    return (stowed, total)


# ── copy-sync status (winuser / mnt) ──────────────────────────

def check_copy_status(
    pkg: Path, pt: PkgType,
) -> Dict[str, int]:
    """
    Check sync status for all syncable files in *pkg*.

    Returns a dict with counts:
      synced           — both sides exist and are identical
      outdated_local   — WSL side is newer than Windows
      outdated_remote  — Windows side is newer than WSL
      missing_remote   — exists in WSL only (not on Windows)
      missing_local    — exists on Windows only
      skipped          — excluded or too large
      error            — stat failed
    """
    from .discover import list_syncable_files, winuser_rel_path
    from .utils import is_excluded

    counts = {
        "synced": 0, "outdated_local": 0, "outdated_remote": 0,
        "missing_remote": 0, "missing_local": 0,
        "skipped": 0, "error": 0,
    }

    if not pkg.is_dir():
        return counts

    files = list_syncable_files(pkg, pt)
    for f in files:
        # size check
        try:
            if config.MAX_SYNC_SIZE_BYTES > 0 and f.stat().st_size > config.MAX_SYNC_SIZE_BYTES:
                counts["skipped"] += 1
                continue
        except OSError:
            counts["error"] += 1
            continue

        # Determine the corresponding Windows path
        if pt == PkgType.WINUSER:
            rel = winuser_rel_path(pkg, f)
            win_path = config.MNT_C / "Users" / config.WIN_USERNAME / rel
        elif pt == PkgType.MNT:
            try:
                rel = f.relative_to(config.WSL_MNT_BASE)
            except ValueError:
                counts["error"] += 1
                continue
            win_path = config.MNT_C / rel
        else:
            continue

        wsl_exists = f.exists()
        try:
            win_exists = win_path.exists() or win_path.is_symlink()
        except OSError:
            win_exists = True

        if not wsl_exists:
            counts["missing_local"] += 1
            continue
        if not win_exists:
            counts["missing_remote"] += 1
            continue

        try:
            wsl_stat = f.stat()
            win_stat = win_path.stat()
        except OSError:
            counts["error"] += 1
            continue

        # Consider files identical if mtime within 1 s AND same size
        if (abs(wsl_stat.st_mtime - win_stat.st_mtime) < 1.0
                and wsl_stat.st_size == win_stat.st_size):
            counts["synced"] += 1
        elif wsl_stat.st_mtime > win_stat.st_mtime:
            counts["outdated_local"] += 1
        else:
            counts["outdated_remote"] += 1

    return counts


def status_text(counts: Dict[str, int]) -> str:
    """Human-readable one-line summary of sync status."""
    parts = []
    if counts["synced"]:
        parts.append(f"{counts['synced']} synced")
    if counts["outdated_local"]:
        parts.append(f"{counts['outdated_local']} local-newer")
    if counts["outdated_remote"]:
        parts.append(f"{counts['outdated_remote']} remote-newer")
    if counts["missing_remote"]:
        parts.append(f"{counts['missing_remote']} missing-win")
    if counts["missing_local"]:
        parts.append(f"{counts['missing_local']} missing-wsl")
    if counts["skipped"]:
        parts.append(f"{counts['skipped']} skipped")
    if not parts:
        return "empty"
    return ", ".join(parts)
