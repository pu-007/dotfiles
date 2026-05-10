# Dotfiles

Managed by **[WOTS](wots/)** — WSL Dotfile Stow Tool.

```bash
# Quick start
pixi install
pixi run wots --help
pixi run stats
pixi run sync
```

## Package types

| Type | Suffix | Target |
|------|--------|--------|
| user | `.user` | `~/` |
| config | `.config` | `~/.config/` |
| local | `.local` | `~/.local/` |
| root | `.root` | `/` (sudo) |
| winuser | `.winuser` | `C:\Users\{name}\` |
| winconfig | `.winconfig` | `C:\Users\{name}\.config\` |
| winlocal | `.winlocal` | `C:\Users\{name}\AppData\Local\` |
| winroaming | `.winroaming` | `C:\Users\{name}\AppData\Roaming\` |
| meta | `.meta` | manual |

## Common commands

```bash
# Statistics
pixi run stats

# List packages (filter unsynced)
pixi run wots list --diff --type root

# Sync all
pixi run sync

# Sync specific type
pixi run sync-user    # user + config
pixi run sync-root    # sudo, requires confirmation
pixi run sync-win     # Windows copy-sync

# Dry run (preview)
pixi run sync-dry

# Create a new package (interactive)
pixi run wots create ~/.config/newapp
pixi run wots create /mnt/c/Users/zion/AppData/Roaming/MyApp
```
