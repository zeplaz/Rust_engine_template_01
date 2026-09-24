# OPS witness spine scan — unified index + integrity hook (MCP-WIT-022 / MCP-OPS-REPORT-001).
# Usage:
#   powershell -File tools/orchestrator/scripts/ops_intelligence_scan.ps1
# Enforce fail:
#   $env:RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE = "1"
#   powershell -File tools/orchestrator/scripts/ops_intelligence_scan.ps1 -Enforce
#
# Authority: python -m rust_engine_mcp.cli ops-intelligence-scan (CLI/MCP parity).

param(
    [switch]$Enforce,
    [int]$WindowHours = 168
)

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
Set-Location $RepoRoot

$McpPy = Join-Path $RepoRoot "tools\mcp\python"
$env:PYTHONPATH = $McpPy

$Args = @("-m", "rust_engine_mcp.cli", "ops-intelligence-scan", "--window-hours", "$WindowHours")
if ($Enforce -or $env:RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE -eq "1") {
    $Args += "--enforce"
    $env:RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE = "1"
}

Write-Host "[ops] ops-intelligence-scan (MCP-OPS-REPORT-001)..."
python @Args
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ops] scan failed — exit $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "[ops] done -> unified_witness_index + ops_report + ops_dashboard_live + triage_live + mcp_ops_report_001_live"
exit 0
