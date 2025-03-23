oh-my-posh init pwsh | Invoke-Expression

function l {
    eza --git -a --icons -l
}
function la {
    eza -a --icons --no-git
}
function ll {
    eza -a --total-size --git-repos --icons -l
}
function lT {
    eza --tree -a -I '.git'
}
function lt {
    lT -L
}