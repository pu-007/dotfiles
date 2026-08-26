# ============================================================
# PowerShell Profile
# ============================================================
# Philosophy:
#   - Keep PowerShell native
#   - zsh-like interactive experience
#   - Minimal custom aliases/functions
#   - Safe for VS Code / AI / scripts
# ============================================================


# ============================================================
# 1. UTF-8
# ============================================================

$PSDefaultParameterValues['Out-File:Encoding'] = 'utf8'
$PSDefaultParameterValues['Set-Content:Encoding'] = 'utf8'
$PSDefaultParameterValues['Add-Content:Encoding'] = 'utf8'

[Console]::InputEncoding = [System.Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)


# ============================================================
# 2. Environment
# ============================================================

$env:SHELL = 'pwsh'
$env:EDITOR = 'nvim'
$env:VISUAL = 'nvim'

$env:DOTNET_CLI_UI_LANGUAGE = 'zh-CN'
$env:DOTNET_SYSTEM_GLOBALIZATION_INVARIANT = '0'

$env:FZF_DEFAULT_OPTS = '--preview-window=hidden'


# ============================================================
# 3. zoxide
# ============================================================

if (Get-Command zoxide -ErrorAction SilentlyContinue) {
    Invoke-Expression (& { zoxide init powershell | Out-String })
}


# ============================================================
# 4. WSL Interop
# ============================================================

if (Get-Command Import-WslCommand -ErrorAction SilentlyContinue) {

    # Use Linux nvim from WSL
    Import-WslCommand 'nvim'

    if (-not $WslDefaultParameterValues) {
        $WslDefaultParameterValues = @{}
    }

    $WslDefaultParameterValues['eza'] = "-I 'NTUSER.DAT*|ntuser.*'"
}


# ============================================================
# PSReadLine
# ============================================================

if (Get-Command Set-PSReadLineOption -ErrorAction SilentlyContinue) {

    # Vi editing
    Set-PSReadLineOption -EditMode Vi

    # History
    Set-PSReadLineOption -HistoryNoDuplicates
    Set-PSReadLineOption -HistorySearchCursorMovesToEnd
    Set-PSReadLineOption -MaximumHistoryCount 10000

    # No terminal bell
    Set-PSReadLineOption -BellStyle None

    # History prediction
    try {
        Set-PSReadLineOption -PredictionSource History
        Set-PSReadLineOption -PredictionViewStyle ListView
    }
    catch {
        # Ignore on older PSReadLine versions
    }

    # Ctrl + L
    Set-PSReadLineKeyHandler `
        -Chord Ctrl+l `
        -Function ClearScreen

    # --------------------------------------------------------
    # Vi cursor
    # --------------------------------------------------------

    $viModeChangeHandler = {
        param($mode)

        if ($mode -eq 'Command') {
            # Command mode → block cursor
            Write-Host -NoNewLine "`e[1 q"
        }
        else {
            # Insert mode → line cursor
            Write-Host -NoNewLine "`e[5 q"
        }
    }

    Set-PSReadLineOption `
        -ViModeIndicator Script `
        -ViModeChangeHandler $viModeChangeHandler
}


# ============================================================
# 6. PSFzf
# ============================================================

if (Get-Command Set-PsFzfOption -ErrorAction SilentlyContinue) {

    # Ctrl + T → file/path search
    Set-PsFzfOption `
        -PSReadlineChordProvider 'Ctrl+t'

    # Ctrl + R → history search
    Set-PsFzfOption `
        -PSReadlineChordReverseHistory 'Ctrl+r'

    # Tab → fzf completion
    Set-PsFzfOption -TabExpansion

    # Alt + C → fuzzy directory navigation
    Set-PsFzfOption -AltCCommand {
        param($Location)
        Set-Location $Location
    }

    Set-PSReadLineKeyHandler `
        -Key Tab `
        -ScriptBlock {
            Invoke-FzfTabCompletion
        }

    # Optional fuzzy aliases
    Set-PsFzfOption `
        -EnableAliasFuzzyEdit `
        -EnableAliasFuzzyHistory `
        -EnableAliasFuzzySetLocation `
        -EnableAliasFuzzyZLocation `
        -EnableAliasFuzzyGitStatus
}


# ============================================================
# 7. Minimal aliases
# ============================================================
# Only aliases that do not change PowerShell semantics.

Set-Alias -Name v -Value nvim
Set-Alias -Name e -Value explorer.exe


# ============================================================
# 8. eza
# ============================================================

function l {
    eza --git -a --icons -l
}

function ll {
    eza -a --total-size --git-repos --icons -l
}

function lt {
    eza --tree -a -I '.git' -L 2
}

function lT {
    eza --tree -a -I '.git'
}
