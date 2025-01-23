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

# 设置历史记录保存的文件路径
export HISTFILE=~/.zsh_history

# 设置历史记录保存的条目数
export HISTSIZE=10000
export SAVEHIST=10000

# 历史记录相关配置
setopt globdots

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
### jump dir
function j() {
    if [[ "$argv[1]" == "-"* ]]; then
        _zlua "$@"
    else
        cd "$@" 2> /dev/null || _zlua "$@"
    fi
}

### for sync_directory_change
sync_directory_change() {
  pwd | tr -d '\n' > "$home/.workdir"
}

autoload -U add-zsh-hook
add-zsh-hook chpwd sync_directory_change

### zsh plugins
# blank aliases
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

expand-alias-space() {
  [[ $LBUFFER =~ "\<(${(j:|:)baliases})\$" ]]; insertBlank=$?
  if [[ ! $LBUFFER =~ "\<(${(j:|:)ialiases})\$" ]]; then
    zle _expand_alias
  fi
  zle self-insert
  if [[ "$insertBlank" = "0" ]]; then
    zle backward-delete-char
  fi
}
zle -N expand-alias-space

bindkey " " expand-alias-space
bindkey -M isearch " " magic-space

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
(( ${+_comps} )) && _comps[zinit]=_zinit
export fpath=($fpath ~/.config/zsh/completions/)
autoload -U compinit && compinit
# for command-not-found:
# sudo pkgfile --update
### plugins
zinit wait lucid for \
      OMZL::functions.zsh\
	    OMZL::clipboard.zsh \
      OMZP::extract \
      OMZP::cp \
      OMZL::git.zsh \
      OMZP::archlinux \
      OMZP::colored-man-pages \
      OMZP::command-not-found \
      OMZP::copypath \
      OMZL::termsupport.zsh \
      OMZP::git \
        atinit"ZINIT[COMPINIT_OPTS]=-C; zicompinit; zicdreplay" \
      zdharma-continuum/fast-syntax-highlighting \
      zsh-users/zsh-completions \
      OMZP::sudo \
      jeffreytse/zsh-vi-mode \
      skywind3000/z.lua

function zvm_after_init() {
  source ~/.config/zsh/fzf.zsh
}
zinit wait lucid is-snippet for \
  ~/.config/zsh/commands.zsh

zinit wait'2' lucid is-snippet for \
  ~/.config/zsh/powershell.zsh \
  ~/.config/zsh/br \
  ~/.config/zsh/conda.zsh \

zinit ice as"command" from"gh-r" \
          atclone"./starship init zsh > init.zsh; ./starship completions zsh > _starship" \
          atpull"%atclone" src"init.zsh"
zinit light starship/starship

zinit light-mode for \
      Aloxaf/fzf-tab

bindkey -M vicmd 'H' beginning-of-line
bindkey -M vicmd 'L' end-of-line

