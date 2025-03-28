#!/bin/bash

# 定义一个函数来替换隐私 token
replace_token() {
  local file=$1
  local temp_file=$(mktemp)

  # 使用 sed 替换隐私 token，这里假设隐私 token 匹配正则表达式
  sed -E 's/(api_key|token|secret)=.*$/\1=REDACTED/gI' "$file" >"$temp_file" && mv "$temp_file" "$file"
  sed -E 's/(api_key|token|secret)\:.*$/\1\: REDACTED/gI' "$file" >"$temp_file" && mv "$temp_file" "$file"

  # 检查是否进行了替换
  if git diff --quiet "$file"; then
    echo "No tokens found in $file"
  else
    echo "Token replaced in $file"
    git add "$file" # 将修改后的文件添加到暂存区
  fi
}

replace_token ./wsl.winuser/.config/yasb/config.yaml

# # 遍历所有传入的文件
# for file in "$@"; do
#   replace_token "$file"
# done
