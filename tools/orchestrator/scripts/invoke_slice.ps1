# Start implementation from continuation_queue.json (output of plan_slice).
# Usage: .\tools\orchestrator\scripts\invoke_slice.ps1
#        .\tools\orchestrator\scripts\invoke_slice.ps1 -SliceId SLICE-TRIAGE-VM-06

param(
    [string] $SliceId = "",
    [int] $Priority = 1
)

$ErrorActionPreference = "Stop"
$orch = Split-Path $PSScriptRoot -Parent
$root = Split-Path $orch -Parent
$queuePath = Join-Path $root "tools/orchestrator/queues/continuation_queue.json"
if (-not (Test-Path $queuePath)) {
    Write-Error "Missing $queuePath — run plan_slice.ps1 first"
}
$queue = Get-Content $queuePath -Raw | ConvertFrom-Json
if ($queue.Count -eq 0) {
    Write-Error "continuation_queue.json is empty — run plan_slice.ps1"
}
$slice = $null
if ($SliceId) {
    $slice = $queue | Where-Object { $_.id -eq $SliceId } | Select-Object -First 1
    if (-not $slice) { Write-Error "No slice with id $SliceId" }
} else {
    $slice = $queue | Sort-Object { [int]$_.priority } | Select-Object -First 1
}
$lane = switch -Regex ($slice.lane) {
    "VM" { "VM" }
    "Fire" { "Other" }
    "Stage5" { "Stage5" }
    "Ops" { "Other" }
    default { "Other" }
}
$cmd = Join-Path $PSScriptRoot "invoke_handoff.ps1"
$next = ($slice.commands | Select-Object -First 1)
if (-not $next) { $next = "cargo test -p proc_A_dine01 --lib" }
& $cmd -Goal $slice.title -Lane $lane -Owner "@$($slice.agent)" -Witness $slice.witness -NextAction $next
Write-Host "Slice: $($slice.id) track=$($slice.track) playbook=$($slice.playbook)"
