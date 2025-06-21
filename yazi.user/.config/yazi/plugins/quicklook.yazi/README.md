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

4. Setup Python Environment

To activate automatically, you need to install `pyautogui` in your Python environment.

```bash
pip install pyautogui
```

Or you can choose vbs commands to activate QuickLook, but it may not work in some cases.

```Lua
[[powershell.exe -Command "Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.Interaction]::AppActivate('QuickLook')"]]
```

The method will be opptional in the future, and you can choose to use it or not.

## Brief Introduction

1. if in Windows mounted dir

transform current file/dir path like `/mnt/c/Users/zion/AppData/Local/Programs/QuickLook`

to win_native_path like `C:\Users\zion\AppData\Local\Programs\QuickLook`

2. if in WSL innter dir

transform current file/dir path like `/home/zion/Downloads`

to win_wslnet_path like `\\wsl.localhost\Arch\home\zion\Downloads`

3. exec `QuickLook.exe '$file_or_dir'`

(Note that the new path should be enclosed in quotation marks to prevent ambiguity caused by spaces, use async ways to exec)
