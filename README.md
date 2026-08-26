# Zion's Dotfiles / 锡安的点文件

个人 WSL / Linux / Windows 点文件仓库。由 **WOTS** 按目录后缀同步。

> **WOTS 引擎已独立。** 源码、构建、CLI 与实现文档在兄弟仓库 [`../wots-src`](../wots-src)。
> 本仓库只保留点文件包、`just` 编排，以及已安装的 `./wots` 二进制。

---

## 包类型

包以 `<name>.<suffix>/` 放在仓库根目录：

| 后缀 | 目标 | 策略 |
| :--- | ---- | :--- |
| `.user` | `~/` | GNU Stow |
| `.config` | `~/.config/` | GNU Stow |
| `.root` | `/` | `sudo ln -sf` |
| `.meta` | — | 手动 |
| `.winuser` | `C:\Users\{name}\` | Robocopy / pwsh |
| `.winroot` | `C:\` | Robocopy / pwsh |

```text
~/dotfiles/          # 本仓库
├── zsh.user/
├── nvim.config/
├── wsl.root/
├── git.winuser/
├── packages.meta/
└── wots             # 由 ~/wots-src 构建安装

~/wots-src/          # 独立仓库：引擎源码
```

---

## 使用

需要 [GNU Stow](https://www.gnu.org/software/stow/)、[Just](https://github.com/casey/just)，以及已安装的 `./wots`（在 `~/wots-src` 执行 `just build`）。

```bash
git clone https://github.com/pu-007/dotfiles.git ~/dotfiles
cd ~/dotfiles
just            # 全部命令
just sync-dry   # 预览同步
just sync       # 同步全部包
just refresh    # 一键系统维护
```

| 命令 | 说明 |
| ---- | ---- |
| `just sync` / `sync-type` / `sync-app` / `sync-root` | 同步 |
| `just list` / `stats` / `diff` | 查询 |
| `just create` | 从现有路径建包 |
| `just refresh` | 保护 → 更新 → 备份 → 清理 → 远端同步 → 恢复 |

`justfile` 顶部 `CONFIRM := "true"` 时，同步前会打印路径映射并要求确认。

引擎开发（`build` / `test` / `lint`）请到 [`../wots-src`](../wots-src)。文档见 [docs/](docs/)。

---

## 环境变量

| 变量 | 默认 | 说明 |
| ---- | ---- | ---- |
| `DOTFILES_DIR` | `$HOME/dotfiles` | 仓库根 |
| `WSL_DISTRO` | `archlinux` | WSL 发行版名 |
| `WSL_MNT` | `/mnt/c` | C 盘挂载点 |
| `WIN_USER` | （必须） | Windows 用户名 |
| `WOTS_CONCURRENT` | `8` | 并发数 |
| `WOTS_MAX_SIZE_MB` | `50` | 跳过更大的文件 |

## 许可证

MIT
