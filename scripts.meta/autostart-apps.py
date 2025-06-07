# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "pyautogui",
# ]
# ///

from typing import Callable
import subprocess
import pyautogui
import os
from time import sleep


def find_window_by_title(title: str,
                         action: Callable,
                         timeout: float = 5.0) -> list | None:
    app_window = []
    elapsed_time = 0
    while not app_window and elapsed_time < timeout:
        app_window = pyautogui.getWindowsWithTitle(title)
        if not app_window:
            sleep(0.5)
            elapsed_time += 0.5
    if action:
        action(app_window)
    return app_window if app_window else None


# 启动应用，支持通过 title 来判断是否成功启动，支持自定义快捷键来隐藏窗口的 hook, lazy 在窗口激活后延迟多少秒发送快捷键
def launch_and_hind_app(commands, window_title, hotkey_combination, delay=0):

    exe_name = os.path.basename(
        commands if isinstance(commands, str) else commands[0])
    try:
        subprocess.run(commands, check=True)
        print(f"{exe_name} 启动成功。")
        find_window_by_title(
            window_title,
            lambda w: w[0].activate() if w else None,
            timeout=5,
        )
        sleep(delay)
        try:
            pyautogui.hotkey(*hotkey_combination)
            print(f"快捷键 {hotkey_combination} 已发送，窗口应已处理。")
        except Exception as e:
            print(f"发送快捷键失败：{str(e)}")
    except subprocess.CalledProcessError:
        print(f"启动 {exe_name} 失败")


# 启动应用，不隐藏，不检测是否启动，支持自定义 hook 函数
def launch_app(commands, hook=None, cwd=None, delay=0):
    exe_name = os.path.basename(
        commands if isinstance(commands, str) else commands[0])
    try:
        subprocess.Popen(commands, cwd=cwd, shell=True)
        print(f"{exe_name} 启动成功。")
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

launch_and_hind_app([r"wt.exe", "-w", "_quake", "-p", "Arch_quake"],
                    "Arch_quake", ["alt", "`"],
                    delay=2)

launch_app(
    [
        r"C:\Program Files\komorebi\bin\komorebic-no-console.exe", "start",
        "--ahk", "--bar"
    ],
    cwd=r"C:\Program Files\komorebi",
)


def _close_all_matched_windows(window_list: list):
    return [i.close() for i in window_list]


def _close_unexpected_window():
    find_window_by_title("Arch", _close_all_matched_windows)
    find_window_by_title("CapsLockX-Core.ahk", _close_all_matched_windows)


launch_app(
    # 先打开 quick look 然后再次打开 capslockx，可以避免空格键无法出发 quicklook，原因未知
    # 又突然好了，可以不用打开两次 CapsLockX 了，鬼知道为什么，啥也没干
    [
        r"C:\Users\zion\AppData\Local\Programs\QuickLook\QuickLook.exe",
        "/autorun"
    ],
    cwd=r"C:\Users\zion\AppData\Local\Programs\QuickLook",
    delay=1,
    hook=lambda: launch_app(
        [r"C:\Users\zion\Apps\CapsLockX\CapsLockX.exe"],
        cwd=r"C:\Users\zion\Apps\CapsLockX",
        # 会莫名其妙地出现一个 wt 或者 auto hotkey 窗口，原因未知，只能自动关闭
        hook=lambda: _close_unexpected_window(),
        delay=2,
    ),
)

# for lazy load

# sleep(3)

launch_app(
    [r"C:\Users\zion\AppData\Local\Programs\Ollama\ollama app.exe"],
    cwd=r"C:\Users\zion\AppData\Local\Programs\Ollama",
)

# launch_app(
#     # C:\Users\zion\AppData\Local\Microsoft\WinGet\Links\catime.exe
#     [r"C:\Users\zion\AppData\Local\Microsoft\WinGet\Links\catime.exe"], )
