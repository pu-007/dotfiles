# 本仓库如何使用 WOTS

> 引擎 CLI、包类型细节与实现见独立仓库 [`../../wots-src/docs/usage.md`](../../wots-src/docs/usage.md)。

本仓库只调用已安装的 `./wots`。构建 / 测试 / lint 在 `~/wots-src`。

---

## 包约定

包以 `<name>.<suffix>/` 放在仓库根：

| 后缀 | 目标 | 策略 |
| :--- | ---- | :--- |
| `.user` | `~/` | GNU Stow |
| `.config` | `~/.config/` | GNU Stow |
| `.root` | `/` | `sudo ln -sf` |
| `.meta` | — | 手动 |
| `.winuser` | `C:\Users\{name}\` | Robocopy / pwsh |
| `.winroot` | `C:\` | Robocopy / pwsh |

`.winuser` 包内若有与 Windows 用户名同名的子目录，同步时会剥掉该层。

---

## Just 命令

```bash
just            # 全部命令
just sync-dry
just sync
just refresh
```

| 命令 | 说明 |
| ---- | ---- |
| `just sync` | 同步全部（`CONFIRM=true` 时先确认路径） |
| `just sync-type <type>` | 按类型；root 用 `sync-root` |
| `just sync-app <app>` | 按包名，支持 `git.user` 后缀 |
| `just sync-dry` | 干运行 |
| `just sync-root` | sudo 同步 root 包 |
| `just stats` / `stats-json` | 统计 |
| `just list` / `list-json` / `list-type` | 列包 |
| `just diff` / `diff-type` / `diff-app` | 差异 |
| `just create <args>` | 创建新包 |
| `just refresh` | 保护 → 更新 → 备份 → 清理 → 远端同步 → 恢复 |

`justfile` 顶部 `CONFIRM := "true"` 时，同步前打印 **仓库路径 → 目标路径** 并要求确认。直接跑 `./wots sync` 时默认同样确认，`-y` 跳过。

---

## 环境变量

| 变量 | 默认 | 说明 |
| ---- | ---- | ---- |
| `DOTFILES_DIR` | `$HOME/dotfiles` | 仓库根 |
| `WSL_DISTRO` | `archlinux` | WSL 发行版 |
| `WSL_MNT` | `/mnt/c` | C 盘挂载点 |
| `WIN_USER` | （必须） | 也可用 `--win-user` |
| `WOTS_CONCURRENT` | `8` | 并发数 |
| `WOTS_MAX_SIZE_MB` | `50` | 跳过更大的文件 |
