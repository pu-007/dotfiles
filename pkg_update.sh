zsh -ic "zinit update --all"
yay -Syu --noconfirm
pacman -Qqe >packages.meta/pacman.txt
git add ./packages.meta/
git commit -m "pkg update"
scoop cleanup -a -g -k
sudo pkgfile --update
