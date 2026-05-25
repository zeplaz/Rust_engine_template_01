# N-11: Run FULL_APP visual proof (requires GPU window).
# Output: debug_runs/stage5_full_app_live.json
$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
Set-Location $RepoRoot

$env:STAGE5_VERBOSE = "1"
Write-Host "[visual] cargo run -- --test visual (proof -> debug_runs/stage5_full_app_live.json)"
cargo run -- --test visual
exit $LASTEXITCODE
