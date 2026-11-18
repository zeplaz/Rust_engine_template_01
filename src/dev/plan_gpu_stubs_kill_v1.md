# PLAN-GPU-STUBS-KILL-v1 — Priority GPU upgrades (host pixel & mesh debt)

| Field | Value |
|:---|:---|
| **Id** | **PLAN-GPU-STUBS-KILL-v1** |
| **Status** | **ACTIVE · P0 / HIGH** |
| **Date** | 2026-11-17 |
| **Trigger** | Operator: finish GPU stubs + priority GPU upgrades (eliminate host pixel & mesh debt) |
| **Parent** | [`plan_final_gpu_presentation_v1.md`](plan_final_gpu_presentation_v1.md) · [`plan_rpc1_gpu_terrain_ab_v1.md`](plan_rpc1_gpu_terrain_ab_v1.md) · [`plan_render_production_cleanup_v1.md`](plan_render_production_cleanup_v1.md) |
| **Pick board** | [`gpu_todos_v1.md`](gpu_todos_v1.md) § PLAN-GPU-STUBS-KILL |

## Locked architecture decision

**Sole terrain display authority = single world bake texture** (flat sprite + PixelWorld ground sample + minimap). Arbitrary blending, roads, and scorch live as GPU samples on that bake / companion textures — **not** discrete tile quads.

| Was | Decision |
|:---|:---|
| Flip `terrain_instanced_draw` ON | **DELETE** the dormant pass (`GS-03`) |
| Keep `GpuTilemap` gated (`DR-MIG-TILEMAP`) | **RETIRE** variant + close deferral as product-wontfix (`GS-04`) |
| Host atlas scissor “RTT” | **REPLACE** with true GPU RTT/compute bake (`GS-01` / **Upgrade A**) |

Do **not** reintroduce a second terrain display path “for safety.”

## Rule

A path that is **named or witnessed as GPU** but still does host-side pixel authorship, rebuilds full meshes per frame, clears instances, writes text stubs, or leaves `rtt_pass_wired` meaning “host write” is **debt**. P0 = eliminate or rename honestly.

---

## 1. Priority GPU upgrades (host pixel & mesh debt)

### A. Terrain land pixels — true GPU RTT bake shader · **P0** (`GS-01` + `GS-02`)

**Authority design:** [`design_gpu_rtt_bake_water_displace_v1.md`](design_gpu_rtt_bake_water_displace_v1.md) Part 1.

| | |
|:---|:---|
| **Now** | [`terrain_gpu_bake_spike`](../render/pipelines/terrain_gpu_bake_spike.rs) samples atlas on CPU + host scissor write; [`tile_world_fallback`](../render/tile_world_fallback.rs) still paints dirty chunks on CPU |
| **Upgrade** | CPU owns **`R16Uint` control** only (`mat6 \| road2 \| biome8`) + dirty-rect buffer. Bevy RenderNode samples atlas → world bake RT. Then **GPU-MIPS**. Layout: design §1. |
| **Payoff** | Zero CPU pixel blending in Simulation; Editor writes the small ID texture + dirty rects only; Sim/Editor share one bake path |
| **Exit** | Host `write_atlas_to_world_bake` diagnose-only or gone; `rtt_pass_wired` means Render-graph pass; Sim land paint off when bake ready |

### B. Fire map tint — decouple scorch from terrain raster · **P0** (`GPU-B`)

| | |
|:---|:---|
| **Now** | [`overlay_heat_scorch_decal01`](../render/fire_vfx/emit.rs) tint applied into terrain raster chunks on CPU ([`map_tile_raster`](../gui/map_tile_raster.rs)) → dirty re-upload on fire spread |
| **Upgrade** | Dedicated low-res **Scorch/Heat Accumulation** texture (`R8Unorm`) via GPU compute or stamped decals; sample **multiplicatively** in terrain display shaders (flat + voxel) |
| **Payoff** | Fire spread does not invalidate terrain land chunks / force CPU re-uploads |
| **Exit** | Scorch no longer written into land RGBA; witness + eye: burn tint without `tile_raster` dirty storm |

### C. Vegetation (flat) — instanced GPU quads · **P0** (`GPU-C`)

| | |
|:---|:---|
| **Now** | [`world_map_vegetation_mesh`](../render/world_map_vegetation_mesh.rs) rebuilds CPU vertex/index geometry from extract rows and uploads full meshes |
| **Upgrade** | Static quad + **GPU instance buffer** (`InstanceData`: pos, scale, rot, atlas UV index, wind phase). Prefer: deterministic placement → **compute** fills instance buffer + `draw_indexed_indirect` (bypass CPU extract) |
| **Exit** | No per-change full mesh rebuild for flat veg; non-zero instance draw in playcheck |

### D. Water (voxel) — GPU vertex displacement · **P0** (`GPU-D`)

**Authority design:** [`design_gpu_rtt_bake_water_displace_v1.md`](design_gpu_rtt_bake_water_displace_v1.md) Part 2. **Keep DF-3 pack** (R=depth, GB=flow, A=foam); bed from separate GPU height bind.

| | |
|:---|:---|
| **Now** | [`WaterFieldMaterial3d`](../render/pixel_world/water_gpu.rs) builds CPU grid mesh every update; FS-only in `water_field_3d.wgsl` (“vertices are pre-displaced”) |
| **Upgrade** | Static high-density grid once; VS samples water field + bed texture → `position.y`; FS keeps foam/normals (Part 3.4) |
| **Payoff** | No per-frame CPU vertex alloc/upload for 3D water; 60+ FPS dense grids |
| **Exit** | Mesh buffers stable across depth updates; only field/bed textures + uniforms change |

---

## 2. High-value architecture + Part 3 practices

Full matrix: [`design_gpu_rtt_bake_water_displace_v1.md`](design_gpu_rtt_bake_water_displace_v1.md) Part 3.

| ID | Pri | Practice | Exit |
|:---|:---:|:---|:---|
| **GS-03 / GS-04** | P0 | **Delete** `terrain_instanced_draw` + retire `GpuTilemap` | Sole bake texture |
| **GPU-MIPS** | P1 · **SHIPPED** | §3.1 Bake mip chain after GPU RTT | No zoom-out shimmer |
| **GPU-B** | P0 | §3.2 Scorch `R8Unorm` companion | Fire ≠ land dirty |
| **GPU-C** | P0 | §3.3 Veg instances / indirect | No full mesh rebuild |
| **GPU-D + FS** | P0 | §3.4 VS displace + FS foam/normals | Dense water, no CPU mesh |
| **GS-09** | P1 · **SHIPPED** | §3.5 Minimap zero-copy (default GPU) | No CpuBridge in play |
| **GPU-CULL** | P2 | §3.6 AABB compute cull → indirect | Large-map scale |
| **GPU-ARRAY** | P1 · **SHIPPED** | Texture2DArray materials → bake RTT; PixelWorld land greedy run-merge | No atlas mip UV bleed |
| **GS-07** | P0 | Retire capture text stubs | Zero text-as-PNG |
| **GS-05** | P0 | Prove water particle draw | DamFlood instances |
| **GS-WITNESS** | P0 | Host bake ≠ `rtt_pass` | Probe fails on lie |
| **GS-06 / GS-08** | P1 · **SHIPPED** | Fire LOD = CPU policy; spark wired; smoke_volume deferred ES-4 | Honest naming |

---

## Stub inventory (cross-walk)

| ID | Stub | Maps to | Pri |
|:---|:---|:---|:---:|
| GS-01 | Host “RTT” bake | **Upgrade A** | P0 |
| GS-02 | CPU land paint under `GpuBake` | **Upgrade A** (Sim eliminate) | P0 |
| GS-03 | Dormant instanced terrain | **§2 DELETE** | P0 |
| GS-04 | `GpuTilemap` never constructed | **§2 RETIRE** | P0 |
| GS-05 | Water particle doc lie | Prove draw | P0 |
| GS-07 | Capture text stubs | Delete/redirect | P0 |
| GPU-B | Scorch in land raster | **Upgrade B** | P0 |
| GPU-C | Veg full-mesh rebuild | **Upgrade C** | P0 |
| GPU-D | Voxel water CPU mesh | **Upgrade D** | P0 |
| GS-09 | Minimap CpuBridge default | **§2 default GPU** | P1 |
| GPU-MIPS / ARRAY / CULL | — | **§2** | P1–P2 |
| GS-06 / GS-08 | Fire stub comments | Rename/finish | P1 |

**Honest keep (not debt):** water field pack→WGSL (2D), fire/weather Core2d particles, Bevy UI. PixelWorld **land** MeshBuf geometry stays until Texture2DArray/greedy (**GPU-ARRAY**); water MeshBuf Y-rebuild is **GPU-D** debt.

## Kill ladder (mandatory)

```text
GS-07 capture stub out
  → A / GS-01 GPU RTT bake (material-id upload + WGSL scissor)
  → A' / GS-02 Sim land paint off
  → B scorch texture decouple
  → GS-03+04 DELETE instanced + retire GpuTilemap
  → D voxel water VS displace
  → C flat veg instances (then optional compute indirect)
  → GS-05 water particles prove
  → GS-WITNESS honesty
  → GS-09 minimap default GPU
  → GPU-MIPS bake mips
  → GPU-ARRAY Texture2DArray + greedy
  → GPU-CULL indirect frustum (scale)
  → GS-06/08 rename/finish
```

## Anti-goals

- Do **not** enable or keep `GpuTilemap` / instanced terrain as a parallel display path.
- Do **not** leave `rtt_pass_wired=true` for host memcpy.
- Do **not** bake fire/weather into land bake indices (scorch = companion texture only).
- Do **not** reintroduce CPU water MeshBuf for PixelWorld after **D**.
