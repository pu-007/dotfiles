import time
from typing import Optional
import pyautogui


def activate_window_by_prefix(
    prefix: str, timeout: Optional[float] = 10.0, interval: float = 0.20
) -> bool:
    start_time = time.time()

    while timeout is None or (time.time() - start_time) < timeout:
        target_win = next(
            (w for w in pyautogui.getAllWindows() if w.title.startswith(prefix)), None
        )

        if target_win:
            try:
                target_win.activate()
                return True
            except Exception as e:
                # 捕获可能的系统权限或窗口状态异常（如窗口刚好被销毁）
                print(f"激活窗口时发生异常: {e}")
                return False

        time.sleep(interval)

    print(f"等待超时：在 {timeout} 秒内未找到前缀为 '{prefix}' 的窗口")
    return False


# 使用示例
if __name__ == "__main__":
    success = activate_window_by_prefix("QuickLook - ", timeout=3.0)
    if success:
        print("窗口已成功激活！")
