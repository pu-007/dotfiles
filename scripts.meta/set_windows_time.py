import sys
import os
import requests
import datetime
import ctypes

def get_network_time():
    """从 worldtimeapi.org 获取网络时间 (Asia/Shanghai)"""
    try:
        # 直接请求Asia/Shanghai时区的时间
        response = requests.get("http://worldtimeapi.org/api/timezone/Asia/Shanghai", timeout=5)
        response.raise_for_status()  # 如果请求失败则抛出异常
        
        data = response.json()
        datetime_str = data['datetime']
        
        # 解析ISO 8601格式的日期时间字符串
        network_time = datetime.datetime.fromisoformat(datetime_str)
        
        return network_time
    except requests.exceptions.RequestException as e:
        print(f"错误: 获取网络时间失败 - {e}")
        return None
    except Exception as e:
        print(f"错误: 解析时间数据时发生未知错误 - {e}")
        return None

def set_system_time(network_time, dry_run=False):
    """设置Windows系统时间，或在dry run模式下显示时间差异"""
    if dry_run:
        local_time = datetime.datetime.now()
        print("\n--- 模拟运行模式 (Dry Run) ---")
        print(f"当前系统时间: {local_time.strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"将要设置时间: {network_time.strftime('%Y-%m-%d %H:%M:%S')}")
        print("系统时间将不会被修改。")
        return

    try:
        # 格式化日期为 "YYYY-MM-DD"
        date_str = network_time.strftime('%Y-%m-%d')
        # 格式化时间为 "HH:MM:SS"
        time_str = network_time.strftime('%H:%M:%S')

        print(f"正在设置系统日期为: {date_str}")
        os.system(f'date {date_str}')

        print(f"正在设置系统时间为: {time_str}")
        os.system(f'time {time_str}')
        
        print("\n成功更新系统时间！")
        
    except Exception as e:
        print(f"错误: 设置系统时间失败 - {e}")
        print("请确保您以管理员权限运行此脚本。")

def is_admin():
    """检查当前用户是否为管理员"""
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False

def main():
    """主函数"""
    # 1. 检查是否为 dry run 模式
    dry_run = '--dry-run' in sys.argv

    # 2. 如果不是 dry run，则检查操作系统和管理员权限
    if not dry_run:
        if sys.platform != 'win32':
            print("错误: 此脚本只能在 Windows 系统上运行。")
            return
            
        if not is_admin():
            print("错误: 需要管理员权限来修改系统时间。")
            print("请右键单击脚本并选择 '以管理员身份运行'。")
            return

    # 3. 获取网络时间
    print("正在从网络获取最新时间...")
    now = get_network_time()

    # 4. 如果成功获取时间，则设置或显示时间
    if now:
        if not dry_run:
            print(f"获取到的当前网络时间是: {now.strftime('%Y-%m-%d %H:%M:%S')}")
        set_system_time(now, dry_run=dry_run)

if __name__ == '__main__':
    main()
    # 让窗口在结束后暂停，以便用户可以看到输出
    os.system("pause")