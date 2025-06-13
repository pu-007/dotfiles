set ws=wscript.CreateObject("wscript.shell")

ws.run "python \\wsl.localhost\Arch\home\pu\dotfiles\scripts.meta\autostart-apps.py", 0
ws.run "python \\wsl.localhost\Arch\home\pu\Source\cut_in_xiaoai\src", 0

set ws = Nothing
