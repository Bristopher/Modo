# Switch-FluxerInstance.ps1
# -------------------------------------------------------------------------
# Pick which Fluxer server the desktop app connects to, then (re)launch it.
#
# What it does:
#   1. Shows a menu of servers (edit the $Servers list below to taste).
#   2. Fully closes Fluxer if it's running (window OR tray).
#   3. Writes the chosen URL into Fluxer's settings.json.
#   4. Relaunches the app pointed at that server.
#
# Usage:
#   Right-click  -> "Run with PowerShell"
#   or:  powershell -ExecutionPolicy Bypass -File .\Switch-FluxerInstance.ps1
#   or:  .\Switch-FluxerInstance.ps1 -Url https://my.server.example   (skip menu)
#   Add  -Canary  if you run the Fluxer Canary build.
# -------------------------------------------------------------------------

[CmdletBinding()]
param(
    [string]$Url,        # optional: set this URL directly, skip the menu
    [switch]$Canary      # use the Fluxer Canary build instead of stable
)

# Works on Windows PowerShell 5.1 (built into Windows) and PowerShell 7+.
$ErrorActionPreference = 'Stop'

# === EDIT ME: the servers your friends can pick from =====================
# Name shown in the menu  =  URL the app will connect to
$Servers = [ordered]@{
    'Bigweld (self-hosted)'  = 'https://fluxer.bigweld.duckdns.org'
    'Official Fluxer'        = 'https://web.fluxer.app'
    'Official Fluxer Canary' = 'https://web.canary.fluxer.app'
}
# =========================================================================

# --- Channel-specific names (don't usually need to change these) ---------
if ($Canary) {
    $ProcName    = 'Fluxer Canary'
    $StorageDir  = 'fluxercanary'
    $ExeLeaf     = 'Fluxer Canary.exe'
} else {
    $ProcName    = 'Fluxer'
    $StorageDir  = 'fluxer'
    $ExeLeaf     = 'Fluxer.exe'
}

$SettingsDir  = Join-Path $env:APPDATA $StorageDir
$SettingsFile = Join-Path $SettingsDir 'settings.json'

function Write-Title($t) { Write-Host ""; Write-Host $t -ForegroundColor Cyan }

# Find the installed Fluxer executable so we can relaunch it.
function Find-FluxerExe {
    # 1) If it's running, take the exact path it's running from.
    $proc = Get-Process -Name $ProcName -ErrorAction SilentlyContinue |
            Where-Object { $_.Path } | Select-Object -First 1
    if ($proc) { return $proc.Path }

    # 2) Known install locations (NSIS per-user, Squirrel, Program Files).
    $candidates = @(
        (Join-Path $env:LOCALAPPDATA ("Programs\{0}\{1}" -f $ProcName, $ExeLeaf)),
        (Join-Path $env:LOCALAPPDATA ("{0}\{1}"          -f $ProcName, $ExeLeaf)),
        (Join-Path $env:LOCALAPPDATA ("{0}\{1}"          -f $StorageDir, $ExeLeaf)),
        (Join-Path ${env:ProgramFiles}       ("{0}\{1}"  -f $ProcName, $ExeLeaf)),
        (Join-Path ${env:ProgramFiles(x86)}  ("{0}\{1}"  -f $ProcName, $ExeLeaf))
    )
    foreach ($c in $candidates) { if ($c -and (Test-Path $c)) { return $c } }

    # 3) Start Menu shortcut -> resolve its target.
    $startMenu = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
    $lnk = Get-ChildItem -Path $startMenu -Filter ("{0}.lnk" -f $ProcName) -Recurse -ErrorAction SilentlyContinue |
           Select-Object -First 1
    if ($lnk) {
        try {
            $shell  = New-Object -ComObject WScript.Shell
            $target = $shell.CreateShortcut($lnk.FullName).TargetPath
            if ($target -and (Test-Path $target)) { return $target }
        } catch { }
    }

    return $null
}

# Fully close the app (window + tray + any stray helpers).
function Stop-Fluxer {
    $running = Get-Process -Name $ProcName -ErrorAction SilentlyContinue
    if (-not $running) { Write-Host "Fluxer is not running." -ForegroundColor DarkGray; return }

    Write-Host "Closing Fluxer..." -ForegroundColor Yellow
    $running | Stop-Process -Force -ErrorAction SilentlyContinue

    # Wait up to ~5s for the processes to actually exit.
    for ($i = 0; $i -lt 25; $i++) {
        if (-not (Get-Process -Name $ProcName -ErrorAction SilentlyContinue)) { break }
        Start-Sleep -Milliseconds 200
    }
    if (Get-Process -Name $ProcName -ErrorAction SilentlyContinue) {
        Write-Warning "Fluxer is still closing; continuing anyway."
    } else {
        Write-Host "Fluxer closed." -ForegroundColor DarkGray
    }
}

# Validate + normalize a server URL.
function Resolve-Url($candidate) {
    $u = ($candidate).Trim().TrimEnd('/')
    if ($u -notmatch '^https?://[^/\s]+') {
        throw "That doesn't look like a valid URL: '$candidate'  (expected something like https://chat.example.com)"
    }
    return $u
}

# --- Pick the target URL -------------------------------------------------
$target = $null

if ($Url) {
    $target = Resolve-Url $Url
} else {
    Write-Title "Which Fluxer server do you want to use?"
    $names = @($Servers.Keys)
    for ($i = 0; $i -lt $names.Count; $i++) {
        "{0,3}) {1,-26} {2}" -f ($i + 1), $names[$i], $Servers[$names[$i]] | Write-Host
    }
    Write-Host "  C) Custom URL..."
    Write-Host "  Q) Quit (don't change anything)"
    Write-Host ""

    while (-not $target) {
        $choice = (Read-Host "Enter your choice").Trim()
        if ($choice -match '^[Qq]$') { Write-Host "No changes made."; return }
        if ($choice -match '^[Cc]$') {
            $custom = Read-Host "Paste the full server URL (e.g. https://chat.example.com)"
            try { $target = Resolve-Url $custom } catch { Write-Host $_.Exception.Message -ForegroundColor Red }
            continue
        }
        $n = 0
        if ([int]::TryParse($choice, [ref]$n) -and $n -ge 1 -and $n -le $names.Count) {
            $target = Resolve-Url $Servers[$names[$n - 1]]
        } else {
            Write-Host "Pick a number from the list, C for custom, or Q to quit." -ForegroundColor Red
        }
    }
}

Write-Host ""
Write-Host "Selected server: $target" -ForegroundColor Green

# --- Close Fluxer BEFORE writing settings (find exe first while it runs) --
$exe = Find-FluxerExe
Stop-Fluxer

# --- Write settings.json -------------------------------------------------
if (-not (Test-Path $SettingsDir)) {
    New-Item -ItemType Directory -Path $SettingsDir -Force | Out-Null
}
$json = "{`r`n  ""app_url"": ""$target""`r`n}`r`n"
Set-Content -Path $SettingsFile -Value $json -Encoding UTF8
Write-Host "Saved -> $SettingsFile" -ForegroundColor DarkGray

# --- Relaunch ------------------------------------------------------------
if (-not $exe) { $exe = Find-FluxerExe }   # re-find in case it was only resolvable while running
if ($exe -and (Test-Path $exe)) {
    Write-Host "Launching Fluxer..." -ForegroundColor Yellow
    Start-Process -FilePath $exe
    Write-Host "Done. Fluxer is starting on $target" -ForegroundColor Green
} else {
    Write-Warning "Couldn't find Fluxer.exe to launch automatically."
    Write-Host  "The server was saved -- just open Fluxer normally and it'll connect to:" -ForegroundColor Yellow
    Write-Host  "  $target"
}

# Keep the window open if it was double-clicked / run-with-PowerShell.
if ($Host.Name -eq 'ConsoleHost' -and -not $Url) {
    Write-Host ""
    Read-Host "Press Enter to close"
}
