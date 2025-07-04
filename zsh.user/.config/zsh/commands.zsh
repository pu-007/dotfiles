function gzr() {
  cd $(git rev-parse --show-toplevel)
}



alias py="python"
alias apy="source .venv/bin/activate"
alias wy="p python"
alias xz="x z"
alias uz="x uz"
alias iy="ipython"
alias adb="adb.exe"
alias fastboot="fastboot.exe"
alias ollama="ollama.exe"
alias wsl="wsl.exe"
ialias wg="winget.exe"
ialias scrcpy="scrcpy.exe"
alias padmode="scrcpy --new-display=1920x1080 --video-codec=h265 --always-on-top --fullscreen --disable-screensaver --video-buffer=50 --audio-buffer=200 --gamepad=uhid"
ialias winget="winget.exe"
alias als="alias | rg "
alias gdd="$DOTFILE_STORE/scripts.meta/git_staged_summary.sh"
alias md2pdf="python $DOTFILE_STORE/scripts.meta/md2pdf.py"
export AI_COMMIT_PROMPT='"Commit staged files directly with a commit message generated based on the infomation displayed below. The message must follow the Conventional Commit style(i.e. <type>[optional scope]: <description>) in English and intelligently determine whether to include a detailed description based on the diff. If there are multiple lines, use mutile -m in git args instead of \n in message"'
ialias gac="gdd --no-color | gptme --non-interactive -t shell $AI_COMMIT_PROMPT"
ialias 'gc@'='git reset --soft HEAD^'
alias aic="gaa; gac"
alias aicp="aic; git push"
alias lg="lazygit"
alias b="bat"


alias ci="win32yank.exe -i"
alias co="win32yank.exe -o"
alias re-cmp=": rm .zcompdump; compinit"
alias uc="zinit update home--pu--.config--zsh/commands.zsh"
alias ec="v $DOTFILE_STORE/zsh.user/.config/zsh/commands.zsh"
alias re-pkg="pacman -Qdqt | xargs yay -Rnsc"
alias re-cmake-install="sudo xargs rm < install_manifest.txt"
alias re-mirror="sudo reflector --country China --protocol https --latest 10 --sort rate --save /etc/pacman.d/mirrorlist"

# ssh to openwrt server
alias wrt="ssh root@192.168.100.1"

alias yai="yay -S"
alias yas="yay -Ss"

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
# alias -s py=v
alias -s js=v
alias -s c=v
alias -s txt=v
alias -s md=v
alias -s ini=v
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
