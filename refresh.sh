#!/bin/bash
# refresh - Securely update and clean the system while protecting dotfiles state

DOTFILES_DIR="$HOME/dotfiles"
TIMESTAMP=$(date +'%Y-%m-%d %H:%M:%S')
HAS_STASH=false

echo "--- Refreshing System: $TIMESTAMP ---"

# --- [ SAFETY FUNCTIONS ] ---

# Protect local changes in dotfiles
git_protect() {
    if [ -d "$DOTFILES_DIR/.git" ]; then
        if [[ -n $(git -C "$DOTFILES_DIR" status --porcelain) ]]; then
            echo ">> Local changes detected in dotfiles. Stashing..."
            git -C "$DOTFILES_DIR" stash push -m "Auto-stash before refresh at $TIMESTAMP"
            HAS_STASH=true
        fi
    fi
}

# Restore local changes
git_restore() {
    if [ "$HAS_STASH" = true ]; then
        echo ">> Restoring stashed changes..."
        git -C "$DOTFILES_DIR" stash pop
    fi
}

# --- [ TASKS ] ---

do_update() {
    echo ">> Updating packages..."
    
    # Zsh plugins
    [ -f "$HOME/.zinit/bin/zinit.zsh" ] && zsh -ic "zinit update --all"

    # Arch Linux
    if command -v yay &> /dev/null; then
        yay -Syu --noconfirm
    fi

    sudo pkgfile --update 2>/dev/null

    # Runtimes
    if command -v conda &> /dev/null; then conda update --all -y; fi
    #sudo npm -g update -y
    # sudo npm install -g @google/gemini-cli
    if command -v npm &> /dev/null; then sudo npm install -g aicommit2; fi

    # WSL / Windows Sync
    if command -v komorebic.exe &> /dev/null; then komorebic.exe fetch-app-specific-configuration; fi
    if [ -d "/mnt/c/Users/zion/AppData/Roaming/Rime" ]; then
        git -C "/mnt/c/Users/zion/AppData/Roaming/Rime" pull
    fi
}

do_backup() {
    echo ">> Exporting package metadata..."
    mkdir -p "$DOTFILES_DIR/packages.meta"
    
    if command -v pacman &> /dev/null; then
        pacman -Qqe > "$DOTFILES_DIR/packages.meta/pacman.txt"
    fi

    if [ -d "$DOTFILES_DIR/.git" ]; then
        git -C "$DOTFILES_DIR" add "$DOTFILES_DIR/packages.meta/"
        # Only commit if there are changes in the meta folder
        if ! git -C "$DOTFILES_DIR" diff --cached --quiet; then
            git -C "$DOTFILES_DIR" commit -m "chore(pkg): auto-update package list ($TIMESTAMP)"
        else
            echo ">> No package changes to commit."
        fi
    fi
}

do_cleanup() {
    echo ">> Deep cleaning caches..."
    
    # Dev Caches
    [ -d "$HOME/.cache/uv" ] && uv cache clean
    [ -d "$HOME/.cache/pip" ] && pip cache purge
    if [ -d "$HOME/.cache/huggingface/hub" ]; then
        rm -rf "$HOME/.cache/huggingface/hub" "$HOME/.cache/huggingface/download"
    fi
    
    command -v go &> /dev/null && go clean -cache -modcache
    command -v npm &> /dev/null && npm cache clean --force
    command -v yay &> /dev/null && yay -Sc --noconfirm
    command -v scoop &> /dev/null && scoop cleanup -a -g -k

    # System Garbage
    # TODO safe docker cleanup job
    #docker system prune -a -f
    # ssh root@192.168.100.1 docker system prune -a -f
    #  docker volume prune

    sudo trash-empty -f --all-users 2>/dev/null || trash-empty -f
}

# --- [ EXECUTION ] ---

git_protect
do_update
do_backup
do_cleanup
git add .
git commit -m "chore(refresh.sh): update, backup and cleanup"
git_restore

echo "--- Refresh Complete at $(date +'%H:%M:%S') ---"
