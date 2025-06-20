import asyncio
import subprocess
from typing import Callable, Coroutine, List, Union, Optional

import pyautogui
import psutil

DETACHED_PROCESS = 0x00000008


async def async_find_window_by_title(title: str,
                                     hook: Optional[Callable] = None,
                                     timeout: float = 5.0,
                                     interval: float = 0.3) -> Optional[list]:
    """
    Asynchronously finds a window by its title and performs a hook function.
    Runs blocking pyautogui calls in a separate thread.
    """
    elapsed_time = 0
    app_window = []
    while not app_window and elapsed_time < timeout:
        # Run blocking I/O in a separate thread to not block the event loop
        app_window = await asyncio.to_thread(pyautogui.getWindowsWithTitle,
                                             title)
        if not app_window:
            await asyncio.sleep(interval)
            elapsed_time += interval

    if app_window and hook:
        # Also run the action in a thread if it's a blocking call
        if asyncio.iscoroutinefunction(hook):
            await hook(app_window)
        else:
            await asyncio.to_thread(hook, app_window)

    return app_window if app_window else None


async def async_launch_app(
    commands: Union[str, List[str]],
    cwd: Optional[str] = None,
    delay: float = 0,
    hook: Optional[Callable[[], Union[None, Coroutine]]] = None,
):
    await asyncio.to_thread(
        lambda: subprocess.Popen(commands,
                                 shell=False,
                                 close_fds=True,
                                 cwd=cwd,
                                 creationflags=DETACHED_PROCESS))
    if delay > 0:
        await asyncio.sleep(delay)
    if hook:
        if asyncio.iscoroutinefunction(hook):
            await hook()
        else:
            # Run sync hook in a thread to avoid blocking
            await asyncio.to_thread(hook)


def create_hide_window_hook(window_title: str,
                            hotkey_combination: List[str],
                            delay: float = 0):
    """
    Factory function that creates an async hook to find, activate, and hide a window.
    """

    async def hide_window_hook():
        """The actual hook that will be executed to hide a window."""
        if delay > 0:
            await asyncio.sleep(delay)

        def activate_window(w: list):
            """Activates the first window in the list."""
            if w and w[0]:
                print(f"Activating window for hotkey: {w[0].title}")
                w[0].activate()

        # Wait for the window to appear and activate it
        window = await async_find_window_by_title(window_title,
                                                  hook=activate_window,
                                                  timeout=10)

        if not window:
            print(
                f"Warning: Window with title '{window_title}' not found for sending hotkey."
            )
            return

        try:
            print(
                f"Sending hotkey {hotkey_combination} to hide window '{window_title}'."
            )
            await asyncio.to_thread(pyautogui.hotkey, *hotkey_combination)
            print(f"Hotkey {hotkey_combination} sent successfully.")
        except Exception as e:
            print(f"Failed to send hotkey for '{window_title}': {e}")

    return hide_window_hook


def _close_all_matched_windows(window_list: list):
    """Action to close all windows in a list."""
    if not window_list:
        return
    print(f"Closing {len(window_list)} window(s)...")
    for i in window_list:
        try:
            print(f"Closing window: {i.title}")
            i.close()
        except Exception as e:
            print(f"Could not close window {i.title}: {e}")


async def _wait_for_prog(name,
                         timeout: int = 5,
                         interval: float = 0.3) -> bool:
    for _ in range(timeout):
        for proc in psutil.process_iter(['name']):
            if proc.name() == name:
                return True
        await asyncio.sleep(interval)
    return False


async def launch_quicker_clipboard():
    await async_launch_app(r"C:\Program Files\Quicker\Quicker.exe")
    await asyncio.sleep(5)
    await asyncio.to_thread(pyautogui.hotkey, "ctrl", "shift", "x")
    await async_find_window_by_title("剪贴板", _close_all_matched_windows)


async def launch_quicklook_and_capslockx():
    await async_launch_app([
        r"C:\Users\zion\AppData\Local\Programs\QuickLook\QuickLook.exe",
        "-autorun"
    ])
    await _wait_for_prog("QuickLook.exe")
    await async_launch_app([r"C:\Users\zion\Apps\CapsLockX\CapsLockX.exe"], )
    await async_find_window_by_title("CapsLockX-Core.ahk",
                                     _close_all_matched_windows)


async def _close_doubao_window_async():
    await async_find_window_by_title("豆包 - 字节跳动旗下 AI 智能助手 - 豆包",
                                     _close_all_matched_windows)


async def launch_windowsterminal_with_quake():
    # TODO : fix logic
    await async_launch_app(commands=[
        "wt.exe", "-w", "_quake", "-p", "special_quake_window_title"
    ],
                           hook=create_hide_window_hook(
                               window_title="special_quake_window_title",
                               hotkey_combination=["alt", "`"],
                           ),
                           delay=2)
    await async_find_window_by_title("Arch", _close_all_matched_windows)


async def main():
    # fmt: off
    await asyncio.gather(
        asyncio.create_task(
            async_launch_app(
                [r"C:\Program Files\Everything\Everything.exe", "-startup"])),
        asyncio.create_task(
            async_launch_app(
                r"C:\Users\zion\AppData\Local\Doubao\Application\Doubao.exe",
                hook=_close_doubao_window_async,
            )),
        # Launch komorebi
        asyncio.create_task(
            async_launch_app([
                r"C:\Program Files\komorebi\bin\komorebic-no-console.exe",
                "start",
            ])),
        asyncio.create_task(
            async_launch_app(
                r"C:\Users\zion\AppData\Local\Programs\utools\uTools.exe")),
        asyncio.create_task(
            async_launch_app([
                r"C:\Program Files\AutoHotkey\v2\AutoHotkey.exe",
                r"C:\Users\zion\komorebi.ahk"
            ])),
        asyncio.create_task(
            async_launch_app(r"C:\Program Files\YASB\yasb.exe")),
        asyncio.create_task(
            async_launch_app(r"C:\Program Files\Mem Reduct\memreduct.exe")),
        # With QuickLook launched, execute capslockx.
        asyncio.create_task(launch_quicklook_and_capslockx()),
        asyncio.create_task(
            async_launch_app(
                r"C:\Users\zion\AppData\Local\Programs\Motrix\Motrix.exe")),
        asyncio.create_task(
            async_launch_app(
                r"C:\Users\zion\AppData\Roaming\AltSnap\AltSnap.exe")),
        asyncio.create_task(
            async_launch_app([
                r'C:\Users\zion\AppData\Local\Programs\Ollama\ollama app.exe'
            ])),
        # Launch Windows Terminal Quake mode and hide it via hook
        asyncio.create_task(launch_windowsterminal_with_quake()),
        #
        # Launch Quicker clipboard
        asyncio.create_task(launch_quicker_clipboard()),
        asyncio.create_task(
            async_launch_app(
                r"C:\Users\zion\AppData\Local\Programs\twinkle-tray\Twinkle Tray.exe"
            )),
        asyncio.create_task(
            async_launch_app(
                r"C:\Users\zion\AppData\Local\Programs\PixPin\PixPin.exe")),
        asyncio.create_task(
            async_launch_app(
                r"C:\Program Files (x86)\Tobias Erichsen\loopMIDI\loopMIDI.exe"
            )),
        asyncio.create_task(
            async_launch_app([
                r"C:\Program Files (x86)\Stardock\Fences\Fences.exe",
                "/startup"
            ])),
        asyncio.create_task(
            async_launch_app(
                r"C:\Users\zion\Apps\Controller Companion\ControllerCompanion.exe"
            ))
        )
    # fmt: on


if __name__ == "__main__":
    pyautogui.FAILSAFE = False
    asyncio.run(main())
