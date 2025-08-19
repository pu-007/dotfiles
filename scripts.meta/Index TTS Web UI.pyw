import socket
import subprocess
import platform
import webbrowser

# --- 配置区 ---
HOST = "127.0.0.1"
PORT = 7860
# 这是你要在后台启动的完整命令
COMMAND_TO_RUN = "wsl.exe zsh -ic 'VLLM_USE_V1=0 /opt/miniconda3/bin/conda run -n index-tts-vllm --cwd /home/pu/Source/index-tts-vllm python /home/pu/Source/index-tts-vllm/webui.py'"


def is_port_in_use(port: int, host: str = '127.0.0.1') -> bool:
    """
    检查指定的 TCP 端口是否正在被监听。
    """
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        # connect_ex 返回错误码，0表示成功（端口已占用），其他值表示失败（端口空闲）
        return s.connect_ex((host, port)) == 0


def start_service_silently():
    """
    在后台静默启动服务进程，不显示任何窗口。
    """
    # 仅在 Windows 系统上使用 creationflags
    if platform.system() == "Windows":
        # subprocess.CREATE_NO_WINDOW 是一个关键标志，用于防止弹出cmd窗口
        # 这对于从 GUI 应用或无UI脚本调用控制台程序非常有用
        creation_flags = subprocess.CREATE_NO_WINDOW

        # 使用 Popen 启动进程，它不会阻塞脚本的执行
        subprocess.Popen(COMMAND_TO_RUN, creationflags=creation_flags)

        # 如果你想打印日志或调试，可以取消下面这行的注释，但它只会在你从命令行运行时显示
        # print("服务启动命令已发送，进程将在后台运行。")
    else:
        # 在非 Windows 系统上的等效操作（尽管此脚本主要为 Windows 设计）
        # 使用 shell=True 并重定向输出来实现后台运行
        subprocess.Popen(COMMAND_TO_RUN,
                         shell=True,
                         stdout=subprocess.DEVNULL,
                         stderr=subprocess.DEVNULL,
                         stdin=subprocess.DEVNULL)


if __name__ == "__main__":
    # 首先检查当前运行环境是否为 Windows
    if platform.system() != "Windows":
        # 如果想让脚本在非Windows环境下也能运行，可以添加提示信息
        # sys.exit("此脚本主要为 Windows 环境设计，用于静默启动进程。")
        pass  # 或者直接允许执行，使用上面 else 分支的逻辑

    if is_port_in_use(PORT, HOST):
        # 服务已在运行，什么也不做
        # 如果需要调试，可以取消下面这行的注释
        # print(f"服务已在运行 (端口 {PORT} 已被占用)。")
        webbrowser.open(f"http://{HOST}:{PORT}")
    else:
        # 服务未运行，启动它
        start_service_silently()

