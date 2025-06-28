import asyncio
import subprocess

from typing import List, Union, Optional
from time import time

import pyautogui
import psutil


async def _close_windows_by_title(title: str,
                                  timeout: float = 5.0,
                                  interval: float = 0.3) -> None:
    """
    Asynchronously finds a window by its title and performs a hook function.
    Runs blocking pyautogui calls in a separate thread.
    """
    end_time = time() + timeout
    while time() <= end_time:
        windows = await asyncio.to_thread(pyautogui.getWindowsWithTitle, title)
        if windows:
            for window in windows:
                window.close()
            return
        await asyncio.sleep(interval)


async def _async_launch_app(
    commands: Union[str, List[str]],
    cwd: Optional[str] = None,
):
    await asyncio.to_thread(
        lambda: subprocess.Popen(commands,
                                 shell=False,
                                 close_fds=True,
                                 cwd=cwd,
                                 creationflags=subprocess.DETACHED_PROCESS))


def _find_first_process_sync(name: str) -> Optional[psutil.Process]:
    """
    同步地查找第一个匹配名称的进程。
    找到一个后立即返回，以提高效率。
    """
    # 使用生成器表达式 (p for p in ...) 和 next()
    # next() 会在找到第一个元素后立即停止迭代
    try:
        return next(p for p in psutil.process_iter(['name'])
                    if p.name() == name)
    except StopIteration:
        # 如果生成器耗尽（即没有找到匹配的进程），next() 会抛出 StopIteration
        return None


async def _wait_for(name, timeout: float = 5.0, interval: float = 0.3) -> None:
    end_time = time() + timeout
    while time() <= end_time:
        matched_process = await asyncio.to_thread(_find_first_process_sync,
                                                  name)
        if matched_process:
            return
        await asyncio.sleep(interval)


async def launch_quicklook_and_capslockx():
    await _async_launch_app([
        r"C:\Users\zion\AppData\Local\Programs\QuickLook\QuickLook.exe",
        "-autorun"
    ])
    await _wait_for("QuickLook.exe")
    await _async_launch_app([r"C:\Users\zion\Apps\CapsLockX\CapsLockX.exe"], )
    await _close_windows_by_title("CapsLockX-Core.ahk")


async def launch_quake_and_clipboard():
    await _async_launch_app(r"C:\Program Files\Quicker\Quicker.exe")
    await _async_launch_app(commands=[
        "wt.exe", "-w", "_quake", "-p", "special_quake_window_title"
    ])
    await _close_windows_by_title("Arch")
    await asyncio.sleep(5)
    await asyncio.to_thread(pyautogui.hotkey, "alt", "`")
    await asyncio.sleep(1)
    await asyncio.to_thread(pyautogui.hotkey, "ctrl", "shift", "x")
    await _close_windows_by_title("剪贴板")


async def launch_doubao():
    await _async_launch_app(
        r"C:\Users\zion\AppData\Local\Doubao\Application\Doubao.exe")
    await _close_windows_by_title("豆包 - 字节跳动旗下 AI 智能助手 - 豆包")


def launch(commands: str | list, cwd: str | None = None) -> asyncio.Task:
    return asyncio.create_task(_async_launch_app(commands, cwd))


async def main():
    await asyncio.gather(
        asyncio.create_task(launch_quicklook_and_capslockx()),
        asyncio.create_task(launch_doubao()),
        asyncio.create_task(launch_quake_and_clipboard()),
        launch([r"C:\Program Files\Everything\Everything.exe", "-startup"]),
        launch([
            r"C:\Program Files\komorebi\bin\komorebic-no-console.exe",
            "start",
        ]),
        launch(r"C:\Users\zion\AppData\Local\Programs\utools\uTools.exe"),
        launch([
            r"C:\Program Files\AutoHotkey\v2\AutoHotkey.exe",
            r"C:\Users\zion\komorebi.ahk"
        ]),
        launch(r"C:\Program Files\YASB\yasb.exe"),
        launch(r"C:\Program Files\Mem Reduct\memreduct.exe"),
        launch(r"C:\Users\zion\AppData\Local\Programs\Motrix\Motrix.exe"),
        launch(r"C:\Users\zion\AppData\Roaming\AltSnap\AltSnap.exe"),
        launch(
            r"C:\Users\zion\AppData\Local\Programs\twinkle-tray\Twinkle Tray.exe"
        ),
        launch(r"C:\Users\zion\AppData\Local\Programs\PixPin\PixPin.exe"),
        launch(
            r"C:\Program Files (x86)\Tobias Erichsen\loopMIDI\loopMIDI.exe"),
        launch(
            [r"C:\Program Files (x86)\Stardock\Fences\Fences.exe",
             "/startup"]),
        launch(
            r"C:\Users\zion\Apps\Controller Companion\ControllerCompanion.exe"
        ),
        launch([
            r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe',
            r'--no-startup-window', r'--win-session-start'
        ]),
        launch(
            r"C:\Program Files\Pantum\ptm6700\SCANNER\PushScan\ptm6700PushMonitor.exe"
        ),
        launch([
            r"C:\Windows\System32\DriverStore\FileRepository\realtekservice.inf_amd64_d2d4c5f34960aaac\RtkAudUService64.exe",
            "-background"
        ]),
        launch(r"C:\Program Files\Rime\weasel-0.17.4\WeaselServer.exe"),
        launch([r'C:\Users\zion\AppData\Local\Programs\Ollama\ollama app.exe'
                ]),
        # launch(
        #     task_input=[
        #         r"C:\Users\zion\AppData\local\Programs\podman-desktop\Podman Desktop.exe",
        #         "--minimized"
        #     ],
        #     cwd=r"C:\Users\zion\AppData\local\Programs\podman-desktop")
    )


if __name__ == "__main__":
    pyautogui.FAILSAFE = False
    asyncio.run(main())
