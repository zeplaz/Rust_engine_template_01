# Optional post-build orchestration hook.
# Usage: $env:RUST_ENGINE_ORCHESTRATE = "1"; cargo check
# Or call directly after check/test.

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
Set-Location $RepoRoot

Write-Host "[orchestrator] running pipeline..."
cargo run --quiet --manifest-path tools/orchestrator/Cargo.toml -- --skip-test
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "[orchestrator] reports -> tools/orchestrator/reports/"
