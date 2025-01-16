# ------------------------------------------------------------------------------
# Description
# -----------
# Adds 'p' before the command when triggered.
# ------------------------------------------------------------------------------
# Authors
# -------
# * Dongweiming <ciici123@gmail.com>
# * Subhaditya Nath <github.com/subnut>
# * Marc Cornellà <github.com/mcornella>
# * Carlo Sala <carlosalag@protonmail.com>
# ------------------------------------------------------------------------------

__add-p-replace-buffer() {
  local old=$1 new=$2 space=${2:+ }

  # If the cursor is positioned in the $old part of the text, make
  # the substitution and leave the cursor after the $new text
  if [[ $CURSOR -le ${#old} ]]; then
    BUFFER="${new}${space}${BUFFER#$old }"
    CURSOR=${#new}
  # Otherwise just replace $old with $new in the text before the cursor
  else
    LBUFFER="${new}${space}${LBUFFER#$old }"
  fi
}

add-p-command-line() {
  # If line is empty, get the last run command from history
  [[ -z $BUFFER ]] && LBUFFER="$(fc -ln -1)"

  # Save beginning space
  local WHITESPACE=""
  if [[ ${LBUFFER:0:1} = " " ]]; then
    WHITESPACE=" "
    LBUFFER="${LBUFFER:1}"
  fi

  {
    # Toggle the 'p' prefix on and off
    case "$BUFFER" in
      p\ *) __add-p-replace-buffer "p" "" ;;
      *) LBUFFER="p $LBUFFER" ;;
    esac
  } always {
    # Preserve beginning space
    LBUFFER="${WHITESPACE}${LBUFFER}"

    # Redisplay edit buffer (compatibility with zsh-syntax-highlighting)
    zle && zle redisplay # Only run redisplay if zle is enabled
  }
}

zle -N add-p-command-line

# Defined shortcut keys: [Tab][Tab]
bindkey -M emacs '^\\' add-p-command-line
bindkey -M vicmd '^\\' add-p-command-line
bindkey -M viins '^\\' add-p-command-line
