set ws=wscript.CreateObject("wscript.shell")

ws.run "pixi run -m '\\wsl.localhost\archlinux\home\pu\dotfiles\scripts.meta\pixi.toml' autostart", 0

set ws = Nothing
