# WOTS — WSL Dotfile Stow Tool

Unified dotfile management for WSL, Linux, and Windows.

## Package types

| Type     | Directory       | Sync method          | Target                       |
|----------|-----------------|----------------------|------------------------------|
| user     | `*.user/`       | `stow --adopt`       | `$HOME`                      |
| root     | `*.root/`       | `sudo stow --adopt`  | `/`                          |
| winuser  | `*.winuser/`    | copy + compare       | `C:\Users\{name}`            |
| meta     | `*.meta/`       | manual only          | —                            |
| mnt      | `c.mnt/`        | copy + compare       | `C:\`                        |

## Quick start

```bash
# Install dependencies
pixi install

# Show help
pixi run wots --help

# Show repository statistics
pixi run stats

# List all packages
pixi run list

# Preview what sync would do
pixi run sync-dry

# Sync everything
pixi run sync
```

## Commands

### `wots create`

Create a new package from existing config files.  Type is auto-detected.

```bash
# Linux config (auto-detected as user)
wots create ~/.config/nvim

# Multiple files with custom name
wots create ~/.zshrc ~/.zsh_aliases -a zsh

# Windows config (auto-detected as winuser)
wots create /mnt/c/Users/zion/AppData/Roaming/MyApp

# Force a specific type
wots create ~/Documents/notes --type meta
```

### `wots sync`

Sync packages to their targets: stow for user/root, copy-compare for winuser/mnt.

```bash
# Sync everything
wots sync

# Sync only user packages
wots sync --type user

# Push c.mnt to Windows (WSL → C:\)
wots sync --type mnt --direction to-windows

# Pull Windows changes into WSL (C:\ → WSL)
wots sync --type mnt --direction to-wsl

# Bidirectional sync (newer wins) — default
wots sync --type winuser

# Sync a specific package
wots sync --type user --app nvim

# Preview (no changes)
wots sync --dry-run

# Verbose per-file output
wots sync --verbose
```

### `wots stats`

Repository statistics with sync status.

```bash
wots stats
wots stats --verbose
wots stats --json     # machine-readable
```

### `wots list`

Package listing with per-package status.

```bash
wots list
wots list --type user
wots list --type winuser
wots list --json
```

## How sync works

### Stow (user / root)
Uses GNU Stow to create symlinks from `$HOME` or `/` into the package directory.

### Copy-sync (winuser / mnt)
Windows files use **copy** (not symlink) for maximum compatibility.  Each sync:

1. Compares file modification time + size on both sides
2. Copies the **newer** version to the other side (in `sync` mode)
3. Or enforces a single direction (`to-windows` / `to-wsl`)

Async batch operations run multiple copies in parallel (configurable via `WOTS_CONCURRENT` env var, default 15).

## Configuration

Edit `wots/config.py` or set environment variables:

| Variable           | Default            | Description                     |
|--------------------|--------------------|---------------------------------|
| `DOTFILES_DIR`     | `~/dotfiles`       | Root of dotfiles repository     |
| `WSL_DISTRO`       | `archlinux`        | WSL distribution name           |
| `WIN_USER`         | `$USER`            | Windows username                |
| `WOTS_CONCURRENT`  | `15`               | Max parallel Windows copies     |
| `WOTS_MAX_SIZE_MB` | `50`               | Skip files larger than this     |

Exclusion patterns in `config.py` skip `.git`, `.pixi`, `node_modules`, etc.

## Migrating from old scripts

| Old script                    | New command                          |
|-------------------------------|--------------------------------------|
| `bash stow.sh`                | `wots sync --type user`              |
| `bash linux_stow.sh`          | `wots sync`                          |
| `bash mnt2win.sh`             | `wots sync --type mnt`               |
| `python user_creat_stow_app.py` | `wots create`                      |
| `python config_linker.py`     | `wots sync --type mnt`               |

## Creating a winuser package (Windows config)

```bash
# Pull existing Windows config into a new winuser package
wots create /mnt/c/Users/zion/.gitconfig -a git

# This creates:  git.winuser/zion/.gitconfig
# Then sync it:
wots sync --type winuser --app git --direction to-windows
```

## Package structure example

```
~/dotfiles/
├── nvim.user/           # user  type → stow to ~
│   └── .config/
│       └── nvim/
│           └── init.lua
├── pacman.root/          # root  type → sudo stow to /
│   └── etc/
│       └── pacman.conf
├── git.winuser/          # winuser type → copy-sync to C:\Users\zion\
│   └── zion/
│       └── .gitconfig
├── android.meta/         # meta  type → manual
│   └── cphrase.ini
└── c.mnt/                # mnt   type → legacy Windows mirror
    └── Users/
        └── zion/
            └── .wslconfig
```
