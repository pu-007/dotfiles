alias aic="GEMINI_API_KEY= aicommit2 --auto-select --include-body"
alias Aic="gaa; aic"
alias acp="aic; git push"
alias Acp="Aic; git push"

alias j="jj"
function gzr() {
  cd $(git rev-parse --show-toplevel)
}
ialias 'gc@'='git reset --soft HEAD^'
alias lg="lazygit"
function ghd() {
  # 检查参数是否提供
  if [ -z "$1" ]; then
    echo "Usage: ghd <github_blob_url>"
    return 1
  fi

  # 提取 GitHub blob URL 的各个部分
  local url="$1"
  local raw_url=$(echo "$url" | sed 's/github.com/raw.githubusercontent.com/g' | sed 's/blob\//\//g')

  # 下载文件并保存到当前目录
  local filename=$(basename "$raw_url")
  curl -L -o "$filename" "$raw_url"

  if [ $? -eq 0 ]; then
    echo "File downloaded successfully: $filename"
  else
    echo "Failed to download file from $raw_url"
    return 1
  fi
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
alias md2pdf="python $DOTFILE_STORE/scripts.meta/md2pdf.py"
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
alias yar="yay -Rnsc"
alias yaif="yay -S --overwrite='*'"

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

alias -s html=v
alias -s css=v
alias -s ts=v
# alias -s py=v
alias -s py='env python'
alias -s js=v
alias -s c=v
alias -s txt=v
alias -s md=v
alias -s ini=v
alias -s txt=v
alias -s toml=v
alias -s {yaml,yml}=v
alias -s json=v
