# DamFlood playtest notes (operator)

Captured from live playcheck + eye notes. Track fixes against this list.

## UX / panel

1. **Dam ops card is keyboard-only and unclear.** No knobs/buttons; click-to-focus then keys. Card copy now labels CHANNEL / KEYS / NOW + presets explicitly. Follow-up: real Bevy UI sliders/buttons (designer + coder).

## Rendering (PLAN-FINAL-GPU-PRESENTATION-v1)

2. **Half-map / glitchy voxels.** Root cause: field revision thrash + `CHUNKS_PER_FRAME=12`. Fixed: coarser stamp + 48 chunks/frame.
3. **Water both views = one GPU field.** Composite pack (static ⊕ ROI) → `water_field.wgsl` (flat) + PixelWorld `water_gpu` mesh sampling the same Image. CPU water MeshBuf + CPU raster water paint **removed**.
4. **Terrain display.** Simulation always `GpuBake`; atlas→world bake **default ON** (`TERRAIN_GPU_BAKE_SPIKE=0` to opt out); tactical sprite binds bake when ready. `TERRAIN_CPU_FALLBACK` ignored in Simulation.
5. **Overall 3D jank.** Placeholder site columns / lighting polish still open; iterate water look in `assets/shaders/water/water_field*.wgsl`.

## Combat / scenario

6. **Missile never hit dam; launcher/guys not seen.** Launch pad now `MilitaryBase` with launcher mast placeholder + infantry formation at pad. Strike failsafe no longer latches on pending-only; scenario force-closes turbines during fill; strike queue no longer drops orders if ballistics book is briefly missing.
7. **Matrix reds still expected until re-playcheck:** `s2_downstream_ratio`, `s3_drowned`, `s4_hydro_mw`, `s5_impact_err_m`, `s7_wave`, `s9_burn`, debug perf avg/p95.

## Camera

8. **MMB in voxel world should rotate, not pan.** Landed: map pan ignores MMB in PixelWorld; `pixel_world_mmb_orbit_system` turns yaw ~48px / 90°.

## Pre-playtest checklist (2026-11-17)

- [x] Simulation `GpuBake` only; bake spike default ON; tactical+voxel ground bind bake
- [x] Water GPU field composite (static⊕ROI); flat `water_field.wgsl`; voxel `WaterFieldMaterial3d` + `water_field_3d.wgsl`
- [x] CPU water MeshBuf + CPU raster water paint + motion dirty cadence retired
- [x] Jet foam particles from `SurfaceWaterVisualFrame` flows
- [x] Catalog ribbons skipped inside wired GPU field ROI (field shader owns surface)
- [x] MMB orbit in voxel; MilitaryBase pad + unit placeholders; strike failsafe
- [x] BOOK-S2 GPU-CULL.1: fire frustum sphere cull + `draw_indirect` (RND-02/12 PARTIAL)
- [x] BOOK-S3: veg snapped sample + frustum mesh filter (GPU instance buffer UNWIRED)
- [x] BOOK-S4: water grid zoom LOD (`water_grid_step_for_zoom`)
- [ ] Operator: `python tools/sim_review/playcheck.py` + eye both views (flat / `v`); sparks off-frustum; veg pan; water LOD

```powershell
python tools/sim_review/playcheck.py
```
