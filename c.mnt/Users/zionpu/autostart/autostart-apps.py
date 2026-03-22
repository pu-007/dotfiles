import asyncio
import subprocess
from dataclasses import dataclass
from time import time
from typing import Union, List

import pyautogui

# 提前关闭安全保护，防止意外中止
pyautogui.FAILSAFE = False

# ==========================================
# 1. 配置区域 (Configuration)
# ==========================================


@dataclass
class AppConfig:
    cmd: Union[str, List[str]]
    cwd: str | None = None
    hide_window: bool = True


# 统一维护的开机自启应用列表
STARTUP_APPS = [
    # AppConfig(r"C:\Users\zion\scoop\apps\cc-switch\current\cc-switch.exe"),
    # AppConfig(r"C:\Users\zionpu\AppData\Local\Focust\focust.exe"),
    # AppConfig(r"C:\Users\zionpu\AppData\Local\health-reminder\health-reminder.exe"),
    # AppConfig(r"C:\Users\zionpu\Apps\KeyStats\KeyStats.exe"),
    # AppConfig(r"C:\Program Files\KDE Connect\bin\kdeconnect-indicator.exe"),
    # AppConfig(r"C:\Users\zionpu\Apps\IME_Indicator\IME-Indicator.exe"),
    # AppConfig(r"C:\Program Files\Rime\weasel-0.17.4\WeaselServer.exe"),
    # AppConfig(
    #     [
    #         r"C:\Windows\System32\DriverStore\FileRepository\realtekservice.inf_amd64_d2d4c5f34960aaac\RtkAudUService64.exe",
    #         "-background",
    #     ]
    # ),
    # AppConfig(r"C:\Users\zionpu\Apps\Controller Companion\ControllerCompanion.exe"),
    AppConfig(
        [r"C:\Users\zionpu\AppData\Local\Programs\QuickLook\QuickLook.exe", "-autorun"]
    ),
    AppConfig(r"C:\Users\zionpu\AppData\Local\Doubao\Application\Doubao.exe"),
    AppConfig(r"C:\Program Files\EcoPaste-Sync\EcoPaste-Sync.exe"),
    AppConfig(r"C:\Program Files\flomo\flomo.exe"),
    AppConfig(["wt.exe", "-w", "_quake", "-p", "special_quake_window_title"]),
    AppConfig(r"C:\Program Files\Quicker\Quicker.exe"),
    AppConfig([r"C:\Program Files\Everything\Everything.exe", "-startup"]),
    AppConfig([r"C:\Program Files\komorebi\bin\komorebic-no-console.exe", "start"]),
    AppConfig(
        [
            r"C:\Users\zionpu\AppData\Local\Programs\AutoHotkey\v2\AutoHotkey64.exe",
            r"C:\Users\zionpu\komorebi.ahk",
        ]
    ),
    AppConfig(r"C:\Program Files (x86)\滴答清单\TickTick.exe"),
    AppConfig(r"C:\Program Files\YASB\yasb.exe"),
    AppConfig(r"C:\Users\zionpu\AppData\Local\Programs\utools\uTools.exe"),
    AppConfig(r"C:\Program Files\Mem Reduct\memreduct.exe"),
    AppConfig(r"C:\Users\zionpu\AppData\Roaming\AltSnap\AltSnap.exe"),
    AppConfig(r"C:\Users\zionpu\AppData\Local\Programs\PixPin\PixPin.exe"),
    AppConfig([r"C:\Program Files (x86)\Stardock\Fences\Fences.exe", "/startup"]),
    AppConfig(
        r"C:\Program Files\Pantum\ptm6700\SCANNER\PushScan\ptm6700PushMonitor.exe"
    ),
    AppConfig(r"C:\Users\zionpu\Apps\capslockpp\CapsLock++.exe"),
    AppConfig(r"C:\Program Files\Docker\Docker\Docker Desktop.exe"),
    AppConfig([r"C:\Program Files (x86)\PasteIntoFile\PasteIntoFile.exe", "tray"]),
    AppConfig(
        ["pixi", "run", "-m", r"C:\Users\zionpu\cut_in_xiaoai\pyproject.toml", "start"]
    ),
    AppConfig(
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
]

# ==========================================
# 2. 核心功能方法 (Core Functions)
# ==========================================


async def launch_app_async(config: AppConfig):
    """
    异步多线程启动应用程序。
    利用线程池将进程创建请求瞬间并发甩给 Windows 操作系统。
    """

    def _start_process():
        try:
            flags = subprocess.DETACHED_PROCESS
            startupinfo = None

            if config.hide_window:
                flags |= subprocess.CREATE_NO_WINDOW
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
                startupinfo.wShowWindow = 0  # SW_HIDE

            subprocess.Popen(
                config.cmd,
                shell=False,
                close_fds=True,
                cwd=config.cwd,
                creationflags=flags,
                startupinfo=startupinfo,
            )
        except Exception as e:
            cmd_name = config.cmd[0] if isinstance(config.cmd, list) else config.cmd
            print(f"启动失败 [{cmd_name}]: {e}")

    # 将阻塞的 Popen 调用推入 asyncio 底层的线程池执行
    await asyncio.to_thread(_start_process)


async def manage_window_by_title(
    title: str, action: str = "close", timeout: float = 10.0, interval: float = 0.5
) -> bool:
    end_time = time() + timeout
    while time() <= end_time:
        windows = await asyncio.to_thread(pyautogui.getWindowsWithTitle, title)
        if windows:
            for w in windows:
                w.close() if action == "close" else w.minimize()
            return True
        await asyncio.sleep(interval)
    return False


async def auto_login_wechat(
    wechat_path: str = r"C:\Program Files\Tencent\Weixin\Weixin.exe",
):
    from pywinauto import Application
    from pywinauto.findwindows import ElementNotFoundError

    app = Application(backend="uia").start(wechat_path)
    login_dlg = app.window(title="微信")

    start_time = time()
    while time() - start_time < 15:
        if login_dlg.exists(timeout=0):
            break
        await asyncio.sleep(0.5)
    else:
        print("超时：未能找到微信登录窗口")
        return

    try:
        rect = login_dlg.rectangle()
        init_width, init_height = rect.width(), rect.height()
        login_dlg.set_focus()
        login_dlg.type_keys("{ENTER}")
    except Exception as e:
        print(f"操作登录窗口失败: {e}")
        return

    logged_in = False
    monitor_start = time()
    while time() - monitor_start < 20:
        try:
            current_dlg = app.window(title="微信")
            if current_dlg.exists(timeout=0):
                new_rect = current_dlg.rectangle()
                if (
                    new_rect.width() > init_width + 50
                    or new_rect.height() > init_height + 50
                ):
                    logged_in = True
                    break
        except (ElementNotFoundError, Exception):
            pass
        await asyncio.sleep(0.5)

    if logged_in:
        try:
            app.window(title="微信").close()
            print("微信已登录并隐藏")
        except Exception as e:
            print(f"微信隐藏失败: {e}")


# ==========================================
# 3. 主流程 (Main Flow)
# ==========================================


async def main():
    # 将所有的普通应用启动任务打包
    startup_tasks = [launch_app_async(app) for app in STARTUP_APPS]

    # 【核心改动】：使用 gather 将所有任务一次性全部推入事件循环！
    # 这意味着 30个软件的启动 + 微信自动化登录 + 等待豆包窗口关闭，全部在同一时间点并发执行。
    await asyncio.gather(
        *startup_tasks,
        auto_login_wechat(),
        manage_window_by_title("豆包", action="close", timeout=15.0),
        manage_window_by_title("滴答清单", action="close", timeout=15.0),
        manage_window_by_title("archlinux", action="close", timeout=15.0),
    )


if __name__ == "__main__":
    asyncio.run(main())
