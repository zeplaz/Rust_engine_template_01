# ENGINE-DEEP-DEBUG-001 — intrusive debug build for minimap / GPU / schedule recovery.

$ErrorActionPreference = 'Stop'
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
Set-Location $RepoRoot

$Profile = "dev-deep-debug"
$Features = "engine_deep_debug"

Write-Host "[deep-debug] cargo build --profile $Profile --features $Features"
cargo build --profile $Profile --features $Features @args
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$Bin = Join-Path $RepoRoot "target\dev-deep-debug\proc_A_dine01.exe"
Write-Host "[deep-debug] built -> $Bin"
Write-Host "[deep-debug] run: tools/orchestrator/scripts/run_deep_debug.ps1 --test visual --stay-open"
