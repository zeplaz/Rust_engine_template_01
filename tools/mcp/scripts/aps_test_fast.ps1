# APS pytest — fast tier (no Tk windows). Full: pass "full" as first arg.
param(
    [ValidateSet("fast", "full", "gui")]
    [string]$Tier = "fast"
)

$ErrorActionPreference = "Stop"
$env:APS_TEST_HEADLESS = "1"
$env:RUST_ENGINE_BEVY_PREVIEW = "0"
$pyRoot = Join-Path $PSScriptRoot "..\python"
Set-Location $pyRoot

switch ($Tier) {
    "gui"  { $expr = "aps_gui" }
    "full" { $expr = "aps and not e0_e2_relaunch" }
    default { $expr = "aps and not aps_gui and not e0_e2_relaunch" }
}

python -m pytest tests/ -k $expr -q --tb=short
