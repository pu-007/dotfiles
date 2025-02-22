function gzr() {
  cd $(git rev-parse --show-toplevel)
}


function y() {
	local tmp="$(mktemp -t "yazi-cwd.XXXXXX")" cwd
	yazi "$@" --cwd-file="$tmp"
	if cwd="$(command cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
		builtin cd -- "$cwd"
	fi
	rm -f -- "$tmp"
}

alias py="python"
alias wy="p python"
alias iy="ipython"
alias adb="adb.exe"
alias fastboot="fastboot.exe"
alias ollama="ollama.exe"
alias wsl="wsl.exe"
alias wg="winget.exe"
ialias winget="winget.exe"
alias als="alias | rg "
ialias gac="ai-commit --PROVIDER=ollama --MODEL=qwen2.5-coder  --commit-type 'Grasp the main points and short, git commit messages only, no other comments'"
ialias 'gc@'='git reset --soft HEAD^'
alias gaac="gaa; gac"

alias re-cmp=": rm .zcompdump; compinit"
alias re-cmd="zinit update home--pu--.config--zsh/commands.zsh"
alias re-pkg="pacman -Qdqt | xargs yay -Rnsc"
alias re-cmake-install="sudo xargs rm < install_manifest.txt"
# utilize Cline in place of Cursor
# alias re-cursor="p 'irm https://raw.githubusercontent.com/yuaotian/go-cursor-help/refs/heads/master/scripts/run/cursor_win_id_modifier.ps1 | iex'"

alias yai="yay -S"

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
alias -s txt=v
alias -s toml=v
alias -s {yaml,yml}=v
alias -s json=v

update-all () {
        echo "Updating pip packages..."
        pip list --outdated | awk 'NR > 2 {print $1}' | xargs -n1 pip install --upgrade --progress-bar on
        echo "Pip packages updated."
        echo "Updating conda packages..."
        conda update --all -y
        echo "Conda packages updated."
        echo "Updating npm global packages..."
        sudo npm outdated -g --json | jq -r 'keys | .[]' | xargs -n1 sudo npm install -g
        echo "npm global packages updated."
        echo "Updating pacman packages..."
        yay -Syyu --noconfirm
        echo "Pacman packages updated."
}
