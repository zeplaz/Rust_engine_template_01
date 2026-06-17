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

if ($env:RUST_ENGINE_LG_PRESET_HOOK -ne "0") {
    Write-Host "[orchestrator] landscape grammar preset batch (MCP-LG-VALID-PRESET-001)..."
    $McpPy = Join-Path $RepoRoot "tools\mcp\python"
    $env:PYTHONPATH = $McpPy
    python -m rust_engine_mcp.cli landscape-grammar-presets-witness
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

if ($env:RUST_ENGINE_WITNESS_HONESTY_HOOK -ne "0") {
    Write-Host "[orchestrator] witness integrity scan (MCP-WIT-023)..."
    $McpPy = Join-Path $RepoRoot "tools\mcp\python"
    $env:PYTHONPATH = $McpPy
    python (Join-Path $RepoRoot "tools\orchestrator\scripts\witness_honesty_lib.py") run-hook
    $witExit = $LASTEXITCODE
    if ($witExit -ne 0 -and $env:RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE -eq "1") {
        exit $witExit
    }
}
