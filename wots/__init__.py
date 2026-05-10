"""
WOTS — WSL Dotfile Stow Tool
=============================
Unified dotfile management for WSL & Linux & Windows.

Package types:
  .user    → stow to $HOME
  .root    → stow to / (sudo)
  .winuser → copy-sync to C:\\Users\\{name}
  .meta    → manual storage
  c.mnt/   → legacy Windows mirror (compat)

Usage:
  python -m wots create ~/.config/nvim
  python -m wots sync
  python -m wots sync --type winuser
  python -m wots stats
  python -m wots list
"""

__version__ = "1.0.0"
__all__ = [
    "config", "types", "utils", "discover",
    "sync", "status", "display", "cli",
]
