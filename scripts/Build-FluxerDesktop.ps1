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

# --- Locate paths --------------------------------------------------------
$RepoRoot   = Split-Path -Parent $PSScriptRoot          # scripts\ -> repo root
$DesktopDir = Join-Path $RepoRoot 'fluxer_desktop'
if (-not (Test-Path $DesktopDir)) { Fail "Can't find fluxer_desktop at $DesktopDir" }

$channel = if ($Canary) { 'canary' } else { 'stable' }
$product = if ($Canary) { 'Fluxer Canary' } else { 'Fluxer' }

Write-Host "Fluxer desktop build" -ForegroundColor Green
Write-Host "  repo:    $RepoRoot"
Write-Host "  channel: $channel"
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
    $genBody = @"
const base = require('./electron-builder.config.cjs');
delete base.linux; // Windows-only build; upstream linux block fails eb26 schema
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
    $tmp = Join-Path $DesktopDir '.eb-win.generated.cjs'
    if (Test-Path $tmp) { Remove-Item $tmp -Force -ErrorAction SilentlyContinue }
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
    Write-Host "`nHand this .exe to friends. After install, run Switch-FluxerInstance.ps1" -ForegroundColor Green
    Write-Host "to point it at your server (it defaults to web.fluxer.app otherwise)."
} else {
    Write-Host "`nBuild finished but no .exe found in $OutDir" -ForegroundColor Yellow
    Write-Host "Check the electron-builder output above; artifacts list:"
    Get-ChildItem -Path $OutDir -ErrorAction SilentlyContinue | Select-Object Name, Length | Format-Table -AutoSize
}
