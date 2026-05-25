import sys
import os
import requests
import datetime
import ctypes
import email.utils


# 声明 Windows SYSTEMTIME 结构体
class SYSTEMTIME(ctypes.Structure):
    _fields_ = [
        ("wYear", ctypes.c_ushort),
        ("wMonth", ctypes.c_ushort),
        ("wDayOfWeek", ctypes.c_ushort),
        ("wDay", ctypes.c_ushort),
        ("wHour", ctypes.c_ushort),
        ("wMinute", ctypes.c_ushort),
        ("wSecond", ctypes.c_ushort),
        ("wMilliseconds", ctypes.c_ushort),
    ]


def get_network_time():
    """尝试从多个中国国内优先的源获取网络时间，并统一返回时区感知的 UTC datetime 对象"""

    sources = [
        # 源 1：百度 HTTP Header (国内极度稳定，直连极快，不受特定API失效影响)
        {
            "name": "百度 (HTTP Header)",
            "url": "https://www.baidu.com",
            "type": "header",
        },
        # 源 2：腾讯 (HTTP Header)
        {
            "name": "腾讯 (HTTP Header)",
            "url": "https://www.tencent.com",
            "type": "header",
        },
        # 源 3：淘宝 (HTTP Header)
        {
            "name": "淘宝 (HTTP Header)",
            "url": "https://www.taobao.com",
            "type": "header",
        },
        # 源 4：苏宁时间 API (北京时间)
        {
            "name": "苏宁 API",
            "url": "http://quan.suning.com/getSysTime.do",
            "type": "suning",
        },
        # 源 5：淘宝时间戳 API
        {
            "name": "淘宝 API",
            "url": "http://api.m.taobao.com/rest/api3.do?api=mtop.common.getTimestamp",
            "type": "taobao",
        },
        # 源 6：WorldTimeAPI (国外备用源)
        {
            "name": "WorldTimeAPI",
            "url": "http://worldtimeapi.org/api/timezone/Asia/Shanghai",
            "type": "worldtimeapi",
        },
    ]

    for source in sources:
        try:
            print(f"正在尝试从 [{source['name']}] 获取时间...")

            # 设置 3 秒超时，避免网络拥堵导致卡死
            if source["type"] == "header":
                # 先尝试轻量的 HEAD 请求，只获取响应头以加快响应速度
                try:
                    response = requests.head(source["url"], timeout=3)
                    response.raise_for_status()
                except Exception:
                    # 如果某些环境下的 CDN 拦截了 HEAD 请求，则降级使用 GET
                    response = requests.get(source["url"], timeout=3)
                    response.raise_for_status()

                date_str = response.headers.get("Date")
                if date_str:
                    # 解析 HTTP Header 标准格式 (RFC 2822)，此格式自带 UTC 时区信息
                    utc_dt = email.utils.parsedate_to_datetime(date_str)
                    print(f"成功从 [{source['name']}] 获取时间。")
                    return utc_dt

            elif source["type"] == "suning":
                response = requests.get(source["url"], timeout=3)
                response.raise_for_status()
                data = response.json()
                time_str = data.get("sysTime2")  # 格式如: "2026-05-25 18:00:00"
                if time_str:
                    local_dt = datetime.datetime.strptime(time_str, "%Y-%m-%d %H:%M:%S")
                    # 苏宁返回的是北京时间 (UTC+8)
                    beijing_tz = datetime.timezone(datetime.timedelta(hours=8))
                    utc_dt = local_dt.replace(tzinfo=beijing_tz).astimezone(
                        datetime.timezone.utc
                    )
                    print(f"成功从 [{source['name']}] 获取时间。")
                    return utc_dt

            elif source["type"] == "taobao":
                response = requests.get(source["url"], timeout=3)
                response.raise_for_status()
                data = response.json()
                # 淘宝返回的是毫秒级时间戳
                timestamp_ms = int(data["data"]["t"])
                utc_dt = datetime.datetime.fromtimestamp(
                    timestamp_ms / 1000.0, tz=datetime.timezone.utc
                )
                print(f"成功从 [{source['name']}] 获取时间。")
                return utc_dt

            elif source["type"] == "worldtimeapi":
                response = requests.get(source["url"], timeout=3)
                response.raise_for_status()
                data = response.json()
                timestamp = data["unixtime"]
                utc_dt = datetime.datetime.fromtimestamp(
                    timestamp, tz=datetime.timezone.utc
                )
                print(f"成功从 [{source['name']}] 获取时间。")
                return utc_dt

        except Exception as e:
            print(f"提示: 从 [{source['name']}] 获取时间失败，尝试下一个源。原因: {e}")
            continue

    return None


def set_system_time(utc_time, dry_run=False):
    """设置 Windows 系统时间，或在 dry run 模式下显示时间差异"""
    if dry_run:
        # 获取当前系统本地时间 (有时区感知)
        local_time = datetime.datetime.now(datetime.timezone.utc).astimezone()
        # 将获取到的网络 UTC 时间转换为本地时区进行对比
        network_local = utc_time.astimezone()

        print("\n--- 模拟运行模式 (Dry Run) ---")
        print(f"当前系统时间: {local_time.strftime('%Y-%m-%d %H:%M:%S %Z')}")
        print(f"网络同步时间: {network_local.strftime('%Y-%m-%d %H:%M:%S %Z')}")

        diff = (network_local - local_time).total_seconds()
        print(f"时间相差: {diff:.3f} 秒")
        print("系统时间将不会被修改。")
        return

    try:
        # 统一转为标准 UTC 格式
        utc_datetime = utc_time.astimezone(datetime.timezone.utc)

        sys_time = SYSTEMTIME()
        sys_time.wYear = utc_datetime.year
        sys_time.wMonth = utc_datetime.month
        sys_time.wDayOfWeek = 0  # Win32 API 会自动计算，传入0即可
        sys_time.wDay = utc_datetime.day
        sys_time.wHour = utc_datetime.hour
        sys_time.wMinute = utc_datetime.minute
        sys_time.wSecond = utc_datetime.second
        sys_time.wMilliseconds = int(utc_datetime.microsecond / 1000)

        # 转换为本地显示格式打印，便于用户核对
        local_display = utc_datetime.astimezone()
        print(
            f"正在更新系统本地时间为: {local_display.strftime('%Y-%m-%d %H:%M:%S %Z')}"
        )

        # 调用 Windows API 修改系统 UTC 时间
        result = ctypes.windll.kernel32.SetSystemTime(ctypes.byref(sys_time))

        if result:
            print("\n成功更新系统时间及硬件时钟！")
        else:
            error_code = ctypes.get_last_error()
            print(f"\n错误: 修改系统时间失败，Windows 错误代码: {error_code}")
            print("请确保您已右键选择 '以管理员身份运行' 此脚本。")

    except Exception as e:
        print(f"错误: 设置系统时间失败 - {e}")


def is_admin():
    """检查当前用户是否为管理员"""
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False


def main():
    """主函数"""
    # 1. 检查是否为 dry run 模式
    dry_run = "--dry-run" in sys.argv

    # 2. 如果不是 dry run，则检查操作系统和管理员权限
    if not dry_run:
        if sys.platform != "win32":
            print("错误: 此脚本只能在 Windows 系统上运行。")
            return

        if not is_admin():
            print("错误: 需要管理员权限来修改系统时间。")
            print("请右键单击脚本并选择 '以管理员身份运行'。")
            return

    # 3. 获取网络时间
    print("正在从网络获取最新时间...")
    utc_time = get_network_time()

    # 4. 如果成功获取时间，则设置或显示时间
    if utc_time:
        set_system_time(utc_time, dry_run=dry_run)
    else:
        print("\n错误: 无法从所有配置的网络源获取时间。请检查您的网络连接或代理设置。")


if __name__ == "__main__":
    main()
    # 让窗口在结束后暂停，以便用户可以看到输出
    os.system("pause")
