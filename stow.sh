#!/bin/bash

# 连接 *.user 目录到 ~
for dir in *.user; do
  if [ -d "$dir" ]; then
    # 使用 stow 连接
    echo stow --adopt -t ~ "$dir"
    stow --adopt -t ~ "$dir"
  fi
done

# # 连接 *.root 目录到 /
# for dir in *.root; do
#   if [ -d "$dir" ]; then
#     # 使用 stow 连接
#     echo sudo stow --adopt -t / "$dir"
#     sudo stow --adopt -t / "$dir"
#   fi
# done

# 连接 *.winuser 目录到 C:/Users/zion
# for dir in *.winuser; do
#   if [ -d "$dir" ]; then
#     echo cp -r $dir/. /mnt/c/Users/zion/
#     cp -r $dir/. /mnt/c/Users/zion/
#   fi
# done
