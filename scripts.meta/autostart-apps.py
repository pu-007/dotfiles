import asyncio
import subprocess
from dataclasses import dataclass
from time import time
from typing import Union, List

import pyautogui
from pywinauto import Application
from pywinauto.findwindows import ElementNotFoundError

# 提前关闭安全保护，防止鼠标移动到角落导致脚本抛出异常中止
pyautogui.FAILSAFE = False

# ==========================================
# 1. 配置区域 (Configuration)
# ==========================================


@dataclass
class AppConfig:
    """定义应用启动配置的数据类，方便统一管理"""

    cmd: Union[str, List[str]]
    cwd: str = None
    hide_window: bool = True


# 统一维护的开机自启应用列表 (增删应用只需在这里修改)
STARTUP_APPS = [
    AppConfig(r"C:\Users\zion\scoop\apps\cc-switch\current\cc-switch.exe"),
    AppConfig(r"C:\Users\zion\AppData\Local\Focust\focust.exe"),
    AppConfig(r"C:\Users\zion\AppData\Local\health-reminder\health-reminder.exe"),
    AppConfig(r"C:\Users\zion\Apps\KeyStats\KeyStats.exe"),
    AppConfig(r"C:\Program Files\KDE Connect\bin\kdeconnect-indicator.exe"),
    AppConfig(r"C:\Users\zion\AppData\Local\Doubao\Application\Doubao.exe"),
    AppConfig(r"C:\Program Files\EcoPaste-Sync\EcoPaste-Sync.exe"),
    AppConfig(r"C:\Program Files\flomo\flomo.exe"),
    AppConfig(["wt.exe", "-w", "_quake", "-p", "special_quake_window_title"]),
    AppConfig(r"C:\Program Files\Quicker\Quicker.exe"),
    AppConfig([r"C:\Program Files\Everything\Everything.exe", "-startup"]),
    AppConfig([r"C:\Program Files\komorebi\bin\komorebic-no-console.exe", "start"]),
    AppConfig(
        [
            r"C:\Program Files\AutoHotkey\v2\AutoHotkey.exe",
            r"C:\Users\zion\komorebi.ahk",
        ]
    ),
    AppConfig(r"C:\Program Files\YASB\yasb.exe"),
    AppConfig(r"C:\Users\zion\AppData\Local\Programs\utools\uTools.exe"),
    AppConfig(r"C:\Program Files\Mem Reduct\memreduct.exe"),
    AppConfig(r"C:\Users\zion\AppData\Roaming\AltSnap\AltSnap.exe"),
    AppConfig(r"C:\Users\zion\AppData\Local\Programs\PixPin\PixPin.exe"),
    AppConfig([r"C:\Program Files (x86)\Stardock\Fences\Fences.exe", "/startup"]),
    AppConfig(r"C:\Users\zion\Apps\Controller Companion\ControllerCompanion.exe"),
    AppConfig(
        r"C:\Program Files\Pantum\ptm6700\SCANNER\PushScan\ptm6700PushMonitor.exe"
    ),
    AppConfig(
        [
            r"C:\Windows\System32\DriverStore\FileRepository\realtekservice.inf_amd64_d2d4c5f34960aaac\RtkAudUService64.exe",
            "-background",
        ]
    ),
    AppConfig(r"C:\Program Files\Rime\weasel-0.17.4\WeaselServer.exe"),
    AppConfig(
        [r"C:\Users\zion\AppData\Local\Programs\QuickLook\QuickLook.exe", "-autorun"]
    ),
    AppConfig(r"C:\Users\zion\Apps\capslockpp\CapsLock++.exe"),
    AppConfig(r"C:\Program Files\Docker\Docker\Docker Desktop.exe"),
    AppConfig([r"C:\Program Files (x86)\PasteIntoFile\PasteIntoFile.exe", "tray"]),
    AppConfig(r"C:\Users\zion\Apps\IME_Indicator\IME-Indicator.exe"),
    AppConfig(
        ["pythonw", r"\\wsl.localhost\Arch\home\pu\Source\cut_in_xiaoai\main.py"]
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


def launch_app_sync(config: AppConfig):
    """
    同步启动应用程序。
    说明：subprocess.Popen 本身就是非阻塞的，无需使用 async 包装。
    """
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


async def manage_window_by_title(
    title: str, action: str = "close", timeout: float = 10.0, interval: float = 0.5
) -> bool:
    """
    异步管理（关闭/最小化）指定标题的窗口。整合了以前的 close 和 minimize 方法。
    """
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
    """微信全自动登录并隐藏的逻辑封装"""
    app = Application(backend="uia").start(wechat_path)

    # 1. 等待登录窗口出现
    login_dlg = app.window(title="微信")
    start_time = time()
    while time() - start_time < 15:
        if login_dlg.exists(timeout=0):
            break
        await asyncio.sleep(0.5)
    else:
        print("超时：未能找到微信登录窗口")
        return

    # 2. 记录初始大小并发送回车
    try:
        rect = login_dlg.rectangle()
        init_width, init_height = rect.width(), rect.height()
        login_dlg.set_focus()
        login_dlg.type_keys("{ENTER}")
    except Exception as e:
        print(f"操作登录窗口失败: {e}")
        return

    # 3. 监控窗口变化判断登录成功
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
        except ElementNotFoundError:
            pass  # 过渡瞬间的正常现象
        except Exception:
            pass  # 忽略 COM 接口刷新异常

        await asyncio.sleep(0.5)

    # 4. 登录成功后隐藏
    if logged_in:
        try:
            app.window(title="微信").close()
            print("微信已登录并隐藏至托盘")
        except Exception as e:
            print(f"微信隐藏失败: {e}")
    else:
        print("微信登录超时或未检测到窗口变化")


# ==========================================
# 3. 主流程 (Main Flow)
# ==========================================


async def main():
    # 1. 瞬间遍历触发所有基础应用的启动 (由于是进程分离，瞬间完成，不会阻塞)
    for app_config in STARTUP_APPS:
        launch_app_sync(app_config)

    # 2. 并发执行需要长时间轮询和等待的异步任务（例如微信UI自动化、等待豆包窗口出现并关闭）
    await asyncio.gather(
        auto_login_wechat(),
        manage_window_by_title("豆包", action="close", timeout=15.0),
    )


if __name__ == "__main__":
    asyncio.run(main())
