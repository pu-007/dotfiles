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

ialias eza="eza -I 'NTUSER.DAT*|ntuser.*'"
ialias l="eza --git -a --icons -l  "
ialias la="eza  -a --icons --no-git "
ialias ll="eza -a --total-size --git-repos --icons -l "
ialias lT="eza --tree -a -I '.git'"
alias lt="lT -L "
alias z=j
alias v="vi"
# ialias rv="nvim +'FzfLua oldfiles'"
ialias rv="cat ~/.config/nvim/recent_files.txt | fzf | xargs nvim"
ialias e="explorer.exe ."
ialias ex="explorer.exe .;exit 0"
ialias p="powershell.exe"
ialias c="cmd.exe"
ialias a="gptme"
balias h="$home/"

### application options
export CRYPTOGRAPHY_OPENSSL_NO_LEGACY=1
export home="/mnt/c/Users/zion"
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
### for sync_directory_change
sync_directory_change() {
  pwd | tr -d '\n' > "$home/.workdir"
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

### autoload zinit
if [[ ! -f $HOME/.local/share/zinit/zinit.git/zinit.zsh ]]; then
    print -P "%F{33} %F{220}Installing %F{33}ZDHARMA-CONTINUUM%F{220} Initiative Plugin Manager (%F{33}zdharma-continuum/zinit%F{220})…%f"
    command mkdir -p "$HOME/.local/share/zinit" && command chmod g-rwX "$HOME/.local/share/zinit"
    command git clone https://github.com/zdharma-continuum/zinit "$HOME/.local/share/zinit/zinit.git" && \
        print -P "%F{33} %F{34}Installation successful.%f%b" || \
        print -P "%F{160} The clone has failed.%f%b"
fi
source "$HOME/.local/share/zinit/zinit.git/zinit.zsh"
autoload -Uz _zinit
autoload -U compinit && compinit
(( ${+_comps} )) && _comps[zinit]=_zinit
export fpath=($fpath ~/.config/zsh/completions/)
### plugins
zinit wait lucid for \
  jeffreytse/zsh-vi-mode

zinit wait lucid is-snippet for \
  ~/.config/zsh/zoxide.zsh \
  ~/.config/zsh/commands.zsh \
  ~/.config/zsh/conda.zsh

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
    ~/.config/zsh/powershell.zsh \
    ~/.config/zsh/fzf.zsh

  # for command-not-found:
  # sudo pkgfile --update
  zinit wait lucid for \
    Aloxaf/fzf-tab \
      reset-prompt nocd atload"zle .reset-prompt" as"command" from"gh-r" atclone"./starship init zsh > init.zsh; ./starship completions zsh > _starship" atpull"%atclone" src"init.zsh" \
    starship/starship \
    OMZP::sudo \
    OMZP::extract \
    OMZP::cp \
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
}
