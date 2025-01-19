#!/bin/bash

# 遍历当前目录下所有 *.winuser 目录
for dotfiles_dir in ./*.winuser; do
  if [ -d "$dotfiles_dir" ]; then
    echo "Processing $dotfiles_dir..."

    # 遍历 dotfiles_dir 下的文件和目录
    for file in $(find "$dotfiles_dir" -mindepth 1 -type f -o -type d); do
      # 获取相对路径
      rel_path="${file#$dotfiles_dir/}"

      # 源路径 (Windows)
      src_path="/mnt/c/Users/zion/$rel_path"

      # 目标路径 (dotfiles_dir)
      dest_path="$dotfiles_dir/$rel_path"

      # 如果是文件夹，则创建目标文件夹
      if [ -d "$file" ]; then
        echo "Creating directory: $dest_path"
        mkdir -p "$dest_path"
      fi

      # 如果是文件，则复制文件到目标路径
      if [ -f "$file" ]; then
        echo "Copying file: $src_path to $dest_path"
        cp -rf "$src_path" "$dest_path" # 使用 -f 强制覆盖
      fi
    done
  else
    echo "$dotfiles_dir is not a directory. Skipping."
  fi
done
