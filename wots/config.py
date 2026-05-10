"""
WOTS configuration — edit these constants to match your setup.
Environment variables take precedence over defaults.
"""
import os
from pathlib import Path

# ── Paths ──────────────────────────────────────────────────────────
DOTFILES_DIR = Path(
    os.environ.get("DOTFILES_DIR", Path.home() / "dotfiles")
).resolve()

# ── WSL / Windows settings ─────────────────────────────────────────
WSL_DISTRO_NAME = os.environ.get("WSL_DISTRO", "archlinux")
WIN_USERNAME   = os.environ.get("WIN_USER", os.environ.get("USER", "zion"))

# ── Derived paths ──────────────────────────────────────────────────
USER_TARGET  = Path.home()
ROOT_TARGET  = Path("/")
WSL_MNT_BASE = DOTFILES_DIR / "c.mnt"          # c.mnt ↔ C:\
MNT_C        = Path("/mnt/c")                  # /mnt/c = C:\ in WSL

# ── Performance ────────────────────────────────────────────────────
# Max concurrent Windows copy operations (async batch)
MNT_MAX_CONCURRENT = int(os.environ.get("WOTS_CONCURRENT", "15"))

# ── Exclusion ──────────────────────────────────────────────────────
# Glob-style patterns; matched against path parts.
# Files / dirs matching any pattern are skipped during sync.
EXCLUDE_PATTERNS = [
    ".git",
    ".pixi",
    "__pycache__",
    "node_modules",
    ".mypy_cache",
    ".ruff_cache",
    "*.pyc",
    ".DS_Store",
    "Thumbs.db",
]

# Skip syncing files larger than this (bytes).  0 = no limit.
MAX_SYNC_SIZE_BYTES = int(os.environ.get(
    "WOTS_MAX_SIZE_MB", "50"
)) * 1024 * 1024
