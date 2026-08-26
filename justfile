# =============================================================================
# Dotfiles 编排 (Just Orchestrator)
# 维护本仓库 + 调用已安装的 ./wots。引擎源码与构建见 ../wots-src
# =============================================================================

dotfiles := justfile_directory()
timestamp := `date +'%Y-%m-%d %H:%M:%S'`
stash_msg := "WOTS_AUTO_STASH_" + timestamp
wots := dotfiles / "wots"
WIN_USER := "zion"

# 同步前手动确认文件路径（"true" 开启 / "false" 跳过确认）
CONFIRM := "false"
confirm_flag := if CONFIRM == "true" { "" } else { "--yes" }

c_reset := '\033[0m'
c_green := '\033[1;32m'
c_blue := '\033[1;34m'
c_yellow := '\033[1;33m'
c_red := '\033[1;31m'
c_gray := '\033[1;90m'
c_bold := '\033[1m'

default:
    @just --list --unsorted

# =============================================================================
# 系统维护
# =============================================================================

# [一键执行] 保护工作区 -> 更新依赖 -> 备份元数据 -> 深度清理 -> 同步远程 -> 恢复本地
[group('0. 系统维护')]
refresh: protect update backup cleanup sync-remote restore
    @echo -e "\n{{ c_green }}{{ c_bold }}=== Refresh done at $(date +'%H:%M:%S') ==={{ c_reset }}"

# [1/6] 检查并保护工作区状态
[group('0. 系统维护')]
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

# [2/6] 更新系统包与开发环境
[group('0. 系统维护')]
update:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [2/6] 更新系统包与开发环境...{{ c_reset }}"

    [ -f "$HOME/.zinit/bin/zinit.zsh" ] && { echo -e "{{ c_gray }}  ↳ 更新 zinit...{{ c_reset }}"; zsh -ic "zinit update --all" || true; } || true
    command -v yay &>/dev/null && { echo -e "{{ c_gray }}  ↳ 更新 yay 包...{{ c_reset }}"; yay -Syu --noconfirm || true; } || true
    command -v pkgfile &>/dev/null && { echo -e "{{ c_gray }}  ↳ 更新 pkgfile...{{ c_reset }}"; sudo pkgfile --update 2>/dev/null || true; } || true

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

    command -v npm &>/dev/null && { echo -e "{{ c_gray }}  ↳ 更新 NPM 全局依赖...{{ c_reset }}"; sudo npm install -g aicommit2 || true; } || true
    command -v komorebic.exe &>/dev/null && { echo -e "{{ c_gray }}  ↳ 更新 komorebic 配置...{{ c_reset }}"; komorebic.exe fetch-app-specific-configuration || true; } || true

# [3/6] 导出元数据并收集系统自动变更
[group('0. 系统维护')]
backup:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [3/6] 导出元数据并收集系统自动变更...{{ c_reset }}"
    mkdir -p "{{ dotfiles }}/packages.meta"

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
    safe_backup "pip" "pip list" "{{ dotfiles }}/packages.meta/pip.txt"

    cd "{{ dotfiles }}"
    git add packages.meta/
    git add -u

    if ! git diff --cached --quiet; then
        CHANGED_FILES=$(git diff --cached --name-only | awk -F'/' '{print $NF}' | sort -u | paste -sd ", " -)
        echo -e "{{ c_yellow }}  ↳ 发现系统自动产生的变更，准备静默提交: [$CHANGED_FILES]{{ c_reset }}"

        COMMIT_TITLE="chore(sync): auto-update dependencies ({{ timestamp }})"
        COMMIT_DESC="Files dynamically updated by automated script: - $CHANGED_FILES"

        git commit --no-verify -m "$COMMIT_TITLE" -m "$COMMIT_DESC" > /dev/null
        echo -e "{{ c_green }}  ↳ 自动化变更已保存至本地 Git 树。{{ c_reset }}"
    else
        echo -e "{{ c_gray }}  ↳ 依赖和元数据无变更，跳过提交。{{ c_reset }}"
    fi

# [4/6] 深度清理系统与开发环境缓存
[group('0. 系统维护')]
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

# [5/6] 同步代码至远端仓库 (Pull & Push)
[group('0. 系统维护')]
sync-remote:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [5/6] 同步代码至远端仓库 (Pull & Push)...{{ c_reset }}"
    cd "{{ dotfiles }}"

    if ! git pull --rebase --autostash; then
        echo -e "{{ c_red }}  ↳ Pull 遇到严重冲突或网络错误！正在中止 Rebase 以保护当前工作区...{{ c_reset }}"
        git rebase --abort 2>/dev/null || true
        echo -e "{{ c_yellow }}  ↳ 放弃本次 Push，请手动检查冲突，但本地代码不受影响。{{ c_reset }}"
    else
        echo -e "{{ c_gray }}  ↳ Pull 完成，正在推送...{{ c_reset }}"
        if git push; then
            echo -e "{{ c_green }}  ↳ 远端同步成功。{{ c_reset }}"
        else
            echo -e "{{ c_red }}  ↳ 推送失败，请检查网络或远端写权限！{{ c_reset }}"
        fi
    fi

# [6/6] 还原用户先前的未完成状态
[group('0. 系统维护')]
restore:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "\n{{ c_blue }}▶ [6/6] 还原用户先前的未完成状态...{{ c_reset }}"
    cd "{{ dotfiles }}"

    STASH_REF=$(git stash list | grep "{{ stash_msg }}" | head -n 1 | cut -d: -f1 || true)

    if [ -n "$STASH_REF" ]; then
        echo -e "{{ c_yellow }}  ↳ 找到用户执行前的状态备份 ($STASH_REF)，正在精准还原...{{ c_reset }}"
        if git stash pop --index "$STASH_REF" > /dev/null; then
            echo -e "{{ c_green }}  ↳ 状态恢复完毕。{{ c_reset }}"
        else
            echo -e "{{ c_red }}  ↳ 恢复 Stash 时发现文件冲突！该 Stash 仍保留在栈中，请手动解决 (git status){{ c_reset }}"
        fi
    else
        echo -e "{{ c_gray }}  ↳ 初始状态无修改，无需恢复。{{ c_reset }}"
    fi

# =============================================================================
# 配置管理（调用 ./wots）
# =============================================================================

[group('1. 配置管理')]
create +args:
    @{{ wots }} create --win-user {{ WIN_USER }} {{ args }}

# [同步] 同步所有包 (CONFIRM=true 时同步前确认路径)
[group('1. 配置管理')]
sync *args:
    @{{ wots }} sync --win-user {{ WIN_USER }} {{ confirm_flag }} {{ args }}

# [同步] 按类型同步 (user/config/root/winuser/winroot)
[group('1. 配置管理')]
sync-type type *args:
    @{{ wots }} sync --win-user {{ WIN_USER }} --type {{ type }} {{ confirm_flag }} {{ args }}

# [同步] 按包名同步 (支持后缀: git.user → 自动识别 User)
[group('1. 配置管理')]
sync-app app *args:
    @{{ wots }} sync --win-user {{ WIN_USER }} --app {{ app }} {{ confirm_flag }} {{ args }}

# [同步] 干运行预览
[group('1. 配置管理')]
sync-dry *args:
    @{{ wots }} sync --win-user {{ WIN_USER }} --dry-run {{ args }}

# [同步] 同步 root 包 (sudo, 跳过确认)
[group('1. 配置管理')]
sync-root *args:
    @sudo {{ wots }} sync --win-user {{ WIN_USER }} --type root --bypass {{ args }}

[group('1. 配置管理')]
stats *args:
    @{{ wots }} stats --win-user {{ WIN_USER }} {{ args }}

[group('1. 配置管理')]
stats-json:
    @{{ wots }} stats --win-user {{ WIN_USER }} --json

[group('1. 配置管理')]
list *args:
    @{{ wots }} list --win-user {{ WIN_USER }} {{ args }}

[group('1. 配置管理')]
list-json:
    @{{ wots }} list --win-user {{ WIN_USER }} --json

[group('1. 配置管理')]
list-type type *args:
    @{{ wots }} list --win-user {{ WIN_USER }} --type {{ type }} {{ args }}

[group('1. 配置管理')]
diff *args:
    @{{ wots }} diff --win-user {{ WIN_USER }} {{ args }}

[group('1. 配置管理')]
diff-type type *args:
    @{{ wots }} diff --win-user {{ WIN_USER }} --type {{ type }} {{ args }}

[group('1. 配置管理')]
diff-app app *args:
    @{{ wots }} diff --win-user {{ WIN_USER }} --app {{ app }} {{ args }}
