# TODO: zoxide sync

### better alias
typeset -g baliases=()

balias() {
  alias -g $@
  args="$@"
  args=${args%%\=*}
  baliases+=(${args##* })
}

# ignored aliases
typeset -g ialiases=()

ialias() {
  alias $@
  args="$@"
  args=${args%%\=*}
  ialiases+=(${args##* })
}

function y() {
  local tmp="$(mktemp -t "yazi-cwd.XXXXXX")" cwd
  yazi "$@" --cwd-file="$tmp"
  if cwd="$(command cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
    builtin cd -- "$cwd"
  fi
  rm -f -- "$tmp"
}
ialias ff="fastfetch"
ialias eza="eza -I 'NTUSER.DAT*|ntuser.*'"
ialias l="eza --git -a --icons -l  "
ialias la="eza  -a --icons --no-git "
ialias ll="eza -a --total-size --git-repos --icons -l "
ialias lT="eza --tree -a -I '.git'"
ialias md="mkdir -p"
ialias rm="trash-put"
alias lt="lT -L "
alias v="vi"
alias R="source ~/.zshrc"
ialias e="explorer.exe ."
ialias ex="explorer.exe .;exit 0"
ialias p="pwsh.exe -NoProfile"
export win_home="/mnt/c/Users/zionpu"
balias 'c:'="/mnt/c/"
balias 'd:'="/mnt/d/"
balias 'e:'="/mnt/e/"
balias h="$win_home/"

### application options

export GO111MODULE=on

export VCPKG_ROOT=$HOME/.local/share/vcpkg
export CRYPTOGRAPHY_OPENSSL_NO_LEGACY=1
export LIBGL_ALWAYS_INDIRECT=1

export _ZL_MATCH_MODE=1
export _ZL_HYPHEN=1

### shell options
export BROWSER="/usr/bin/wslview"
export EDITOR="vi"
export TERMINFO=/usr/share/terminfo

### 历史记录相关配置
export HISTFILE=~/.zsh_history
export HISTSIZE=10000
export SAVEHIST=10000
setopt EXTENDED_HISTORY       # 为历史记录中的命令添加时间戳
setopt HIST_EXPIRE_DUPS_FIRST # 先删除旧的重复命令
setopt HIST_IGNORE_DUPS       # 忽略重复命令
setopt HIST_FIND_NO_DUPS      # 搜索时不显示重复命令
setopt HIST_IGNORE_ALL_DUPS   # 忽略所有重复命令
setopt HIST_SAVE_NO_DUPS      # 保存时不保存重复命令
setopt HIST_REDUCE_BLANKS     # 去除命令中的多余空白
setopt HIST_VERIFY            # 执行历史命令前先显示
setopt SHARE_HISTORY          # 所有终端共享历史记录
setopt INC_APPEND_HISTORY     # 命令执行后立即追加到历史记录文件
setopt HIST_IGNORE_SPACE      # 命令前有空格时不保存到历史记录
setopt HIST_NO_STORE          # history命令本身不被保存

setopt auto_cd
setopt auto_pushd
setopt pushd_ignore_dups
setopt pushdminus

setopt interactivecomments # 交互式注释
### for sync_directory_change
sync_directory_change() {
  pwd | tr -d '\n' >"$win_home/.workdir"
}

autoload -U add-zsh-hook
add-zsh-hook chpwd sync_directory_change
### arrows to search history
autoload -U up-line-or-beginning-search
autoload -U down-line-or-beginning-search
zle -N up-line-or-beginning-search
zle -N down-line-or-beginning-search
# In Defense of Maintaining Search History Despite the Absence of FZF-History-Search
bindkey "^[[A" up-line-or-beginning-search   # Up
bindkey "^[[B" down-line-or-beginning-search # Down

# ================================================
#  autoload zinit
# ================================================
if [[ ! -f $HOME/.local/share/zinit/zinit.git/zinit.zsh ]]; then
  print -P "%F{33} %F{220}Installing %F{33}ZDHARMA-CONTINUUM%F{220} Initiative Plugin Manager (%F{33}zdharma-continuum/zinit%F{220})…%f"
  command mkdir -p "$HOME/.local/share/zinit" && command chmod g-rwX "$HOME/.local/share/zinit"
  command git clone https://github.com/zdharma-continuum/zinit "$HOME/.local/share/zinit/zinit.git" &&
    print -P "%F{33} %F{34}Installation successful.%f%b" ||
    print -P "%F{160} The clone has failed.%f%b"
fi

fpath=(~/.config/zsh/completions/ $fpath)

source "$HOME/.local/share/zinit/zinit.git/zinit.zsh"

autoload -U compinit && compinit

# ================================================
# FZF + fzf-tab 现代极简配置（支持隐藏文件 & 高级配色版）
# ================================================

# ====================== 1. FZF 全局默认选项（极简透明风） ======================
# --border=none           : 移除全包围边框
# --preview-window        : 仅保留预览区左侧分隔线
# --pointer / --marker    : 恢复原生 > 和 *
# --color                 : 透明背景，高级玫瑰红游标/高亮，暗灰蓝分隔线
export FZF_DEFAULT_OPTS="
  --layout=reverse
  --height=60%
  --multi
  --info=inline-right
  --border=none
  --preview-window='right:55%:border-left:wrap'
  --pointer='>'
  --marker='*'
  --color='bg:-1,bg+:-1,fg:-1,fg+:#ffffff,hl:#e06c75,hl+:#e06c75,pointer:#e06c75,marker:#e06c75,prompt:#61afef,query:-1,border:#4b5263,separator:#4b5263'
"

# ====================== 2. 快捷键专用选项 ======================

# [Ctrl-T] 找文件快捷键：在当前目录查找文件并输出路径
export FZF_CTRL_T_OPTS="
  --walker-skip .git,node_modules,target,dist
  --bind 'ctrl-/:change-preview-window(down|hidden|)'
  --preview 'bat --style=numbers --color=always --line-range :500 {} 2>/dev/null || ls -l {}'
"

# [Ctrl-R] 历史命令快捷键：搜索历史命令
export FZF_CTRL_R_OPTS="
  --bind 'ctrl-y:execute-silent(echo -n {2..} | win32yank.exe -i)+abort'
  --color header:italic
  --header 'Press CTRL-Y to copy command to clipboard'
"

# [Alt-C] 找目录快捷键：查找子目录并自动 cd 进去 (加入 -a 以显示树状隐藏文件)
export FZF_ALT_C_OPTS="
  --walker-skip .git,node_modules,target
  --preview 'eza --tree -a --level=2 --color=always {} 2>/dev/null || ls -la {}'
"

# ====================== 3. Zsh Completion 通用设置 ======================
zstyle ':completion:*' menu no
zstyle ':completion:*:descriptions' format '%F{yellow}[%d]%f'
zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}

zstyle ':completion:*' matcher-list 'm:{a-zA-Z}={A-Za-z}' 'r:|[._-]=* r:|=*' 'l:|=* r:|=*'
setopt GLOB_DOTS

# ====================== 4. fzf-tab 行为与快捷键优化 ======================
zstyle ':fzf-tab:*' use-fzf-default-opts yes
zstyle ':fzf-tab:*' fzf-command fzf

# 快捷键：Tab (在目录间连续进入) | < > (切换分组)
zstyle ':fzf-tab:*' continuous-trigger 'tab'
zstyle ':fzf-tab:*' switch-group '<' '>'

# 快捷键：Ctrl-/ (切换预览布局) | Ctrl-Space (多选标记)
zstyle ':fzf-tab:*' fzf-flags \
  --tiebreak=length,begin,index \
  --bind 'ctrl-space:toggle' \
  --bind 'ctrl-/:change-preview-window(down|hidden|)'

# ====================== 5. 极致优化的 Preview 智能预览配置 ======================
# 核心修复：给 eza 和 ls 加上了 -a / -la 参数，确保完整显示隐藏的 dotfiles
zstyle ':fzf-tab:complete:*:*' fzf-preview '
  if [[ -e "$realpath" ]]; then
    if [[ -d "$realpath" ]]; then
      eza -1a --color=always --icons=always "$realpath" 2>/dev/null || ls -la "$realpath"
    elif [[ -f "$realpath" ]]; then
      bat --style=numbers --color=always --line-range :500 "$realpath" 2>/dev/null || 
      pistol "$realpath" 2>/dev/null || 
      cat "$realpath"
    fi
  fi
'

zstyle ':fzf-tab:complete:git-(add|diff|restore):*' fzf-preview 'git diff "$word" | delta 2>/dev/null || git diff "$word"'
zstyle ':fzf-tab:complete:git-log:*' fzf-preview 'git log --color=always --oneline "$word"'

zstyle ':fzf-tab:complete:kill:*' fzf-preview 'ps --pid="$word" -o cmd --no-headers'
zstyle ':fzf-tab:complete:man:*' fzf-preview 'man "$word" | bat -plman --color=always 2>/dev/null || man "$word"'
# ================================================
#  plugins
# ================================================
zinit wait lucid for \
  jeffreytse/zsh-vi-mode

zinit wait'!0' lucid is-snippet nocd for \
  ~/.config/zsh/starship.zsh \
  ~/.config/zsh/fzf.zsh \
  atinit'export IM_SELECT_EXE_PATH="/mnt/c/Users/zionpu/im-select.exe"' \
  https://raw.githubusercontent.com/pu-007/im-select-ahk.nvim/refs/heads/main/zsh/im-select-vimmode.zsh

zinit wait lucid is-snippet for \
  atload"ialias z='__zoxide_z'; ialias zi='__zoxide_zi'" \
  ~/.config/zsh/zoxide.zsh \
  ~/.config/zsh/commands.zsh \
  ~/.config/zsh/powershell.zsh

function expand-alias-space() {
  [[ $LBUFFER =~ "\<(${(j:|:)baliases})\$" ]]
  insertBlank=$?
  if [[ ! $LBUFFER =~ "\<(${(j:|:)ialiases})\$" ]]; then
    zle _expand_alias
  fi
  zle self-insert
  if [[ "$insertBlank" = "0" ]]; then
    zle backward-delete-char
  fi
}

function zvm_after_init() {
  zle -N expand-alias-space
  bindkey " " expand-alias-space
  bindkey -M isearch " " magic-space
  bindkey -M vicmd 'H' beginning-of-line
  bindkey -M vicmd 'L' end-of-line
  bindkey -M vicmd 'k' up-line-or-beginning-search
  bindkey -M vicmd 'j' down-line-or-beginning-search
  bindkey "^[[A" up-line-or-beginning-search   # Up
  bindkey "^[[B" down-line-or-beginning-search # Down
  zle -N fzf-history-widget
  bindkey -M vicmd '^R' fzf-history-widget
  bindkey -M viins '^R' fzf-history-widget

  zinit wait lucid for \
    oldkingOK/pinyin-completion \
    OMZP::cp \
    OMZP::colored-man-pages \
    OMZP::command-not-found \
    OMZP::copypath \
    OMZL::functions.zsh OMZL::clipboard.zsh \
    OMZL::git.zsh \
    OMZL::termsupport.zsh \
    OMZP::git \
    atinit"ZINIT[COMPINIT_OPTS]=-C; zicompinit; zicdreplay" \
    zdharma-continuum/fast-syntax-highlighting \
    atload"compdef _adb adb.exe" \
    zsh-users/zsh-completions \
    Aloxaf/fzf-tab \
    atinit'export ZSH_AI_PROVIDER="gemini"' \
    matheusml/zsh-ai
}

export PATH="$PATH:$HOME/.local/bin"
export PATH="$PATH:$HOME/.moon/bin:"
export PATH="$PATH:$HOME/go/bin"
export DOTFILE_STORE="$HOME/dotfiles"
export CUDA_HOME="/opt/cuda/"
export HF_ENDPOINT="https://hf-mirror.com"

wots() { $DOTFILE_STORE/wots "$@"; }

alias cls="clear"
# use system python for yay instaed of miniconda's one
ialias yay='PATH="/usr/bin:$PATH" yay'
