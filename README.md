# 🚀 Zion's Dotfiles / 锡安的点文件

> **WOTS (Stow Backwards) — 统一的 WSL/Linux/Windows 点文件管理引擎，Rust 实现，零运行时依赖。**
>
> Built in Rust. Zero runtime dependencies. 使用 Rust 构建，单二进制文件即可运行。

这是我的个人系统配置仓库。WOTS 使用**单一编译二进制**弥合 **Linux (WSL)** 与 **Windows** 环境之间的鸿沟：Linux 配置通过 **GNU Stow**（符号链接）管理，Windows 配置通过 **Robocopy**（原生多线程拷贝）同步。

---

## 🛠️ 设计理念 / Philosophy

WSL 上的 dotfile 管理通常涉及两个独立世界：

1. **Linux**：通过符号链接管理（GNU Stow）
2. **Windows**：需要将配置文件强制拷贝到特定的 `AppData` / `User` 路径

WOTS 用**单一 CLI** 统一处理两者，通过目录命名约定自动检测包类型和目标路径。

---

## 🏗️ 核心工具链 / Core Toolchain

| 组件 / Component | 作用 / Role |
|---|---|
| **wots** (Rust binary) | 点文件管理引擎：create, sync, stats, list, diff |
| **[Just](https://github.com/casey/just)** | 任务编排器 — `just refresh` 执行完整系统维护周期 |
| **[GNU Stow](https://www.gnu.org/software/stow/)** | Linux 包符号链接管理 |
| **[Robocopy](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/robocopy)** | 多线程 Windows 文件同步（主引擎） |
| **pwsh.exe / xcopy** | Windows 同步回退方案（robocopy 不可用时） |

---

## 🚀 快速开始 / Quick Start

### 前置条件 / Prerequisites

- **WSL2**（或原生 Linux）
- **[GNU Stow](https://www.gnu.org/software/stow/)** — Linux 符号链接
- **[Just](https://github.com/casey/just)** — 推荐，提供便捷命令
- **[Rust](https://rustup.rs/)** — 仅在从源码构建时需要

### 安装与构建 / Setup & Build

```bash
git clone https://github.com/pu-007/dotfiles.git
cd dotfiles

# 构建 Rust 二进制（Release 优化版）
just build

# 或使用调试构建（包含回溯信息）
just build-debug

# 运行静态检查
just lint

# 运行测试
just test

# 查看 wots 帮助
just wots --help
```

**无需 Python、pixi、虚拟环境**。`wots_bin` 是独立的静态链接二进制文件。

---

## 🕹️ Just 命令体系 / Just Command Reference

**本仓库所有操作均通过 `just` 命令组织。** 运行 `just` 查看完整命令列表。

### 系统维护 / System Maintenance

| 命令 / Command | 说明 / Description |
|---|---|
| `just refresh` | **一键全流程**：保护→更新→备份→清理→同步远端→恢复 |
| `just build` | 编译 Rust Release 二进制 |
| `just build-debug` | 编译 Rust Debug 二进制 |
| `just test` | 运行 `cargo test` |
| `just lint` | 运行 `cargo clippy` |

### 同步操作 / Sync Operations

| 命令 / Command | 等价于 / Equivalent | 说明 / Description |
|---|---|---|
| `just sync` | `wots sync` | 同步所有包到目标 |
| `just sync-type <type>` | `wots sync --type <type>` | 按类型同步；`root` 类型自动加 `sudo` |
| `just sync-app <app>` | `wots sync --app <app>` | 按包名同步 |
| `just sync-dry` | `wots sync --dry-run` | 预览同步（干运行） |
| `just sync-root` | `sudo wots sync --type root --bypass` | 同步 root 包（跳过确认） |

### 信息查询 / Information

| 命令 / Command | 等价于 / Equivalent | 说明 / Description |
|---|---|---|
| `just stats` | `wots stats` | 仓库统计：包数、文件数、大小、状态 |
| `just stats-json` | `wots stats --json` | JSON 格式统计 |
| `just list` | `wots list` | 列出所有包及状态 |
| `just list-type <type>` | `wots list --type <type>` | 按类型列出包 |
| `just list-json` | `wots list --json` | JSON 格式列出包 |

### 差异对比 / Diff

| 命令 / Command | 等价于 / Equivalent | 说明 / Description |
|---|---|---|
| `just diff` | `wots diff` | 显示所有差异 |
| `just diff-type <type>` | `wots diff --type <type>` | 按类型查看差异 |
| `just diff-app <app>` | `wots diff --app <app>` | 按包名查看差异 |

### 创建包 / Create

| 命令 / Command | 等价于 / Equivalent | 说明 / Description |
|---|---|---|
| `just create <args>` | `wots create <args>` | 创建新包 |

---

## 📂 命名约定 / Naming Convention

WOTS 通过目录后缀自动检测包类型。所有包存放在 `DOTFILES_DIR`（默认 `~/dotfiles`）：

| 后缀 / Suffix | 类型 / Type | 目标路径 / Target | 同步策略 / Strategy |
|:---|:---|---|:---|
| `.user` | `user` | `~/` | GNU Stow (symlink) |
| `.config` | `config` | `~/.config/` | GNU Stow (symlink) |
| `.local` | `local` | `~/.local/` | GNU Stow (symlink) |
| `.root` | `root` | `/` | `ln -sf` (sudo) |
| `.meta` | `meta` | N/A | 手动 (metadata) |
| `.winuser` | `winuser` | `C:\Users\{name}\` | Robocopy / Copy |
| `.winconfig` | `winconfig` | `C:\Users\{name}\.config\` | Robocopy / Copy |
| `.winlocal` | `winlocal` | `C:\Users\{name}\AppData\Local\` | Robocopy / Copy |
| `.winroaming` | `winroaming` | `C:\Users\{name}\AppData\Roaming\` | Robocopy / Copy |

### 目录结构示例 / Example Layout

```
~/dotfiles/
├── zsh.user/           → stow 到 ~/
│   ├── .zshrc
│   └── .zsh/
├── nvim.config/        → stow 到 ~/.config/
│   └── nvim/init.lua
├── wsl.root/           → 链接到 / (sudo)
│   └── etc/wsl.conf
├── git.winuser/        → 拷贝到 C:\Users\pu\
│   └── .gitconfig
├── Terminal.winconfig/ → 拷贝到 C:\Users\pu\.config\
│   └── Windows Terminal/settings.json
└── packages.meta/      → 手动元数据存储
    ├── pacman.txt
    └── npm.txt
```

---

## ⚙️ Wots CLI 详解 / CLI Reference

### `wots create` — 创建新包 / Create a Package

```bash
just create [OPTIONS] [SOURCES]...
# 等价: wots create [OPTIONS] [SOURCES]...
```

| 选项 / Option | 说明 / Description |
|---|---|
| `SOURCES` | 一个或多个源文件/目录（支持 `~` 展开） |
| `-a, --app-name <NAME>` | 自定义包名（默认自动检测） |
| `-t, --type <TYPE>` | 显式指定类型：`user`, `config`, `local`, `root`, `meta`, `winuser`, `winconfig`, `winlocal`, `winroaming` |
| `-y, --yes` | 跳过所有确认提示 |
| `-n, --dry-run` | 仅预览，不移动或拷贝文件 |
| `--no-stow` | 创建后不自动 stow（Linux 类型） |
| `--no-sync` | 创建后不自动 Windows 同步（Windows 类型） |

**示例 / Examples**:

```bash
# 创建 user 包（自动检测类型）
just create ~/.zshrc ~/.zsh

# 显式指定类型和包名
just create ~/.config/nvim/init.lua -t config -a nvim

# 干运行预览
just create ~/.ssh/config -n

# 非交互模式（跳过提示）
just create ~/.gitconfig -y

# 创建 Windows 用户配置
just create /mnt/c/Users/pu/.gitconfig -t winuser -a git

# 创建 root 配置
just create /etc/wsl.conf -t root -a wsl
```

创建后自动 stow（Linux）或 sync（Windows），除非传入 `--no-stow` / `--no-sync`。

**交互行为**：交互模式下会提示确认类型和包名，按 Enter 接受默认值，或输入替代值。

---

### `wots sync` — 同步到目标 / Sync to Targets

```bash
just sync                # 同步所有
just sync-type <type>    # 按类型同步
just sync-app <app>      # 按包名同步
just sync-dry            # 干运行预览
just sync-root           # 同步 root 包（sudo）
```

| 选项 / Option | 说明 / Description |
|---|---|
| `-t, --type <TYPE>` | 仅同步该类型的包 |
| `--app <NAME>` | 仅同步指定包（按包名） |
| `-n, --dry-run` | 仅预览，不做实际修改 |
| `--bypass` | 跳过 root 确认提示 |
| `-q, --quiet` | 减少输出 |

**Linux 同步**（user, config, local, root）：使用 GNU Stow `--adopt` 模式（将目标已有文件纳入仓库），冲突时回退为逐文件 `ln -sf`。root 包使用 `sudo ln -sf`。

**Windows 同步**（winuser, winconfig, winlocal, winroaming）：优先使用 `robocopy.exe` 的 `/MIR`（镜像）和 `/MT:8`（8线
程），不可用时回退为 `pwsh.exe` + `xcopy`/`copy`。

---

### `wots stats` — 仓库统计 / Statistics

```bash
just stats          # 文本格式
just stats-json     # JSON 格式
```

| 选项 / Option | 说明 / Description |
|---|---|
| `-j, --json` | JSON 格式输出 |

显示每种类型包的数量、文件统计、总大小和同步状态。

**输出示例**:

```
WOTS Repository  —  /home/pu/dotfiles

Type         Pkgs  Files       Size  Status
──────────────────────────────────────────────────────────
user            2     12    45.3 KB  12/12 stowed
config          3     45   123.7 KB  45/45 stowed
root            1      2     1.2 KB  2/2 stowed
winuser         2      8    18.4 KB  6 synced, 2 needs-sync
──────────────────────────────────────────────────────────
TOTAL           8     67   188.6 KB
```

---

### `wots list` — 列出包 / List Packages

```bash
just list               # 列出所有
just list-type <type>   # 按类型筛选
just list-json          # JSON 格式
```

| 选项 / Option | 说明 / Description |
|---|---|
| `-t, --type <TYPE>` | 按类型筛选 |
| `-j, --json` | JSON 格式输出 |

列出包名、类型、文件数、大小、同步状态和路径。

**输出示例**:

```
Package                  Type     Files       Size  Status
────────────────────────────────────────────────────────────────────
zsh                      user         6    28.1 KB  stowed
fzf                      user         6    17.2 KB  stowed
nvim                     config      30    89.4 KB  stowed
git                      winuser      3    12.3 KB  3 synced
...
```

---

### `wots diff` — 显示差异 / Show Differences

```bash
just diff               # 查看所有差异
just diff-type <type>   # 按类型查看
just diff-app <app>     # 按包名查看
```

| 选项 / Option | 说明 / Description |
|---|---|
| `-t, --type <TYPE>` | 按类型筛选 |
| `--app <NAME>` | 仅显示指定包的差异 |

显示仓库与目标之间不同步的文件：

- **Linux (stow) 包**: 列出尚未建立链接的文件
- **Windows 包**: 显示 mtime/size 不一致或目标缺失的文件

**输出示例**:

```
!   git — 2 needs-sync
    needs-sync: .gitconfig
    missing-win: .gitattributes
!   nvim — 43/45 stowed
    not-stowed: /home/pu/.config/nvim/lua/plugins.lua
```

---

## ♻️ 完整系统刷新 / Full System Refresh: `just refresh`

`just refresh` 流水线自动化完整的系统维护周期：

1. **Protect** — 将未提交的本地修改 stash
2. **Update** — 更新系统包（yay, zinit, nvim plugins, npm 等）
3. **Backup** — 导出 pacman/npm 包列表到 `packages.meta/`
4. **Cleanup** — 清理缓存（uv, pip, npm, go, yay, scoop）
5. **Sync Remote** — `git pull --rebase` + `git push`
6. **Restore** — 恢复之前 stash 的本地修改

```bash
just refresh
```

---

## 🌐 环境变量 / Environment Variables

| 变量 / Variable | 默认值 / Default | 说明 / Description |
|---|---|---|
| `DOTFILES_DIR` | `$HOME/dotfiles` | 仓库根目录 |
| `WSL_DISTRO` | `archlinux` | WSL 发行版名称（用于 UNC 路径构造） |
| `WSL_MNT` | `/mnt/c` | WSL 挂载 C: 盘的位置 |
| `WIN_USER` | 自动检测 | Windows 用户名（`/mnt/c/Users` 下首个非系统用户） |
| `WOTS_CONCURRENT` | `8` | 最大并发同步操作数 |
| `WOTS_MAX_SIZE_MB` | `50` | 跳过超过此大小的文件（MB） |

---

## 🏗️ 源码构建 / Building from Source

```bash
# Debug 构建
cargo build --manifest-path wots/Cargo.toml

# Release 构建（优化 + LTO，单二进制）
cargo build --release --manifest-path wots/Cargo.toml
cp wots/target/release/wots ./wots_bin
```

或使用 `just build` / `just build-debug`。

**Rust 依赖**：Rust 1.80+ (edition 2024)。Cargo 自动获取 `clap`、`walkdir`、`rayon`、`serde`、`anyhow`、`colored`、`glob`、`shellexpand` — 全部纯 Rust，无系统库依赖（除 libc）。

**系统工具**（运行时）：`stow`（Linux）、`robocopy.exe` 或 `pwsh.exe`（WSL 上 Windows 同步）。

---

## 📁 仓库结构 / Repository Layout

```
dotfiles/
├── wots_bin                  # 预编译 Rust 二进制
├── wots/                     # Rust 源码 crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs           # 入口点
│       ├── cli.rs            # CLI 定义 (clap)
│       ├── config.rs         # 配置与环境变量
│       ├── create.rs         # Create 命令
│       ├── discover.rs       # 包发现与类型推断
│       ├── display.rs        # 终端输出 (colored)
│       ├── status.rs         # 状态检查 (stow + copy)
│       ├── sync.rs           # 同步引擎 (stow + robocopy/pwsh)
│       ├── types.rs          # 包类型枚举
│       └── util.rs           # 文件系统工具
├── justfile                  # 任务编排器 (Just)
├── packages.meta/            # 系统元数据 (pacman/npm 列表)
├── packages.user/            # 用户级 dotfile 包
├── packages.config/          # XDG 配置包
├── packages.root/            # 系统级配置包
├── packages.winuser/         # Windows 用户配置包
├── packages.winconfig/       # Windows 配置包
├── packages.winlocal/        # Windows LocalAppData 包
├── packages.winroaming/      # Windows RoamingAppData 包
├── README.md
└── review_report.md          # Rust 实现代码审查报告
```

---

## ⚠️ 声明 / Disclaimer

这是个人配置。欢迎参考架构。如需采用此工作流，你需要：

1. `wots/` — Rust crate（或 `wots_bin` 二进制文件）
2. `justfile` — 任务编排器
3. `packages.*/` — 按上述命名约定组织的目录

---

## 📝 许可证 / License

MIT
