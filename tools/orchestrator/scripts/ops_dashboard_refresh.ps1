# Refresh OPS oversight dashboard witness (processes + drift + run_events).
$ErrorActionPreference = 'Stop'
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
Set-Location $RepoRoot
$McpPy = Join-Path $RepoRoot "tools\mcp\python"
$env:PYTHONPATH = $McpPy
$WindowHours = 168
if ($args.Count -ge 1) { $WindowHours = [int]$args[0] }
Write-Host "[ops-dashboard] refresh window=${WindowHours}h ..."
python -m rust_engine_mcp.cli ops-dashboard-refresh --window-hours $WindowHours
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "[ops-dashboard] -> debug_runs/agent_ops/ops_dashboard_live.json"
Write-Host "[ops-dashboard] HTML: tools/orchestrator/dashboard/ops_dashboard.html"
Write-Host "[ops-dashboard] Grafana: tools/orchestrator/dashboard/grafana_ops_overview.json"
