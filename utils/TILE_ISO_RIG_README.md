# Tile iso rig (`Tile_iso_rig_v1.blend`)

**What it is:** camera, lights, and keyframe targets only — **no building meshes, no civ truck reference geometry**.

**What it is not:** a saved assembly. Module GLBs live in `assets/staging/assemblies/<id>.blend` under collection `ASSEMBLY` only.

## Rebuild

```powershell
python -m rust_engine_mcp.cli build-iso-rig
```

Extracts camera/light keyframes from legacy `Light_keysshotsetup.blend` when present; otherwise builds a minimal procedural rig.

## Manual ship stills

1. Open clean assembly blend (or import GLBs into `ASSEMBLY`).
2. File → Append → `utils/Tile_iso_rig_v1.blend` → collection `TILE_ISO_RIG` (or let bake append automatically).
3. Assign materials to modules.
4. Run `keyframe_render.py` addon → PNG folder → `tile-atlas-pack`.

## Cleanup bad staging blends

```powershell
python tools/mcp/scripts/cleanup_assembly_blends.py
```

Deletes `*.blend` / `*.blend1` under `assets/staging/assemblies/` that incorrectly embedded the old full rig, then rebuilds from `*.json` snapshots.
