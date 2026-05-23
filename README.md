# 🚀 Zion's Dotfiles / 锡安的点文件

> **WOTS (WSL Dotfile Stow Tool) — 统一的 WSL/Linux/Windows 点文件管理引擎，Rust 实现，零运行时依赖。**
>
> **WOTS (WSL Dotfile Stow Tool) — Unified WSL/Linux/Windows dotfile management engine built in Rust with zero runtime dependencies.**

Built in Rust. Single binary. / Rust 构建，单二进制文件运行。

WOTS bridges **WSL Linux** (GNU Stow symlinks) and **Windows** (Robocopy multi-threaded mirroring) with a single CLI. Package type is auto-detected from directory suffix naming conventions.

WOTS 通过**单一 CLI** 弥合 **WSL Linux**（GNU Stow 符号链接）与 **Windows**（Robocopy 多线程镜像）之间的鸿沟。包类型通过目录后缀命名约定自动检测。

---

## 🛠️ 设计理念 / Philosophy

Managing dotfiles on WSL typically involves two separate worlds:

1. **Linux**: symlink management via GNU Stow
2. **Windows**: forced copies of config files to specific `AppData` / `User` paths

WOTS unifies both with a single CLI, using directory naming conventions to auto-detect package type and target path.

---

WSL 上的 dotfile 管理通常涉及两个独立世界：

1. **Linux**：通过 GNU Stow 管理符号链接
2. **Windows**：通过 Robocopy 将配置文件拷贝到 `AppData` / `User` 路径

WOTS 用单一 CLI 统一处理两者，通过目录后缀自动检测包类型和目标路径。

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

- **WSL2**（或原生 Linux / or native Linux）
- **[GNU Stow](https://www.gnu.org/software/stow/)** — Linux 符号链接
- **[Just](https://github.com/casey/just)** — 推荐，提供便捷命令
- **[Rust](https://rustup.rs/)** — 仅从源码构建时需要 / only needed for building from source

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

**无需 Python、pixi、虚拟环境。** `wots_bin` 是独立的静态链接二进制文件（约 1.7 MB）。

---

## 🕹️ Just 命令体系 / Just Command Reference

**本仓库所有操作均通过 `just` 命令组织。** 运行 `just` 查看完整命令列表。

### 构建 / Build

| 命令 / Command | 说明 / Description |
|---|---|
| `just build` | 编译 Rust Release 二进制 |
| `just build-debug` | 编译 Rust Debug 二进制 |
| `just test` | 运行 `cargo test` |
| `just lint` | 运行 `cargo clippy` |
| `just wots <args>` | 直接运行 wots 二进制 |

### 系统维护 / System Maintenance

| 命令 / Command | 说明 / Description |
|---|---|
| `just refresh` | **一键全流程**：保护→更新→备份→清理→同步远端→恢复 |

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
| `just stats` | `wots stats` | 仓库统计 |
| `just stats-json` | `wots stats --json` | JSON 格式统计 |
| `just list` | `wots list` | 列出所有包 |
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

所有包以**后缀目录**形式存放在 `DOTFILES_DIR`（默认 `~/dotfiles`）中。WOTS 通过目录后缀自动检测类型。

Packages are stored as **suffixed directories** directly under `DOTFILES_DIR` (default `~/dotfiles`). WOTS auto-detects type from the suffix.

| 后缀 / Suffix | 类型 / Type | 目标路径 / Target | 同步策略 / Strategy |
|:---|---|---|:---|
| `.user` | `user` | `~/` | GNU Stow (symlink) |
| `.config` | `config` | `~/.config/` | GNU Stow (symlink) |
| `.local` | `local` | `~/.local/` | GNU Stow (symlink) |
| `.root` | `root` | `/` | `ln -sf` (sudo) |
| `.meta` | `meta` | N/A | 手动管理 / Manual |
| `.winuser` | `winuser` | `C:\Users\{name}\` | Robocopy mirror / pwsh copy |
| `.winconfig` | `winconfig` | `C:\Users\{name}\.config\` | Robocopy mirror / pwsh copy |
| `.winlocal` | `winlocal` | `C:\Users\{name}\AppData\Local\` | Robocopy mirror / pwsh copy |
| `.winroaming` | `winroaming` | `C:\Users\{name}\AppData\Roaming\` | Robocopy mirror / pwsh copy |

### 目录结构示例 / Example Layout

```text
~/dotfiles/
├── zsh.user/            → stow 到 ~/
│   ├── .zshrc
│   └── .zsh/
├── nvim.config/         → stow 到 ~/.config/
│   └── nvim/init.lua
├── wsl.root/            → sudo 链接到 /
│   └── etc/wsl.conf
├── git.winuser/         → robocopy 到 C:\Users\pu\
│   └── .gitconfig
├── powershell.winuser/  → robocopy 到 C:\Users\pu\
│   └── Documents/PowerShell/profile.ps1
├── packages.meta/       → 手动管理（包列表等元数据）
│   ├── pacman.txt
│   └── npm.txt
└── scripts.meta/        → 手动管理（脚本备份）
```

> **注意**: 旧版 README 曾描述 `packages.user/`、`packages.config/` 等子目录结构。实际实现使用**扁平后缀**布局：`<name>.<suffix>/` 直接放在 `DOTFILES_DIR` 根目录下。唯一的 `packages.` 前缀目录是 `packages.meta/`（因其命名不遵循后缀约定）。

---

## ⚙️ Wots CLI 详解 / CLI Reference

### `wots create` — 创建新包 / Create a Package

```bash
just create [OPTIONS] [SOURCES]...
```

| 选项 / Option | 说明 / Description |
|---|---|
| `SOURCES` | 源文件/目录（支持 `~` 展开） |
| `-a, --app-name <NAME>` | 自定义包名 |
| `-t, --type <TYPE>` | 显式指定类型：`user`, `config`, `local`, `root`, `meta`, `winuser`, `winconfig`, `winlocal`, `winroaming` |
| `-y, --yes` | 跳过所有确认提示 |
| `-n, --dry-run` | 仅预览，不移动文件 |
| `--no-stow` | 创建后不自动 stow（Linux 类型） |
| `--no-sync` | 创建后不自动 Windows 同步（Windows 类型） |

**存储策略**: Linux config (`user/config/local`) 类型使用 **move**（从原始位置移动到仓库，然后 stow 回 symlink）；Windows 和 meta 类型使用 **copy**。Move 操作使用临时文件 + 原子 rename，并通过文件数量/大小验证拷贝完整性。

**示例 / Examples**:

```bash
# 创建 user 包（自动检测类型）
just create ~/.zshrc ~/.zsh

# 显式指定类型和包名
just create ~/.config/nvim/init.lua -t config -a nvim

# 干运行预览
just create ~/.ssh/config -n

# 非交互模式
just create ~/.gitconfig -y

# 创建 Windows 用户配置
just create /mnt/c/Users/pu/.gitconfig -t winuser -a git

# 创建 root 配置
just create /etc/wsl.conf -t root -a wsl
```

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
| `--app <NAME>` | 仅同步指定包 |
| `-n, --dry-run` | 仅预览 |
| `--bypass` | 跳过 root 确认提示 |
| `-q, --quiet` | 减少输出 |

**Linux 同步** (`user`, `config`, `local`, `root`): 使用 GNU Stow `--adopt` 模式。若 stow 失败则降级为逐文件 `ln -sf`。`root` 包使用 `sudo ln -sf`。

**Windows 同步** (`winuser`, `winconfig`, `winlocal`, `winroaming`): 优先使用 `robocopy.exe` 的 `/MIR`（镜像）和 `/MT:8`（8线程）。robocopy 不可用时回退为 `pwsh.exe` + `xcopy`。

**状态索引**: Windows 同步使用 `.wots_index.json` 索引缓存文件元数据（mtime、size、blake3 哈希），避免重复比较已知已同步的文件。索引采用原子写入（`.tmp` → `rename`）。

### `wots stats` — 仓库统计 / Statistics

```bash
just stats          # 文本表格
just stats-json     # JSON 格式
```

显示每种类型包的数量、文件统计、总大小和同步状态。

### `wots list` — 列出包 / List Packages

```bash
just list               # 列出所有
just list-type <type>   # 按类型筛选
just list-json          # JSON 格式
```

### `wots diff` — 显示差异 / Show Differences

```bash
just diff               # 所有差异
just diff-type <type>   # 按类型
just diff-app <app>     # 按包名
```

显示仓库与目标之间不同步的文件：Linux 包列出未建立链接的文件，Windows 包显示 mtime/size/blake3 不一致或目标缺失的文件。

---

## 📁 仓库结构 / Repository Layout

```text
dotfiles/
├── wots_bin                  # 预编译 Rust 二进制 (~1.7 MB)
├── wots/                     # Rust 源码 crate
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── src/
│   │   ├── main.rs           # 入口点 + stats/list/diff 命令实现
│   │   ├── lib.rs            # 模块导出
│   │   ├── cli.rs            # CLI 定义 (clap derive)
│   │   ├── config.rs         # 配置与环境变量 (LazyLock globals)
│   │   ├── create.rs         # Create 命令（含原子拷贝验证）
│   │   ├── discover.rs       # 包发现、类型检测、路径构建
│   │   ├── display.rs        # 终端输出（表格渲染、提示确认）
│   │   ├── status.rs         # 同步状态检查、索引管理（1057 lines）
│   │   ├── sync.rs           # 同步引擎（stow + robocopy/pwsh）
│   │   ├── types.rs          # PkgType 枚举（9 变体）+ 方法
│   │   └── util.rs           # 文件系统工具函数
│   └── tests/
│       └── integration.rs    # 集成测试（27 test functions, 347 lines）
├── justfile                  # 任务编排器 (281 lines)
├── .pre-commit-config.yaml   # Git pre-commit hooks
├── .wots_index.json          # 同步状态索引（git-ignored 缓存）
│
├── <name>.user/              # 用户级 dotfile 包（~30 个包）
├── <name>.config/            # XDG 配置包
├── <name>.root/              # 系统级配置包
├── <name>.meta/              # 元数据/手动管理包
├── <name>.winuser/           # Windows 用户配置包
├── <name>.winconfig/         # Windows 配置包
├── <name>.winlocal/          # Windows LocalAppData 包
├── <name>.winroaming/        # Windows RoamingAppData 包
└── README.md
```

---

## 🏗️ 代码质量分析 / Code Quality Analysis

### 总体评分 / Overall

| 维度 / Aspect | 评分 / Rating | 说明 / Notes |
|---|---|---|
| 架构设计 / Architecture | ★★★★☆ | 模块分离清晰，11 个 `.rs` 文件职责明确 |
| Rust 惯用性 / Idiomatic Rust | ★★★★☆ | clap derive, rayon, thiserror, serde, LazyLock |
| 错误处理 / Error Handling | ★★★★☆ | anyhow + 自定义上下文, broken-pipe 处理 |
| 单元测试 / Unit Tests | ★★★☆☆ | types.rs 和 discover.rs 覆盖良好，但 sync/create/util 测试缺失 |
| 集成测试 / Integration Tests | ★★★★☆ | 27 个测试覆盖状态同步核心场景 |
| 文档 / Documentation | ★★★☆☆ | 注释较少，依赖 README 和类型系统 |

### 源码统计 / Source Statistics

| 文件 / File | 行数 / Lines | 职责 |
|---|---|---|
| `status.rs` | 1,057 | 同步索引、状态检查、blake3 哈希对比 |
| `sync.rs` | 459 | stow + robocopy/pwsh 同步编排 |
| `discover.rs` | 439 | 包发现、类型检测、路径映射（含完整单测） |
| `types.rs` | 365 | PkgType 枚举 + 方法（含完整单测） |
| `main.rs` | 325 | 入口 + stats/list/diff 命令内联实现 |
| `create.rs` | 315 | 包创建（含原子拷贝验证） |
| `util.rs` | 292 | 文件系统工具 |
| `display.rs` | 181 | 终端输出和交互 |
| `cli.rs` | 100 | clap CLI 定义 |
| `config.rs` | 97 | LazyLock 全局配置 |
| `tests/integration.rs` | 347 | 集成测试 |
| **总计 / Total** | **3,977** | |

### 优缺点分析 / Strengths & Issues

#### 优点 / Strengths

1. **原子操作 / Atomic operations**: `create.rs` 使用 tmp → rename 模式避免文件损坏；`status.rs` 索引保存同理
2. **并发同步 / Parallel sync**: 使用 rayon 线程池并行处理包统计和 Windows 文件拷贝
3. **降级策略 / Graceful degradation**: robocopy 不可用时自动回退为 pwsh + xcopy；stow 失败时降级为逐文件 ln
4. **状态持久化 / State persistence**: `.wots_index.json` 缓存 blake3 哈希，避免重复比较已知已同步文件
5. **Broken-pipe 处理**: main.rs 正确静默处理 SIGPIPE
6. **安全模式写入 / Safe writes**: pacman/npm 备份使用 `.tmp` 后缀，出错不覆盖原文件

#### 待改进项 / Areas for Improvement

| # | 问题 / Issue | 严重度 | 建议 / Suggestion |
|---|---|---|---|
| 1 | **`wots_bin` 提交到 Git** | 中 | 1.7 MB 二进制不应版本化。考虑 GitHub Releases 或 git LFS |
| 2 | **`main.rs` 命令逻辑内联** | 中 | `cmd_stats`/`cmd_list`/`cmd_diff` 内联在 main.rs 中（~280 行）应抽取到独立模块 |
| 3 | **`status.rs` 过大** | 中 | 1,057 行的单文件应拆分为 `index.rs` + `status.rs` |
| 4 | **sync.rs / create.rs 零单测** | 中 | 这些核心模块缺少 `#[cfg(test)]` 块 |
| 5 | **索引加载静默降级** | 低 | `SyncIndex::load_from` 在 JSON 解析失败时静默返回空索引 (`unwrap_or_default`)，可能丢失同步状态 — 应输出 warning |
| 6 | **HashSet 导入未使用** | 低 | `status.rs:2` 的 `HashSet` 仅用于 `#[cfg(test)]` 中的 `keys_cloned`，可加 `#[allow(unused_imports)]` |
| 7 | **`synced: false` 字段默认值** | 低 | `IndexEntry.synced` 默认为 `false`，但语义应为"已确认同步"才标记为 `true`。对于 WSL 侧单方面变更的场景，需要验证不会导致索引"毒化" — 集成测试已覆盖此回归 |
| 8 | **集成测试 WSL 依赖** | 低 | 7 个核心集成测试需要 `/mnt/c/Windows` 存在，Linux CI 环境被跳过 |

### 测试覆盖 / Test Coverage

**单元测试**: `types.rs` 和 `discover.rs` 有完整的单元测试（类型方法、包解析、路径构建等）。`status.rs` 在 `#[cfg(test)]` 中有少量单元测试。

**未覆盖的模块**: `sync.rs`, `create.rs`, `config.rs`, `display.rs`, `util.rs`, `cli.rs` 没有单元测试。

**集成测试** (`tests/integration.rs`): 27 个测试函数，覆盖：
- 同步状态场景：双方同步、WSL 编辑、Windows 编辑、Windows 删除、WSL 删除、双方删除
- 内容哈希：相同/不同内容的 blake3 哈希对比
- CopyStatusCounts 计数器正确性
- SyncIndex 序列化/反序列化
- 符号链接检测
- stow 状态检查（空包/有文件包）
- copy_status 详细/批量/非目录
- 包发现和排除规则
- `FileSyncStatus` 标签唯一性
- `PkgType` 序列化往返
- 大文件格式化

---

## 🌐 环境变量 / Environment Variables

| 变量 / Variable | 默认值 / Default | 说明 / Description |
|---|---|---|
| `DOTFILES_DIR` | `$HOME/dotfiles` | 仓库根目录 |
| `WSL_DISTRO` | `archlinux` | WSL 发行版名称 |
| `WSL_MNT` | `/mnt/c` | WSL 挂载 C: 盘的位置 |
| `WIN_USER` | 自动检测 | Windows 用户名（`/mnt/c/Users` 下首个非系统用户） |
| `WOTS_CONCURRENT` | `8` | 最大并发同步操作数 |
| `WOTS_MAX_SIZE_MB` | `50` | 跳过超过此大小的文件（MB） |

---

## 🏗️ 源码构建 / Building from Source

```bash
# Debug 构建
cargo build --manifest-path wots/Cargo.toml

# Release 构建（优化 + LTO，单二进制 ~1.7 MB）
cargo build --release --manifest-path wots/Cargo.toml
cp wots/target/release/wots ./wots_bin
```

或使用 `just build` / `just build-debug`。

**Rust 依赖**: Rust 1.80+ (edition 2024)。`clap`, `walkdir`, `rayon`, `serde`, `serde_json`, `anyhow`, `colored`, `glob`, `shellexpand`, `blake3`, `comfy-table` — 全部纯 Rust。

**系统工具（运行时）**: `stow`（Linux）、`robocopy.exe` 或 `pwsh.exe`（WSL 上 Windows 同步）。

---

## ⚠️ 声明 / Disclaimer

这是个人配置。欢迎参考架构。如需采用此工作流，你需要：

1. `wots/` — Rust crate（或预编译 `wots_bin` 二进制）
2. `justfile` — 任务编排器
3. 按上述后缀命名约定组织的包目录

---

## 📝 许可证 / License

MIT
