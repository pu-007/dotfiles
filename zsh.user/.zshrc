# vim: foldmethod=marker foldlevel=0
# ================================================
#  My ZSH Configuration
# ================================================

# {{{ 1. Environment Variables & Paths
export win_home="/mnt/c/Users/zionpu"
export GO111MODULE=on
export VCPKG_ROOT="$HOME/.local/share/vcpkg"
export CRYPTOGRAPHY_OPENSSL_NO_LEGACY=1
export LIBGL_ALWAYS_INDIRECT=1

export BROWSER="/usr/bin/wslview"
export EDITOR="vi"
export TERMINFO="/usr/share/terminfo"
export DOTFILE_STORE="$HOME/dotfiles"
export CUDA_HOME="/opt/cuda/"
export HF_ENDPOINT="https://hf-mirror.com"

# 合并 PATH，修复结尾多余冒号的问题
export PATH="$HOME/.local/bin:$HOME/.moon/bin:$HOME/go/bin:/usr/bin:$PATH"
# }}}

# {{{ 2. Shell Options & History
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
setopt GLOB_DOTS           # 补全时包含隐藏文件
# }}}

# {{{ 3. Functions & Aliases
typeset -g baliases=()
balias() {
  alias -g "$@"
  local args="$@"
  args=${args%%\=*}
  baliases+=(${args##* })
}

typeset -g ialiases=()
ialias() {
  alias "$@"
  local args="$@"
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

# 目录同步 Hook
sync_directory_change() {
  pwd | tr -d '\n' >"$win_home/.workdir"
}
autoload -U add-zsh-hook
add-zsh-hook chpwd sync_directory_change

# 普通命令别名
alias cls="clear"
alias v="vi"
alias R="source ~/.zshrc"
alias lt="lT -L "

ialias ff="fastfetch"
ialias eza="eza -I 'NTUSER.DAT*|ntuser.*'"
ialias l="eza --git -a --icons -l"
ialias la="eza -a --icons --no-git"
ialias ll="eza -a --total-size --git-repos --icons -l"
ialias lT="eza --tree -a -I '.git'"
ialias md="mkdir -p"
ialias rm="trash-put"
ialias e="explorer.exe ."
ialias ex="explorer.exe .;exit 0"
ialias p="pwsh.exe -NoProfile"
ialias yay='PATH="/usr/bin:$PATH" yay'

# 全局别名
balias 'c:'="/mnt/c/"
balias 'd:'="/mnt/d/"
balias 'e:'="/mnt/e/"
balias h="$win_home/"

wots() { "$DOTFILE_STORE/wots" "$@"; }
# }}}

# {{{ 4. Zinit Initialization & Completion
if [[ ! -f $HOME/.local/share/zinit/zinit.git/zinit.zsh ]]; then
  print -P "%F{33} %F{220}Installing %F{33}ZDHARMA-CONTINUUM%F{220} Initiative Plugin Manager...%f"
  command mkdir -p "$HOME/.local/share/zinit" && command chmod g-rwX "$HOME/.local/share/zinit"
  command git clone https://github.com/zdharma-continuum/zinit "$HOME/.local/share/zinit/zinit.git" &&
    print -P "%F{33} %F{34}Installation successful.%f%b" || print -P "%F{160} The clone has failed.%f%b"
fi

fpath=(~/.config/zsh/completions/ $fpath)
source "$HOME/.local/share/zinit/zinit.git/zinit.zsh"

# 基础补全初始化
autoload -U compinit && compinit

# Zsh Completion 通用设置
zstyle ':completion:*' menu no
zstyle ':completion:*:descriptions' format '%F{yellow}[%d]%f'
zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}
zstyle ':completion:*' matcher-list 'm:{a-zA-Z}={A-Za-z}' 'r:|[._-]=* r:|=*' 'l:|=* r:|=*'
# }}}

# {{{ 5. FZF & fzf-tab Configuration
# FZF 全局默认选项
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
export FZF_CTRL_T_OPTS="
  --walker-skip .git,node_modules,target,dist
  --bind 'ctrl-/:change-preview-window(down|hidden|)'
  --preview 'bat --style=numbers --color=always --line-range :500 {} 2>/dev/null || ls -l {}'
"
export FZF_CTRL_R_OPTS="
  --bind 'ctrl-y:execute-silent(echo -n {2..} | win32yank.exe -i)+abort'
  --color header:italic
  --header 'Press CTRL-Y to copy command to clipboard'
"
export FZF_ALT_C_OPTS="
  --walker-skip .git,node_modules,target
  --preview 'eza --tree -a --level=2 --color=always {} 2>/dev/null || ls -la {}'
"

# fzf-tab 行为与快捷键优化
zstyle ':fzf-tab:*' use-fzf-default-opts yes
zstyle ':fzf-tab:*' fzf-command fzf
zstyle ':fzf-tab:*' continuous-trigger 'tab'
zstyle ':fzf-tab:*' switch-group '<' '>'
zstyle ':fzf-tab:*' fzf-flags \
  --tiebreak=length,begin,index \
  --bind 'ctrl-space:toggle' \
  --bind 'ctrl-/:change-preview-window(down|hidden|)'

# Preview 智能预览配置
zstyle ':fzf-tab:complete:*:*' fzf-preview '
  if [[ -e "$realpath" ]]; then
    if [[ -d "$realpath" ]]; then
      eza -1a --color=always --icons=always "$realpath" 2>/dev/null || ls -la "$realpath"
    elif [[ -f "$realpath" ]]; then
      bat --style=numbers --color=always --line-range :500 "$realpath" 2>/dev/null || pistol "$realpath" 2>/dev/null || cat "$realpath"
    fi
  fi
'
zstyle ':fzf-tab:complete:git-(add|diff|restore):*' fzf-preview 'git diff "$word" | delta 2>/dev/null || git diff "$word"'
zstyle ':fzf-tab:complete:git-log:*' fzf-preview 'git log --color=always --oneline "$word"'
zstyle ':fzf-tab:complete:kill:*' fzf-preview 'ps --pid="$word" -o cmd --no-headers'
zstyle ':fzf-tab:complete:man:*' fzf-preview 'man "$word" | bat -plman --color=always 2>/dev/null || man "$word"'
# }}}

# {{{ 6. Plugins Setup (Zinit)
# Snippets & Configs
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

# Plugins (注意：已将这里从 zvm_after_init 中抽出)
zinit wait lucid for \
  oldkingOK/pinyin-completion \
  OMZP::cp \
  OMZP::colored-man-pages \
  OMZP::command-not-found \
  OMZP::copypath \
  OMZL::functions.zsh \
  OMZL::clipboard.zsh \
  OMZL::git.zsh \
  OMZL::termsupport.zsh \
  OMZP::git \
  zsh-users/zsh-completions \
  Aloxaf/fzf-tab \
  atinit'export ZSH_AI_PROVIDER="gemini"' \
  matheusml/zsh-ai \
  jeffreytse/zsh-vi-mode \
  atinit"ZINIT[COMPINIT_OPTS]=-C; zicompinit; zicdreplay" \
  atload"compdef _adb adb.exe; compdef _files trash-put" \
  zdharma-continuum/fast-syntax-highlighting
# }}}

# {{{ 7. Custom Keybindings & Hooks
autoload -U up-line-or-beginning-search
autoload -U down-line-or-beginning-search

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

# ZSH-VI-MODE Hook 函数 (只放置动态触发的键盘绑定逻辑)
function zvm_after_init() {
  zle -N expand-alias-space
  zle -N up-line-or-beginning-search
  zle -N down-line-or-beginning-search
  zle -N fzf-history-widget

  # 全局以及插入模式映射
  bindkey " " expand-alias-space
  bindkey -M isearch " " magic-space
  bindkey "^[[A" up-line-or-beginning-search   # Up
  bindkey "^[[B" down-line-or-beginning-search # Down
  bindkey -M viins '^R' fzf-history-widget

  # Vi Command 模式映射
  bindkey -M vicmd 'H' beginning-of-line
  bindkey -M vicmd 'L' end-of-line
  bindkey -M vicmd 'k' up-line-or-beginning-search
  bindkey -M vicmd 'j' down-line-or-beginning-search
  bindkey -M vicmd '^R' fzf-history-widget
}
# }}}
