# MCP-PILOT-GRAMMAR-001 prep - materials-on-assembly (CLI/MCP parity; NOT ship bake)
# Unblocks assembly blend + PBR profiles; manual keyframe_render remains human step.
# Policy: src/dev/mcp_orchestrator_tile_fix_warehouse_slice_v2.md

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
Set-Location $RepoRoot

$StagingSnap = "assets/staging/assemblies/industrial_west_7x5_s39_9fa1.json"
$Blend = "assets/staging/assemblies/industrial_west_7x5_s39_9fa1.blend"

Write-Host "=== 1. Grammar pilot snapshot (P0 green - do not overwrite placement-only) ==="
if (-not (Test-Path $StagingSnap)) {
    Write-Error "Missing grammar pilot snapshot: $StagingSnap - run PILOT-GRAMMAR-E2E-001 first"
}

Push-Location tools/mcp/python

Write-Host "=== 2. Generate shell material PNGs ==="
python -m rust_engine_mcp.cli generate-material-textures --profile steel_panel_01
python -m rust_engine_mcp.cli generate-material-textures --profile roof_metal_01
python -m rust_engine_mcp.cli generate-material-textures --profile brick_red_01
python -m rust_engine_mcp.cli generate-material-textures --profile wood_plank_01

Write-Host "=== 3. assembly-build-run (ASSEMBLY-only blend + PBR materials) ==="
python -m rust_engine_mcp.cli assembly-build-run "$RepoRoot/$StagingSnap"
if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }

Pop-Location

$LogPath = Join-Path $RepoRoot "debug_runs\art_pipeline\asm_industrial_west_7x5_s39_9fa1.log"
if (Test-Path $LogPath) {
    $warns = Select-String -Path $LogPath -Pattern "ASSEMBLY_MATERIAL_WARN" -SimpleMatch
    if ($warns) {
        Write-Host ""
        Write-Host "ERROR: Materials were NOT applied in Blender (see log):" -ForegroundColor Red
        Write-Host "  $LogPath"
        Write-Host "  Fix assembly_import.py ops.export_glb import, then re-run this script."
        Write-Host ""
        exit 1
    }
    $applied = Select-String -Path $LogPath -Pattern "ASSEMBLY_MATERIAL " -SimpleMatch
    if (-not $applied) {
        Write-Host "WARN: No ASSEMBLY_MATERIAL lines in log - open blend and confirm pbr_* materials."
    } else {
        Write-Host "Materials applied in headless build (see ASSEMBLY_MATERIAL lines in log)."
    }
}

Write-Host ""
Write-Host "=== Prep complete ==="
Write-Host "Blend: $Blend"
Write-Host ""
Write-Host "=== Part 2 - Manual keyframe (NOT run by this script) ==="
Write-Host '1. Open blend in Blender (File -> Open -> path above)'
Write-Host '2. Append utils/Tile_iso_rig_v1.blend collection TILE_ISO_RIG (File -> Append)'
Write-Host '3. Install keyframe addon - run ONE of:'
Write-Host "     tools\mcp\scripts\open_keyframe_render.cmd"
Write-Host '   Or: Blender -> Edit -> Preferences -> Add-ons -> Install -> utils/keyframe_render.py -> enable'
Write-Host '4. Output -> Keyframes panel -> Render Keyframes to Images - 24 PNG files'
Write-Host '   Folder: assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4/'
Write-Host "5. Then: tools\mcp\scripts\operator_warehouse_keyframe_finish.cmd"
Write-Host ""
Write-Host "Runbook: src/dev/pilot_grammar_operator_runbook_v1.md"
Write-Host "Do NOT run tile_compile_minimum_bake for ship art."
