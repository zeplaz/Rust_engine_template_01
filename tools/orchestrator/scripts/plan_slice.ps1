# Refresh development plan from witnesses + triage boards.
# Usage: .\tools\orchestrator\scripts\plan_slice.ps1
#        .\tools\orchestrator\scripts\plan_slice.ps1 -Top 3 -SkipCargo

param(
    [int] $Top = 5,
    [switch] $SkipCargo
)

$ErrorActionPreference = "Stop"
$orch = Split-Path $PSScriptRoot -Parent
$root = Split-Path $orch -Parent
$args = @("--manifest-path", (Join-Path $root "tools/orchestrator/Cargo.toml"), "--", "--plan-slice", "--plan-slice-top", "$Top")
if ($SkipCargo) { $args += "--skip-cargo" }
Push-Location $root
try {
    cargo @args
    Write-Host ""
    Write-Host "Report: tools/orchestrator/reports/plan_slice.md"
    Write-Host "Queue:  tools/orchestrator/queues/continuation_queue.json"
    Write-Host "Index:  src/dev/development_plan_index.md"
} finally {
    Pop-Location
}
