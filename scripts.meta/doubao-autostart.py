# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "pyautogui",
# ]
# ///

import subprocess
import pyautogui
import os
from time import sleep

def launch_and_hind_app(commands, window_title, hotkey_combination, lazy=0):

    exe_name = os.path.basename(commands if isinstance(commands, str) else commands[0])
    try:
        subprocess.run(commands, check=True)
        print(f"{exe_name} 启动成功。")
        app_window = None
        while not app_window:
            app_window = pyautogui.getWindowsWithTitle(window_title)
        app_window[0].activate()
        sleep(lazy)
        try:
            pyautogui.hotkey(*hotkey_combination)
            print(f"快捷键 {hotkey_combination} 已发送，窗口应已处理。")
        except Exception as e:
            print(f"发送快捷键失败：{str(e)}")
    except subprocess.CalledProcessError:
        print(f"启动 {exe_name} 失败")

# 调用豆包
launch_and_hind_app(
    r"C:\Users\zion\AppData\Local\Doubao\Application\Doubao.exe",
    "豆包 - 字节跳动旗下 AI 智能助手 - 豆包",
    ["ctrl", "shift", "a"]
)

launch_and_hind_app(
    # EXE: wt.exe -w _quake -p "Arch_quake"
    # hide key: alt+`
    [r"wt.exe", "-w", "_quake", "-p", "Arch_quake"],
    "Arch_quake",
    ["alt", '`'],
    lazy=1.5
)
