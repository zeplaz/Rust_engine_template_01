# Main-thread orchestrator Shift A→B (executable debug + cleanup + sim-grade).
# Writes debug_runs/main_thread_orchestrator_live.json
#
# Usage:
#   .\tools\orchestrator\scripts\main_thread_shift.ps1
#   .\tools\orchestrator\scripts\main_thread_shift.ps1 -FullBuild

param(
    [switch]$FullBuild
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
Set-Location $repoRoot

$args = @("--main-thread-shift")
if (-not $FullBuild) {
    $args += @("--skip-cargo", "--skip-clippy", "--skip-test")
}

Write-Host "[main-thread-shift] cargo orchestrate $($args -join ' ')"
cargo orchestrate @args

$proof = Join-Path $repoRoot "debug_runs\main_thread_orchestrator_live.json"
if (Test-Path $proof) {
    Write-Host "[main-thread-shift] proof: $proof"
    Get-Content $proof -Raw | ConvertFrom-Json | Select-Object profile, ok, highest_severity | Format-List
} else {
    Write-Warning "proof missing: $proof"
    exit 1
}
