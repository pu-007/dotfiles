# =============================================================================
# 🚀 WOTS & Dotfiles 现代自动化管理脚本 (Rust Engine + Just Orchestrator)
# 🛡️ 工业级优化版：具备防崩溃、防锁死、状态精准恢复特性
# =============================================================================

# -----------------------------------------------------------------------------
# 🌐 全局变量配置
# -----------------------------------------------------------------------------
dotfiles := justfile_directory()
timestamp := `date +'%Y-%m-%d %H:%M:%S'`
stash_msg := "WOTS_AUTO_STASH_" + timestamp
wots := dotfiles / "wots"
wots_crate := dotfiles / "wots-src"
comp_dir := dotfiles / "zsh.user" / ".config" / "zsh" / "completions"

# 🎨 UI 色彩与排版配置
c_reset := '\033[0m'
c_green := '\033[1;32m'
c_blue := '\033[1;34m'
c_yellow := '\033[1;33m'
c_red := '\033[1;31m'
c_gray := '\033[1;90m'
c_bold := '\033[1m'

# 默认列出所有可用命令
default:
    @just --list --unsorted

# =============================================================================
# 🛠️ 1. 系统维护 (System Maintenance)
# =============================================================================

# [一键执行] 保护工作区 -> 更新依赖 -> 备份元数据 -> 深度清理 -> 同步远程 -> 恢复本地
[group('0. 系统维护 (System Maintenance)')]
refresh: _protect _update _backup _cleanup _sync_remote _restore
    @echo -e "\n{{ c_green }}{{ c_bold }}🎉 === WOTS Refresh All Done at $(date +'%H:%M:%S') ==={{ c_reset }}"

# =============================================================================
# ⚙️ 2. 配置管理 (Wots CLI)
# =============================================================================

# [构建] 编译 Rust 二进制 (release) + 更新补全脚本
[group('1. 构建 (Build)')]
build *args:
    @cargo build --release --manifest-path {{ wots_crate }}/Cargo.toml {{ args }}
    @cp -f {{ wots_crate }}/target/release/wots {{ wots }}
    @chmod +x {{ wots }}
    @echo -e "{{ c_green }}✓  Build complete: {{ wots }}{{ c_reset }}"
    @mkdir -p "{{ comp_dir }}"
    @{{ wots }} completion zsh > "{{ comp_dir }}/_wots"
    @echo -e "{{ c_green }}✓  Completion updated: {{ comp_dir }}/_wots{{ c_reset }}"

# [构建] 编译 Rust 二进制 (debug, with backtraces) + 更新补全脚本
[group('1. 构建 (Build)')]
build-debug *args:
    @cargo build --manifest-path {{ wots_crate }}/Cargo.toml {{ args }}
    @cp -f {{ wots_crate }}/target/debug/wots {{ wots }}
    @chmod +x {{ wots }}
    @echo -e "{{ c_green }}✓  Debug build complete: {{ wots }}{{ c_reset }}"
    @mkdir -p "{{ comp_dir }}"
    @{{ wots }} completion zsh > "{{ comp_dir }}/_wots"
    @echo -e "{{ c_green }}✓  Completion updated: {{ comp_dir }}/_wots{{ c_reset }}"

# [测试] 运行 cargo test
[group('1. 构建 (Build)')]
test *args:
    @cargo test --manifest-path {{ wots_crate }}/Cargo.toml {{ args }}

# [测试] 运行 cargo clippy
[group('1. 构建 (Build)')]
lint *args:
    @cargo clippy --manifest-path {{ wots_crate }}/Cargo.toml -- -D warnings {{ args }}

# [补全] 打印指定 shell 的补全脚本到 stdout
#   just completion zsh          → zsh  补全脚本
#   just completion bash         → bash 补全脚本
#   just completion fish         → fish 补全脚本
[group('1. 构建 (Build)')]
completion shell='zsh':
    @{{ wots }} completion {{ shell }}

# [补全] 安装 zsh 补全到仓库 (zsh.user)，stow 到 ~/.config/zsh/completions/
#   安装后执行 exec zsh 激活。之后 wots create/sync/stats <TAB> 即可补全子命令。
#   注意: just wots ... 不支持子命令补全，请使用独立的 wots 命令。
[group('1. 构建 (Build)')]
completion-install shell='zsh':
    @echo -e "{{ c_blue }}▶ Installing {{ shell }} completion for wots...{{ c_reset }}"
    @mkdir -p "{{ comp_dir }}"
    @{{ wots }} completion {{ shell }} > "{{ comp_dir }}/_wots"
    @echo -e "{{ c_green }}✓  Installed: {{ comp_dir }}/_wots{{ c_reset }}"
    @echo -e "{{ c_gray }}  ↳ Run 'exec {{ shell }}' or restart your shell to activate.{{ c_reset }}"

# -----------------------------------------------------------------------------
# ⚙️ 2. 配置管理 (Wots CLI)
#  可用类型: user, config, local, root, meta, winuser, winconfig, winlocal, winroaming
# -----------------------------------------------------------------------------

# 运行任意 wots 命令 (默认显示帮助)
# 用法: just wots -- list --json

# 创建新包: just create ~/.zshrc -t user -a zsh
[group('2. 配置管理 (Wots CLI)')]
create +args:
    @{{ wots }} create {{ args }}

# 同步所有包到目标 (转发额外参数: just sync -- -h)
[group('2. 配置管理 (Wots CLI)')]
sync *args:
    @{{ wots }} sync {{ args }}

# 按类型同步 (root 由 wots 内部处理 sudo): just sync-type root
[group('2. 配置管理 (Wots CLI)')]
sync-type type *args:
    @{{ wots }} sync --type {{ type }} {{ args }}

# 按包名同步: just sync-app git
[group('2. 配置管理 (Wots CLI)')]
sync-app app *args:
    @{{ wots }} sync --app {{ app }} {{ args }}

# 预览同步 (干运行): just sync-dry --type winuser
[group('2. 配置管理 (Wots CLI)')]
sync-dry *args:
    @{{ wots }} sync --dry-run {{ args }}

# 同步 root 包 (带 sudo, 跳过确认)
[group('2. 配置管理 (Wots CLI)')]
sync-root *args:
    @sudo {{ wots }} sync --type root --bypass {{ args }}

# 仓库统计: 包数量、文件数、大小、状态
[group('2. 配置管理 (Wots CLI)')]
stats *args:
    @{{ wots }} stats {{ args }}

# 列出所有包及其状态
[group('2. 配置管理 (Wots CLI)')]
list *args:
    @{{ wots }} list {{ args }}

# 按类型列出包: just list-type user
[group('2. 配置管理 (Wots CLI)')]
list-type type *args:
    @{{ wots }} list --type {{ type }} {{ args }}

# 显示差异: 哪些文件需要同步
[group('2. 配置管理 (Wots CLI)')]
diff *args:
    @{{ wots }} diff {{ args }}

# 按类型查看差异: just diff-type winuser
[group('2. 配置管理 (Wots CLI)')]
diff-type type *args:
    @{{ wots }} diff --type {{ type }} {{ args }}

# 按包名查看差异: just diff-app git
[group('2. 配置管理 (Wots CLI)')]
diff-app app *args:
    @{{ wots }} diff --app {{ app }} {{ args }}

# =============================================================================
# 📦 核心流水线子任务 (Private Sub-tasks) - 列表隐身，提供模块化原子操作
# =============================================================================

# 🛡️ [1/6] 保护工作区
_protect:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [1/6] 检查并保护工作区状态...{{ c_reset }}"
    cd "{{ dotfiles }}"

    if ! git diff --quiet || ! git diff --cached --quiet || [ -n "$(git ls-files --others --exclude-standard)" ]; then
        echo -e "{{ c_yellow }}  ↳ 发现未提交的更改，正在执行安全 Stash 隔离 (ID: {{ timestamp }})...{{ c_reset }}"
        git stash push --include-untracked -m "{{ stash_msg }}" > /dev/null
    else
        echo -e "{{ c_gray }}  ↳ 工作区干净，无需隔离。{{ c_reset }}"
    fi

# 🔄 [2/6] 更新依赖
_update:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [2/6] 更新系统包与开发环境...{{ c_reset }}"

    # 【修复陷阱】: 每行末尾附带 || true，防止 set -e 在最后一行导致脚本异常退出
    [ -f "$HOME/.zinit/bin/zinit.zsh" ] && { echo -e "{{ c_gray }}  ↳ 更新 zinit...{{ c_reset }}"; zsh -ic "zinit update --all" || true; } || true
    command -v yay &>/dev/null && { echo -e "{{ c_gray }}  ↳ 更新 yay 包...{{ c_reset }}"; yay -Syu --noconfirm || true; } || true
    command -v pkgfile &>/dev/null && { echo -e "{{ c_gray }}  ↳ 更新 pkgfile...{{ c_reset }}"; sudo pkgfile --update 2>/dev/null || true; } || true

    # Cli Tools init script for zsh
    fzf --zsh > ./zsh.user/.config/zsh/fzf.zsh
    starship init zsh > ./zsh.user/.config/zsh/starship.zsh
    zoxide init zsh > ./zsh.user/.config/zsh/zoxide.zsh

    # 【UI 优化】: 解决 nvim headless 严重刷屏问题。将其置于后台静默运行，前台单行显示加载动画
    if command -v nvim &>/dev/null; then
        echo -ne "{{ c_gray }}  ↳ 同步 Neovim 插件 (静默进行中) "
        # 将无用输出重定向到临时文件
        nvim --headless "+Lazy! sync" +qa >/tmp/wots_nvim_lazy.log 2>&1 &
        NVIM_PID=$!
        # 显示单行进度点
        while kill -0 $NVIM_PID 2>/dev/null; do
            echo -ne "."
            sleep 0.5
        done
        echo -e " Done.{{ c_reset }}"
    fi

    command -v npm &>/dev/null && { echo -e "{{ c_gray }}  ↳ 更新 NPM 全局依赖...{{ c_reset }}"; sudo npm install -g @google/gemini-cli aicommit2 || true; } || true
    command -v komorebic.exe &>/dev/null && { echo -e "{{ c_gray }}  ↳ 更新 komorebic 配置...{{ c_reset }}"; komorebic.exe fetch-app-specific-configuration || true; } || true

# 💾 [3/6] 备份元数据与自动提交
_backup:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [3/6] 导出元数据并收集系统自动变更...{{ c_reset }}"
    mkdir -p "{{ dotfiles }}/packages.meta"

    # 采用安全写入模式 (.tmp)，防止命令报错时源文件被意外清空
    command -v pacman &>/dev/null && pacman -Qqe > "{{ dotfiles }}/packages.meta/pacman.txt.tmp" && mv "{{ dotfiles }}/packages.meta/pacman.txt.tmp" "{{ dotfiles }}/packages.meta/pacman.txt" || true
    command -v npm &>/dev/null && npm list -g --depth=0 > "{{ dotfiles }}/packages.meta/npm.txt.tmp" && mv "{{ dotfiles }}/packages.meta/npm.txt.tmp" "{{ dotfiles }}/packages.meta/npm.txt" || true
    rm -f "{{ dotfiles }}/packages.meta/*.tmp" 2>/dev/null || true

    cd "{{ dotfiles }}"

    # 收集 packages.meta (含新增 uniget 备份等) 与工作区其他已被追踪的文件 (lazy-lock.json)
    git add packages.meta/
    git add -u

    if ! git diff --cached --quiet; then
        CHANGED_FILES=$(git diff --cached --name-only | awk -F'/' '{print $NF}' | sort -u | paste -sd ", " -)
        echo -e "{{ c_yellow }}  ↳ 发现系统自动产生的变更，准备静默提交: [$CHANGED_FILES]{{ c_reset }}"

        COMMIT_TITLE="chore(sync): auto-update dependencies ({{ timestamp }})"
        COMMIT_DESC="Files dynamically updated by automated script: - $CHANGED_FILES"

        # --no-verify 强行跳过钩子，防止自动流程被卡死
        git commit --no-verify -m "$COMMIT_TITLE" -m "$COMMIT_DESC" > /dev/null
        echo -e "{{ c_green }}  ↳ ✅ 自动化变更已保存至本地 Git 树。{{ c_reset }}"
    else
        echo -e "{{ c_gray }}  ↳ 依赖和元数据无变更，跳过提交。{{ c_reset }}"
    fi

# 🧹 [4/6] 深度清理
_cleanup:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [4/6] 深度清理系统与开发环境缓存...{{ c_reset }}"

    # 全部添加 || true 护盾
    [ -d "$HOME/.cache/uv" ] && uv cache clean >/dev/null || true
    [ -d "$HOME/.cache/pip" ] && pip cache purge >/dev/null || true
    rm -rf "$HOME/.cache/huggingface/hub" "$HOME/.cache/huggingface/download" 2>/dev/null || true
    command -v go &>/dev/null && go clean -cache -modcache || true
    command -v npm &>/dev/null && npm cache clean --force >/dev/null 2>&1 || true
    command -v yay &>/dev/null && yay -Sc --noconfirm >/dev/null || true
    command -v scoop &>/dev/null && scoop cleanup -a -g -k >/dev/null 2>&1 || true
    command -v trash-empty &>/dev/null && { sudo trash-empty -f --all-users 2>/dev/null || trash-empty -f; } || true

    echo -e "{{ c_yellow }}  ↳ (提示) Docker 清理建议定期手动检查: ssh root@xxx docker system prune -a -f{{ c_reset }}"

# ☁️ [5/6] 远端同步
_sync_remote:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [5/6] 同步代码至远端仓库 (Pull & Push)...{{ c_reset }}"
    cd "{{ dotfiles }}"

    # pull 发生冲突立即中止，保护工作区不锁死
    if ! git pull --rebase --autostash; then
        echo -e "{{ c_red }}  ↳ ⚠️ Pull 遇到严重冲突或网络错误！正在中止 Rebase 以保护当前工作区...{{ c_reset }}"
        git rebase --abort 2>/dev/null || true
        echo -e "{{ c_yellow }}  ↳ ⚠️ 放弃本次 Push，请手动检查冲突，但本地代码不受影响。{{ c_reset }}"
    else
        echo -e "{{ c_gray }}  ↳ Pull 完成，正在推送...{{ c_reset }}"
        if git push; then
            echo -e "{{ c_green }}  ↳ ✅ 远端同步成功。{{ c_reset }}"
        else
            echo -e "{{ c_red }}  ↳ ⚠️ 推送失败，请检查网络或远端写权限！{{ c_reset }}"
        fi
    fi

# 🪄 [6/6] 恢复工作区
_restore:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [6/6] 还原用户先前的未完成状态...{{ c_reset }}"
    cd "{{ dotfiles }}"

    STASH_REF=$(git stash list | grep "{{ stash_msg }}" | head -n 1 | cut -d: -f1 || true)

    if [ -n "$STASH_REF" ]; then
        echo -e "{{ c_yellow }}  ↳ 找到用户执行前的状态备份 ($STASH_REF)，正在精准还原...{{ c_reset }}"
        # --index 保留暂存区 (add 过的内容) 的完美状态
        if git stash pop --index "$STASH_REF" > /dev/null; then
            echo -e "{{ c_green }}  ↳ ✅ 状态恢复完毕！可以继续你的开发。{{ c_reset }}"
        else
            echo -e "{{ c_red }}  ↳ ⚠️ 恢复 Stash 时发现文件冲突！该 Stash 仍保留在栈中以供安全恢复，请手动解决冲突 (git status){{ c_reset }}"
        fi
    else
        echo -e "{{ c_gray }}  ↳ 初始状态无修改，无需恢复。{{ c_reset }}"
    fi
