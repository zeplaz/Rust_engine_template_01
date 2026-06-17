# CI entry: orchestrate after check (fast path).
param(
    [switch]$WitnessIntegrity
)

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
Set-Location $RepoRoot
cargo check -p proc_A_dine01
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& (Join-Path $RepoRoot "tools\orchestrator\scripts\check_live_proof_containment.ps1") -HardFail
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& (Join-Path $RepoRoot "tools\orchestrator\scripts\check_visual_runbook_no_raster_env.ps1")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo rustc -p proc_A_dine01 --lib -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test -p proc_A_dine01 --lib stage5 -- --nocapture
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$RunWitnessIntegrity = $WitnessIntegrity -or ($env:RUST_ENGINE_CI_WITNESS_INTEGRITY -eq "1")
if ($RunWitnessIntegrity) {
    Write-Host "[ci] witness integrity stage (MCP-WIT-043)..."
    $McpPy = Join-Path $RepoRoot "tools\mcp\python"
    Push-Location $McpPy
    python -m pytest tests/test_witness_honesty.py tests/test_queue_integrity.py tests/test_aps_imports.py -q --tb=short
    if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }
    python -m rust_engine_mcp.cli mcp-witness-honesty-validator-witness
    if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }
    python -m rust_engine_mcp.cli queue-integrity-reconcile-witness | Out-Null
    # report-only refresh — repo contradictions expected; pytest is the gate
    Pop-Location
}

cargo run --manifest-path tools/orchestrator/Cargo.toml -- --skip-clippy --skip-test
exit $LASTEXITCODE
