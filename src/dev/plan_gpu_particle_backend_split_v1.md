# GPU particle backend / fire VFX frontend split (v1)

## Layer map

| Layer | Path | Authority | Must NOT know |
|-------|------|-----------|---------------|
| **Backend (generic)** | `render/gpu_instanced_quad.rs`, `render/gpu_particle_draw.rs` | Instance stride, expand compute, registry upload, dispatch counts | Fire heat thresholds, chunk LOD bands, scatter policy |
| **Fire frontend** | `render/fire_vfx/` | Projection → instance rows, scatter, LOD shaping, witness | WGSL bind group layout ids |
| **Draw / raster** | `gpu_particle_draw.rs`, `gpu_fire_particle_raster.rs` | Shader dispatch, expanded vertex scratch | Sim fire ecology |
| **WGSL** | `assets/shaders/fire/fire_particle.wgsl`, `fire_particle_draw.wgsl` | Expand + fragment read | Rust witness gates |

Schedule slot: `FireVisualFrameSet::ProjectGpu` in `fire_visual_extract.rs` (unchanged).

## Stable IDs — do not rename until second particle consumer (R7/D3)

| ID | Value | Breaks if renamed |
|----|-------|-------------------|
| `FIRE_PARTICLE_INSTANCES_BUFFER` | `BufferId(3)` | registry, WGSL group 1, tests |
| `FIRE_PARTICLE_EXPANDED_VERTICES_BUFFER` | `BufferId(6)` | expand pass, raster |
| `FIRE_PARTICLE_INSTANCE_FORMAT` | `PackedFormatId(3)` | `gpu_packed_formats` |
| `WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP` | `BindGroupId(2)` | render graph node |
| `WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP` | `BindGroupId(3)` | render graph node |

Instance byte stride: **32** (`GpuInstancedQuadInstance` / `GpuParticleInstance` alias).

## Phases (no big-bang)

0. This doc + layer boundaries  
1. Generic instance + uniform view types (`gpu_instanced_quad.rs`)  
2. Extract fire domain → `fire_vfx/`; `gpu_particles.rs` = facade  
3. `InstancedParticleExpandPipeline` config (shader from handle)  
4. Witness + FX-FIRE-SPARK gates → `fire_vfx/witness.rs`  
5. WGSL struct comments / uniform field ownership  
6. Tests + stage5 witness refresh  

### RTT operator lane — Track B (Item 5 + 2a) — **lib SHIPPED 2026-07-04**

**Hub:** [`plan_tactical_map_rtt_lane_v1.md`](plan_tactical_map_rtt_lane_v1.md) · HANDOFF § PLAN-TACTICAL-MAP-RTT-v1 · witness `debug_runs/rtt_lane_witness_live.json`

| ID | Maps to phase | Action |
|----|---------------|--------|
| RTT-B5-001 | **2a** | Shared `ViewUniform` / `ParticleViewGlobals` from `ExtractedCameraMetrics` |
| RTT-B5-002 | **5** | Fire: `fire_particle_draw.wgsl` + raster use extracted view only |
| RTT-B5-003 | **5** | Water: same View uniform pattern |
| RTT-B5-004 | **6** | `--test vfx` + stage5 witness refresh |

**Exit:** sparks/precip visible on tactical map RTT; no `MainWorldCamera` query in raster sync.

**Deferred:** R7 weather/precip generic spine — gated on D3 baseline (`plan_cleanup_v1.md`).

## Forbidden

- Renaming `FIRE_*` buffers before a second instanced-quad consumer ships  
- Second fire extract path (spark compute reads existing frame only)  
- Moving witness gates into draw/backend layers  
