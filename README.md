这是一个为你量身定制的英文版 `README.md`。它既展示了你强大的跨平台配置工作流，又清晰地声明了个人属性（便于他人参考借鉴）。

---

````markdown
# 🚀 Zion's Dotfiles

> **WOTS (Stow Backwards) — A unified dotfile engine for WSL, Linux, and Windows.**

This repository is my personal system configuration home. It leverages modern tooling to bridge the gap between **Linux (WSL)** and **Windows** environments. By using **Python**, **Pixi**, and **Just**, I've transformed the traditional, fragile dotfile management into a robust, automated orchestration pipeline.

---

## 🛠️ The Philosophy

Managing dotfiles on WSL usually involves two worlds that don't talk to each other:

1. **Linux**: Managed via symlinks (GNU Stow).
2. **Windows**: Requires force-copying configuration files to specific AppData/User paths.

This project solves this by using a **unified engine (`_wots`)** that handles both, orchestrated by a single command-line interface.

---

## 🏗️ Core Toolchain

- **[Pixi](https://pixi.sh/)**: Environment & Dependency Management. No more "Python environment hell."
- **[Just](https://github.com/casey/just)**: Task Orchestrator. Replaces scattered Shell scripts with a clean, unified API.
- **[GNU Stow](https://www.gnu.org/software/stow/)**: The classic, reliable engine for Linux symlink management.
- **WOTS Engine (`_wots/`)**: A custom Python engine I built to handle Windows file synchronization, package detection, and metadata backup.

---

## 🚀 Quick Start

### Prerequisites

- **WSL2** (Arch Linux recommended)
- **pixi** (for environment)
- **just** (for task execution)
- **GNU Stow** (for symlinking)

### Setup

```bash
git clone https://github.com/pu-007/dotfiles.git
cd dotfiles
pixi install
```
````

---

## 🕹️ Workflow: `just refresh`

The heart of this repo is the `just refresh` command. It automates the entire system maintenance lifecycle in one go:

1.  **Protect**: Automatically `git stash` local, uncommitted changes.
2.  **Update**: Updates system packages (yay, zinit, nvim, npm, etc.).
3.  **Backup**: Exports system metadata (pacman package lists).
4.  **Cleanup**: Cleans cache (uv, pip, go, npm, etc.).
5.  **Sync**: Pulls/Pushes to Git.
6.  **Restore**: Pops your local changes back.

---

## 📂 Naming Convention (Package Types)

WOTS auto-detects package types by folder suffixes. Here is how I organize my files:

| Suffix       | Target Path        | Sync Strategy    |
| :----------- | :----------------- | :--------------- |
| `.user`      | `~/`               | GNU Stow         |
| `.config`    | `~/.config/`       | GNU Stow         |
| `.root`      | `/`                | ln (sudo)        |
| `.winuser`   | `C:\Users\{name}\` | Force-Copy       |
| `.winconfig` | `AppData\Roaming\` | Force-Copy       |
| `.meta`      | N/A                | Metadata Storage |

---

## ⚙️ Available Commands

Run `just` to see all available tasks. Here are the highlights:

- `just sync`: Deploy all configurations to their respective targets.
- `just create <path>`: Create a new managed package from an existing file/dir.
- `just stats`: Repository statistics (package counts, file sizes, sync status).
- `just list`: Detailed list of managed packages.
- `just diff`: Identify unsynced files and local differences.

---

## ⚠️ Disclaimer & Inspiration

**This repository is a personal setup tailored to my workflow.**
If you are looking for a reliable way to manage dotfiles across WSL, feel free to **reference** the architecture.

**If you want to adopt this workflow:**
You only really need to copy these three components into your own repository:

1. `_wots/` (The engine)
2. `justfile` (The orchestrator)
3. `pixi.toml` (The environment)

Then, rename your folders with the suffixes defined above, and you're ready to go!

---

## 📝 License

MIT

```

---

### 给你的开源小贴士：
1. **GitHub 仓库设置**：记得在 GitHub 仓库主页右侧的 "About" 栏里添加描述，例如："A cross-platform dotfile management engine for WSL, Linux, and Windows using Pixi and Just."
2. **README 渲染**：GitHub 会自动渲染 `README.md`。这种排版在移动端和桌面端阅读体验都很好。
3. **Template 的可能性**：如果你之后想让这个项目火起来，可以考虑写一个 `template.justfile` 和 `template.pixi.toml`，把你的个人路径（如 `zion`）全部改成变量占位符，这样别人就能更轻松地 fork 了！
```
