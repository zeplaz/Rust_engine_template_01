# TILE-FIX Phase C — @designer-mcp (CLI/MCP parity only; no ad-hoc Python audit)
# Slice: src/dev/mcp_orchestrator_tile_fix_warehouse_slice_v1.md + v2 art_quality gate
# Operator: run AFTER manual PNGs — see src/dev/pilot_grammar_operator_runbook_v1.md

param(
    [string]$StagingFolder = "assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4",
    [string]$BuildingDef = "tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json"
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
Set-Location $RepoRoot

$BDEF = $BuildingDef
$VC = "assets/configs/buildings/visual_config_warehouse_industrial_west_v2.json"
$META = ($StagingFolder.TrimEnd('/')).Replace('\', '/') + "/atlas_meta.json"

Push-Location tools/mcp/python

Write-Host "=== Step 1: validate-report visual_config ==="
python -m rust_engine_mcp.cli validate-report visual_config $VC --compress 3
if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }

Write-Host "=== Step 2: validate-report atlas_meta_v2 ==="
python -m rust_engine_mcp.cli validate-report atlas_meta_v2 $META --compress 3
if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }

Write-Host "=== Step 3: write-tile-fix-10-witness ==="
python -m rust_engine_mcp.cli write-tile-fix-10-witness --building $BDEF
# exit 1 expected until keyframe_manual.export present

Write-Host "=== Step 4: validate-report tile_promotion ==="
python -m rust_engine_mcp.cli validate-report tile_promotion $BDEF --compress 3

Write-Host "=== Step 5: write-tile-fix-designer-g4-witness ==="
python -m rust_engine_mcp.cli write-tile-fix-designer-g4-witness --building $BDEF
$exit = $LASTEXITCODE

Pop-Location
exit $exit
