#requires -Version 7
# Dev loop: build + run the LOCALHOST stack on http://localhost:8080, then follow logs.
# Bound to Ctrl+Shift+B (see .vscode/tasks.json). Forces localhost build args so it works
# regardless of what .env is set to (your Unraid/duckdns settings are left untouched).

$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root

# Force a localhost build/runtime, independent of .env (which may be set for Unraid).
$env:BASE_DOMAIN   = 'localhost'
$env:PUBLIC_SCHEME = 'http'
$env:PUBLIC_PORT   = '8080'

$files = @('-f', 'compose.yaml', '-f', 'compose.localhost.yaml')

function Invoke-Step($desc, [scriptblock]$block) {
	Write-Host "==> $desc" -ForegroundColor Cyan
	& $block
	if ($LASTEXITCODE -ne 0) { Write-Host "FAILED: $desc (exit $LASTEXITCODE)" -ForegroundColor Red; exit $LASTEXITCODE }
}

if (-not (Test-Path config/config.json)) {
	Invoke-Step 'Seeding localhost config (config/config.json)' {
		node scripts/seed_config.mjs config/config.json --from config/config.localhost.template.json
	}
} else {
	Write-Host '==> config/config.json exists — leaving it as-is' -ForegroundColor DarkGray
}

Invoke-Step 'Building images (first run is slow: Rust/WASM + Erlang + SPA)' {
	docker compose @files build fluxer_gateway fluxer_server
}

Invoke-Step 'Starting stack' {
	docker compose @files up -d
}

docker compose @files ps
Write-Host ''
Write-Host '  App:   http://localhost:8080/' -ForegroundColor Green
Write-Host '  Mail:  http://localhost:8080/mailpit/' -ForegroundColor Green
Write-Host '  Disco: http://localhost:8080/.well-known/fluxer' -ForegroundColor Green
Write-Host ''
Write-Host 'Following logs (Ctrl+C stops following — containers keep running)...' -ForegroundColor DarkGray
docker compose @files logs -f fluxer_server fluxer_gateway
