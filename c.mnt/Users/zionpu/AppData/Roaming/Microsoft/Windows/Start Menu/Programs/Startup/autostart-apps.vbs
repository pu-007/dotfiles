set ws=wscript.CreateObject("wscript.shell")

ws.run "pixi run -m C:\Users\zionpu\autostart\pixi.toml autostart", 0

set ws = Nothing
