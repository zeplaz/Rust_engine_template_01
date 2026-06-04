# MCP-PILOT-GRAMMAR-001 prep — materials-on-assembly (CLI/MCP parity; NOT ship bake)
# Unblocks assembly blend + PBR profiles; manual keyframe_render remains human step.
# Policy: src/dev/mcp_orchestrator_tile_fix_warehouse_slice_v2.md

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
Set-Location $RepoRoot

$ProdSnap = "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json"
$StagingSnap = "assets/staging/assemblies/industrial_west_4x2_s43_a879.json"
$Blend = "assets/staging/assemblies/industrial_west_4x2_s43_a879.blend"

Write-Host "=== 1. Sync production snapshot (material_profile on placements) ==="
Copy-Item -Force $ProdSnap $StagingSnap

Push-Location tools/mcp/python

Write-Host "=== 2. Generate shell material PNGs ==="
python -m rust_engine_mcp.cli generate-material-textures --profile steel_panel_01
python -m rust_engine_mcp.cli generate-material-textures --profile roof_metal_01
python -m rust_engine_mcp.cli generate-material-textures --profile brick_red_01
python -m rust_engine_mcp.cli generate-material-textures --profile wood_plank_01

Write-Host "=== 3. assembly-build-run (ASSEMBLY-only blend + materials) ==="
python -m rust_engine_mcp.cli assembly-build-run "$RepoRoot/$StagingSnap"
if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }

Pop-Location

Write-Host "=== Prep complete ==="
Write-Host "Blend: $Blend"
Write-Host "Next (human): Append Tile_iso_rig_v1 -> keyframe_render 24 cells -> tile-atlas-pack -> designer_mcp_warehouse_phase_c.ps1"
Write-Host "Do NOT run tile_compile_minimum_bake for ship art."
