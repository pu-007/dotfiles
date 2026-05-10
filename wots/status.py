"""Status checks for stow and force-copy packages."""
from __future__ import annotations

import asyncio
import subprocess as _sp
from pathlib import Path
from typing import Dict, List, Tuple

from . import config
from .types import PkgType
from .discover import list_syncable_files, build_win_path


def _is_symlink(path: Path) -> bool:
    """Like path.is_symlink() but falls back to sudo test -L
    for paths the user cannot stat (e.g. /root/...)."""
    try:
        if path.is_symlink():
            return True
    except PermissionError:
        pass
    try:
        return _sp.run(["sudo", "test", "-L", str(path)], capture_output=True).returncode == 0
    except Exception:
        return False


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

        linked = _is_symlink(dest)
        if linked:
            linked = dest.exists() or _is_symlink(dest)
        if not linked:
            p = dest.parent
            while p != target and p != p.parent:
                if _is_symlink(p) and (p.exists() or _is_symlink(p)):
                    linked = True
                    break
                p = p.parent
        if linked:
            stowed += 1

    return (stowed, total)


# ── force-copy status (Windows types — WSL→Win only) ─────────

def check_copy_status(pkg: Path, pt: PkgType) -> Dict[str, int]:
    """
    Check force-copy status for all syncable files in *pkg*.

    Dotfiles repo is source of truth — only WSL→Win direction.

    Returns:
      synced           — both sides exist and are identical
      outdated_local   — WSL side is newer (needs sync)
      missing_remote   — exists in WSL only
      skipped          — excluded or too large
      error            — stat failed
    """
    counts = {
        "synced": 0, "outdated_local": 0,
        "missing_remote": 0, "skipped": 0, "error": 0,
    }

    if not pkg.is_dir():
        return counts

    files = list_syncable_files(pkg)
    for f in files:
        try:
            if config.MAX_SYNC_SIZE_BYTES > 0 and f.stat().st_size > config.MAX_SYNC_SIZE_BYTES:
                counts["skipped"] += 1
                continue
        except OSError:
            counts["error"] += 1
            continue

        win_path = build_win_path(f, pkg, pt)

        try:
            win_exists = win_path.exists() or win_path.is_symlink()
        except OSError:
            win_exists = True

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
        # else: Windows is newer — ignored (dotfiles always wins)

    return counts


# ── async wrappers ────────────────────────────────────────────

async def check_stow_status_async(pkg: Path, pt: PkgType) -> Tuple[int, int]:
    return await asyncio.to_thread(check_stow_status, pkg, pt)

async def check_copy_status_async(pkg: Path, pt: PkgType) -> Dict[str, int]:
    return await asyncio.to_thread(check_copy_status, pkg, pt)


def status_text(counts: Dict[str, int]) -> str:
    """Human-readable one-line summary of sync status (WSL→Win only)."""
    parts = []
    if counts.get("synced"):
        parts.append(f"{counts['synced']} synced")
    if counts.get("outdated_local"):
        parts.append(f"{counts['outdated_local']} needs-sync")
    if counts.get("missing_remote"):
        parts.append(f"{counts['missing_remote']} missing-win")
    if counts.get("skipped"):
        parts.append(f"{counts['skipped']} skipped")
    if not parts:
        return "empty"
    return ", ".join(parts)
