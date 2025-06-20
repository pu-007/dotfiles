# GLOBAL PROFILE @ C:\Program Files\PowerShell\7\profile.ps1
# Install-Module PSReadLine
# Install-Module PSFzf
# Install-Module WslInterop

$env:SHELL = "pwsh"
$env:EDITOR = "vim"
$env:FZF_DEFAULT_OPTS = "--preview-window=hidden"

### PSReadLine
Set-PSReadLineOption -EditMode Vi
Set-PSReadLineKeyHandler -Chord Ctrl+l -Function ClearScreen
# This example emits a cursor change VT escape in response to a Vi mode change.
Write-Host -NoNewLine "`e[5 q"
function OnViModeChange {
    if ($args[0] -eq 'Command') {
        # Set the cursor to a blinking block.
        Write-Host -NoNewLine "`e[1 q"
    }
    else {
        # Set the cursor to a blinking line.
        Write-Host -NoNewLine "`e[5 q"
    }
}
Set-PSReadLineOption -ViModeIndicator Script -ViModeChangeHandler $Function:OnViModeChange

###PSFzf
Set-PsFzfOption -PSReadlineChordProvider 'Ctrl+t' -PSReadlineChordReverseHistory 'Ctrl+r'
$commandOverride = [ScriptBlock] { param($Location) Write-Host $Location }
Set-PsFzfOption -AltCCommand $commandOverride
Set-PSReadLineKeyHandler -Key Tab -ScriptBlock { Invoke-FzfTabCompletion }
Set-PsFzfOption -TabExpansion
Set-PsFzfOption -EnableAliasFuzzyEdit -EnableAliasFuzzyFasd -EnableAliasFuzzyHistory -EnableAliasFuzzyKillProcess -EnableAliasFuzzySetLocation -EnableAliasFuzzyScoop -EnableAliasFuzzySetEverything -EnableAliasFuzzyZLocation -EnableAliasFuzzyGitStatus
### WslInterop
# use native linux commands from WSL instead of GOW.
Import-WslCommand "yay", "awk", "find", "grep", "head", "less", "man", "sed", "seq", "ssh", "sudo", "tail", "touch", "vim", "wc"

#### Alias
if (-not $WslDefaultParameterValues) {
    $WslDefaultParameterValues = @{}
}
$WslDefaultParameterValues["eza"] = "-I 'NTUSER.DAT*|ntuser.*'"
function l {
    eza  --git -a --icons -l
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

function v {
    vim $args
}
function e {
    explorer .
}


#region conda initialize
# !! Contents within this block are managed by 'conda init' !!
If (Test-Path "C:\ProgramData\miniconda3\Scripts\conda.exe") {
    (& "C:\ProgramData\miniconda3\Scripts\conda.exe" "shell.powershell" "hook") | Out-String | ?{$_} | Invoke-Expression
}
#endregion
