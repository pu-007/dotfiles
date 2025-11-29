alias aic="GEMINI_API_KEY= aicommit2 --auto-select --include-body"
alias Aic="gaa; aic"
alias acp="aic; git push"
alias Acp="Aic; git push"
alias G="gemini"
alias dkc="docker compose"
alias dcu="docker compose up -d"

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

function speedup-audio() {
    local speed=$1
    shift  # 移除第一个参数（速率），剩下的都是文件

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

export ITTS_DIR="/mnt/c/Users/zion/Apps/index-tts-vllm"

function itts {
  local itts_script=("python" "$ITTS_DIR/itts.py")
  local success_keyword="可用的语音角色"
  local max_wait=60       # 启动 Docker 后最大等待时间（秒）
  local sleep_sec=3       # 每次检测间隔

  # 检测服务是否可用
  check_service() {
    if output=$(timeout 5 "${itts_script[@]}" --get-voices 2>&1); then
      echo "$output" | grep -q "$success_keyword"
      return $?
    else
      return 1
    fi
  }

  # 先检测一次
  if check_service; then
    #echo "服务已就绪，直接执行命令..."
    "${itts_script[@]}" "$@"
    return 0
  fi

  # API 未启动，立即启动 Docker
  echo "API 未启动，启动 Docker 容器..."
  (cd "$ITTS_DIR" && docker compose up -d)

  # 等待服务初始化完成
  echo "等待服务初始化完成..."
  local ready=0
  for ((i=0; i<max_wait; i+=sleep_sec)); do
    if check_service; then
      ready=1
      break
    fi
    echo "$(date +'%H:%M:%S') - API 仍未就绪，等待 ${sleep_sec}s..."
    sleep $sleep_sec
  done

  if [ $ready -eq 0 ]; then
    echo "服务启动超时，未能检测到关键词 '$success_keyword'"
    return 1
  fi

  echo "服务已就绪，执行命令..."
  "${itts_script[@]}" "$@"
}

alias 3critts="co | itts --repeat-text -v candy,pu,cat"
alias 3crsitts="co | itts --repeat-text --shuffle-text -v candy,pu,cat"
alias 2critts="co | itts --repeat-text -v pu,cat"
alias 2crsitts="co | itts --repeat-text --shuffle-text -v pu,cat"
