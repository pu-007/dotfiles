# =============================================================================
# WOTS & Dotfiles 现代管理脚本 (Powered by Just & Pixi)
# =============================================================================

# 全局变量
dotfiles   := env_var("HOME") + "/dotfiles"
timestamp  := `date +'%Y-%m-%d %H:%M:%S'`
stash_msg  := "WOTS_AUTO_STASH"

# 🎨 UI 色彩配置
c_reset  := '\033[0m'
c_green  := '\033[1;32m'
c_blue   := '\033[1;34m'
c_yellow := '\033[1;33m'
c_red    := '\033[1;31m'
c_gray   := '\033[1;90m'

# 默认列出所有可用命令（加上 --unsorted 取消字母乱序，按本文件定义顺序输出）
default:
    @just --list --unsorted

# =============================================================================
# 🛠️ 1. 系统维护 (System Maintenance)
# =============================================================================

# [一键执行] 保护 -> 更新 -> 备份 -> 清理 -> 同步远程 -> 恢复本地
[group('1. 系统维护 (System Maintenance)')]
refresh: _protect _update _backup _cleanup _sync_remote _restore
    @echo -e "{{c_green}}🎉 === Refresh Complete at $(date +'%H:%M:%S') ==={{c_reset}}"

# =============================================================================
# ⚙️ 2. 配置管理 (Wots CLI)
# =============================================================================

# [核心] 运行 wots 基础命令。用法: just wots [参数]
[group('2. 配置管理 (Wots CLI)')]
wots *args:
    @pixi run wots {{args}}

# 创建新配置包。用法: just create ~/.config/nvim -a nvim -t config
[group('2. 配置管理 (Wots CLI)')]
create +args:
    @pixi run wots create {{args}}

# 同步所有配置 (覆盖 Windows)
[group('2. 配置管理 (Wots CLI)')]
sync:
    @pixi run wots sync

# 同步特定类型。可选: user, root, winuser。用法: just sync-type user
[group('2. 配置管理 (Wots CLI)')]
sync-type type:
    @{{ if type == "root" { "sudo " } else { "" } }}pixi run wots sync --type {{type}}

# 预览同步差异 (Dry run)
[group('2. 配置管理 (Wots CLI)')]
sync-dry:
    @pixi run wots sync --dry-run

# 查看仓库与包统计
[group('2. 配置管理 (Wots CLI)')]
stats:
    @pixi run wots stats

# 查看所有包列表
[group('2. 配置管理 (Wots CLI)')]
list:
    @pixi run wots list

# 查看需要同步的文件差异
[group('2. 配置管理 (Wots CLI)')]
diff:
    @pixi run wots list --unsynced


# =============================================================================
# 📦 私有子任务 (Private Sub-tasks) - 带有 "_" 前缀，列表中自动隐身
# =============================================================================

# 1. 保护工作区
_protect:
    #!/usr/bin/env bash
    echo -e "{{c_blue}}=== [1/6] 检查工作区状态 ==={{c_reset}}"
    cd "{{dotfiles}}" || exit 1
    
    if ! git diff --quiet || ! git diff --cached --quiet || [ -n "$(git ls-files --others --exclude-standard)" ]; then
        echo -e "{{c_yellow}}>> 发现未提交的 dotfiles 更改，正在安全 Stash...{{c_reset}}"
        git stash push --include-untracked -m "{{stash_msg}}"
    else
        echo -e "{{c_green}}>> 工作区干净，无需 Stash。{{c_reset}}"
    fi

# 2. 更新依赖
_update:
    #!/usr/bin/env bash
    echo -e "{{c_blue}}=== [2/6] 更新系统包和开发环境 ==={{c_reset}}"
    [ -f "$HOME/.zinit/bin/zinit.zsh" ] && zsh -ic "zinit update --all" || true
    command -v yay &>/dev/null && yay -Syu --noconfirm || true
    sudo pkgfile --update 2>/dev/null || true
    command -v nvim &>/dev/null && nvim --headless "+Lazy! sync" +qa || true
    sudo npm install -g @google/gemini-cli || true
    command -v npm &>/dev/null && sudo npm install -g aicommit2 || true
    command -v komorebic.exe &>/dev/null && komorebic.exe fetch-app-specific-configuration || true
    [ -d "/mnt/c/Users/zion/AppData/Roaming/Rime" ] && git -C "/mnt/c/Users/zion/AppData/Roaming/Rime" pull || true

# 3. 备份元数据
_backup:
    #!/usr/bin/env bash
    echo -e "{{c_blue}}=== [3/6] 导出包列表元数据 ==={{c_reset}}"
    mkdir -p "{{dotfiles}}/packages.meta"
    command -v pacman &>/dev/null && pacman -Qqe > "{{dotfiles}}/packages.meta/pacman.txt" || true
    
    cd "{{dotfiles}}" || exit 1
    git add packages.meta/
    if ! git diff --cached --quiet; then
        echo -e "{{c_yellow}}>> 提交包列表更新...{{c_reset}}"
        git commit -m "chore(pkg): auto-update package list ({{timestamp}})"
    else
        echo -e "{{c_gray}}>> 包列表无变化，跳过提交。{{c_reset}}"
    fi

# 4. 深度清理
_cleanup:
    #!/usr/bin/env bash
    echo -e "{{c_blue}}=== [4/6] 深度清理系统缓存 ==={{c_reset}}"
    [ -d "$HOME/.cache/uv" ] && uv cache clean || true
    [ -d "$HOME/.cache/pip" ] && pip cache purge || true
    rm -rf "$HOME/.cache/huggingface/hub" "$HOME/.cache/huggingface/download" 2>/dev/null || true
    command -v go &>/dev/null && go clean -cache -modcache || true
    command -v npm &>/dev/null && npm cache clean --force || true
    command -v yay &>/dev/null && yay -Sc --noconfirm || true
    command -v scoop &>/dev/null && scoop cleanup -a -g -k || true
    sudo trash-empty -f --all-users 2>/dev/null || trash-empty -f || true
    echo -e "{{c_yellow}}💡 (提示) Docker 清理建议手动检查: ssh root@192.168.100.1 docker system prune -a -f{{c_reset}}"

# 5. 安全推送 (在恢复本地状态之前执行，保持工作区干净以允许 Rebase)
_sync_remote:
    #!/usr/bin/env bash
    echo -e "{{c_blue}}=== [5/6] 同步远程仓库 (Pull & Push) ==={{c_reset}}"
    cd "{{dotfiles}}" || exit 1
    git pull --rebase || echo -e "{{c_yellow}}⚠️ Pull 遇到问题，尝试继续 Push...{{c_reset}}"
    git push || echo -e "{{c_red}}⚠️ 推送失败，请检查网络或远端冲突！{{c_reset}}"

# 6. 恢复工作区 (最后一步，把之前的代码还给你)
_restore:
    #!/usr/bin/env bash
    echo -e "{{c_blue}}=== [6/6] 恢复本地状态 ==={{c_reset}}"
    cd "{{dotfiles}}" || exit 1
    
    # 精确匹配刚刚创建的 Stash 的 ID
    STASH_REF=$(git stash list | grep "{{stash_msg}}" | head -n 1 | cut -d: -f1 || true)
    
    if [ -n "$STASH_REF" ]; then
        echo -e "{{c_yellow}}>> 找到自动备份 ($STASH_REF)，正在恢复...{{c_reset}}"
        git stash pop "$STASH_REF" || echo -e "{{c_red}}⚠️ 恢复 Stash 时发生冲突，请手动处理！(git status){{c_reset}}"
    else
        echo -e "{{c_gray}}>> 没有需要恢复的拦截状态。{{c_reset}}"
    fi
