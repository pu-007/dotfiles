"""
Sync operations: stow (user/root) + copy-sync (winuser/mnt) + async batch.

All Windows operations use **copy** (not symlink) for maximum compatibility.
File comparison is based on mtime + size.
"""
from __future__ import annotations

import asyncio
import shutil
import subprocess
from pathlib import Path
from typing import Callable, Dict, List, Optional, Tuple

from . import config
from .types import PkgType
from .discover import list_syncable_files, winuser_rel_path
from .utils import has_stow, has_pwsh, is_wsl, is_excluded


# ═══════════════════════════════════════════════════════════════════════════════
#  Stow (user / root)
# ═══════════════════════════════════════════════════════════════════════════════

def do_stow(
    pkg: Path,
    target: Path,
    *,
    sudo: bool = False,
    dry_run: bool = False,
) -> bool:
    """Run `stow --adopt -t <target> <pkg>` inside DOTFILES_DIR."""
    from .display import success, error, dim
    from .utils import run

    if not has_stow():
        error("GNU Stow is not installed — cannot stow packages.")
        return False

    pkg_name = pkg.name
    try:
        run(
            ["stow", "-v", "--adopt", "-t", str(target), pkg_name],
            sudo=sudo, cwd=config.DOTFILES_DIR, dry_run=dry_run,
        )
        if dry_run:
            dim(f"    DRY-RUN  would stow  {pkg_name}  →  {target}")
        else:
            success(f"Stowed  {pkg_name}  →  {target}")
        return True
    except subprocess.CalledProcessError:
        return False


# ═══════════════════════════════════════════════════════════════════════════════
#  Windows copy (sync, blocking)
# ═══════════════════════════════════════════════════════════════════════════════

def _win_copy(
    wsl_src: Path,
    win_dst: str,       # e.g. "C:\\Users\\zion\\.gitconfig"
    *,
    is_dir: bool = False,
    dry_run: bool = False,
) -> bool:
    """
    Copy a file / directory from WSL to Windows via  pwsh.exe + cmd /c copy|xcopy.
    """
    from .display import dim, error

    wsl_unc = f"\\\\wsl$\\{config.WSL_DISTRO_NAME}" + str(wsl_src).replace("/", "\\")

    if is_dir:
        parts = ["xcopy", "/E", "/I", "/Y", f'"{wsl_unc}"', f'"{win_dst}"']
    else:
        parts = ["copy", "/Y", f'"{wsl_unc}"', f'"{win_dst}"']

    cmd_str = " ".join(parts)
    full = ["pwsh.exe", "-NoProfile", "-Command", f"cmd /c {cmd_str}"]
    dim(f"  WIN  {' '.join(full)}")

    if dry_run:
        return True

    try:
        r = subprocess.run(
            full, capture_output=True, text=True,
            cwd=str(config.MNT_C), encoding="gbk", errors="replace",
        )
        if r.stdout.strip():
            dim(f"       {r.stdout.strip()}")
        if r.stderr.strip():
            dim(f"       {r.stderr.strip()}")
        if r.returncode != 0:
            error(f"Windows copy failed (rc={r.returncode}): {cmd_str}")
            return False
        return True
    except Exception as e:
        error(f"Exception running Windows copy: {e}")
        return False


def _wsl_copy(
    win_src: Path,        # /mnt/c/Users/zion/...
    wsl_dst: Path,        # c.mnt/Users/zion/...  or  {pkg}.winuser/zion/...
    *,
    is_dir: bool = False,
    dry_run: bool = False,
) -> bool:
    """Copy a file / directory from Windows (/mnt/c) into WSL."""
    from .display import dim, error

    if dry_run:
        dim(f"    DRY-RUN  would copy  {win_src}  →  {wsl_dst}")
        return True

    try:
        wsl_dst.parent.mkdir(parents=True, exist_ok=True)
        if is_dir:
            shutil.copytree(str(win_src), str(wsl_dst), dirs_exist_ok=True)
        else:
            shutil.copy2(str(win_src), str(wsl_dst))
        return True
    except Exception as e:
        error(f"Copy failed: {e}")
        return False


# ═══════════════════════════════════════════════════════════════════════════════
#  Async Windows copy (for batch operations)
# ═══════════════════════════════════════════════════════════════════════════════

async def _win_copy_async(
    wsl_src: Path,
    win_dst: str,
    *,
    is_dir: bool = False,
    dry_run: bool = False,
) -> bool:
    """Async version of _win_copy — uses asyncio subprocess."""
    from .display import dim, error

    wsl_unc = f"\\\\wsl$\\{config.WSL_DISTRO_NAME}" + str(wsl_src).replace("/", "\\")
    if is_dir:
        parts = ["xcopy", "/E", "/I", "/Y", f'"{wsl_unc}"', f'"{win_dst}"']
    else:
        parts = ["copy", "/Y", f'"{wsl_unc}"', f'"{win_dst}"']

    cmd_str = " ".join(parts)
    full = ["pwsh.exe", "-NoProfile", "-Command", f"cmd /c {cmd_str}"]

    if dry_run:
        return True

    try:
        proc = await asyncio.create_subprocess_exec(
            *full,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            cwd=str(config.MNT_C),
        )
        stdout, stderr = await proc.communicate()
        if proc.returncode != 0:
            error(f"Windows copy failed (rc={proc.returncode}): {cmd_str}")
            if stderr:
                dim(f"       {stderr.decode('gbk', errors='replace')}")
            return False
        return True
    except Exception as e:
        error(f"Exception: {e}")
        return False


async def _wsl_copy_async(
    win_src: Path,
    wsl_dst: Path,
    *,
    is_dir: bool = False,
    dry_run: bool = False,
) -> bool:
    """Async WSL-side copy (runs in thread to avoid blocking)."""
    loop = asyncio.get_running_loop()
    return await loop.run_in_executor(
        None, _wsl_copy, win_src, wsl_dst, is_dir, dry_run,
    )


# ═══════════════════════════════════════════════════════════════════════════════
#  Single file sync (sync version — used by create)
# ═══════════════════════════════════════════════════════════════════════════════

def sync_one_file(
    wsl_path: Path,
    win_path: Path,
    *,
    direction: str = "sync",
    dry_run: bool = False,
) -> str:
    """
    Sync a single file between WSL and Windows.

    direction:
      "to-windows"  — WSL → Windows (push)
      "to-wsl"      — Windows → WSL (pull)
      "sync"        — bidirectional, newer wins

    Returns one of:
      'copied_to_win', 'copied_to_wsl', 'up_to_date',
      'missing_source', 'error'
    """
    from .display import dim, warning, error

    wsl_exists = wsl_path.exists()
    try:
        win_exists = win_path.exists() or win_path.is_symlink()
    except OSError:
        win_exists = True

    # ── direction: to-windows ──
    if direction == "to-windows":
        if not wsl_exists:
            warning(f"WSL source missing: {wsl_path}")
            return "missing_source"
        if _win_copy(wsl_path, _win_path_str(win_path),
                     is_dir=wsl_path.is_dir(), dry_run=dry_run):
            return "copied_to_win"
        return "error"

    # ── direction: to-wsl ──
    if direction == "to-wsl":
        if not win_exists:
            warning(f"Windows source missing: {win_path}")
            return "missing_source"
        if _wsl_copy(win_path, wsl_path, is_dir=win_path.is_dir(), dry_run=dry_run):
            return "copied_to_wsl"
        return "error"

    # ── direction: sync (bidirectional) ──
    if wsl_exists and not win_exists:
        if _win_copy(wsl_path, _win_path_str(win_path),
                     is_dir=wsl_path.is_dir(), dry_run=dry_run):
            return "copied_to_win"
        return "error"

    if win_exists and not wsl_exists:
        if _wsl_copy(win_path, wsl_path, is_dir=win_path.is_dir(), dry_run=dry_run):
            return "copied_to_wsl"
        return "error"

    # both exist — compare
    try:
        ws = wsl_path.stat()
        wn = win_path.stat()
    except OSError as e:
        error(f"Stat failed: {e}")
        return "error"

    if abs(ws.st_mtime - wn.st_mtime) < 1.0 and ws.st_size == wn.st_size:
        return "up_to_date"

    if ws.st_mtime > wn.st_mtime:
        if _win_copy(wsl_path, _win_path_str(win_path),
                     is_dir=wsl_path.is_dir(), dry_run=dry_run):
            return "copied_to_win"
        return "error"
    elif wn.st_mtime > ws.st_mtime:
        if _wsl_copy(win_path, wsl_path, is_dir=win_path.is_dir(), dry_run=dry_run):
            return "copied_to_wsl"
        return "error"
    else:
        return "up_to_date"


# ═══════════════════════════════════════════════════════════════════════════════
#  Async single file sync (for batch)
# ═══════════════════════════════════════════════════════════════════════════════

async def _sync_one_async(
    wsl_path: Path,
    win_path: Path,
    *,
    direction: str = "sync",
    dry_run: bool = False,
    sem: asyncio.Semaphore | None = None,
) -> str:
    """Async version of sync_one_file — uses semaphore for concurrency."""
    from .display import dim, warning, error

    async def _do_win_copy():
        async with sem if sem else _null_context():
            return await _win_copy_async(
                wsl_path, _win_path_str(win_path),
                is_dir=wsl_path.is_dir(), dry_run=dry_run,
            )

    async def _do_wsl_copy():
        async with sem if sem else _null_context():
            return await _wsl_copy_async(
                win_path, wsl_path,
                is_dir=win_path.is_dir(), dry_run=dry_run,
            )

    wsl_exists = wsl_path.exists()
    try:
        win_exists = win_path.exists() or win_path.is_symlink()
    except OSError:
        win_exists = True

    if direction == "to-windows":
        if not wsl_exists:
            return "missing_source"
        ok = await _do_win_copy()
        return "copied_to_win" if ok else "error"

    if direction == "to-wsl":
        if not win_exists:
            return "missing_source"
        ok = await _do_wsl_copy()
        return "copied_to_wsl" if ok else "error"

    # sync
    if wsl_exists and not win_exists:
        ok = await _do_win_copy()
        return "copied_to_win" if ok else "error"
    if win_exists and not wsl_exists:
        ok = await _do_wsl_copy()
        return "copied_to_wsl" if ok else "error"

    try:
        ws = wsl_path.stat()
        wn = win_path.stat()
    except OSError:
        return "error"

    if abs(ws.st_mtime - wn.st_mtime) < 1.0 and ws.st_size == wn.st_size:
        return "up_to_date"
    if ws.st_mtime > wn.st_mtime:
        ok = await _do_win_copy()
        return "copied_to_win" if ok else "error"
    elif wn.st_mtime > ws.st_mtime:
        ok = await _do_wsl_copy()
        return "copied_to_wsl" if ok else "error"
    return "up_to_date"


# ═══════════════════════════════════════════════════════════════════════════════
#  Async batch sync (with progress)
# ═══════════════════════════════════════════════════════════════════════════════

def sync_batch(
    items: List[Tuple[Path, Path]],   # [(wsl_path, win_path), ...]
    *,
    direction: str = "sync",
    dry_run: bool = False,
    max_concurrent: int = 0,
    progress_cb: Callable[[str, int, int], None] | None = None,
) -> Dict[str, int]:
    """
    Sync a batch of (wsl_path, win_path) pairs in parallel.

    Returns counts by result category.
    """
    return asyncio.run(_sync_batch_async(
        items, direction=direction, dry_run=dry_run,
        max_concurrent=max_concurrent, progress_cb=progress_cb,
    ))


async def _sync_batch_async(
    items: List[Tuple[Path, Path]],
    *,
    direction: str = "sync",
    dry_run: bool = False,
    max_concurrent: int = 0,
    progress_cb: Callable[[str, int, int], None] | None = None,
) -> Dict[str, int]:
    """
    Async core of sync_batch.

    Uses a semaphore to cap concurrent Windows pwsh.exe calls.
    """
    from .display import dim

    mc = max_concurrent or config.MNT_MAX_CONCURRENT
    sem = asyncio.Semaphore(max(1, mc))

    counts: Dict[str, int] = {
        "copied_to_win": 0, "copied_to_wsl": 0, "up_to_date": 0,
        "missing_source": 0, "error": 0, "skipped": 0,
    }
    total = len(items)
    done = 0

    async def _one(wsl_p: Path, win_p: Path):
        nonlocal done
        # Skip excluded or too-large files
        if is_excluded(wsl_p):
            return "skipped"
        try:
            if config.MAX_SYNC_SIZE_BYTES > 0 and wsl_p.stat().st_size > config.MAX_SYNC_SIZE_BYTES:
                return "skipped"
        except OSError:
            return "error"

        result = await _sync_one_async(
            wsl_p, win_p, direction=direction, dry_run=dry_run, sem=sem,
        )
        done += 1
        if progress_cb:
            progress_cb(result, done, total)
        return result

    tasks = [_one(ws, wn) for ws, wn in items]
    results = await asyncio.gather(*tasks, return_exceptions=True)

    for r in results:
        if isinstance(r, Exception):
            counts["error"] += 1
        else:
            counts[r] = counts.get(r, 0) + 1

    return counts


# ═══════════════════════════════════════════════════════════════════════════════
#  Helpers
# ═══════════════════════════════════════════════════════════════════════════════

def _win_path_str(p: Path) -> str:
    """Convert a /mnt/c-based Path to a C:\\ string for Windows commands."""
    s = str(Path("C:/") / p.relative_to(config.MNT_C))
    return s.replace("/", "\\")


class _null_context:
    """Async no-op context manager."""
    async def __aenter__(self):
        return None
    async def __aexit__(self, *args):
        pass


def prepare_sync_items(
    pkg: Path, pt: PkgType,
) -> List[Tuple[Path, Path]]:
    """
    Build a list of (wsl_path, win_path) pairs for every syncable file
    in *pkg* (any copy-sync type).
    """
    from .discover import list_syncable_files, build_mnt_path
    items: List[Tuple[Path, Path]] = []

    if not pt.uses_copy_sync:
        return items

    files = list_syncable_files(pkg, pt)
    for f in files:
        win_path = build_mnt_path(f, pt)
        items.append((f, win_path))

    return items


def build_winuser_wsl_path(pkg: Path, win_path_on_mnt: Path, pt: PkgType | None = None) -> Path:
    """
    Given a package and a /mnt/c/... path, return the corresponding
    path inside the package (reverse of build_mnt_path).

    For winuser packages, the package root maps to C:\\Users\\{WIN_USERNAME}\\
    For winconfig,          → C:\\Users\\{WIN_USERNAME}\\.config\\
    etc.
    """
    # Try to figure out the relative path from the target
    target = pt.sync_target if pt else None
    if target:
        try:
            rel = win_path_on_mnt.relative_to(
                Path(str(target).replace("C:", str(config.MNT_C)))
            )
            return pkg / rel
        except ValueError:
            pass

    # Fallback: relative to /mnt/c
    try:
        rel = win_path_on_mnt.relative_to(config.MNT_C)
    except ValueError:
        rel = win_path_on_mnt
    return pkg / rel
