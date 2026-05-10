"""
Sync operations: stow (user/root) + force-copy (winuser) + async batch.

All Windows operations use **copy** (not symlink) for maximum compatibility.
Dotfiles repo is always the source of truth — WSL always overwrites Windows.
"""

from __future__ import annotations

import asyncio
import subprocess
from pathlib import Path
from typing import Callable, Dict, List, Optional, Tuple

from . import config
from .types import PkgType
from .discover import list_syncable_files, build_win_path
from .utils import has_stow, has_pwsh, is_wsl, is_excluded


# ═════════════════════════════════════════════════════════════════
#  Stow (user / root)
# ═════════════════════════════════════════════════════════════════

def do_stow(pkg: Path, target: Path, *, sudo: bool = False, dry_run: bool = False) -> bool:
    """Run `stow --adopt -t <target> <pkg>` inside DOTFILES_DIR."""
    from .display import success, error, warning, dim
    from .utils import run

    if not has_stow():
        error("GNU Stow is not installed — cannot stow packages.")
        return False

    pkg_name = pkg.name

    if sudo:
        if dry_run:
            dim(f"    DRY-RUN  would link  {pkg_name}  →  {target}  (file-by-file)")
        else:
            dim(f"  {pkg_name}  →  {target}  (file-by-file)")
        try:
            _stow_file_by_file(pkg, target, sudo=True, dry_run=dry_run)
            if not dry_run:
                success(f"Linked (file-by-file)  {pkg_name}  →  {target}")
            return True
        except Exception:
            error(f"File-by-file link failed for {pkg_name}")
            return False

    try:
        run(["stow", "-v", "--adopt", "-t", str(target), pkg_name],
            sudo=False, cwd=config.DOTFILES_DIR, dry_run=dry_run)
        if dry_run:
            dim(f"    DRY-RUN  would stow  {pkg_name}  →  {target}")
        else:
            success(f"Stowed  {pkg_name}  →  {target}")
        return True
    except subprocess.CalledProcessError as e:
        stderr = (e.stderr or "").strip()
        if "existing target is not owned by stow" in stderr:
            conflict_paths: list[str] = []
            for ln in stderr.splitlines():
                if "existing target is not owned by stow:" in ln:
                    p = ln.split("existing target is not owned by stow:")[-1].strip()
                    if p:
                        conflict_paths.append(p)
            warning(f"Stow conflict in {pkg_name}: {', '.join(conflict_paths)}")
            warning("  → Retrying file-by-file (ln -sf) ...")
            try:
                _stow_file_by_file(pkg, target, sudo=False, dry_run=dry_run)
                if dry_run:
                    dim(f"    DRY-RUN  would stow  {pkg_name}  →  {target}")
                else:
                    success(f"Linked (file-by-file)  {pkg_name}  →  {target}")
                return True
            except Exception:
                error(f"File-by-file link also failed for {pkg_name}")
                return False
        elif "Permission denied" in stderr or "cannot stow" in stderr:
            warning(f"Stow permission error in {pkg_name}: {stderr[:200]}")
        else:
            error(f"Stow failed for {pkg_name}: {stderr[:300]}")
        return False


def _stow_file_by_file(pkg: Path, target: Path, *, sudo: bool, dry_run: bool):
    """Stow by creating individual symlinks for each file."""
    from .display import dim

    for f in sorted(pkg.rglob("*")):
        if not f.is_file() or ".git" in f.parts:
            continue
        rel = f.relative_to(pkg)
        dest = target / rel
        if sudo:
            subprocess.run(["sudo", "mkdir", "-p", str(dest.parent)], check=dry_run is False)
        else:
            dest.parent.mkdir(parents=True, exist_ok=True)
        src = f.resolve()
        if dry_run:
            dim(f"      ln -sf {src} {dest}")
        else:
            if sudo:
                subprocess.run(["sudo", "ln", "-sf", str(src), str(dest)], check=True)
            else:
                if dest.exists() or dest.is_symlink():
                    dest.unlink()
                dest.symlink_to(src)


# ═════════════════════════════════════════════════════════════════
#  Windows copy (blocking)
# ═════════════════════════════════════════════════════════════════

def _win_copy(wsl_src: Path, win_dst: str, *, is_dir: bool = False, dry_run: bool = False) -> bool:
    """Copy a file/dir from WSL to Windows via pwsh.exe + cmd /c copy|xcopy."""
    from .display import dim, error

    wsl_unc = f"\\\\wsl$\\{config.WSL_DISTRO_NAME}" + str(wsl_src).replace("/", "\\")
    parts = (["xcopy", "/E", "/I", "/Y"] if is_dir else ["copy", "/Y"]) + [f'"{wsl_unc}"', f'"{win_dst}"']
    cmd_str = " ".join(parts)
    full = ["pwsh.exe", "-NoProfile", "-Command", f"cmd /c {cmd_str}"]

    if dry_run:
        dim(f"    DRY-RUN  {' '.join(full)}")
        return True

    try:
        r = subprocess.run(full, capture_output=True, text=True,
                           cwd=str(config.MNT_C), encoding="gbk", errors="replace")
        if r.returncode != 0:
            error(f"Windows copy failed (rc={r.returncode}): {cmd_str}")
            return False
        return True
    except Exception as e:
        error(f"Exception running Windows copy: {e}")
        return False


# ═════════════════════════════════════════════════════════════════
#  Async Windows copy (for batch)
# ═════════════════════════════════════════════════════════════════

async def _win_copy_async(wsl_src: Path, win_dst: str, *, is_dir: bool = False,
                          dry_run: bool = False) -> bool:
    """Async version of _win_copy — uses asyncio subprocess."""
    from .display import dim, error

    wsl_unc = f"\\\\wsl$\\{config.WSL_DISTRO_NAME}" + str(wsl_src).replace("/", "\\")
    parts = (["xcopy", "/E", "/I", "/Y"] if is_dir else ["copy", "/Y"]) + [f'"{wsl_unc}"', f'"{win_dst}"']
    cmd_str = " ".join(parts)
    full = ["pwsh.exe", "-NoProfile", "-Command", f"cmd /c {cmd_str}"]

    if dry_run:
        return True

    try:
        proc = await asyncio.create_subprocess_exec(
            *full, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE,
            cwd=str(config.MNT_C))
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


# ═════════════════════════════════════════════════════════════════
#  Single file sync (force-copy — used by create)
# ═════════════════════════════════════════════════════════════════

def sync_one_file(wsl_path: Path, win_path: Path, *, dry_run: bool = False) -> str:
    """Force-copy a single file from WSL to Windows.
    Returns: 'copied_to_win', 'missing_source', 'error'."""
    if not wsl_path.exists():
        return "missing_source"
    if _win_copy(wsl_path, _win_path_str(win_path), is_dir=wsl_path.is_dir(), dry_run=dry_run):
        return "copied_to_win"
    return "error"


# ═════════════════════════════════════════════════════════════════
#  Async single file sync (for batch)
# ═════════════════════════════════════════════════════════════════

async def _sync_one_async(wsl_path: Path, win_path: Path, *,
                          dry_run: bool = False,
                          sem: asyncio.Semaphore | None = None) -> str:
    """Async force-copy WSL → Windows — uses semaphore for concurrency."""

    async def _do():
        async with sem if sem else _null_context():
            return await _win_copy_async(wsl_path, _win_path_str(win_path),
                                         is_dir=wsl_path.is_dir(), dry_run=dry_run)

    if not wsl_path.exists():
        return "missing_source"
    ok = await _do()
    return "copied_to_win" if ok else "error"


# ═════════════════════════════════════════════════════════════════
#  Async batch sync (with progress)
# ═════════════════════════════════════════════════════════════════

def sync_batch(items: List[Tuple[Path, Path]], *,
               dry_run: bool = False, max_concurrent: int = 0,
               progress_cb: Callable[[str, int, int], None] | None = None,
               ) -> Dict[str, int]:
    """Force-copy a batch of (wsl_path, win_path) pairs in parallel.
    Returns counts by result category."""
    return asyncio.run(_sync_batch_async(items, dry_run=dry_run,
                                         max_concurrent=max_concurrent,
                                         progress_cb=progress_cb))


async def _sync_batch_async(items: List[Tuple[Path, Path]], *,
                            dry_run: bool = False, max_concurrent: int = 0,
                            progress_cb: Callable[[str, int, int], None] | None = None,
                            ) -> Dict[str, int]:
    """Async core of sync_batch. Uses a semaphore to cap concurrent pwsh.exe calls."""
    mc = max_concurrent or config.SYNC_MAX_CONCURRENT
    sem = asyncio.Semaphore(max(1, mc))

    counts: Dict[str, int] = {"copied_to_win": 0, "missing_source": 0, "error": 0, "skipped": 0}
    total = len(items)
    done = 0

    async def _one(wsl_p: Path, win_p: Path):
        nonlocal done
        if is_excluded(wsl_p):
            return "skipped"
        try:
            if config.MAX_SYNC_SIZE_BYTES > 0 and wsl_p.stat().st_size > config.MAX_SYNC_SIZE_BYTES:
                return "skipped"
        except OSError:
            return "error"

        result = await _sync_one_async(wsl_p, win_p, dry_run=dry_run, sem=sem)
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


# ═════════════════════════════════════════════════════════════════
#  Helpers
# ═════════════════════════════════════════════════════════════════

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


def prepare_sync_items(pkg: Path, pt: PkgType) -> List[Tuple[Path, Path]]:
    """Build a list of (wsl_path, win_path) pairs for every syncable file in *pkg*."""
    if not pt.uses_copy_sync:
        return []
    return [(f, build_win_path(f, pkg, pt)) for f in list_syncable_files(pkg)]
