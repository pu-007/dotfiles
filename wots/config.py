"""
WOTS configuration — edit constants or set env vars to match your setup.
"""
import os
from pathlib import Path

# ── Paths ──────────────────────────────────────────────────────────
DOTFILES_DIR = Path(
    os.environ.get("DOTFILES_DIR", Path.home() / "dotfiles")
).resolve()

# ── WSL / Windows ──────────────────────────────────────────────────
WSL_DISTRO_NAME = os.environ.get("WSL_DISTRO", "archlinux")

# Auto-detect Windows username from /mnt/c/Users (WSL) or env
def _detect_win_username() -> str:
    if os.environ.get("WIN_USER"):
        return os.environ["WIN_USER"]
    mnt_users = Path("/mnt/c/Users")
    if mnt_users.exists():
        skip = {"Public", "Default", "Default User", "All Users", "desktop.ini"}
        for d in sorted(mnt_users.iterdir()):
            if d.is_dir() and d.name not in skip and not d.name.startswith("."):
                return d.name
    return os.environ.get("USER", "user")

WIN_USERNAME = _detect_win_username()

# ── Derived target paths ───────────────────────────────────────────
USER_TARGET   = Path.home()
CONFIG_TARGET = USER_TARGET / ".config"
LOCAL_TARGET  = USER_TARGET / ".local"
ROOT_TARGET   = Path("/")

WINUSER_TARGET    = Path(f"C:/Users/{WIN_USERNAME}")
WINCONFIG_TARGET  = WINUSER_TARGET / ".config"
WINLOCAL_TARGET   = WINUSER_TARGET / "AppData" / "Local"
WINROAMING_TARGET = WINUSER_TARGET / "AppData" / "Roaming"

# /mnt/c mount point
MNT_C = Path("/mnt/c")


# ── Performance ────────────────────────────────────────────────────
MNT_MAX_CONCURRENT = int(os.environ.get("WOTS_CONCURRENT", "15"))

# ── Exclusion ──────────────────────────────────────────────────────
EXCLUDE_PATTERNS = [
    ".git", ".pixi", "__pycache__", "node_modules",
    ".mypy_cache", ".ruff_cache", "*.pyc", ".DS_Store", "Thumbs.db",
]
MAX_SYNC_SIZE_BYTES = int(os.environ.get(
    "WOTS_MAX_SIZE_MB", "50"
)) * 1024 * 1024
