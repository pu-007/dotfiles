set ws=wscript.CreateObject("wscript.shell")

ws.run "python \\wsl.localhost\Arch\home\pu\dotfiles\scripts.meta\autostart-apps.py", 0
ws.run "python \\wsl.localhost\Arch\home\pu\Source\cut_in_xiaoai\main.py", 0
ws.run "aria2c --dir='D:\Downloads' --enable-rpc --rpc-listen-all=true --rpc-allow-origin-all=true --file-allocation=falloc -c -D -x 16 -s 16 -j 10 -k 1M --disk-cache=256M --enable-dht=true --bt-enable-lpd=true --enable-peer-exchange=true", 0

set ws = Nothing
