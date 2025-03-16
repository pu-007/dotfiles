set ws=wscript.CreateObject("wscript.shell")

ws.run "uv run \\wsl.localhost\Arch\home\pu\Source\autostart-apps\doubao-autostart.py", 0
ws.run "uv run --with paho-mqtt,pyautogui,screeninfo  \\wsl.localhost\Arch\home\pu\Source\cut_in_xiaoai\src", 0