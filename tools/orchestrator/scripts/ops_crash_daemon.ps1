# OPS crash daemon — Python loop (not cron). Polls DCC/process exits, data drops, exports Prometheus.
$ErrorActionPreference = 'Stop'
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
Set-Location $RepoRoot
$McpPy = Join-Path $RepoRoot "tools\mcp\python"
$env:PYTHONPATH = $McpPy

$IntervalSec = 30
if ($args.Count -ge 1) { $IntervalSec = [int]$args[0] }

Write-Host "[ops-crash-daemon] interval=${IntervalSec}s — Ctrl+C to stop"
Write-Host "[ops-crash-daemon] triage -> debug_runs/agent_ops/triage_live.json"
Write-Host "[ops-crash-daemon] prometheus -> debug_runs/agent_ops/prometheus/rust_engine_ops.prom"

python -m rust_engine_mcp.cli ops-crash-daemon --interval-sec $IntervalSec
