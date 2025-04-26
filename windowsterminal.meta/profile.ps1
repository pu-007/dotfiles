# GLOBAL PROFILE @ C:\Program Files\PowerShell\7\profile.ps1
# Install-Module PSReadLine
# Install-Module PSFzf
# Install-Module WslInterop

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

### WslInterop
# use native linux commands from WSL instead of GOW.
Import-WslCommand "yay", "awk", "fd", "find", "grep", "head", "less", "man", "sed", "seq", "ssh", "sudo", "tail", "touch", "vim", "wc", "bat"

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
