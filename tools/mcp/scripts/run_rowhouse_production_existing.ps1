# Continue rowhouse production on EXISTING staging PNGs (no new geometry / no headless keyframe).
# Typical upstream: pt2_production_ortho_seed.py or prior tile-batch-run staging folder.
# Ship path for artists remains keyframe_render -> tile-atlas-pack; this script is register/pack only.

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot\..\..\..
$env:RUST_ENGINE_TILE_DRY_RUN = "0"

$Batch = "tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json"
$Staging = "assets/staging/tiles/tile_rowhouse_victorian_production_v1"

if (-not (Test-Path $Staging)) {
    Write-Error "Missing staging folder: $Staging (run pt2_production_ortho_seed.py or export keyframe stills first)"
}

Write-Host "=== tile-batch-run (pack + register from existing staging) ==="
python -m rust_engine_mcp.cli tile-batch-run $Batch
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "=== index + production bake witnesses ==="
python tools/mcp/scripts/mcp_prod_tile_index_finalize.py
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Done. Atlas: assets/textures/buildings_iso/production/rowhouse_victorian_production_v1_atlas.png"
