import asyncio
import subprocess

from typing import List, Union, Optional
from time import time

import pyautogui
import psutil


async def _close_windows_by_title(
    title: str, timeout: float = 10.0, interval: float = 0.2
) -> list | None:
    """
    Asynchronously finds a window by its title and performs a hook function.
    Runs blocking pyautogui calls in a separate thread.
    """
    end_time = time() + timeout
    while time() <= end_time:
        windows = await asyncio.to_thread(pyautogui.getWindowsWithTitle, title)
        if windows:
            return [w.close() for w in windows]
        await asyncio.sleep(interval)


async def _minimize_windows_by_title(
    title: str, timeout: float = 10.0, interval: float = 0.2
) -> list | None:
    end_time = time() + timeout
    while time() <= end_time:
        windows = await asyncio.to_thread(pyautogui.getWindowsWithTitle, title)
        if windows:
            return [w.minimize() for w in windows]
        await asyncio.sleep(interval)


async def _async_launch_app(
    commands: Union[str, List[str]],
    cwd: Optional[str] = None,
):
    try:
        startupinfo = subprocess.STARTUPINFO()
        startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        startupinfo.wShowWindow = 0  # SW_HIDE

        await asyncio.to_thread(
            lambda: subprocess.Popen(
                commands,
                shell=False,
                close_fds=True,
                cwd=cwd,
                creationflags=subprocess.CREATE_NO_WINDOW | subprocess.DETACHED_PROCESS,
                startupinfo=startupinfo,
            )
        )
    except FileNotFoundError as e:
        print(f"Error launching application: {e}")
    except Exception as e:
        print(f"An unexpected error occurred: {e}")


def _find_first_process_sync(name: str) -> Optional[psutil.Process]:
    """
    同步地查找第一个匹配名称的进程。
    找到一个后立即返回，以提高效率。
    """
    # 使用生成器表达式 (p for p in ...) 和 next()
    # next() 会在找到第一个元素后立即停止迭代
    try:
        return next(p for p in psutil.process_iter(["name"]) if p.name() == name)
    except StopIteration:
        # 如果生成器耗尽（即没有找到匹配的进程），next() 会抛出 StopIteration
        return None


async def _wait_for(name, timeout: float = 10.0, interval: float = 0.2) -> None:
    end_time = time() + timeout
    while time() <= end_time:
        matched_process = await asyncio.to_thread(_find_first_process_sync, name)
        if matched_process:
            return
        await asyncio.sleep(interval)


async def close_windows_matching_titles():
    await _close_windows_by_title("豆包")


def launch(commands: str | list, cwd: str | None = None) -> asyncio.Task:
    return asyncio.create_task(_async_launch_app(commands, cwd))


async def main():
    await asyncio.gather(
        asyncio.create_task(close_windows_matching_titles()),
        launch(r"C:\Users\zion\Apps\KeyStats\KeyStats.exe"),
        launch(r"C:\Program Files\KDE Connect\bin\kdeconnect-indicator.exe"),
        launch(r"C:\Users\zion\AppData\Local\Doubao\Application\Doubao.exe"),
        launch(r"C:\Program Files\EcoPaste-Sync\EcoPaste-Sync.exe"),
        launch(r"C:\Program Files\flomo\flomo.exe"),
        launch(["wt.exe", "-w", "_quake", "-p", "special_quake_window_title"]),
        launch(r"C:\Program Files\Quicker\Quicker.exe"),
        launch([r"C:\Program Files\Everything\Everything.exe", "-startup"]),
        launch(
            [
                r"C:\Program Files\komorebi\bin\komorebic-no-console.exe",
                "start",
            ]
        ),
        launch(
            [
                r"C:\Program Files\AutoHotkey\v2\AutoHotkey.exe",
                r"C:\Users\zion\komorebi.ahk",
            ]
        ),
        launch(r"C:\Program Files\YASB\yasb.exe"),
        launch(r"C:\Users\zion\AppData\Local\Programs\utools\uTools.exe"),
        launch(r"C:\Program Files\Mem Reduct\memreduct.exe"),
        launch(r"C:\Users\zion\AppData\Roaming\AltSnap\AltSnap.exe"),
        launch(r"C:\Users\zion\AppData\Local\Programs\PixPin\PixPin.exe"),
        launch([r"C:\Program Files (x86)\Stardock\Fences\Fences.exe", "/startup"]),
        launch(r"C:\Users\zion\Apps\Controller Companion\ControllerCompanion.exe"),
        launch(
            r"C:\Program Files\Pantum\ptm6700\SCANNER\PushScan\ptm6700PushMonitor.exe"
        ),
        launch(
            [
                r"C:\Windows\System32\DriverStore\FileRepository\realtekservice.inf_amd64_d2d4c5f34960aaac\RtkAudUService64.exe",
                "-background",
            ]
        ),
        launch(r"C:\Program Files\Rime\weasel-0.17.4\WeaselServer.exe"),
        launch(
            [
                r"C:\Users\zion\AppData\Local\Programs\QuickLook\QuickLook.exe",
                "-autorun",
            ]
        ),
        launch(r"C:\Users\zion\Apps\capslockpp\CapsLock++.exe"),
        launch(r"C:\Program Files\Docker\Docker\Docker Desktop.exe"),
        launch(r"C:\Users\zion\Apps\ProjectEye\ProjectEye.exe"),
        launch([r"C:\Program Files (x86)\PasteIntoFile\PasteIntoFile.exe", "tray"]),
        launch(r"C:\Users\zion\Apps\IME_Indicator\IME-Indicator.exe"),
        launch(
            ["pythonw", r"\\wsl.localhost\Arch\home\pu\Source\cut_in_xiaoai\main.py"]
        ),
        launch(
            [
                "aria2c",
                "--dir=D:\\Downloads",
                "--enable-rpc",
                "--rpc-listen-all=true",
                "--rpc-allow-origin-all=true",
                "--file-allocation=falloc",
                "-c",
                "-x",
                "16",
                "-s",
                "16",
                "-j",
                "10",
                "-k",
                "1M",
                "--disk-cache=256M",
                "--enable-dht=true",
                "--bt-enable-lpd=true",
                "--enable-peer-exchange=true",
            ]
        ),
    )


if __name__ == "__main__":
    asyncio.run(main())
    pyautogui.FAILSAFE = False
