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
sudo npm install -g aicommit2
komorebic.exe fetch-app-specific-configuration
pip cache purge
docker system prune
sudo trash-empty -f --all-users
