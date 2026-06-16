# Operator finish — manual warehouse keyframe stills → pack + marker + Phase C validators
# Run AFTER Blender Part 2 (24 PNGs on disk). See: src/dev/pilot_grammar_operator_runbook_v1.md

param(
    [string]$StagingFolder = "assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4",
    [string]$BuildingDef = "tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json"
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
Set-Location $RepoRoot

$Staging = Join-Path $RepoRoot ($StagingFolder -replace '/', '\')
if (-not (Test-Path $Staging)) {
    New-Item -ItemType Directory -Path $Staging -Force | Out-Null
}

$States = @('clean_day', 'clean_night_on', 'damaged_night_on')
$Missing = @()
$Found = 0
foreach ($state in $States) {
    for ($f = 0; $f -lt 8; $f++) {
        $name = "${state}_f$f.png"
        $path = Join-Path $Staging $name
        if (Test-Path $path) { $Found++ } else { $Missing += $name }
    }
}

Write-Host "=== Manual still check: $Found / 24 in $StagingFolder ==="
if ($Missing.Count -gt 0) {
    Write-Host "Missing PNGs (first 12):"
    $Missing | Select-Object -First 12 | ForEach-Object { Write-Host "  $_" }
    Write-Error @"
Stop: need 24 PNGs named clean_day_f0..f7, clean_night_on_f0..f7, damaged_night_on_f0..f7
See src/dev/pilot_grammar_operator_runbook_v1.md Part 2 (Blender) then re-run this script.
"@
}

# Remove fake headless marker if present
$BadMarker = Join-Path $Staging 'keyframe_manual.export'
if (Test-Path $BadMarker) {
    try {
        $body = Get-Content $BadMarker -Raw | ConvertFrom-Json
        if ($body.method -eq 'blender_keyframe_light_rig') {
            Remove-Item $BadMarker -Force
            Write-Host "Removed headless keyframe_manual.export marker"
        }
    } catch { }
}

$ExportedAt = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
$Marker = @{
    export_mode = 'keyframe_manual'
    method      = 'keyframe_render.py'
    exported_at = $ExportedAt
    operator    = 'manual'
    cell_count  = 24
    states      = $States
    facings     = 8
    assembly_id = 'industrial_west_7x5_s39_9fa1'
    notes       = 'Warehouse PILOT Track B — not headless tile_keyframe_bake'
} | ConvertTo-Json -Depth 4
$Marker | Set-Content -Path (Join-Path $Staging 'keyframe_manual.export') -Encoding utf8
Write-Host "Wrote keyframe_manual.export"

Push-Location tools/mcp/python

Write-Host "=== tile-atlas-pack (tilemapgen -pk) ==="
python -m rust_engine_mcp.cli tile-atlas-pack "$Staging" --keyframe-rename
if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }

Pop-Location

Write-Host "=== Phase C validators (designer_mcp_warehouse_phase_c.ps1) ==="
& (Join-Path $PSScriptRoot 'designer_mcp_warehouse_phase_c.ps1') -StagingFolder $StagingFolder -BuildingDef $BuildingDef
exit $LASTEXITCODE
