# PLAN-FINAL-GPU-PRESENTATION-v1 — Simulation display is GPU-only

| Field | Value |
|:---|:---|
| **Id** | **PLAN-FINAL-GPU-PRESENTATION-v1** |
| **Status** | **ACTIVE** |
| **Date** | 2026-11-17 |
| **Purpose** | Lock the **ship** presentation pipeline for Simulation: one GPU-bound display path per domain; CPU paint/mesh water and Simulation `CpuRaster` are forbidden. |
| **Related** | [`plan_rpc1_gpu_terrain_ab_v1.md`](plan_rpc1_gpu_terrain_ab_v1.md) · [`plan_dam_flood_benchmark_v1.md`](plan_dam_flood_benchmark_v1.md) DF-3 · [`plan_render_production_cleanup_v1.md`](plan_render_production_cleanup_v1.md) · **stub kill P0** [`plan_gpu_stubs_kill_v1.md`](plan_gpu_stubs_kill_v1.md) · **destruction look** [`plan_destruction_presentation_v1.md`](plan_destruction_presentation_v1.md) (**DESTROY-T2** [`plan_destroy_t2_mask_v1.md`](plan_destroy_t2_mask_v1.md)) |

## Locked ship contract (Simulation)

```text
Terrain pixels  → GpuBake world texture (atlas→world bake ON by default) → GPU sprite / PixelWorld ground sample
Water pixels    → SurfaceWaterGpuField (static depth ⊕ ROI) → water_field.wgsl (2D) + water_field_3d.wgsl (PixelWorld)
Fire / weather  → live Core2d GPU overlays (never baked into terrain)
Particles       → gpu_* particle draws only
UI / HUD        → Bevy UI (unchanged)
```

**Forbidden in Simulation play:**

- `TerrainRenderAuthority::CpuRaster` as display authority
- CPU water paint into `tile_world_fallback` RGBA (`apply_water_surface_overlay_subregion`)
- PixelWorld CPU water `MeshBuf` quads
- Dual “catalog CPU mirror” of water motion when the GPU field is wired

**Editor** may keep `CpuRaster` for live paint tools.

**Honest note:** atlas→world bake is host scissor write today (RPC-1-004/005); display authority flips to that Image (RPC-1-006). True GPU RTT = **Upgrade A**; scorch companion = **B**; veg instances = **C**; voxel water VS displace = **D**. **Delete** `terrain_instanced_draw` / retire `GpuTilemap` — see [`plan_gpu_stubs_kill_v1.md`](plan_gpu_stubs_kill_v1.md).

## PixelWorld

- Ground columns stay MeshBuf (geometry), textured from the **same** world bake / raster Image the flat view uses.
- Water surface is **only** the GPU field shader (no translucent CPU quads).
- Pick / focus height may probe depth on CPU from static+ROI books — not from a water mesh.

## Exit

- Flat + voxel water match (one texture, two shaders).
- Minimap / tactical terrain_source = `gpu_bake` when bake ready (default).
- `TERRAIN_CPU_FALLBACK` ignored in Simulation (editor only).
- `TERRAIN_GPU_BAKE_SPIKE=0` opt-out for diagnose; default ON.
- validate-report cargo green; DamFlood eye check both views.
