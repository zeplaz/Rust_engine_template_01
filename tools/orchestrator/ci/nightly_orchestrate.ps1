# S-03: Nightly full orchestrate + local history trend.
$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
Set-Location $RepoRoot

$log = "tools/orchestrator/history/nightly_$(Get-Date -Format 'yyyyMMdd_HHmmss').log"
Write-Host "[nightly] full orchestrate -> $log"

cargo orchestrate 2>&1 | Tee-Object -FilePath $log
$code = $LASTEXITCODE

Copy-Item -Force tools/orchestrator/reports/drift_summary.md `
    "tools/orchestrator/history/nightly_drift_latest.md" -ErrorAction SilentlyContinue

exit $code
