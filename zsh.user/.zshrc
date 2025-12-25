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
ialias i="nvim +'FzfLua oldfiles'"
ialias e="explorer.exe ."
ialias ex="explorer.exe .;exit 0"
ialias p="pwsh.exe"
export win_home="/mnt/c/Users/zion"
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

# 补全结果中显示隐藏文件
setopt globdots

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
  pwd | tr -d '\n' > "$win_home/.workdir"
}

autoload -U add-zsh-hook
add-zsh-hook chpwd sync_directory_change
### arrows to search history
autoload -U up-line-or-beginning-search
autoload -U down-line-or-beginning-search
zle -N up-line-or-beginning-search
zle -N down-line-or-beginning-search
# In Defense of Maintaining Search History Despite the Absence of FZF-History-Search
bindkey "^[[A" up-line-or-beginning-search # Up
bindkey "^[[B" down-line-or-beginning-search # Down
# FZF Config and fzf-tab
export FZF_DEFAULT_OPTS="
--layout=reverse -m
--style full
--preview='pistol {}'
--highlight-line
--color='
  fg:#ebdbb2 fg+:#8ec07c
  bg: bg+:#282828 alt-bg:
  hl:#689d6a hl+:#8ec07c
  pointer:#8ec07c marker:#689d6a
  header:#689d6a
  spinner:#689d6a info:#8ec07c
  prompt:#8ec07c query:#ebdbb2
  border:#928374
'"
# CTRL-T - Paste the selected files and directories onto the command-line
# Preview file content using bat (https://github.com/sharkdp/bat)
export FZF_CTRL_T_OPTS="
  --walker-skip .git,node_modules,target
  --bind 'ctrl-/:change-preview-window(down|hidden|)'"
# :CTRL-R - Paste the selected command from history onto the command-line
# CTRL-Y to copy the command into clipboard using pbcopy
export FZF_CTRL_R_OPTS="
  --bind 'ctrl-y:execute-silent(echo -n {2..} | win32yank.exe -i)+abort'
  --color header:italic
  --header 'Press CTRL-Y to copy command into clipboard'"
# ALT-C - cd into the selected directory
# Print tree structure in the preview window
export FZF_ALT_C_OPTS="
  --walker-skip .git,node_modules,target"
# disable sort when completing `git checkout`
zstyle ':completion:*:git-checkout:*' sort false
# set descriptions format to enable group support
# NOTE: don't use escape sequences (like '%F{red}%d%f') here, fzf-tab will ignore them
zstyle ':completion:*:descriptions' format '[%d]'
# set list-colors to enable filename colorizing
zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}
# force zsh not to show completion menu, which allows fzf-tab to capture the unambiguous prefix
zstyle ':completion:*' menu no
# custom fzf flags
# NOTE: fzf-tab does not follow FZF_DEFAULT_OPTS by default
zstyle ':fzf-tab:*' fzf-flags --bind=tab:accept
# To make fzf-tab follow FZF_DEFAULT_OPTS.
# switch group using `<` and `>`
zstyle ':fzf-tab:*' switch-group '<' '>'
zstyle ':fzf-tab:complete:cd:*' fzf-preview 'eza -1 --color=always $realpath'
zstyle ':fzf-tab:complete:z:*' fzf-preview 'eza -1 --color=always $realpath'
zstyle ':fzf-tab:complete:j:*' fzf-preview 'eza -1 --color=always $realpath'


### autoload zinit
if [[ ! -f $HOME/.local/share/zinit/zinit.git/zinit.zsh ]]; then
    print -P "%F{33} %F{220}Installing %F{33}ZDHARMA-CONTINUUM%F{220} Initiative Plugin Manager (%F{33}zdharma-continuum/zinit%F{220})…%f"
    command mkdir -p "$HOME/.local/share/zinit" && command chmod g-rwX "$HOME/.local/share/zinit"
    command git clone https://github.com/zdharma-continuum/zinit "$HOME/.local/share/zinit/zinit.git" && \
        print -P "%F{33} %F{34}Installation successful.%f%b" || \
        print -P "%F{160} The clone has failed.%f%b"
fi
export fpath=($fpath ~/.config/zsh/completions/)
source "$HOME/.local/share/zinit/zinit.git/zinit.zsh"
autoload -U compinit && compinit
### plugins
zinit wait lucid for \
  jeffreytse/zsh-vi-mode \

zinit wait'!0' lucid is-snippet nocd for \
  ~/.config/zsh/starship.zsh

zinit wait lucid is-snippet for \
    atload"ialias z='__zoxide_z'; ialias zi='__zoxide_zi'" \
  ~/.config/zsh/zoxide.zsh \
  ~/.config/zsh/commands.zsh \
  ~/.config/zsh/powershell.zsh \
  ~/.config/zsh/xcmd.zsh \
  ~/.x-cmd.root/X

function expand-alias-space() {
  [[ $LBUFFER =~ "\<(${(j:|:)baliases})\$" ]]; insertBlank=$?
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
  bindkey "^[[A" up-line-or-beginning-search # Up
  bindkey "^[[B" down-line-or-beginning-search # Down

  zinit wait lucid is-snippet for \
    ~/.config/zsh/fzf.zsh \
    ~/.config/zsh/conda.zsh

  zinit wait lucid for \
    oldkingOK/pinyin-completion \
    OMZP::cp \
      atinit"export ZSH_CODEX_PREEXECUTE_COMMENT='true'" \
      atload"bindkey '^O' create_completion"\
    pu-007/zsh_codex \
    OMZP::colored-man-pages \
    OMZP::command-not-found \
    OMZP::copypath \
    OMZL::functions.zsh\
    OMZL::clipboard.zsh \
    OMZL::git.zsh \
    OMZL::termsupport.zsh \
    OMZP::git \
      atinit"ZINIT[COMPINIT_OPTS]=-C; zicompinit; zicdreplay" \
    zdharma-continuum/fast-syntax-highlighting \
      atload"compdef _adb adb.exe" \
    zsh-users/zsh-completions \
    Aloxaf/fzf-tab \
}

export PATH="$PATH:$HOME/.local/bin"
export PATH="$PATH:$HOME/.moon/bin:"
export PATH="$PATH:$HOME/go/bin"
export DOTFILE_STORE="$HOME/dotfiles"
export CUDA_HOME="/opt/cuda/"
export HF_ENDPOINT="https://hf-mirror.com"

# use gemini-balance in docker as backend
export GEMINI_MODEL="gemini-3-flash-preview"
export GOOGLE_GEMINI_BASE_URL="http://192.168.100.1:8000"
export GEMINI_API_KEY="sk-123456"

