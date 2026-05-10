"""Status checks for stow and copy-sync packages."""
from __future__ import annotations

from pathlib import Path
from typing import Dict, List, Tuple

from . import config
from .types import PkgType
from .discover import list_syncable_files, winuser_rel_path


# ── stow status (Linux types) ─────────────────────────────────

def check_stow_status(pkg: Path, pt: PkgType) -> Tuple[int, int]:
    """
    Return (stowed_count, total_stowable).
    Uses pt.sync_target to find the stow destination.
    """
    target = pt.sync_target
    if not pt.uses_stow or target is None or not pkg.is_dir():
        return (0, 0)

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


# ── copy-sync status (Windows types) ──────────────────────────

def check_copy_status(pkg: Path, pt: PkgType) -> Dict[str, int]:
    """
    Check sync status for all syncable files in *pkg*.

    Returns:
      synced           — both sides exist and are identical
      outdated_local   — WSL side is newer than Windows
      outdated_remote  — Windows side is newer than WSL
      missing_remote   — exists in WSL only
      missing_local    — exists on Windows only
      skipped          — excluded or too large
      error            — stat failed
    """
    from .discover import build_mnt_path
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
        try:
            if config.MAX_SYNC_SIZE_BYTES > 0 and f.stat().st_size > config.MAX_SYNC_SIZE_BYTES:
                counts["skipped"] += 1
                continue
        except OSError:
            counts["error"] += 1
            continue

        # Compute Windows-side path via /mnt/c
        win_path = build_mnt_path(f, pt)

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
            ws = f.stat()
            wn = win_path.stat()
        except OSError:
            counts["error"] += 1
            continue

        if abs(ws.st_mtime - wn.st_mtime) < 1.0 and ws.st_size == wn.st_size:
            counts["synced"] += 1
        elif ws.st_mtime > wn.st_mtime:
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
