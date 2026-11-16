# Demo — studio massing → PixelWorld ghost

Presentation-only (DR-VOXEL-OCCUPATION). Does **not** write `FootprintMatrix`.

## One-command (checked-in fixture)

From repo root:

```powershell
$env:RUST_ENGINE_STUDIO_MASSING = "fixture"
# Optional: PixelWorld + build mode as you normally enter play.
cargo run -p proc_A_dine01
```

Loads [`assets/configs/buildings/examples/massing_preview_pilot_rail_warehouse_v1.json`](../../assets/configs/buildings/examples/massing_preview_pilot_rail_warehouse_v1.json) once at startup into `StudioMassingOverride`. Ghost sync paints stacked columns in PixelWorld3D.

## Operator export path

```powershell
# APS Massing tab: Export from pilot/grammar → debug_runs/studio_dash/massing_preview.json
# or ParamFrame (no bpy):
# APS Massing tab: Export from ParamFrame → same path (param_frame_rail_warehouse_v1.json)
# or:
cd tools/mcp/python
python -c "from rust_engine_mcp.massing_from_pilot import export_massing_preview_from_pilot; from pathlib import Path; export_massing_preview_from_pilot(Path('../../debug_runs/studio_dash/massing_preview.json'), 'logistics_rail_warehouse_v0', seed=1)"

python -c "from rust_engine_mcp.massing_from_param_frame import export_massing_preview_from_param_frame; from pathlib import Path; export_massing_preview_from_param_frame(Path('../../debug_runs/studio_dash/massing_preview.json'), Path('../schemas/examples/param_frame_rail_warehouse_v1.json'))"

$env:RUST_ENGINE_STUDIO_MASSING = "1"
cargo run -p proc_A_dine01
```

ParamFrame StudioTask (preview HTML + massing JSON, no Blender):

```powershell
cd tools/mcp/python
python -m rust_engine_mcp.cli studio-task-enqueue --kind param_frame --seed 1 --payload tools/mcp/schemas/examples/param_frame_rail_warehouse_v1.json --run
# writes debug_runs/studio_dash/massing_preview.json + <task_id>_param_frame_massing_qc.html
```

## StudioTask QC

```powershell
cd tools/mcp/python
python -m rust_engine_mcp.cli studio-task-enqueue --kind massing_export --seed 1 --payload assets/configs/buildings/examples/massing_preview_pilot_rail_warehouse_v1.json --run
```

## Lib proof (no game window)

```powershell
cargo test -p proc_A_dine01 --lib voxel_massing::tests::checked_in_pilot_fixture_loads_override
```
