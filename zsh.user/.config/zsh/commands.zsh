ialias eza="eza -I 'NTUSER.DAT*|ntuser.*'"
ialias l="eza --git -a --icons -l  "
ialias la="eza  -a --icons --no-git "
ialias ll="eza -a --total-size --git-repos --icons -l "
ialias lT="eza --tree -a -I '.git'"
alias lt="lT -L "

function gzr() {
  cd $(git rev-parse --show-toplevel)
}

alias v="vi"
ialias e="explorer.exe ."
ialias ex="explorer.exe .;exit 0"
ialias p="powershell.exe"
ialias c="cmd.exe"
ialias z="_zlua"
ialias a="gptme"
balias h="$home/"
ialias b="br"
alias py="python"
alias iy="ipython"
alias adb="adb.exe"
alias fastboot="fastboot.exe"
alias ollama="ollama.exe"
alias wsl="wsl.exe"
alias als="alias | rg "
ialias ai-commit="ai-commit --PROVIDER=ollama --MODEL=qwen2.5  --commit-type 'Grasp the main points and short, git commit messages only, no other comments'"
ialias 'gc@'='git reset --soft HEAD^'

ialias re-cmp=": rm .zcompdump; compinit"
ialias re-cmd="zinit update home--pu--.config--zsh/commands.zsh"
ialias re-pkg="yay -Rnsc `pacman -Qdqt`"

alias -g ...='../..'
alias -g ....='../../..'
alias -g .....='../../../..'
alias -g ......='../../../../..'

ialias -- -='cd -'
ialias 1='cd -1'
ialias 2='cd -2'
ialias 3='cd -3'
ialias 4='cd -4'
ialias 5='cd -5'
ialias 6='cd -6'
ialias 7='cd -7'
ialias 8='cd -8'
ialias 9='cd -9'

alias md="mkdir -p"
#ialias rm='echo "This is not the command you are looking for."; false'
alias rm="trash-put"

alias -s html=v
alias -s css=v
alias -s ts=v
alias -s py=v
alias -s js=v
alias -s c=v
alias -s txt=v
alias -s md=v
alias -s toml=v
alias -s {yaml,yml}=v
alias -s json=v

