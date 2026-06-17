# Modo-ReleaseInfo.ps1
# Prints a ready-to-paste GitHub release tag / title / notes, stamped with the Fluxer (upstream)
# commit this build is based on, so each Modo release records which Fluxer version it tracks.
#
# Usage (from the Pre-Self-Hosting repo root):
#   .\scripts\Modo-ReleaseInfo.ps1            # defaults to version 0.1.0
#   .\scripts\Modo-ReleaseInfo.ps1 -Version 0.2.0

param([string]$Version = "0.1.0")

# Make sure we know the latest upstream Fluxer state (refactor is the self-host branch).
git fetch upstream --quiet 2>$null

$flx = (git merge-base HEAD upstream/refactor 2>$null).Trim()
if (-not $flx) {
    Write-Error "Could not determine the Fluxer base commit (is the 'upstream' remote set to fluxerapp/fluxer?)."
    exit 1
}
$short = $flx.Substring(0, 10)
$modo  = (git rev-parse --short HEAD).Trim()

Write-Host ""
Write-Host "Tag:    v$Version"                              -ForegroundColor Cyan
Write-Host "Title:  Modo v$Version - Fluxer $short"         -ForegroundColor Cyan
Write-Host ""
Write-Host "Release notes (paste below the rest):"          -ForegroundColor Cyan
Write-Host "----------------------------------------"
Write-Host "Built on Fluxer (refactor branch) @ $short"
Write-Host "https://github.com/fluxerapp/fluxer/commit/$flx"
Write-Host ""
Write-Host "Modo source commit: $modo"
Write-Host "----------------------------------------"
