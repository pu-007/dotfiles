#!/bin/bash

sed -i 's/token\:.*$/token\: REDACTED/gI' "$(git rev-parse --show-toplevel | tr -d '\n')"/wsl.winuser/.config/yasb/config.yaml

# # 遍历所有传入的文件
# for file in "$@"; do
#   replace_token "$file"
# done
