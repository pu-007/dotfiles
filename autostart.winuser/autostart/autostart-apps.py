import asyncio
import os
import subprocess
from dataclasses import dataclass
from time import time
from typing import Union, List

import pyautogui

# 提前关闭安全保护，防止鼠标移动到角落时引发异常中止
pyautogui.FAILSAFE = False

# ==========================================
# 1. 任务类型定义 (Task Definitions)
# ==========================================


@dataclass
class AppLaunch:
    """定义一个应用程序启动任务"""

    cmd: Union[str, List[str]]
    cwd: str | None = None
    hide_window: bool = True
    use_shell: bool = False
    # 新增：绑定启动后的窗口管理。例如填入 "豆包"，程序启动后会自动去寻找并关闭该窗口
    after_launch_close: str | None = None
    wait_window_timeout: float = 15.0


@dataclass
class WindowAction:
    """定义一个独立的窗口管理任务（针对非本脚本直接启动的窗口）"""

    title: str
    action: str = "close"  # "close" 或 "minimize"
    timeout: float = 15.0


@dataclass
class WechatAutoLogin:
    """定义一个微信自动登录任务"""

    path: str = r"C:\Program Files\Tencent\Weixin\Weixin.exe"


# 定义配置项的类型提示
TaskType = Union[AppLaunch, WindowAction, WechatAutoLogin]


# ==========================================
# 2. 统一配置清单 (Unified Configuration)
# ==========================================
# 【核心优势】：以后无论是加软件、删软件、加自动关闭窗口，全部只在这里修改！

STARTUP_TASKS: List[TaskType] = [
    # ---------------- 自动化 Hook 任务 ----------------
    # WechatAutoLogin(),
    # ---------------- 独立窗口管理任务 ----------------
    WindowAction(title="archlinux", action="close", timeout=15.0),
    # ---------------- 娇贵 GUI 软件 (Shell启动) ----------------
    AppLaunch(
        cmd=r"C:\Program Files\Quicker\Quicker.exe",
        cwd=r"C:\Program Files\Quicker",
        use_shell=True,
    ),
    AppLaunch(
        cmd=r"C:\Users\zion\Apps\Controller Companion\ControllerCompanion.exe",
        cwd=r"C:\Users\zion\Apps\Controller Companion",
        use_shell=True,
    ),
    AppLaunch(
        cmd=r"C:\Users\zion\Apps\InputTip\InputTip.bat",
        cwd=r"C:\Users\zion\Apps\InputTip",
        use_shell=True,
    ),
    # ---------------- 带随附动作的软件 (启动后自动关闭窗口) ----------------
    AppLaunch(
        cmd=r"C:\Users\zion\AppData\Local\Doubao\Application\Doubao.exe",
        after_launch_close="豆包",
        wait_window_timeout=15.0,
    ),
    AppLaunch(
        cmd=[r"C:\Program Files (x86)\滴答清单\TickTick.exe", "-hide"],
        after_launch_close="滴答清单",
        wait_window_timeout=15.0,
    ),
    AppLaunch(
        cmd=[r"C:\Program Files\MotrixNext\motrix-next.exe", "--autostart"],
        after_launch_close="Motrix Next",
        wait_window_timeout=15.0,
    ),
    AppLaunch(
        cmd=r"C:\Users\zion\AppData\Local\Programs\LocalSend\localsend_app.exe",
        after_launch_close="LocalSend",
        wait_window_timeout=15.0,
    ),
    # ---------------- 普通后台/命令行软件 ----------------
    AppLaunch(
        cmd=r"C:\ProgramData\chocolatey\lib\eartrumpet\tools\EarTrumpet\EarTrumpet.exe"
    ),
    AppLaunch(
        [r"C:\Users\zion\AppData\Local\Programs\QuickLook\QuickLook.exe", "-autorun"]
    ),
    AppLaunch(r"C:\Users\zion\AppData\Local\TieZ\tiez-app.exe"),
    AppLaunch(r"C:\Program Files\flomo\flomo.exe"),
    AppLaunch(["wt.exe", "-w", "_quake", "-p", "Arch_quake"]),
    AppLaunch([r"C:\Program Files\komorebi\bin\komorebic-no-console.exe", "start"]),
    AppLaunch(
        [
            r"C:\Users\zion\apps\NeoCapsLockX\AutoHotkey\AutoHotkeyU64.exe",
            r"C:\Users\zion\apps\NeoCapsLockX\CapsLockX.ahk",
        ]
    ),
    AppLaunch(r"C:\Program Files\YASB\yasb.exe"),
    AppLaunch(
        r"C:\Users\zion\scoop\apps\memreduct\current\memreduct.exe",
        after_launch_close="Mem Reduct",
    ),
    AppLaunch(r"C:\Users\zion\AppData\Roaming\AltSnap\AltSnap.exe"),
    AppLaunch(r"C:\Users\zion\AppData\Local\Programs\PixPin\PixPin.exe"),
    AppLaunch([r"C:\Program Files (x86)\Stardock\Fences\Fences.exe", "/startup"]),
    AppLaunch(r"C:\Program Files\Docker\Docker\Docker Desktop.exe"),
    AppLaunch(
        r"C:\Windows\System32\DriverStore\FileRepository\realtekservice.inf_amd64_26f0df01c9da165d\RtkAudUService64.exe"
    ),
    AppLaunch(
        r"C:\Users\zion\AppData\Local\Microsoft\WinGet\Packages\Martchus.syncthingtray_Microsoft.Winget.Source_8wekyb3d8bbwe\syncthingtray.exe"
    ),
    AppLaunch([r"C:\Program Files\Everything 1.5a\Everything.exe", "-startup"]),
    AppLaunch(r"C:\Users\zion\Apps\FlowWheel\FlowWheel.exe"),
    AppLaunch(r"C:\Users\zion\Apps\Wox\wox-windows-amd64.exe"),
    AppLaunch(
        [
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            "--no-startup-window",
            "/prefetch:5",
        ]
    ),
    AppLaunch([r"C:\Program Files\Tailscale\tailscale-ipn.exe"]),
]


# ==========================================
# 3. 核心执行引擎 (Core Execution Engines)
# ==========================================


async def manage_window_by_title(
    title: str, action: str = "close", timeout: float = 10.0, interval: float = 0.5
) -> bool:
    """循环检测并操作目标窗口"""
    end_time = time() + timeout
    while time() <= end_time:
        windows = await asyncio.to_thread(pyautogui.getWindowsWithTitle, title)
        if windows:
            for w in windows:
                w.close() if action == "close" else w.minimize()
            return True
        await asyncio.sleep(interval)
    return False


async def execute_app_launch(config: AppLaunch):
    """处理 AppLaunch 任务，包含程序启动和关联的窗口处理"""

    def _start_process():
        try:
            if config.use_shell:
                target = config.cmd[0] if isinstance(config.cmd, list) else config.cmd
                original_cwd = os.getcwd()
                if config.cwd and os.path.exists(config.cwd):
                    os.chdir(config.cwd)
                os.startfile(target)
                os.chdir(original_cwd)
                return

            flags = subprocess.DETACHED_PROCESS
            startupinfo = None
            if config.hide_window:
                flags |= subprocess.CREATE_NO_WINDOW
                startupinfo = subprocess.STARTUPINFO()
                startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
                startupinfo.wShowWindow = 0

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

    # 1. 触发底层启动
    await asyncio.to_thread(_start_process)

    # 2. 如果配置了启动后关闭窗口，立刻开始非阻塞监听
    if config.after_launch_close:
        await manage_window_by_title(
            config.after_launch_close, "close", config.wait_window_timeout
        )


async def execute_wechat_login(config: WechatAutoLogin):
    """处理微信自动化登录任务"""
    from pywinauto import Application
    from pywinauto.findwindows import ElementNotFoundError

    app = Application(backend="uia").start(config.path)
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
# 4. 任务调度器 (Task Dispatcher & Main)
# ==========================================


async def main():
    """解析配置列表，生成协程并并发执行"""
    coroutines = []

    # 动态分发任务到对应的执行器
    for task in STARTUP_TASKS:
        if isinstance(task, AppLaunch):
            coroutines.append(execute_app_launch(task))
        elif isinstance(task, WindowAction):
            coroutines.append(
                manage_window_by_title(task.title, task.action, task.timeout)
            )
        elif isinstance(task, WechatAutoLogin):
            coroutines.append(execute_wechat_login(task))

    # 一次性将所有任务推入事件循环，瞬间并发！
    await asyncio.gather(*coroutines)


if __name__ == "__main__":
    asyncio.run(main())
