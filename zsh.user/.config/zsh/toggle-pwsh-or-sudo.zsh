# ------------------------------------------------------------------------------
# Description
# -----------
# Adds 'p -Command' or 'sudo' before the command when triggered.
# ------------------------------------------------------------------------------
# Authors
# -------
# * Dongweiming <ciici123@gmail.com>
# * Subhaditya Nath <github.com/subnut>
# * Marc Cornellà <github.com/mcornella>
# * Carlo Sala <carlosalag@protonmail.com>
# ------------------------------------------------------------------------------

# 通用辅助函数：用于在命令行前缀中替换/剥离指定的命令
__toggle-prefix-replace-buffer() {
  local old=$1 new=$2 space=${2:+ }

  # 如果光标处于 $old 部分的文本中，进行替换并将光标留在新文本后面
  if [[ $CURSOR -le ${#old} ]]; then
    BUFFER="${new}${space}${BUFFER#$old }"
    CURSOR=${#new}
  # 否则，只替换光标前文本中的 $old
  else
    LBUFFER="${new}${space}${LBUFFER#$old }"
  fi
}

# ==============================================================================
# 1. p -Command 切换功能
# ==============================================================================
add-p-command-line() {
  # 如果命令行空，则获取历史记录中最后运行的命令
  [[ -z $BUFFER ]] && LBUFFER="$(fc -ln -1)"

  # 保留行首空格
  local WHITESPACE=""
  if [[ ${LBUFFER:0:1} = " " ]]; then
    WHITESPACE=" "
    LBUFFER="${LBUFFER:1}"
  fi

  {
    # 切换 'p -Command' 前缀的开启和关闭
    # 已修正：收窄匹配模式至 'p\ -Command\ *'，避免误伤其他 'p' 开头的命令
    case "$BUFFER" in
      p\ -Command\ *) __toggle-prefix-replace-buffer "p -Command" "" ;;
      *) LBUFFER="p -Command $LBUFFER" ;;
    esac
  } always {
    # 恢复行首空格
    LBUFFER="${WHITESPACE}${LBUFFER}"

    # 重新显示编辑缓冲区（兼容 zsh-syntax-highlighting）
    zle && zle redisplay
  }
}

zle -N add-p-command-line

# 绑定快捷键：[Ctrl + \]
bindkey -M emacs '^\\' add-p-command-line
bindkey -M vicmd '^\\' add-p-command-line
bindkey -M viins '^\\' add-p-command-line


# ==============================================================================
# 2. sudo 切换功能
# ==============================================================================
sudo-command-line() {
  # 如果命令行空，则获取历史记录中最后运行的命令
  [[ -z $BUFFER ]] && LBUFFER="$(fc -ln -1)"

  # 保留行首空格
  local WHITESPACE=""
  if [[ ${LBUFFER:0:1} = " " ]]; then
    WHITESPACE=" "
    LBUFFER="${LBUFFER:1}"
  fi

  {
    # 切换 'sudo' 前缀的开启和关闭（包含对 sudo -e 的兼容支持）
    case "$BUFFER" in
      sudo\ -e\ *) __toggle-prefix-replace-buffer "sudo -e" "" ;;
      sudo\ *) __toggle-prefix-replace-buffer "sudo" "" ;;
      *) LBUFFER="sudo $LBUFFER" ;;
    esac
  } always {
    # 恢复行首空格
    LBUFFER="${WHITESPACE}${LBUFFER}"

    # 重新显示编辑缓冲区
    zle && zle redisplay
  }
}

zle -N sudo-command-line

# 绑定快捷键：[Alt + \] 
# Zsh 中用 '^[\\' 表示 Alt + \（部分终端中也可以用 '\e\\' 表示）
bindkey -M emacs '^[\\' sudo-command-line
bindkey -M vicmd '^[\\' sudo-command-line
bindkey -M viins '^[\\' sudo-command-line
