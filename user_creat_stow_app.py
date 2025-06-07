#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import pathlib
import shutil
import subprocess
import sys

## ----------------------------------------------------------------------------
## 纯函数 (计算与数据转换)
## ----------------------------------------------------------------------------


def parse_arguments() -> argparse.Namespace:
    """解析命令行参数。"""
    parser = argparse.ArgumentParser(
        description="从 $HOME 目录中的文件创建一个 stow 包，并将其链接回 $HOME。",
        formatter_class=argparse.RawTextHelpFormatter,
        epilog="""
使用示例:
  # 1. 处理单个文件
  user_creat_stow_app.py ~/.config/jj/jj.toml
  (这将创建 jj.user/ 目录, 将文件移至 jj.user/.config/jj/jj.toml, 然后执行 stow)

  # 2. 处理多个文件/目录，并自定义应用名称和目标目录
  user_creat_stow_app.py ~/.zshrc ~/.config/nvim -a my-shell -d ~/dotfiles
  (这将在 ~/dotfiles/ 中创建 my-shell.user/ 目录, 移入文件, 然后执行 stow)
""")
    parser.add_argument(
        "sources",
        nargs="+",
        help="一个或多个来自 $HOME 目录的源文件或目录。",
    )
    parser.add_argument(
        "-a",
        "--app-name",
        type=str,
        default=None,
        help="为应用包指定一个自定义名称。默认为第一个源文件的名称。",
    )
    parser.add_argument(
        "-d",
        "--target-dir",
        type=pathlib.Path,
        default=pathlib.Path.cwd(),
        help="用于创建 {app_name}.user 文件夹的目标目录。默认为当前工作目录。",
    )
    return parser.parse_args()


def determine_app_name(args: argparse.Namespace) -> str:
    """根据参数或第一个源路径确定应用名称。"""
    if args.app_name:
        return args.app_name
    # 从第一个源路径安全地推断名称
    first_source = pathlib.Path(args.sources[0]).expanduser().resolve()
    return first_source.name


## ----------------------------------------------------------------------------
## 具有副作用的函数 (文件系统操作和进程执行)
## ----------------------------------------------------------------------------


def create_and_move_path(source_str: str, stow_package_path: pathlib.Path,
                         home_path: pathlib.Path) -> bool:
    """
    验证、计算目标路径、创建父目录并移动单个源路径。
    如果成功返回 True，否则返回 False。
    """
    source_path = pathlib.Path(source_str).expanduser().resolve()

    print(f"-> 正在处理: {source_path}")

    # 安全检查：确保源路径在 HOME 目录内。这是一个关键保护措施。
    if not str(source_path).startswith(
            str(home_path)) or source_path == home_path:
        print(f"错误: 源 '{source_path}' 不在 HOME 目录 '{home_path}' 内。正在跳过。",
              file=sys.stderr)
        return False

    # 计算相对于 HOME 的路径
    relative_path = source_path.relative_to(home_path)
    destination_path = stow_package_path / relative_path

    # 在 stow 包内创建父目录
    destination_parent = destination_path.parent
    print(f"   创建目录: {destination_parent}")
    destination_parent.mkdir(parents=True, exist_ok=True)

    # 移动源文件/目录
    try:
        print(f"   移动 '{source_path}' 至 '{destination_path}'")
        shutil.move(str(source_path), str(destination_path))
        return True
    except Exception as e:
        print(f"错误: 移动文件失败: {e}", file=sys.stderr)
        return False


def run_stow(stow_package_path: pathlib.Path,
             target_home: pathlib.Path) -> None:
    """执行 stow 命令将包链接到 HOME 目录。"""
    stow_dir = stow_package_path.parent
    stow_package_name = stow_package_path.name

    command = [
        "stow", "-v", "--adopt", "-t",
        str(target_home), stow_package_name
    ]

    print("\n" + "=" * 50)
    print(f"在 '{stow_dir}' 中运行 stow 命令:")
    print(f"$ {' '.join(command)}")
    print("=" * 50)

    try:
        # 使用 check=True，如果命令失败则会引发异常
        result = subprocess.run(command,
                                cwd=stow_dir,
                                check=True,
                                text=True,
                                capture_output=True)
        print("Stow 输出:\n" + result.stdout)
        if result.stderr:
            print("Stow 错误信息:\n" + result.stderr, file=sys.stderr)
        print("\n✅ Stow 操作成功完成！")

    except FileNotFoundError:
        print("错误: 未找到 'stow' 命令。请确保 GNU Stow 已安装并在您的 PATH 中。",
              file=sys.stderr)
        sys.exit(1)
    except subprocess.CalledProcessError as e:
        print(f"错误: 执行 stow 时出错。返回码: {e.returncode}", file=sys.stderr)
        print("Stow 标准输出:\n" + e.stdout, file=sys.stderr)
        print("Stow 标准错误:\n" + e.stderr, file=sys.stderr)
        sys.exit(1)


## ----------------------------------------------------------------------------
## 主程序入口
## ----------------------------------------------------------------------------


def main() -> None:
    """主执行函数。"""
    # 脚本开始时首先检查 stow 是否可用
    if not shutil.which("stow"):
        print("错误: 未找到 'stow' 命令。请确保 GNU Stow 已安装并在您的 PATH 中。",
              file=sys.stderr)
        sys.exit(1)

    args = parse_arguments()
    home = pathlib.Path.home().resolve()
    target_dir = args.target_dir.resolve()

    app_name = determine_app_name(args)
    stow_package_name = f"{app_name}.user"
    stow_package_path = target_dir / stow_package_name

    print("--- Stow 包创建计划 ---")
    print(f"🏠 Home 目录:    {home}")
    print(f"🎯 目标目录:      {target_dir}")
    print(f"📦 应用包名称:      {stow_package_name}\n")

    if stow_package_path.exists():
        print(f"错误: Stow 包目录 '{stow_package_path}' 已存在。请先移除或选择其他名称。",
              file=sys.stderr)
        sys.exit(1)

    # 仅创建最外层的包目录
    stow_package_path.mkdir()

    # 移动所有源文件
    success_count = 0
    for source in args.sources:
        if create_and_move_path(source, stow_package_path, home):
            success_count += 1

    # 如果没有任何文件被成功移动，则退出，避免运行空的 stow
    if success_count == 0:
        print("\n没有文件被成功处理，正在清理空目录并退出。")
        shutil.rmtree(stow_package_path)
        sys.exit(1)

    # 运行 stow
    run_stow(stow_package_path, home)


if __name__ == "__main__":
    main()
