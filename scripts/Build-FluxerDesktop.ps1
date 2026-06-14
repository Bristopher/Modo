# Build-FluxerDesktop.ps1
# -------------------------------------------------------------------------
# Build a Windows installer for the Fluxer desktop app from source.
#
# Why build it yourself: the official published .exe can ONLY connect to
# web.fluxer.app. A from-source build includes the settings.json / app_url
# feature, so it can point at your self-hosted instance (use the companion
# Switch-FluxerInstance.ps1 to pick a server).
#
# This makes NO source changes, so you can keep pulling from the main repo.
# It produces the NSIS installer (not Squirrel) on purpose: NSIS installs
# have no Squirrel Update.exe, so the built-in auto-updater can't silently
# replace your build with the official one.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File .\scripts\Build-FluxerDesktop.ps1
#   Options:
#     -Canary        Build the "Fluxer Canary" variant instead of stable.
#     -SkipInstall   Skip "pnpm install" (faster if deps are already there).
#     -Arch x64|arm64   Target architecture (default: x64).
# -------------------------------------------------------------------------

[CmdletBinding()]
param(
    [switch]$Canary,
    [string]$Brand,        # e.g. "Fluxer Bigweld" -> installs as its OWN app,
                           # separate from official Fluxer (own appId + profile).
    [int]$BuildNumber = 0, # force a personal build number (0 = auto-increment).
    [string]$DefaultServer, # e.g. "https://fluxer.bigweld.duckdns.org" -> the
                            # installer seeds settings.json so the app connects
                            # to this server out of the box (no Switch script).
    [switch]$SkipInstall,
    [ValidateSet('x64', 'arm64')]
    [string]$Arch = 'x64'
)

$ErrorActionPreference = 'Stop'

function Step($msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Fail($msg) { Write-Host "`nERROR: $msg" -ForegroundColor Red; exit 1 }

# electron-builder downloads winCodeSign for Windows targets and extracts it
# with 7-Zip. That archive contains macOS symlinks (darwin .dylib) which Windows
# refuses to create without admin/Developer Mode, so 7-Zip returns a non-zero
# exit and electron-builder aborts. We pre-extract it ourselves into the cache,
# tolerating those 2 symlink failures (the Windows tools extract fine). If the
# cache is already populated, do nothing.
function Initialize-WinCodeSignCache($desktopDir) {
    $cache = Join-Path $env:LOCALAPPDATA 'electron-builder\Cache\winCodeSign'
    $dest  = Join-Path $cache 'winCodeSign-2.6.0'
    if (Test-Path (Join-Path $dest 'windows-10')) {
        Write-Host "  winCodeSign cache OK" -ForegroundColor DarkGray
        return
    }
    Step "Preparing winCodeSign cache (one-time, avoids symlink-privilege error)"
    $sevenZip = (Get-ChildItem (Join-Path $desktopDir 'node_modules') -Recurse -Filter '7za.exe' -ErrorAction SilentlyContinue |
                 Where-Object { $_.FullName -match '7zip-bin' -and $_.FullName -match 'win' } |
                 Select-Object -First 1).FullName
    if (-not $sevenZip) { Fail "Couldn't find bundled 7za.exe (run without -SkipInstall first)." }

    New-Item -ItemType Directory -Force -Path $cache | Out-Null
    $archive = Join-Path $cache 'winCodeSign-2.6.0.7z'
    $url = 'https://github.com/electron-userland/electron-builder-binaries/releases/download/winCodeSign-2.6.0/winCodeSign-2.6.0.7z'
    Write-Host "  downloading winCodeSign-2.6.0..."
    Invoke-WebRequest -Uri $url -OutFile $archive -UseBasicParsing
    if (Test-Path $dest) { Remove-Item $dest -Recurse -Force -ErrorAction SilentlyContinue }
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    # Exit code is ignored on purpose: only the 2 mac symlinks fail.
    & $sevenZip x -snld -bd "-o$dest" $archive | Out-Null
    if (Test-Path (Join-Path $dest 'windows-10')) {
        Write-Host "  winCodeSign cache ready." -ForegroundColor DarkGray
    } else {
        Fail "winCodeSign extraction did not produce the Windows tools."
    }
}

# Repo version core "YYYY.M.D" from the last commit date (no leading zeros so
# it's valid semver). Falls back to 0.0.0 if git isn't available.
function Get-RepoVersionCore($repoRoot) {
    try {
        $d = (& git -C $repoRoot log -1 --date=format:'%Y%m%d' --format=%cd 2>$null)
        if ($LASTEXITCODE -ne 0 -or -not $d) { return $null }
        $d = $d.Trim()
        if ($d -notmatch '^\d{8}$') { return $null }
        return ("{0}.{1}.{2}" -f [int]$d.Substring(0,4), [int]$d.Substring(4,2), [int]$d.Substring(6,2))
    } catch { return $null }
}
function Get-RepoSha($repoRoot) {
    try { $s = (& git -C $repoRoot rev-parse --short=8 HEAD 2>$null); if ($LASTEXITCODE -eq 0 -and $s) { return $s.Trim() } } catch {}
    return $null
}
function Test-RepoDirty($repoRoot) {
    try { return [bool](& git -C $repoRoot status --porcelain 2>$null) } catch { return $false }
}
# Personal, monotonically increasing build counter, stored OUTSIDE the repo so
# it never dirties git. -BuildNumber overrides; otherwise it auto-increments.
function Get-NextBuildNumber($explicit) {
    $dir  = Join-Path $env:LOCALAPPDATA 'FluxerDesktopBuild'
    $file = Join-Path $dir 'build-number.txt'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    if ($explicit -gt 0) {
        $n = $explicit
    } else {
        $cur = 0
        if (Test-Path $file) { [void][int]::TryParse((Get-Content $file -Raw).Trim(), [ref]$cur) }
        $n = $cur + 1
    }
    Set-Content -Path $file -Value $n -Encoding ASCII
    return $n
}

# --- Locate paths --------------------------------------------------------
$RepoRoot   = Split-Path -Parent $PSScriptRoot          # scripts\ -> repo root
$DesktopDir = Join-Path $RepoRoot 'fluxer_desktop'
if (-not (Test-Path $DesktopDir)) { Fail "Can't find fluxer_desktop at $DesktopDir" }

# A branded build piggybacks on the canary "slot" purely for isolation: the
# canary channel gives it a separate userData folder (%APPDATA%\fluxercanary)
# and a distinct appId base WITHOUT any source change. We then rename it via the
# generated config so it shows up as e.g. "Fluxer Bigweld", installed completely
# separately from the official "Fluxer".
if ($Brand) { $Canary = $true }

$channel = if ($Canary) { 'canary' } else { 'stable' }
$product = if ($Brand) { $Brand } elseif ($Canary) { 'Fluxer Canary' } else { 'Fluxer' }

# Where the app keeps its settings.json (the canary slot is used by both -Canary
# and -Brand builds for profile isolation). The installer seeds this folder when
# -DefaultServer is given.
$storageDir = if ($Canary) { 'fluxercanary' } else { 'fluxer' }

if ($DefaultServer) {
    $DefaultServer = $DefaultServer.Trim().TrimEnd('/')
    if ($DefaultServer -notmatch '^https?://[^/\s]+') {
        Fail "DefaultServer '$DefaultServer' is not a valid URL (expected https://host...)."
    }
}

$appIdOverride = $null
if ($Brand) {
    $slug = ($Brand -replace '[^A-Za-z0-9]', '').ToLower()
    if (-not $slug) { Fail "Brand '$Brand' has no usable letters/digits for an appId." }
    $appIdOverride = "app.fluxer.$slug"
}

# --- Compose the version: repo date + sha + personal build counter -------
$verCore = Get-RepoVersionCore $RepoRoot
if (-not $verCore) { $verCore = '0.0.0' }
$repoSha = Get-RepoSha $RepoRoot
$buildNo = Get-NextBuildNumber $BuildNumber
$pre = "b$buildNo"
if ($repoSha) { $pre += ".g$repoSha" }
if (Test-RepoDirty $RepoRoot) { $pre += ".dirty" }
$version = "$verCore-$pre"   # e.g. 2026.6.13-b3.ga5dc6929

Write-Host "Fluxer desktop build" -ForegroundColor Green
Write-Host "  repo:    $RepoRoot"
Write-Host "  product: $product"
Write-Host "  version: $version" -ForegroundColor Green
Write-Host "           (repo $verCore @ $repoSha  |  your build #$buildNo)"
Write-Host "  channel: $channel (storage slot)"
if ($appIdOverride) { Write-Host "  appId:   $appIdOverride" }
Write-Host "  arch:    $Arch"

# --- Check tooling -------------------------------------------------------
Step "Checking tools"
foreach ($tool in @('node', 'pnpm')) {
    $cmd = Get-Command $tool -ErrorAction SilentlyContinue
    if (-not $cmd) { Fail "'$tool' is not installed or not on PATH." }
    Write-Host ("  {0,-5} {1}" -f $tool, (& $tool --version))
}

# --- Environment for this build ------------------------------------------
$env:NODE_ENV      = 'production'
$env:BUILD_CHANNEL = $channel
# We do not code-sign. Disabling cert auto-discovery stops electron-builder
# from downloading/extracting the winCodeSign toolchain, which contains macOS
# symlinks that Windows refuses to extract without admin/Developer Mode
# ("A required privilege is not held by the client").
$env:CSC_IDENTITY_AUTO_DISCOVERY = 'false'

# --- Install deps --------------------------------------------------------
# fluxer_desktop is EXCLUDED from the pnpm workspace (see pnpm-workspace.yaml:
# "- '!fluxer_desktop'"), so it must be installed standalone with
# --ignore-workspace, which gives it its own fluxer_desktop\node_modules.
Push-Location $DesktopDir
try {
    if (-not $SkipInstall) {
        Step "Installing desktop dependencies (pnpm install --ignore-workspace)"
        pnpm install --ignore-workspace
        if ($LASTEXITCODE -ne 0) { Fail "pnpm install (desktop) failed." }
    } else {
        Write-Host "`n(skipping pnpm install)" -ForegroundColor DarkGray
    }

    # --- Bundle main + preload (esbuild) ---------------------------------
    Step "Bundling main/preload (pnpm build)"
    pnpm build
    if ($LASTEXITCODE -ne 0) { Fail "esbuild bundle (pnpm build) failed." }

    # --- Generate a Windows-only config (no source edits) ----------------
    # The tracked electron-builder.config.cjs has a linux.desktop block whose
    # key shape (Name/Comment/Categories/StartupWMClass) is rejected by
    # electron-builder 26's schema validator -- which runs even for a Windows
    # build. We build Windows only, so emit a temp config that drops `linux`.
    # This leaves the tracked file untouched (clean pulls from upstream).
    $genConfig = Join-Path $DesktopDir '.eb-win.generated.cjs'
    $brandLines = ''
    if ($Brand) {
        $brandJson = $Brand | ConvertTo-Json   # safely quotes/escapes the name
        $brandLines = @"
base.productName = $brandJson;          // distinct install dir / exe / Start Menu name
base.appId = '$appIdOverride';          // distinct Windows app identity
"@
    }
    # Generate a custom NSIS include that (a) ALWAYS force-closes a running
    # instance so the installer never stops to ask the user to close it manually,
    # and (b) optionally seeds the server URL at install time.
    #
    # Electron names every process (main + GPU/renderer helpers) "<productName>.exe"
    # on Windows, so a single taskkill /F /T by image name clears all file locks.
    # We do it in customInit (runs in .onInit, before electron-builder's
    # "app is running" check) AND at the top of customInstall (right before files
    # are copied), which covers both the assisted-installer prompt and the copy lock.
    #
    # The seed writes %APPDATA%\<storageDir>\settings.json on first install ONLY (it
    # won't clobber a user who later switched servers), so the app connects to your
    # instance out of the box -- no need to run Switch-FluxerInstance.ps1.
    #
    # The .nsh is generated/cleaned alongside the temp config, so the tracked source
    # stays untouched and upstream pulls stay clean.
    $genNsh   = Join-Path $DesktopDir '.eb-installer.generated.nsh'
    $exeName  = "$product.exe"   # productName drives the Electron exe + helper names

    $seedBlock = ''
    if ($DefaultServer) {
        $settingsBody = "{`"app_url`": `"$DefaultServer`"}"
        $seedBlock = @"
  IfFileExists "`$APPDATA\$storageDir\settings.json" fluxer_seed_done fluxer_seed_write
  fluxer_seed_write:
    CreateDirectory "`$APPDATA\$storageDir"
    FileOpen `$0 "`$APPDATA\$storageDir\settings.json" w
    FileWrite `$0 '$settingsBody'
    FileClose `$0
  fluxer_seed_done:
"@
        Write-Host "  default server: $DefaultServer (seeded into installer)" -ForegroundColor Green
    }

    # customCheckAppRunning REPLACES electron-builder's default "is the app running?"
    # check -- the one that pops the "cannot be closed, click Retry" dialog. By
    # defining it we bypass that prompt entirely and just force-kill. customInit
    # (runs first, in .onInit) and customInstall (right before file copy) repeat the
    # kill as belt-and-suspenders for tray respawns. Full path to taskkill.exe so
    # nsExec doesn't depend on PATH resolution inside the installer.
    #
    # Kill the EXACT installed exe name "<product>.exe" (with /T to sweep the Electron
    # helper subprocesses, which share that exact name). Do NOT use a wildcard like
    # "<product>*": the installer's own filename also begins with the product name
    # ("Fluxer Bigweld Setup ...exe" / versioned), so a wildcard would kill the running
    # installer itself mid-upgrade. The exact name never matches the setup exe.
    $killCmd = "`"`$SYSDIR\taskkill.exe`" /F /T /IM `"$exeName`""
    $nshBody = @"
!macro customCheckAppRunning
  nsExec::Exec '$killCmd'
  Pop `$0
  Sleep 1000
!macroend

!macro customInit
  nsExec::Exec '$killCmd'
  Pop `$1
!macroend

!macro customInstall
  nsExec::Exec '$killCmd'
  Pop `$1
  Sleep 500
$seedBlock
!macroend
"@
    Set-Content -Path $genNsh -Value $nshBody -Encoding ASCII
    $nshJson  = '.eb-installer.generated.nsh' | ConvertTo-Json  # path relative to config
    $nshLines = "base.nsis = Object.assign({}, base.nsis, { include: $nshJson });"
    Write-Host "  auto-close on install: taskkill `"$exeName`" (baked into installer)" -ForegroundColor Green

    $verJson = $version | ConvertTo-Json
    $genBody = @"
const base = require('./electron-builder.config.cjs');
delete base.linux; // Windows-only build; upstream linux block fails eb26 schema
$brandLines
$nshLines
// Stamp our version (package.json is 0.0.0). Shows in the app's About and the
// installer filename: <repo date>-b<your build #>.g<repo sha>.
base.extraMetadata = Object.assign({}, base.extraMetadata, { version: $verJson });
module.exports = base;
"@
    Set-Content -Path $genConfig -Value $genBody -Encoding UTF8

    # --- Make sure the winCodeSign cache won't break extraction ----------
    Initialize-WinCodeSignCache $DesktopDir

    # --- Package the NSIS installer (electron-builder) -------------------
    Step "Packaging NSIS installer (electron-builder)"
    pnpm exec electron-builder --win nsis --$Arch --config .eb-win.generated.cjs
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nelectron-builder failed -- see the output above." -ForegroundColor Red
        Write-Host "If it's a winCodeSign / symlink-privilege error, delete the cache and rerun:" -ForegroundColor Yellow
        Write-Host '  Remove-Item "$env:LOCALAPPDATA\electron-builder\Cache\winCodeSign" -Recurse -Force'
        Write-Host "If it's a missing-module error, rerun once WITHOUT -SkipInstall." -ForegroundColor Yellow
        exit 1
    }
}
finally {
    foreach ($tmp in @('.eb-win.generated.cjs', '.eb-installer.generated.nsh')) {
        $p = Join-Path $DesktopDir $tmp
        if (Test-Path $p) { Remove-Item $p -Force -ErrorAction SilentlyContinue }
    }
    Pop-Location
}

# --- Report output -------------------------------------------------------
$OutDir = Join-Path $DesktopDir 'dist-electron'
Step "Build complete"
$installer = Get-ChildItem -Path $OutDir -Filter '*.exe' -ErrorAction SilentlyContinue |
             Where-Object { $_.Name -notlike '*Setup*Squirrel*' } |
             Sort-Object LastWriteTime -Descending | Select-Object -First 1

if ($installer) {
    Write-Host "`nInstaller ready:" -ForegroundColor Green
    Write-Host "  $($installer.FullName)"
    Write-Host ("  size: {0:N1} MB" -f ($installer.Length / 1MB))
    if ($DefaultServer) {
        Write-Host "`nHand this .exe to friends -- on install it auto-connects to:" -ForegroundColor Green
        Write-Host "  $DefaultServer"
        Write-Host "(They can still switch later with Switch-FluxerInstance.ps1.)" -ForegroundColor DarkGray
    } else {
        Write-Host "`nHand this .exe to friends. After install, run Switch-FluxerInstance.ps1" -ForegroundColor Green
        Write-Host "to point it at your server (it defaults to web.fluxer.app otherwise)."
        Write-Host "Tip: rebuild with -DefaultServer <url> to bake the server into the installer." -ForegroundColor DarkGray
    }
} else {
    Write-Host "`nBuild finished but no .exe found in $OutDir" -ForegroundColor Yellow
    Write-Host "Check the electron-builder output above; artifacts list:"
    Get-ChildItem -Path $OutDir -ErrorAction SilentlyContinue | Select-Object Name, Length | Format-Table -AutoSize
}
