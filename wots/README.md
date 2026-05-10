# WOTS — WSL Dotfile Stow Tool

> **wots** is **stow** backwards.  
> It manages your dotfiles *from* the repo *to* the system, and *back*.

WOTS is a unified CLI tool for managing dotfiles across **WSL**, **Linux**, and **Windows**.  
It handles symlink-based stowing for Linux configs and copy-based force-sync (dotfiles → Windows) for Windows configs.

---

## What can WOTS do?  /  WOTS 能做什么？

| Command | What it does |
|---------|-------------|
| `wots create` | Create a new package from existing config files (interactive, auto-detects type) |
| `wots sync` | Deploy all packages — stow for Linux, force-copy dotfiles → Windows |
| `wots stats` | Show repository statistics with sync status |
| `wots list` | List all packages with file counts and status |
| `wots list -u` | Show only unsynced packages + per-file differences |

## My files at a glance  /  我有哪些文件

```
~/dotfiles/
├── git.user/.gitconfig                 user   → ~/.gitconfig
├── nvim.config/nvim/init.lua           config → ~/.config/nvim/init.lua
├── yazi.config/yazi/yazi.toml           config → ~/.config/yazi/yazi.toml
├── fcitx5.config/fcitx5/...            config → ~/.config/fcitx5/...
├── zsh.user/.zshrc                     user   → ~/.zshrc
├── wsl.root/etc/wsl.conf               root   → /etc/wsl.conf
├── pacman.root/etc/pacman.conf         root   → /etc/pacman.conf
├── git.winuser/.gitconfig              winuser → C:\Users\zion\.gitconfig
├── komorebi.winuser/komorebi.json      winuser → C:\Users\zion\komorebi.json
├── wsl.winuser/.wslconfig              winuser → C:\Users\zion\.wslconfig
├── uv.winuser/.config/uv/              winuser → C:\Users\zion\.config\uv\
├── android.meta/...                    meta   → manual storage
└── scripts.meta/...                    meta   → utility scripts
```

**41 packages, 167 files, 7.0 MB total** — all synced.

## Package types  /  包类型

| Suffix | Target | Sync |
|--------|--------|------|
| `.user` | `~/` | stow symlink |
| `.config` | `~/.config/` | stow symlink |
| `.local` | `~/.local/` | stow symlink |
| `.root` | `/` | stow + file-by-file fallback (sudo) |
| `.winuser` | `C:\Users\{name}\` | force-copy (async) |
| `.winconfig` | `C:\Users\{name}\.config\` | force-copy (async) |
| `.winroaming` | `C:\Users\{name}\AppData\Roaming\` | force-copy (async) |
| `.winlocal` | `C:\Users\{name}\AppData\Local\` | force-copy (async) |
| `.meta` | manual only | none |

## Prerequisites  /  前置条件

- Python ≥ 3.10
- [pixi](https://pixi.sh) (or pip install typer rich)
- GNU Stow (for Linux configs)
- WSL + pwsh.exe (for Windows sync only)

## Getting started  /  快速开始

```bash
# Clone and install
git clone <your-dotfiles-repo>
cd dotfiles
pixi install

# See what you have
pixi run stats
pixi run list

# Preview before syncing
pixi run sync-dry

# Sync everything
pixi run sync

# Sync only Linux configs
pixi run sync-user

# Sync Windows files (WSL required)
pixi run sync-win

# Check for problems
pixi run wots list -u
```

## Creating a new package  /  创建新包

```bash
# Linux config (auto-detected as 'config' type)
wots create ~/.config/myapp

# Home-directory dotfile (auto-detected as 'user')
wots create ~/.myrc

# Windows config
wots create /mnt/c/Users/zion/AppData/Roaming/MyApp

# Force a type
wots create ~/notes --type meta -a mynotes

# Non-interactive
wots create ~/.config/foo --yes
```

## Configuration  /  配置

Edit `wots/config.py` or set environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `DOTFILES_DIR` | `~/dotfiles` | Repo root |
| `WSL_DISTRO` | `archlinux` | WSL distro name |
| `WIN_USER` | auto-detect | Windows username |
| `WOTS_CONCURRENT` | `15` | Max parallel Windows copies |
| `WOTS_MAX_SIZE_MB` | `50` | Skip files larger than this |

Auto-detection: `WIN_USER` is read from `/mnt/c/Users/` if not set explicitly.

## How sync works  /  同步原理

**Linux (stow)**: GNU Stow creates symlinks from the target directory into the package.  
If stow encounters a "not owned by stow" conflict (e.g., system directories), it falls back to file-by-file `ln -sf`.

**Windows (copy)**: No symlinks — files are force-copied from dotfiles repo to Windows via `pwsh.exe` + `cmd /c copy|xcopy`.  
The dotfiles repo is always the source of truth. Async batch operations run multiple copies in parallel.

## Contributing / template  /  模板

A clean template repository will be published soon at a separate repo.  
You'll be able to clone it and start managing your own dotfiles with WOTS immediately.

> Name: **wots** = **stow** backwards. It stows *from* the repo, not *to* it.

## License

MIT
