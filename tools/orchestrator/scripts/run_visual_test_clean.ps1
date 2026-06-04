# Clean env + optional stall probes for: cargo run -p proc_A_dine01 -- --test visual --stay-open
param(
    [switch]$Release,
    [switch]$StallDebug,
    [switch]$QuietLog
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
Set-Location $repoRoot

$clear = @(
    "UI_LAYOUT_DEBUG",
    "STAGE5_VERBOSE",
    "STAGE5_PER_FRAME_HOOKS",
    "STAGE5_READINESS_VERBOSE",
    "VISUAL_DIAG",
    "STREAM_DIAG",
    "SIM_VIEW_SYNC_DEBUG",
    "VIEWPORT_DEBUG_OVERLAY",
    "WORLDGEN_CHROME_DEBUG",
    "STAGE5_FENCE_VERBOSE",
    "VIEW_RUNTIME_AUDIT",
    "MINIMAP_GPU_DEBUG",
    "TACTICAL_VFX_PROOF",
    "PERF_NO_VSYNC",
    "RASTER_MINIMAP",
    "RASTER_CHUNKS_PER_FRAME"
)
foreach ($name in $clear) {
    Remove-Item "Env:$name" -ErrorAction SilentlyContinue
}

# PERF-VIS-001-P1BC — clean run must not carry RASTER_* overrides.
$rasterEnv = Get-ChildItem Env:RASTER_* -ErrorAction SilentlyContinue
if ($rasterEnv) {
    $names = ($rasterEnv | ForEach-Object { $_.Name }) -join ", "
    throw "RASTER_* must be unset for clean visual run (found: $names)"
}
# GPU minimap compositor defaults on when unset (see minimap_gpu_compositor_env_enabled).
Remove-Item Env:MINIMAP_GPU_COMPOSITOR -ErrorAction SilentlyContinue

if ($StallDebug) {
    $env:PERF = "1"
    $env:STALL = "1"
    $env:STALL_SPAN_DEBUG = "1"
    if (-not $QuietLog) {
        $env:RUST_LOG = "warn,stall=info,perf=info"
    }
} else {
    Remove-Item Env:STALL_SPAN_DEBUG -ErrorAction SilentlyContinue
    Remove-Item Env:STALL -ErrorAction SilentlyContinue
    if (-not $QuietLog) {
        $env:PERF = "1"
        $env:RUST_LOG = "warn,error"
    }
}

$args = @("run", "-p", "proc_A_dine01")
if ($Release) { $args += "--release" }
$args += "--", "--test", "visual", "--stay-open"

Write-Host "cwd=$repoRoot"
Write-Host "cargo $($args -join ' ')"
& cargo @args
