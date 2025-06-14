import asyncio
import os
from typing import Callable, Coroutine, List, Union, Optional

import pyautogui
import psutil


async def async_find_window_by_title(title: str,
                                     action: Optional[Callable] = None,
                                     timeout: float = 5.0) -> Optional[list]:
    """
    Asynchronously finds a window by its title and performs an action.
    Runs blocking pyautogui calls in a separate thread.
    """
    elapsed_time = 0
    app_window = []
    while not app_window and elapsed_time < timeout:
        # Run blocking I/O in a separate thread to not block the event loop
        app_window = await asyncio.to_thread(pyautogui.getWindowsWithTitle,
                                             title)
        if not app_window:
            await asyncio.sleep(0.5)
            elapsed_time += 0.5

    if app_window and action:
        # Also run the action in a thread if it's a blocking call
        if asyncio.iscoroutinefunction(action):
            await action(app_window)
        else:
            await asyncio.to_thread(action, app_window)

    return app_window if app_window else None


async def async_launch_app(
    commands: Union[str, List[str]],
    cwd: Optional[str] = None,
    delay: float = 0,
    hook: Optional[Callable[[], Union[None, Coroutine]]] = None,
):
    """
    Asynchronously launches an application.
    Supports an optional hook function (sync or async) after a delay.
    """
    if isinstance(commands, str):
        commands = [commands]
    exe_name = os.path.basename(commands[0])

    try:
        await asyncio.create_subprocess_exec(*commands, cwd=cwd)
        print(f"{exe_name} launched successfully.")
        if delay > 0:
            await asyncio.sleep(delay)
        if hook:
            if asyncio.iscoroutinefunction(hook):
                await hook()
            else:
                # Run sync hook in a thread to avoid blocking
                await asyncio.to_thread(hook)
    except Exception as e:
        print(f"Failed to launch {exe_name}: {e}")


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
                                                  action=activate_window,
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


async def close_doubao_window_async():
    """Finds and closes the Doubao window."""
    print("Hook: Attempting to close Doubao window...")
    await async_find_window_by_title("豆包 - 字节跳动旗下 AI 智能助手 - 豆包",
                                     _close_all_matched_windows)


async def launch_capslockx():
    for _ in range(6):
        for proc in psutil.process_iter(['name']):
            if proc.name() == "QuickLook.exe":
                break
        await asyncio.sleep(1)
    await async_launch_app(
        [r"C:\Users\zion\Apps\CapsLockX\CapsLockX.exe"],
        cwd=r"C:\Users\zion\Apps\CapsLockX",
    )


async def launch_quicklook_and_capslockx_with_cleanup():
    await async_launch_app([
        "pwsh.exe", "-Command",
        r"C:\Users\zion\AppData\Local\Programs\QuickLook\QuickLook.exe",
        "/autorun"
    ])
    await launch_capslockx()
    await async_find_window_by_title("Arch", _close_all_matched_windows)
    await async_find_window_by_title("CapsLockX-Core.ahk",
                                     _close_all_matched_windows)


async def main():
    """
    Main function to create and run all app launch tasks concurrently.
    """

    tasks = [
        # Launch Doubao and then close it via hook
        asyncio.create_task(
            async_launch_app(
                r"C:\Users\zion\AppData\Local\Doubao\Application\Doubao.exe",
                hook=close_doubao_window_async,
            )),
        # Launch komorebi
        asyncio.create_task(
            async_launch_app(
                [
                    r"C:\Program Files\komorebi\bin\komorebic-no-console.exe",
                    "start",
                    "--ahk",
                    "--bar",
                ],
                cwd=r"C:\Program Files\komorebi",
            )),
        # Launch QuickLook, which in turn will launch CapsLockX with its cleanup hook
        asyncio.create_task(launch_quicklook_and_capslockx_with_cleanup()),

        # Launch Ollama
        asyncio.create_task(
            async_launch_app(
                [
                    r"C:\Users\zion\AppData\Local\Programs\Ollama\ollama app.exe"
                ],
                cwd=r"C:\Users\zion\AppData\Local\Programs\Ollama",
            )),
        # Launch Windows Terminal Quake mode and hide it via hook
        asyncio.create_task(
            async_launch_app(
                commands=["wt.exe", "-w", "_quake", "-p", "Arch_quake"],
                hook=create_hide_window_hook(window_title="Arch_quake",
                                             hotkey_combination=["alt", "`"],
                                             delay=3))),
    ]

    print("--- Starting all applications concurrently ---")
    await asyncio.gather(*tasks)
    print("--- All application launch tasks have been processed ---")


if __name__ == "__main__":
    # On Windows, you might need to set a different event loop policy
    # for asyncio subprocesses to work correctly in some environments.
    # asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())
    pyautogui.FAILSAFE = False
    asyncio.run(main())
