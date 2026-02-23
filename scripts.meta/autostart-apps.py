import asyncio
import subprocess

from typing import List, Union, Optional
from time import time

import pyautogui
import psutil

from pywinauto import Application
from pywinauto.findwindows import ElementNotFoundError


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
    commands: Union[str, List[str]], cwd: Optional[str] = None, hide_window=True
):
    try:
        if hide_window:
            startupinfo = subprocess.STARTUPINFO()
            startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            startupinfo.wShowWindow = 0  # SW_HIDE

            await asyncio.to_thread(
                lambda: subprocess.Popen(
                    commands,
                    shell=False,
                    close_fds=True,
                    cwd=cwd,
                    creationflags=subprocess.CREATE_NO_WINDOW
                    | subprocess.DETACHED_PROCESS,
                    startupinfo=startupinfo,
                )
            )
        else:
            await asyncio.to_thread(
                lambda: subprocess.Popen(
                    commands,
                    shell=False,
                    close_fds=True,
                    cwd=cwd,
                    creationflags=subprocess.DETACHED_PROCESS,
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


async def launch_wechat():
    # 启动微信 (使用 uia 后端对现代 UI 兼容性更好)
    app = Application(backend="uia").start(
        r"C:\Program Files\Tencent\Weixin\Weixin.exe"
    )

    # 【1】等待登录窗口出现 (开机时给予 15 秒缓冲)
    login_dlg = app.window(title="微信")
    start_time = time()
    while time() - start_time < 15:
        if login_dlg.exists(timeout=0):
            break
        await asyncio.sleep(0.2)
    else:
        print("超时：未能找到微信登录窗口")
        return

    # 【2】记录初始窗口大小并发送回车
    try:
        # 获取登录窗口的初始长宽
        rect = login_dlg.rectangle()
        init_width, init_height = rect.width(), rect.height()

        # 强制获取焦点并发送回车键
        login_dlg.set_focus()
        login_dlg.type_keys("{ENTER}")
        print("已发送回车登录指令")
    except Exception as e:
        print(f"操作登录窗口失败: {e}")
        return

    # 【3】实时监控窗口变化，判断是否登录成功
    logged_in = False
    monitor_start = time()

    # 给予 20 秒的时间等待网络登录和主窗口加载
    while time() - monitor_start < 20:
        try:
            # 重新寻找名为“微信”的窗口（因为登录后会生成新的同名主窗口）
            current_dlg = app.window(title="微信")

            if current_dlg.exists(timeout=0):
                new_rect = current_dlg.rectangle()
                new_width, new_height = new_rect.width(), new_rect.height()

                # 如果长或宽明显大于登录窗口（加50像素作为容差），说明主窗口已加载
                if new_width > init_width + 50 or new_height > init_height + 50:
                    logged_in = True
                    break

        except ElementNotFoundError:
            # 在登录过渡瞬间，旧窗口销毁、新窗口还未建立时会触发此异常，安全忽略即可
            pass
        except Exception as e:
            # 忽略过渡期间其他由于 COM 接口刷新导致的临时异常
            pass

        await asyncio.sleep(0.2)

    # 【4】登录成功后，隐藏到系统托盘
    if logged_in:
        print("检测到窗口变大，登录成功！")
        # 额外等待 1.5 秒，确保主窗口的内容和后台进程完全初始化完毕，避免过早关闭导致微信退出
        # await asyncio.sleep(1.5)

        try:
            main_dlg = app.window(title="微信")
            # 在微信的默认设置中，触发关闭操作（Close）并不会结束进程，而是隐藏到右下角托盘
            main_dlg.close()
            print("已成功将微信隐藏至任务托盘。")
        except Exception as e:
            print(f"隐藏窗口失败: {e}")
    else:
        print("超时：登录未能成功或窗口大小未发生变化。")


# 测试运行
# asyncio.run(launch_wechat())


def launch(commands: str | list, cwd: str | None = None) -> asyncio.Task:
    return asyncio.create_task(_async_launch_app(commands, cwd))


async def main():
    await asyncio.gather(
        asyncio.create_task(launch_wechat()),
        asyncio.create_task(_close_windows_by_title("豆包")),
        launch(r"C:\Users\zion\scoop\apps\cc-switch\current\cc-switch.exe"),
        launch(r"C:\Users\zion\AppData\Local\Focust\focust.exe"),
        launch(r"C:\Users\zion\AppData\Local\health-reminder\health-reminder.exe"),
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
