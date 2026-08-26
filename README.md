# 🚀 Zion's Dotfiles / 锡安的点文件

> **WOTS (WSL Dotfile Stow Tool)** — 统一的 WSL/Linux/Windows 点文件管理引擎。
> Rust 实现，单二进制，零运行时依赖。Built in Rust, single binary.

WOTS 用**单一 CLI** 弥合 **WSL Linux**（GNU Stow 符号链接）与 **Windows**
（Robocopy 多线程镜像）之间的鸿沟。包类型通过目录后缀自动检测。

---

## 📂 包类型 / Package Types

所有包以 `<name>.<suffix>/` 后缀目录形式存放在 `DOTFILES_DIR`（默认 `~/dotfiles`）：

| 后缀 / Suffix | 目标路径 / Target     | 同步策略 / Strategy         |
| :------------ | --------------------- | :-------------------------- |
| `.user`       | `~/`                  | GNU Stow (symlink)          |
| `.config`     | `~/.config/`          | GNU Stow (symlink)          |
| `.root`       | `/`                   | `sudo ln -sf`               |
| `.meta`       | N/A                   | 手动管理 / Manual           |
| `.winuser`    | `C:\Users\{name}\`    | Robocopy mirror / pwsh copy |
| `.winroot`    | `C:\`                 | Robocopy mirror / pwsh copy |

> **v2.0 变更 / Breaking change**: 移除了 `.local`、`.winlocal`、`.winconfig`、
> `.winroaming` 类型，新增 `.winroot`（Windows 盘根）。迁移说明见
> [docs/wots-usage.md](docs/wots-usage.md)。

```text
~/dotfiles/
├── zsh.user/        → stow 到 ~/
├── nvim.config/     → stow 到 ~/.config/
├── wsl.root/        → sudo 链接到 /
├── git.winuser/     → robocopy 到 C:\Users\{user}\
├── tools.winroot/   → robocopy 到 C:\
└── packages.meta/   → 手动管理
```

---

## ⚡ 快速开始 / Quick Start

前置条件：WSL2（或原生 Linux）、[GNU Stow](https://www.gnu.org/software/stow/)、
[Just](https://github.com/casey/just)、[Rust](https://rustup.rs/)（仅构建时需要）。

```bash
git clone https://github.com/pu-007/dotfiles.git
cd dotfiles

just build      # 构建 Release 二进制 + 安装 zsh 补全
just test       # 运行测试
just lint       # cargo clippy
just sync-dry   # 干运行预览同步
just            # 查看全部命令
```

同步前会打印完整的**仓库路径 → 目标路径**映射清单并要求确认（`justfile` 顶部
`CONFIRM := "true"` 控制），防止写错目标。

---

## 📖 文档 / Documentation

关于 WOTS 的详细内容已拆分至 `docs/`：

| 文档 | 内容 |
| ---- | ---- |
| [docs/wots-usage.md](docs/wots-usage.md) | **使用方法**：完整 CLI 参考、just 命令体系、环境变量、类型迁移说明 |
| [docs/wots-architecture.md](docs/wots-architecture.md) | **技术实现**：模块架构、stow/robocopy 引擎、状态索引、测试策略 |
| [docs/wots-pitfalls.md](docs/wots-pitfalls.md) | **可能踩的坑**：路径/方向/权限/索引 20 条实战注意事项 |
| [docs/wots-maintenance.md](docs/wots-maintenance.md) | **维护方式**：日常流程、增删包类型、故障排查表、已知技术债 |
| [docs/wots-design.md](docs/wots-design.md) | **Rust 设计哲学**：类型即文档、原子性、缓存策略、依赖克制与反面教训 |

---

## 🏗️ 核心工具链 / Core Toolchain

| 组件 | 作用 |
| ---- | ---- |
| **wots** (Rust binary) | 点文件管理引擎：create, sync, stats, list, diff |
| **[Just](https://github.com/casey/just)** | 任务编排器 — `just refresh` 一键系统维护 |
| **[GNU Stow](https://www.gnu.org/software/stow/)** | Linux 符号链接管理 |
| **[Robocopy](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/robocopy)** | Windows 多线程镜像（主引擎） |
| **pwsh.exe** | Windows 同步回退方案 |

---

## 🔧 环境变量 / Environment Variables

| 变量               | 默认值           | 说明                     |
| ------------------ | ---------------- | ------------------------ |
| `DOTFILES_DIR`     | `$HOME/dotfiles` | 仓库根目录               |
| `WSL_DISTRO`       | `archlinux`      | WSL 发行版名称           |
| `WSL_MNT`          | `/mnt/c`         | C 盘挂载点               |
| `WIN_USER`         | （必须设置）     | Windows 用户名           |
| `WOTS_CONCURRENT`  | `8`              | 最大并发同步数           |
| `WOTS_MAX_SIZE_MB` | `50`             | 跳过超过此大小的文件     |

---

## ⚠️ 声明 / Disclaimer

这是个人配置，欢迎参考架构。如需采用此工作流，你需要：
1. `wots-src/` — Rust crate（`just build` 编译出 `./wots`）
2. `justfile` — 任务编排器
3. 按后缀命名约定组织的包目录
4. 可选：`zsh.user/` 中已包含 `_wots` 补全脚本（构建时自动更新）

## 📝 许可证 / License

MIT
