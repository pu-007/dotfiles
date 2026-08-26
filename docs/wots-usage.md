# WOTS 使用方法 / Usage

> WOTS (WSL Dotfile Stow Tool) 的完整使用手册。架构与实现见 [wots-architecture.md](wots-architecture.md)。

---

## 包类型 / Package Types

所有包以**后缀目录**形式存放在 `DOTFILES_DIR`（默认 `~/dotfiles`）中，WOTS 通过后缀自动检测类型：

| 后缀 / Suffix | 类型 | 目标路径 / Target | 同步策略 / Strategy |
| :------------ | ---- | ----------------- | :------------------ |
| `.user`       | `user`   | `~/`                     | GNU Stow（符号链接） |
| `.config`     | `config` | `~/.config/`             | GNU Stow（符号链接） |
| `.root`       | `root`   | `/`                      | `sudo ln -sf`        |
| `.meta`       | `meta`   | N/A                      | 手动管理 / Manual    |
| `.winuser`    | `winuser`| `C:\Users\{name}\`       | Robocopy 镜像 / pwsh |
| `.winroot`    | `winroot`| `C:\`                    | Robocopy 镜像 / pwsh |

> **v2.0 变更**：已移除 `.local`、`.winlocal`、`.winroaming`、`.winconfig` 类型。
> 原 `.winlocal` / `.winroaming` 内容请迁移到 `.winuser` 包内对应子目录
> （如 `AppData/Local/...`）；Windows 系统级文件使用 `.winroot`。

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
├── tools.winroot/       → robocopy 到 C:\
│   └── scripts/tool.cmd
└── packages.meta/       → 手动管理（包列表等元数据）
```

### winuser 包内的可选用户名层 / Optional username layer

`.winuser` 包可以直接放文件，也可以建一层与 Windows 用户名同名的子目录：

```text
git.winuser/
└── zion/            # 若存在该子目录，同步时会剥掉这一层
    └── .gitconfig   → C:\Users\zion\.gitconfig
```

---

## Quick Start / 快速开始

前置条件：WSL2（或原生 Linux）、GNU Stow、[Just](https://github.com/casey/just)、Rust 1.80+（仅构建时需要）。

```bash
git clone https://github.com/pu-007/dotfiles.git
cd dotfiles

just build          # 构建 Release 二进制并安装补全
just test           # 运行测试
just lint           # cargo clippy
just wots --help    # 查看 CLI 帮助
```

**无需 Python、pixi、虚拟环境。** `wots` 是独立静态二进制（约 1.7 MB）。

---

## Just 命令体系 / Just Command Reference

本仓库所有操作均通过 `just` 命令组织。运行 `just` 查看完整列表。

### 构建 / Build

| 命令               | 说明                       |
| ------------------ | -------------------------- |
| `just build`       | 编译 Rust Release 二进制   |
| `just build-debug` | 编译 Debug 二进制          |
| `just test`        | 运行 `cargo test`          |
| `just lint`        | 运行 `cargo clippy`        |
| `just wots <args>` | 直接运行 wots 二进制       |

### 同步操作 / Sync Operations

| 命令                    | 说明                                       |
| ----------------------- | ------------------------------------------ |
| `just sync`             | 同步所有包（CONFIRM=true 时先展示路径确认）|
| `just sync-type <type>` | 按类型同步；`root` 类型建议用 `sync-root`  |
| `just sync-app <app>`   | 按包名同步，支持后缀自动识别类型           |
| `just sync-dry`         | 干运行预览                                 |
| `just sync-root`        | sudo 同步 root 包                          |

> **路径确认 / Path confirmation**：`justfile` 顶部有 `CONFIRM := "true"` 变量。
> 默认开启时，每次同步前会打印完整的 **仓库路径 → 目标路径** 映射清单并要求确认，
> 防止误写目标。改为 `"false"` 则跳过确认（等价于传 `--yes`）。

### 信息查询 / Information & Diff

| 命令                    | 说明           |
| ----------------------- | -------------- |
| `just stats`            | 仓库统计表格   |
| `just stats-json`       | JSON 统计      |
| `just list` / `list-json` / `list-type <type>` | 列出包 |
| `just diff` / `diff-type <type>` / `diff-app <app>` | 显示差异 |
| `just create <args>`    | 创建新包       |

### 系统维护 / System Maintenance

| 命令           | 说明                                                 |
| -------------- | ---------------------------------------------------- |
| `just refresh` | 一键全流程：保护→更新→备份→清理→同步远端→恢复        |

---

## Wots CLI Reference

### `wots create`

```bash
wots create [OPTIONS] [SOURCES]...
```

| 选项                    | 说明                                                       |
| ----------------------- | ---------------------------------------------------------- |
| `SOURCES`               | 源文件/目录（支持 `~` 展开）                               |
| `-u, --win-user <USER>` | Windows 用户名（创建 Windows 类型包时必需）                |
| `-a, --app-name <NAME>` | 自定义包名                                                 |
| `-t, --type <TYPE>`     | 显式类型：`user` `config` `root` `meta` `winuser` `winroot`|
| `-y, --yes`             | 跳过确认提示                                               |
| `-n, --dry-run`         | 仅预览                                                     |
| `--no-stow`             | 创建后不自动 stow（Linux 类型）                            |
| `--no-sync`             | 创建后不自动 Windows 同步                                  |

**存储策略**：`user/config` 类型使用 **move**（临时文件 + 原子 rename + 数量/大小校验）；
其余类型使用 **copy**。

```bash
# 自动检测类型
wots create ~/.zshrc ~/.zsh

# 显式指定类型和包名
wots create ~/.config/nvim/init.lua -t config -a nvim

# Windows 用户配置（必须能通过 --win-user 或 WIN_USER 解析用户名）
wots create /mnt/c/Users/pu/.gitconfig -t winuser -a git -u pu

# Windows 系统级（C:\ 下）
wots create /mnt/c/scripts/tool.cmd -t winroot -a tools -u pu

# root 配置
wots create /etc/wsl.conf -t root -a wsl
```

### `wots sync`

```bash
wots sync --win-user zion                  # 同步全部
wots sync --win-user zion --type winuser   # 按类型
wots sync --win-user zion --app git.user   # 后缀自动识别: type=User, app=git
wots sync --win-user zion --app pwsh.winuser
wots sync --win-user zion --app zsh        # 无后缀: 按 app 名匹配所有类型
```

| 选项                | 说明                                     |
| ------------------- | ---------------------------------------- |
| `-u, --win-user`    | Windows 用户名                           |
| `-t, --type <TYPE>` | 仅同步该类型                             |
| `--app <NAME>`      | 仅同步指定包，支持 `<name>.<suffix>` 格式|
| `-n, --dry-run`     | 仅预览                                   |
| `-y, --yes`         | 跳过路径确认提示                         |
| `--bypass`          | 跳过 root 确认提示                       |
| `-q, --quiet`       | 减少输出                                 |

**同步前路径确认**：默认（未加 `-y`）会先列出每个文件的
`仓库绝对路径 → 目标绝对路径` 映射并要求确认；`-y`、`--dry-run`、`-q` 会跳过。

**Linux 同步**（`user/config/root`）：GNU Stow `--adopt`；stow 冲突或 root 类型降级为逐文件 `ln -sf`。

**Windows 同步**（`winuser/winroot`）：优先 `robocopy.exe /MIR /MT:8`；不可用时回退 `pwsh.exe Copy-Item`。

**状态索引**：`.wots_index.json` 缓存 mtime/size/blake3 哈希避免重复比较，原子写入。

### `wots stats` / `list` / `diff`

```bash
wots stats --win-user zion [--json]
wots list  [--type user] [--json]
wots diff  --win-user zion [--app git.user] [--json]
```

diff 列出不同步的文件：Linux 包显示未建立链接的目标路径；Windows 包显示 mtime/size/blake3 不一致、缺失或内容分叉的文件及状态标签。

---

## 环境变量 / Environment Variables

| 变量               | 默认值             | 说明                              |
| ------------------ | ------------------ | --------------------------------- |
| `DOTFILES_DIR`     | `$HOME/dotfiles`   | 仓库根目录                        |
| `WSL_DISTRO`       | `archlinux`        | WSL 发行版名称（UNC 路径用）      |
| `WSL_MNT`          | `/mnt/c`           | Windows C 盘挂载点                |
| `WIN_USER`         | （必须设置）       | Windows 用户名，可用 `--win-user` |
| `WOTS_CONCURRENT`  | `8`                | 最大并发同步数                    |
| `WOTS_MAX_SIZE_MB` | `50`               | 跳过超过此大小的文件              |
