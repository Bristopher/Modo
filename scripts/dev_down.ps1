#requires -Version 7
# Stop the localhost dev stack (keeps data volumes). Add -Wipe to also delete volumes.
param([switch]$Wipe)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
$files = @('-f', 'compose.yaml', '-f', 'compose.localhost.yaml')
if ($Wipe) {
	Write-Host '==> Stopping and WIPING volumes (DB + uploads)...' -ForegroundColor Yellow
	docker compose @files down -v
} else {
	Write-Host '==> Stopping (data kept)...' -ForegroundColor Cyan
	docker compose @files down
}
