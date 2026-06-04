# {{{ Git & GitHub
alias Aic="gaa; aic2"
alias acp="aic2; git push"
alias Acp="Aic; git push"
alias lg="lazygit"
ialias rg="rg --hidden"
ialias 'gc@'='git reset --soft HEAD^'

function gzr() {
  cd $(git rev-parse --show-toplevel)
}

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
# }}}

# {{{ Docker
alias dk="docker"
alias dkc="docker compose"
alias dkcu="docker compose up -d"
# }}}

# {{{ Python & Pixi
alias py="python"
alias i="pixi"
alias ir="pixi run"
alias apy="source .venv/bin/activate"
alias wy="p python"
alias iy="ipython"
# }}}

# {{{ WSL & Windows Interop
ialias adb="adb.exe"
ialias fastboot="fastboot.exe"
ialias ollama="ollama.exe"
ialias wsl="wsl.exe"
ialias wg="winget.exe"
ialias scrcpy="scrcpy.exe"
alias padmode="scrcpy --new-display=1920x1080 --video-codec=h265 --always-on-top --fullscreen --disable-screensaver --video-buffer=50 --audio-buffer=200 --gamepad=uhid"
ialias winget="winget.exe"
alias ci="win32yank.exe -i"
alias co="win32yank.exe -o"
# }}}

# {{{ System & General Utilities
alias j="just"
alias als="alias | rg "
alias b="bat"
alias re-cmp=": rm .zcompdump; compinit"
alias uc="zinit update home--pu--.config--zsh/commands.zsh"
alias ec="v $DOTFILE_STORE/zsh.user/.config/zsh/commands.zsh"
alias re-cmake-install="sudo xargs rm < install_manifest.txt"
alias wrt="ssh root@192.168.100.1"
# }}}

# {{{ Package Management (Pacman & Yay)
alias re-pkg="pacman -Qdqt | xargs yay -Rnsc"
alias re-mirror="sudo reflector --country China --protocol https --latest 10 --sort rate --save /etc/pacman.d/mirrorlist"
alias yai="yay -S"
alias yas="yay -Ss"
alias yar="yay -Rnsc"
alias yaif="yay -S --overwrite='*'"
# }}}

# {{{ Navigation
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
# }}}

# {{{ Suffix Aliases
alias -s html=v
alias -s css=v
alias -s ts=v
alias -s py='env python'
alias -s js=v
alias -s c=v
alias -s txt=v
alias -s md=v
alias -s ini=v
alias -s toml=v
alias -s {yaml,yml}=v
alias -s json=v
# }}}

# {{{ Media & Audio Utilities
alias xo="xdg-open"

function speedup-audio() {
  local speed=$1
  shift # 移除第一个参数（速率），剩下的都是文件

  if [ -z "$speed" ]; then
    echo "用法: speedup_audio <速率> <文件1> [文件2] ..."
    echo "例如: speedup_audio 0.7 input.mp3"
    return 1
  fi

  # 替换小数点为下划线，避免文件名问题
  speed_label=$(echo "$speed" | tr '.' '_')

  for file in "$@"; do
    # 检查文件是否存在
    if [ ! -f "$file" ]; then
      echo "[警告] 文件不存在: $file"
      continue
    fi

    # 获取文件名和扩展名
    filename=$(basename -- "$file")
    extension="${filename##*.}"
    name="${filename%.*}"

    # 输出文件名：原文件名 + _速率值 + 扩展名
    output="${name}_${speed_label}x.${extension}"

    echo "正在处理: $file -> $output"
    # 使用VBR模式减小体积，质量等级4（可根据需要调整）
    ffmpeg -i "$file" -filter:a "atempo=$speed" -c:a libmp3lame -q:a 4 -map_metadata 0 -y "$output"
  done
}

function mp3t4() {
  ffmpeg -y -f lavfi -i color=black:s=1280x720:r=25 -i "$1" -c:v libx264 -c:a aac -shortest -pix_fmt yuv420p "${1%.mp3}.mp4"
}
# }}}

# vim: set foldmethod=marker foldlevel=0 :
