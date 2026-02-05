# Dotfiles

My dotfiles on Arch WSL & Windows.

- TODO: cc switch and claude code gemini cli settings
- TODO: simplify the link steps
- TODO: add docs
- TODO: all pip and uv changed mirror

- TODO: develop a full-featured and powerful script wo manage dotfiles. Functionsl listed below:

1. transform:

export DOTFILES_STORE="~/dotfiles"

1.1. Existing Linux Config <==> Stow Profile

- A->B {target: ~/.config/example}

  - `mkdir $DOTFILE_STORE/{app_name: example}/.config/example`

  - `mv  ~/.config/example/* $DOTFILE_STORE/{app_name: example}/.config/example/` (rescursive and force)

- B->A

  - Plan 1(link dir, not rerecommended): use `stow --adopt -t ~ "$dir"`

  - Plan 2(link files respectively):yield files, create its parents, and stow them respectively

    1.2. Existing Windows Config <==> Symlinks to Arch on Windows (or copy mode optionally)

- A->B: Windows to WSL stotage

  - 1.1 put your files manually
    put your Windows config file in ~/dotfiles/c.mnt/<your_windows_path>
    e.g. C:\Users\zion\test.conf ==> ~/dotfiles/c.mnt/Users/zion/test.conf

  - 1.2 run this script to create a symlink in WSL (files recommended, directories may cause issues, i.e., it may include extra files)
    <TODO: script to auto move files from Windows to WSL storage> 2. WSL to Windows storage

- B-> A

(speciall file: .wslconfig, komorebi.json)

- 2.1 quick proceed all files, just run:
  `fd . c.mnt -H -t f -x python scripts.meta/config_linker.py --no-confirm-deletion {}`
- specify files or directories manually:
  `python scripts.meta/config_linker.py ~/dotfiles/c.mnt/Users/zion/.config <other_files_or_dirs>`

2. type

`user`: Linux Configs in home
`root`: Linux Configs in root
`mnt`: Windows mount dir
`meta`: spcial files that stored manually

3. a quick and user-friendly interface

4. use simple, structured python code

- TODO: Add others meta type to c.mnt after the script finished
