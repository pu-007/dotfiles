"""Utility functions: shell, filesystem, environment checks."""
from __future__ import annotations

import fnmatch
import shutil
import subprocess
from pathlib import Path
from typing import List, Optional

from . import config


# ── environment checks ─────────────────────────────────────────

def is_wsl() -> bool:
    """True when running under WSL."""
    return Path("/proc/sys/fs/binfmt_misc/WSLInterop").exists()

def has_pwsh() -> bool:
    return shutil.which("pwsh.exe") is not None

def has_stow() -> bool:
    return shutil.which("stow") is not None


# ── filesystem metrics ─────────────────────────────────────────

def count_files(directory: Path) -> int:
    """Count regular files recursively, skipping excluded patterns."""
    if not directory.is_dir():
        return 0
    n = 0
    for p in directory.rglob("*"):
        if p.is_file() and not _is_excluded(p):
            n += 1
    return n

def dir_size(directory: Path) -> int:
    """Total byte size of all regular files, skipping excluded patterns."""
    if not directory.is_dir():
        return 0
    total = 0
    for p in directory.rglob("*"):
        if p.is_file() and not _is_excluded(p):
            total += p.stat().st_size
    return total

def fmt_size(n_bytes: int) -> str:
    """Human-readable byte size."""
    if n_bytes < 1024:
        return f"{n_bytes} B"
    elif n_bytes < 1024 * 1024:
        return f"{n_bytes / 1024:.1f} KB"
    elif n_bytes < 1024 * 1024 * 1024:
        return f"{n_bytes / (1024 * 1024):.1f} MB"
    else:
        return f"{n_bytes / (1024 * 1024 * 1024):.2f} GB"


# ── exclusion ──────────────────────────────────────────────────

def _is_excluded(path: Path) -> bool:
    """Check whether *path* matches any EXCLUDE_PATTERNS."""
    for part in path.parts:
        for pat in config.EXCLUDE_PATTERNS:
            if fnmatch.fnmatch(part, pat):
                return True
    return False

def is_excluded(path: Path) -> bool:
    """Public alias."""
    return _is_excluded(path)


# ── subprocess ─────────────────────────────────────────────────

def run(
    cmd: List[str],
    *,
    sudo: bool = False,
    cwd: Optional[Path] = None,
    dry_run: bool = False,
    check: bool = True,
) -> subprocess.CompletedProcess:
    """Run a command, optionally with sudo."""
    from .display import dim, error, info as _info

    if sudo:
        cmd = ["sudo"] + cmd
    cmd_str = " ".join(cmd)

    if dry_run:
        dim(f"  DRY-RUN  {cmd_str}")
        return subprocess.CompletedProcess(cmd, 0, "", "")

    try:
        r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=check)
        if r.stdout.strip():
            dim(f"       {r.stdout.strip()}")
        if r.stderr.strip():
            dim(f"       {r.stderr.strip()}")
        return r
    except subprocess.CalledProcessError as e:
        error(f"Command failed (rc={e.returncode}): {cmd_str}")
        if e.stdout:
            _info(e.stdout)
        if e.stderr:
            error(e.stderr)
        raise
