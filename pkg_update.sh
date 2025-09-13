zsh -ic "zinit update --all"
yay -Syu --noconfirm
pacman -Qqe >packages.meta/pacman.txt
git add ./packages.meta/
git commit -m "pkg update"
scoop cleanup -a -g -k
sudo pkgfile --update
conda update --all -y
#sudo npm -g update -y
sudo npm install -g @google/gemini-cli
komorebic.exe fetch-app-specific-configuration
