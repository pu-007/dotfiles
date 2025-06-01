import argparse
import subprocess
import pathlib
import shlex

# --- 配置默认输出位置 (当 -o/--output_file 未指定时) ---
# 通过取消注释所需的 DEFAULT_OUTPUT_MODE 并配置 UNIFIED_FOLDER_PATH_CONFIG 来选择一种模式。

# 选项 1: 输出到与输入文件相同的目录。这是默认设置。
DEFAULT_OUTPUT_MODE = "same_as_input"
UNIFIED_FOLDER_PATH_CONFIG = None  # 此模式下未使用

# 选项 2: 输出到位于当前工作目录 (脚本运行的目录) 下的特定“统一文件夹”。
# DEFAULT_OUTPUT_MODE = "unified_cwd"
# UNIFIED_FOLDER_PATH_CONFIG = "my_pandoc_outputs" #将在CWD中创建的子文件夹的名称

# 选项 3: 输出到特定的绝对路径“统一文件夹”。
# DEFAULT_OUTPUT_MODE = "unified_absolute"
# UNIFIED_FOLDER_PATH_CONFIG = "/mnt/d/my_global_pdf_outputs" # WSL中的绝对路径示例
# UNIFIED_FOLDER_PATH_CONFIG = "D:/my_global_pdf_outputs" # Windows中的绝对路径示例
# --- 结束配置 ---


def get_default_output_path(input_file_path: pathlib.Path) -> pathlib.Path:
    """
    如果用户未指定输出路径，则根据 DEFAULT_OUTPUT_MODE 配置确定默认输出路径。
    """
    output_basename = input_file_path.stem + ".pdf"

    if DEFAULT_OUTPUT_MODE == "same_as_input":
        # 输出文件将与输入文件位于同一目录
        return input_file_path.with_suffix(".pdf")

    elif DEFAULT_OUTPUT_MODE == "unified_cwd":
        if not UNIFIED_FOLDER_PATH_CONFIG:
            print(
                "警告: 'unified_cwd' 模式的 UNIFIED_FOLDER_PATH_CONFIG 未设置。默认使用 'pandoc_script_outputs' 文件夹。"
            )
            folder_name = "pandoc_script_outputs"
        else:
            folder_name = UNIFIED_FOLDER_PATH_CONFIG

        output_dir = pathlib.Path.cwd() / folder_name
        try:
            output_dir.mkdir(parents=True, exist_ok=True)
        except OSError as e:
            print(f"警告: 无法创建目录 {output_dir}: {e}。将默认输出到输入文件的目录。")
            return input_file_path.with_suffix(".pdf")
        return output_dir / output_basename

    elif DEFAULT_OUTPUT_MODE == "unified_absolute":
        if not UNIFIED_FOLDER_PATH_CONFIG:
            print(
                "警告: 'unified_absolute' 模式的 UNIFIED_FOLDER_PATH_CONFIG 未设置。将默认输出到输入文件的目录。"
            )
            return input_file_path.with_suffix(".pdf")

        output_dir = pathlib.Path(UNIFIED_FOLDER_PATH_CONFIG)
        if not output_dir.is_absolute():  # 确保配置的是绝对路径
            print(
                f"警告: 'unified_absolute' 模式的 UNIFIED_FOLDER_PATH_CONFIG '{UNIFIED_FOLDER_PATH_CONFIG}' 不是绝对路径。将默认输出到输入文件的目录。"
            )
            return input_file_path.with_suffix(".pdf")
        try:
            output_dir.mkdir(parents=True, exist_ok=True)
        except OSError as e:
            print(f"警告: 无法创建目录 {output_dir}: {e}。将默认输出到输入文件的目录。")
            return input_file_path.with_suffix(".pdf")
        return output_dir / output_basename

    else:  # 未知模式的回退处理
        print(
            f"警告: 未知的 DEFAULT_OUTPUT_MODE '{DEFAULT_OUTPUT_MODE}'。将默认输出到输入文件的目录。"
        )
        return input_file_path.with_suffix(".pdf")


def main():
    parser = argparse.ArgumentParser(
        description="使用 Pandoc 将 Markdown 转换为 PDF，采用预定义的中文友好设置。",
        formatter_class=argparse.RawTextHelpFormatter  # 允许多行帮助信息
    )
    parser.add_argument("input_file", help="输入的 Markdown 文件路径。")
    parser.add_argument("-o",
                        "--output_file",
                        help="输出 PDF 文件或目录的路径 (可选)。\n"
                        "如果指定的是一个目录，PDF 将保存在该目录下，文件名从输入文件派生。\n"
                        "如果未指定，则遵循脚本配置部分中的默认输出行为。",
                        default=None)
    parser.add_argument("-e",
                        "--extra_args",
                        help="要附加的其他 Pandoc 参数字符串 (可选)。\n"
                        "例如: \"--toc --listings -V author='Your Name'\"",
                        default="")

    args = parser.parse_args()

    input_path = pathlib.Path(args.input_file).resolve()  # 获取绝对路径
    if not input_path.is_file():
        print(f"错误: 输入文件未找到: '{input_path}'")
        return

    # --- 确定输出路径 ---
    if args.output_file:
        output_spec = pathlib.Path(args.output_file)
        # 如果 output_spec 是相对路径, 则相对于当前工作目录解析
        if not output_spec.is_absolute():
            output_spec = pathlib.Path.cwd() / output_spec

        # 判断 output_spec 意图是目录还是完整文件路径
        is_intended_as_dir = False
        if output_spec.is_dir():  # 如果它存在并且是一个目录
            is_intended_as_dir = True
        # 如果它不存在，并且（名称为空，例如 "out/" 或 后缀不是 .pdf）
        elif not output_spec.exists() and (output_spec.name == ""
                                           or output_spec.suffix.lower()
                                           != ".pdf"):
            is_intended_as_dir = True

        if is_intended_as_dir:
            output_dir = output_spec.resolve()
            output_filename = input_path.stem + ".pdf"
            output_path = output_dir / output_filename
        else:
            # 假定为完整的文件路径
            output_path = output_spec.resolve()

        # 确保输出文件的父目录存在
        try:
            output_path.parent.mkdir(parents=True, exist_ok=True)
        except OSError as e:
            print(f"错误: 无法为输出文件创建父目录 '{output_path.parent}': {e}")
            return
    else:
        # 未指定 output_file，使用 get_default_output_path 中的默认逻辑
        output_path = get_default_output_path(input_path)
        # --- 输出路径确定完毕 ---

    # Pandoc固定选项 (header-includes 使用原始字符串以正确处理反斜杠)
    # 注意: 字体名称 "JetXW" 基于之前的讨论。
    # header-includes 的值是作为 -V 的单个参数。
    header_includes_value = (r"\usepackage{microtype} "
                             r"\usepackage{booktabs} "
                             r"\setlength{\arrayrulewidth}{0.5pt} "
                             r"\setlength{\heavyrulewidth}{0.5pt} "
                             r"\setlength{\lightrulewidth}{0.5pt} "
                             r"\renewcommand{\arraystretch}{1.3}")

    pandoc_base_command = [
        "pandoc",
        str(input_path),  # Pandoc 通常期望路径是字符串形式
        "-o",
        str(output_path),
        "--pdf-engine=xelatex",
        "-V",
        "documentclass=ctexart",
        "-V",
        "CJKmainfont=JetXW",
        "-V",
        "mainfont=JetXW",
        "-V",
        "monofont=JetXW",
        "-V",
        "sansfont=JetXW",
        "-V",
        "geometry:a4paper,margin=2.5cm",
        "-V",
        "fontsize=12pt",
        "-V",
        "linestretch=1.4",
        "-V",
        f"header-includes={header_includes_value}",  # 以 key=value 形式传递
        "--highlight-style=pygments"
    ]

    additional_options = []
    if args.extra_args:
        try:
            # shlex.split 在 Windows 上默认 posix=False, 其他系统 posix=True
            # 这与各平台典型的 shell 行为一致
            additional_options = shlex.split(args.extra_args)
        except Exception as e:
            print(f"警告: 无法解析 extra_args '{args.extra_args}'。错误: {e}")
            # 可选: 不带 extra_args 继续或中止。目前仅警告。

    pandoc_full_command = pandoc_base_command + additional_options

    print(f"输入文件: {input_path}")
    print(f"输出文件: {output_path}")
    print("将要执行的 Pandoc 命令:")
    try:
        # shlex.join 是 Python 3.8+ 功能, 用于创建适合shell复制粘贴的命令字符串表示
        print(f"  {shlex.join(pandoc_full_command)}")
    except AttributeError:
        # 对于旧版 Python, 打印列表表示形式
        print(f"  {pandoc_full_command}")

    try:
        process = subprocess.run(
            pandoc_full_command,
            capture_output=True,  # 捕获标准输出和标准错误
            text=True,  # 将输出解码为文本 (使用系统默认编码，通常是UTF-8)
            check=False,  # 我们将手动检查 returncode
            encoding='utf-8'  # 显式指定编码
        )

        if process.returncode == 0:
            print(f"\n成功将 '{input_path.name}' 转换为 '{output_path.name}'")
            # Pandoc 即使成功也经常将信息性消息输出到 stderr (例如 LaTeX 的字体警告)
            if process.stderr and process.stderr.strip():
                print("\nPandoc 消息 (stderr):")
                print(process.stderr.strip())
            # stdout 通常较少用于 PDF 转换，但可能包含有用信息
            if process.stdout and process.stdout.strip():
                print("\nPandoc 输出 (stdout):")
                print(process.stdout.strip())
        else:
            print(f"\n错误: Pandoc 执行失败，退出代码 {process.returncode}")
            if process.stdout and process.stdout.strip():
                print("\nPandoc 输出 (stdout):")
                print(process.stdout.strip())
            if process.stderr and process.stderr.strip():
                print("\nPandoc 错误消息 (stderr):")
                print(process.stderr.strip())

    except FileNotFoundError:
        print("\n错误: 未找到 'pandoc' 命令。")
        print("请确保已安装 Pandoc 并将其添加到系统的 PATH 环境变量中。")
    except Exception as e:
        print(f"\n发生意外错误: {e}")


if __name__ == "__main__":
    main()
