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
refresh: protect update backup cleanup sync-remote restore
    @echo -e "\n{{ c_green }}{{ c_bold }}🎉 === WOTS Refresh All Done at $(date +'%H:%M:%S') ==={{ c_reset }}"

# [维护步骤 1/6] 检查并保护工作区状态（防止未提交修改冲突）
[group('0. 系统维护 (System Maintenance)')]
protect:
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

# [维护步骤 2/6] 更新系统包与开发环境
[group('0. 系统维护 (System Maintenance)')]
update:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [2/6] 更新系统包与开发环境...{{ c_reset }}"

    [ -f "$HOME/.zinit/bin/zinit.zsh" ] && { echo -e "{{ c_gray }}  ↳ 更新 zinit...{{ c_reset }}"; zsh -ic "zinit update --all" || true; } || true
    command -v yay &>/dev/null && { echo -e "{{ c_gray }}  ↳ 更新 yay 包...{{ c_reset }}"; yay -Syu --noconfirm || true; } || true
    command -v pkgfile &>/dev/null && { echo -e "{{ c_gray }}  ↳ 更新 pkgfile...{{ c_reset }}"; sudo pkgfile --update 2>/dev/null || true; } || true

    # Cli Tools init script for zsh
    mkdir -p "./zsh.user/.config/zsh"
    command -v fzf &>/dev/null && fzf --zsh > ./zsh.user/.config/zsh/fzf.zsh || true
    command -v starship &>/dev/null && starship init zsh > ./zsh.user/.config/zsh/starship.zsh || true
    command -v zoxide &>/dev/null && zoxide init zsh > ./zsh.user/.config/zsh/zoxide.zsh || true

    if command -v nvim &>/dev/null; then
        echo -ne "{{ c_gray }}  ↳ 同步 Neovim 插件 (静默进行中) "
        nvim --headless "+Lazy! sync" +qa >/tmp/wots_nvim_lazy.log 2>&1 &
        NVIM_PID=$!
        while kill -0 $NVIM_PID 2>/dev/null; do
            echo -ne "."
            sleep 0.5
        done
        echo -e " Done.{{ c_reset }}"
    fi

    command -v npm &>/dev/null && { echo -e "{{ c_gray }}  ↳ 更新 NPM 全局依赖...{{ c_reset }}"; sudo npm install -g @google/gemini-cli aicommit2 || true; } || true
    command -v komorebic.exe &>/dev/null && { echo -e "{{ c_gray }}  ↳ 更新 komorebic 配置...{{ c_reset }}"; komorebic.exe fetch-app-specific-configuration || true; } || true

# [维护步骤 3/6] 导出元数据并收集系统自动变更
[group('0. 系统维护 (System Maintenance)')]
backup:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [3/6] 导出元数据并收集系统自动变更...{{ c_reset }}"
    mkdir -p "{{ dotfiles }}/packages.meta"

    # 定义安全备份工具输出的内部函数
    safe_backup() {
        local cmd="$1"
        local backup_cmd="$2"
        local dest_file="$3"
        if command -v "$cmd" &>/dev/null; then
            if eval "$backup_cmd" > "${dest_file}.tmp" 2>/dev/null; then
                mv "${dest_file}.tmp" "$dest_file"
            else
                rm -f "${dest_file}.tmp"
            fi
        fi
    }

    safe_backup "pacman" "pacman -Qqe" "{{ dotfiles }}/packages.meta/pacman.txt"
    safe_backup "npm" "npm list -g --depth=0" "{{ dotfiles }}/packages.meta/npm.txt"
    safe_backup "uv" "uv tool list" "{{ dotfiles }}/packages.meta/uv.txt"

    cd "{{ dotfiles }}"
    git add packages.meta/
    git add -u

    if ! git diff --cached --quiet; then
        CHANGED_FILES=$(git diff --cached --name-only | awk -F'/' '{print $NF}' | sort -u | paste -sd ", " -)
        echo -e "{{ c_yellow }}  ↳ 发现系统自动产生的变更，准备静默提交: [$CHANGED_FILES]{{ c_reset }}"

        COMMIT_TITLE="chore(sync): auto-update dependencies ({{ timestamp }})"
        COMMIT_DESC="Files dynamically updated by automated script: - $CHANGED_FILES"

        git commit --no-verify -m "$COMMIT_TITLE" -m "$COMMIT_DESC" > /dev/null
        echo -e "{{ c_green }}  ↳ ✅ 自动化变更已保存至本地 Git 树。{{ c_reset }}"
    else
        echo -e "{{ c_gray }}  ↳ 依赖和元数据无变更，跳过提交。{{ c_reset }}"
    fi

# [维护步骤 4/6] 深度清理系统与开发环境缓存
[group('0. 系统维护 (System Maintenance)')]
cleanup:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [4/6] 深度清理系统与开发环境缓存...{{ c_reset }}"

    [ -d "$HOME/.cache/uv" ] && uv cache clean >/dev/null || true
    [ -d "$HOME/.cache/pip" ] && pip cache purge >/dev/null || true
    rm -rf "$HOME/.cache/huggingface/hub" "$HOME/.cache/huggingface/download" 2>/dev/null || true
    command -v go &>/dev/null && go clean -cache -modcache || true
    command -v npm &>/dev/null && npm cache clean --force >/dev/null 2>&1 || true
    command -v yay &>/dev/null && yay -Sc --noconfirm >/dev/null || true
    command -v scoop &>/dev/null && scoop cleanup -a -g -k >/dev/null 2>&1 || true
    command -v trash-empty &>/dev/null && { sudo trash-empty -f --all-users 2>/dev/null || trash-empty -f; } || true

    echo -e "{{ c_yellow }}  ↳ (提示) Docker 清理建议定期手动检查: ssh root@xxx docker system prune -a -f{{ c_reset }}"

# [维护步骤 5/6] 同步代码至远端仓库 (Pull & Push)
[group('0. 系统维护 (System Maintenance)')]
sync-remote:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [5/6] 同步代码至远端仓库 (Pull & Push)...{{ c_reset }}"
    cd "{{ dotfiles }}"

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

# [维护步骤 6/6] 还原用户先前的未完成状态
[group('0. 系统维护 (System Maintenance)')]
restore:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [6/6] 还原用户先前的未完成状态...{{ c_reset }}"
    cd "{{ dotfiles }}"

    STASH_REF=$(git stash list | grep "{{ stash_msg }}" | head -n 1 | cut -d: -f1 || true)

    if [ -n "$STASH_REF" ]; then
        echo -e "{{ c_yellow }}  ↳ 找到用户执行前的状态备份 ($STASH_REF)，正在精准还原...{{ c_reset }}"
        if git stash pop --index "$STASH_REF" > /dev/null; then
            echo -e "{{ c_green }}  ↳ ✅ 状态恢复完毕！可以继续你的开发。{{ c_reset }}"
        else
            echo -e "{{ c_red }}  ↳ ⚠️ 恢复 Stash 时发现文件冲突！该 Stash 仍保留在栈中以供安全恢复，请手动解决冲突 (git status){{ c_reset }}"
        fi
    else
        echo -e "{{ c_gray }}  ↳ 初始状态无修改，无需恢复。{{ c_reset }}"
    fi


# =============================================================================
# ⚙️ 2. 构建任务 (Wots CLI Build)
# =============================================================================

# [构建] 编译 Rust 二进制，可选 profile='release' 或 'debug'
[group('1. 构建 (Build)')]
build profile='release' *args:
    #!/usr/bin/env bash
    set -euo pipefail
    
    CARGO_FLAGS=""
    TARGET_DIR="debug"
    if [ "{{ profile }}" = "release" ]; then
        CARGO_FLAGS="--release"
        TARGET_DIR="release"
    fi

    echo -e "{{ c_blue }}▶ Building Wots in {{ profile }} mode...{{ c_reset }}"
    cargo build $CARGO_FLAGS --manifest-path {{ wots_crate }}/Cargo.toml {{ args }}
    cp -f {{ wots_crate }}/target/$TARGET_DIR/wots {{ wots }}
    chmod +x {{ wots }}
    echo -e "{{ c_green }}✓  Build complete: {{ wots }}{{ c_reset }}"
    
    just _install_completion zsh
    echo -e "{{ c_green }}✓  Completion updated: {{ comp_dir }}/_wots{{ c_reset }}"

# [构建] 编译 Rust 二进制 (debug, 附带 Backtrace 调试信息)
[group('1. 构建 (Build)')]
build-debug *args:
    @just build debug {{ args }}

# [测试] 运行 cargo test
[group('1. 构建 (Build)')]
test *args:
    @cargo test --manifest-path {{ wots_crate }}/Cargo.toml {{ args }}

# [测试] 运行 cargo clippy
[group('1. 构建 (Build)')]
lint *args:
    @cargo clippy --manifest-path {{ wots_crate }}/Cargo.toml -- -D warnings {{ args }}

# [补全] 打印指定 shell 的补全脚本到 stdout
[group('1. 构建 (Build)')]
completion shell='zsh':
    @{{ wots }} completion {{ shell }}

# [补全] 安装补全到本地仓库 zsh.user
[group('1. 构建 (Build)')]
completion-install shell='zsh':
    @echo -e "{{ c_blue }}▶ Installing {{ shell }} completion for wots...{{ c_reset }}"
    @just _install_completion {{ shell }}
    @echo -e "{{ c_green }}✓  Installed: {{ comp_dir }}/_wots{{ c_reset }}"
    @echo -e "{{ c_gray }}  ↳ Run 'exec {{ shell }}' or restart your shell to activate.{{ c_reset }}"

# [私有辅助] 写入补全脚本文件
_install_completion shell:
    @mkdir -p "{{ comp_dir }}"
    @{{ wots }} completion {{ shell }} > "{{ comp_dir }}/_wots"


# =============================================================================
# ⚙️ 3. 配置管理 (Wots CLI Wrappers)
# =============================================================================

[group('2. 配置管理 (Wots CLI)')]
create +args:
    @{{ wots }} create {{ args }}

[group('2. 配置管理 (Wots CLI)')]
sync *args:
    @{{ wots }} sync {{ args }}

[group('2. 配置管理 (Wots CLI)')]
sync-type type *args:
    @{{ wots }} sync --type {{ type }} {{ args }}

[group('2. 配置管理 (Wots CLI)')]
sync-app app *args:
    @{{ wots }} sync --app {{ app }} {{ args }}

[group('2. 配置管理 (Wots CLI)')]
sync-dry *args:
    @{{ wots }} sync --dry-run {{ args }}

[group('2. 配置管理 (Wots CLI)')]
sync-root *args:
    @sudo {{ wots }} sync --type root --bypass {{ args }}

[group('2. 配置管理 (Wots CLI)')]
stats *args:
    @{{ wots }} stats {{ args }}

[group('2. 配置管理 (Wots CLI)')]
list *args:
    @{{ wots }} list {{ args }}

[group('2. 配置管理 (Wots CLI)')]
list-type type *args:
    @{{ wots }} list --type {{ type }} {{ args }}

[group('2. 配置管理 (Wots CLI)')]
diff *args:
    @{{ wots }} diff {{ args }}

[group('2. 配置管理 (Wots CLI)')]
diff-type type *args:
    @{{ wots }} diff --type {{ type }} {{ args }}

[group('2. 配置管理 (Wots CLI)')]
diff-app app *args:
    @{{ wots }} diff --app {{ app }} {{ args }}
