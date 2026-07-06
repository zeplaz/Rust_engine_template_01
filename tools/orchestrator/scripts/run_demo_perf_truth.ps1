# GPU perf-truth baseline — release demo, quiet terminal, disk witnesses only (P3-D).
# See: src/dev/visual_test_runbook_v1.md § Perf truth sign-off
param(
    [int]$Seconds = 60
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
Set-Location $repoRoot

$clear = @(
    "PERF", "STALL", "STALL_SPAN_DEBUG", "PERF_DISK", "SIM_ANALYTICS", "SIM_ANALYTICS_QUIET",
    "UI_LAYOUT_DEBUG", "STAGE5_VERBOSE", "STAGE5_PER_FRAME_HOOKS", "STAGE5_READINESS_VERBOSE",
    "VISUAL_DIAG", "STREAM_DIAG", "SIM_VIEW_SYNC_DEBUG", "VIEWPORT_DEBUG_OVERLAY",
    "WORLDGEN_CHROME_DEBUG", "STAGE5_FENCE_VERBOSE", "VIEW_RUNTIME_AUDIT", "MINIMAP_GPU_DEBUG",
    "TACTICAL_VFX_PROOF", "PERF_NO_VSYNC", "TERRAIN_CPU_FALLBACK", "TERRAIN_GPU_INSTANCED"
)
foreach ($name in $clear) {
    Remove-Item "Env:$name" -ErrorAction SilentlyContinue
}
foreach ($entry in Get-ChildItem Env:RASTER_* -ErrorAction SilentlyContinue) {
    Remove-Item "Env:$($entry.Name)" -ErrorAction SilentlyContinue
}

$env:RUST_LOG = "warn,error"
Write-Host "GPU perf-truth: release demo, quiet disk witnesses (~${Seconds}s then read debug_runs/)"
Write-Host "cwd=$repoRoot"
Write-Host "cargo run -p proc_A_dine01 --release -- --test demo --stay-open"
& cargo run -p proc_A_dine01 --release -- --test demo --stay-open
