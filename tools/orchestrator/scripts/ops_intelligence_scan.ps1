# OPS witness spine scan — unified index + integrity hook (MCP-WIT-022).
# Usage:
#   powershell -File tools/orchestrator/scripts/ops_intelligence_scan.ps1
# Enforce fail:
#   $env:RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE = "1"

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
Set-Location $RepoRoot

$McpPy = Join-Path $RepoRoot "tools\mcp\python"
$env:PYTHONPATH = $McpPy

Write-Host "[ops] witness index..."
python (Join-Path $RepoRoot "tools\orchestrator\scripts\ops_witness_index.py")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[ops] witness integrity hook..."
python (Join-Path $RepoRoot "tools\orchestrator\scripts\witness_honesty_lib.py") run-hook
$hookExit = $LASTEXITCODE

if ($hookExit -ne 0) {
    if ($env:RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE -eq "1") {
        Write-Host "[ops] witness integrity ENFORCE — exit $hookExit" -ForegroundColor Red
        exit $hookExit
    }
    Write-Host "[ops] witness integrity warn-only (set RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE=1 to fail)" -ForegroundColor Yellow
}

Write-Host "[ops] done -> debug_runs/unified_witness_index.json"
exit 0
