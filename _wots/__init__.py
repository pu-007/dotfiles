"""
_WOTS — WSL Dotfile Stow Tool (Core Engine)
==========================================
统一管理 WSL、Linux 和 Windows 配置文件的核心引擎。

项目结构说明:
  .user    → Stow 到 $HOME
  .config  → Stow 到 $HOME/.config
  .root    → Stow 到 / (sudo)
  .winuser → 强制同步到 Windows 用户目录
  .meta    → 仅归档，不执行操作

常用命令 (通过 just 调用):
  just refresh      # 系统全面刷新与备份
  just sync         # 同步所有配置文件
  just create <dir> # 创建新的配置包
"""

# 1. 显式相对导入 (必须导入，__all__ 才能找到这些名称)
from . import cli
from . import config
from . import discover
from . import display
from . import status
from . import sync
from . import types
from . import utils

# 2. 包元数据
__version__ = "1.0.0"
__author__ = "Zion Pu <pu.007@qq.com>"

# 3. 导出定义 (消除 LSP 报错的关键)
__all__ = [
    "cli",
    "config",
    "discover",
    "display",
    "status",
    "sync",
    "types",
    "utils",
]
