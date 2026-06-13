#requires -Version 7
# Follow logs for the localhost dev stack.
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
docker compose -f compose.yaml -f compose.localhost.yaml logs -f
