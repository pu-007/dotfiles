# 默认 shell（可选）
set shell := ["bash", "-c"]

# 一键更新系统 + AUR + Neovim 插件
update-all:
    yay -Syu --noconfirm
    nvim --headless "+Lazy! sync" +qa
    echo "✅ 系统 + Neovim Lazy 全部更新完成"

# 单独更新 Neovim Lazy
lazy-sync:
    nvim --headless "+Lazy! sync" +qa
    echo "✅ Lazy sync 完成"

# 更新 Neovim 本身 + 插件
update-nvim:
    yay -S neovim --noconfirm
    nvim --headless "+Lazy! sync" +qa

# 其他常用命令
clean:
    yay -Sc --noconfirm
    nvim --headless "+Lazy! clean" +qa

# 带参数示例
install pkg:
    yay -S {{pkg}} --noconfirm
