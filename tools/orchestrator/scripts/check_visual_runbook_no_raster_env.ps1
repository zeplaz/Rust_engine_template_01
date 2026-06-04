# PERF-VIS-001 / PLAN-VISUAL-PERF-PRODUCTION-EXEC-001
# Fail if the visual runbook "clean run" path requires RASTER_* env vars.
param(
    [string]$RunbookPath = (Join-Path $PSScriptRoot "../../../src/dev/visual_test_runbook_v1.md")
)

$ErrorActionPreference = "Stop"
$path = Resolve-Path $RunbookPath
$lines = Get-Content $path
$start = ($lines | Select-String -SimpleMatch "reset for a clean perf run" | Select-Object -First 1).LineNumber
$end = ($lines | Select-String -SimpleMatch "Debug-only overrides" | Select-Object -First 1).LineNumber
if (-not $start -or -not $end -or $end -le $start) {
    Write-Error "Could not find clean-run section boundaries in $path"
}
$cleanSection = ($lines[($start - 1)..($end - 2)] -join "`n")

$bad = @(
    'RASTER_MINIMAP',
    'RASTER_CHUNKS_PER_FRAME'
)
$found = @()
foreach ($name in $bad) {
    if ($cleanSection -match [regex]::Escape($name)) {
        $found += $name
    }
}

if ($found.Count -gt 0) {
    $msg = "Clean perf run section must not mention required RASTER_* env vars. Found: $($found -join ', ') in $path. Move RASTER_* to debug-only section only."
    Write-Error $msg
}

Write-Host "OK: clean-run section has no RASTER_* requirements ($path)"
