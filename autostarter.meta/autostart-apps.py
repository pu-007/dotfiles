# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "pyautogui",
# ]
# ///

from gettext import find
import subprocess
import pyautogui
import os
from time import sleep


def find_window_by_title(title):
    app_window = None
    while not app_window:
        app_window = pyautogui.getWindowsWithTitle(title)
    return app_window


# 启动应用，支持通过 title 来判断是否成功启动，支持自定义快捷键来隐藏窗口的 hook, lazy 在窗口激活后延迟多少秒发送快捷键
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


# 启动应用，不隐藏，不检测是否启动，支持自定义 hook 函数
def launch_app(commands, hook=None, cwd=None, delay=0):
    exe_name = os.path.basename(commands if isinstance(commands, str) else commands[0])
    try:
        subprocess.Popen(commands, cwd=cwd, shell=True)
        print(f"{exe_name} 启动成功。")
        if delay > 0:
            sleep(delay)
        if hook:
            hook()
    except Exception as e:
        print(f"启动 {exe_name} 失败: {str(e)}")


# # 调用豆包
launch_and_hind_app(
    r"C:\Users\zion\AppData\Local\Doubao\Application\Doubao.exe",
    "豆包 - 字节跳动旗下 AI 智能助手 - 豆包",
    ["ctrl", "shift", "a"],
)

launch_and_hind_app(
    [r"wt.exe", "-w", "_quake", "-p", "Arch_quake"], "Arch_quake", ["alt", "`"], lazy=2
)

launch_app(
    [r"C:\Program Files\komorebi\bin\komorebic-no-console.exe", "start", "--ahk"],
    cwd=r"C:\Program Files\komorebi",
)

launch_app(
    [r"C:\Users\zion\AppData\Roaming\npm\node_modules\capslockx\CapsLockX.exe"],
    cwd=r"C:\Users\zion\AppData\Roaming\npm\node_modules\capslockx",
)

launch_app(
    [r"C:\Users\zion\AppData\Local\Programs\QuickLook\QuickLook.exe", "/autorun"],
    cwd=r"C:\Users\zion\AppData\Local\Programs\QuickLook",
    delay=1,
    hook=lambda: launch_app(
        [r"C:\Users\zion\AppData\Roaming\npm\node_modules\capslockx\CapsLockX.exe"],
        cwd=r"C:\Users\zion\AppData\Roaming\npm\node_modules\capslockx",
        hook=lambda: find_window_by_title("Arch")[0].close(),
        delay=2,
    ),
)

launch_app(
    [r"C:\Program Files\Yasb\yasb.exe"],
    cwd=r"C:\Program Files\Yasb",
)

launch_app(
    [r"C:\Users\zion\Apps\catime.exe"],
)

launch_app(
    [r"C:\Users\zion\AppData\Local\Programs\Ollama\ollama app.exe"],
    cwd=r"C:\Users\zion\AppData\Local\Programs\Ollama",
)
