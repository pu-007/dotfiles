zsh -ic "zinit update --all"
yay -Syu --noconfirm
pacman -Qqe >packages.meta/pacman.txt
git add ./packages.meta/
git commit -m "pkg update"
scoop cleanup -a -g -k
sudo pkgfile --update
conda update --all -y
#sudo npm -g update -y
# sudo npm install -g @google/gemini-cli
sudo npm install -g aicommit2
komorebic.exe fetch-app-specific-configuration
pip cache purge
#docker system prune -a -f
ssh root@192.168.100.1 docker system prune -a -f
# TODO: RoolBack RX 造成文件系统部分损坏 Trash 目录损坏 一删除就会让 wsl 崩溃 待修复
# sudo trash-empty -f --all-users
git -C /mnt/c/Users/zion/AppData/Roaming/Rime pull
