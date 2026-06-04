# QuickLook.yazi

Use QuickLook to preview files/dirs in yazi on WSL, which is useful for Videos, Images, PDFs, and other file types that QuickLook supports.

![2025-06-14 12-13-36](https://github.com/user-attachments/assets/01d61d10-6c00-4d88-9c79-8c15100440ec)

## Install

1. Install via ya

```bash
ya pkg add pu-007/quicklook
```

2. Lua Setup

add follwing config in `init.lua`

```Lua
require("quicklook"):setup({
  wsl_distro = "Arch",
  quicklook_path = "/mnt/c/Users/zion/AppData/Local/Programs/QuickLook/QuickLook.exe",
  debug = false
})
```

3. Keymap Setup

add follwing config in `keymap.toml`

```toml
[[mgr.prepend_keymap]]
on = ["g", "q"] # or "\\"
run = "plugin quicklook"
desc = "Quick look file/dir"
```

## Requirements

- **WSL** (Windows Subsystem for Linux)
- **QuickLook** installed on Windows
- **pwsh.exe** (PowerShell Core) available in your PATH (usually default in WSL)
- Standard Linux tools: `iconv`, `base64`, `tr` (usually pre-installed)


## How it works

1. **Path Resolution**: Resolves the selected file/directory to a real absolute path (handling symlinks).

2. **Path Conversion**:
   - If in a Windows mounted drive (e.g., `/mnt/c/...`), converts to a native Windows path (`C:\...`).
   - If in a WSL internal directory (e.g., `/home/user/...`), converts to a UNC path (`\\wsl.localhost\Distro\...`).

3. **Window Activation**: Generates and executes a temporary PowerShell script (encoded via Base64) to bring the QuickLook window to the foreground if it's already running.

4. **Execution**: Runs `QuickLook.exe` with the converted path to preview the file.
