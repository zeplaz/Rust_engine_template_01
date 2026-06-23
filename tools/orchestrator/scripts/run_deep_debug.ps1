# Run ENGINE-DEEP-DEBUG build with full env bundle (minimap/GPU/schedule witnesses).
$ErrorActionPreference = 'Stop'
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
Set-Location $RepoRoot

$Bin = Join-Path $RepoRoot "target\dev-deep-debug\proc_A_dine01.exe"
if (-not (Test-Path $Bin)) {
    Write-Host "[deep-debug] binary missing — building..."
    & (Join-Path $PSScriptRoot "build_deep_debug.ps1")
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$env:RUST_ENGINE_DEEP_DEBUG = "1"
$env:RUST_ENGINE_DEEP_DEBUG_JSONL = "1"
$env:MINIMAP_GPU_DEBUG = "1"
$env:VIEW_RUNTIME_AUDIT = "1"
$env:RUST_LOG = "warn,engine_deep_debug=trace,proc_A_dine01::render::minimap=debug,proc_A_dine01=debug,bevy_render=info,bevy_ecs=warn"

Write-Host "[deep-debug] witnesses -> debug_runs/deep_debug/engine_deep_debug_live.json"
Write-Host "[deep-debug] jsonl     -> debug_runs/deep_debug/engine_deep_debug_frames.jsonl"
Write-Host "[deep-debug] launching $Bin --deep-debug $args"

& $Bin --deep-debug @args
