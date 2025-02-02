export LC_ALL=zh_CN.UTF-8
export LANG=zh_CN.UTF-8

export GTK_IM_MODULE=fcitx
export QT_IM_MODULE=fcitx
export XMODIFIERS=@im=fcitx
export SDL_IM_MODULE=fcitx
export GLFW_IM_MODULE=ibus

export MOZ_ENABLE_WAYLAND=0
export DISABLE_WAYLAND=1

ln -sf /mnt/wslg/runtime-dir/wayland-* /run/user/1000/

fcitx5 --disable=wayland -d --verbose '*'=0
