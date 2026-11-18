# DESTROY-T2-MASK-v1 — same-frame GPU destruction mask

| Field | Value |
|:---|:---|
| **Id** | **DESTROY-T2-MASK-v1** |
| **Program** | [`plan_destruction_presentation_v1.md`](plan_destruction_presentation_v1.md) |
| **Status** | **ADOPTED** — mask writers + ground material live (validate Accept) |
| **Date** | 2026-11-17 |
| **Owner** | `@coder` (render + construction callers) |
| **Parents** | PLAN-DESTROY-PRESENTATION-v1 · [`plan_final_gpu_presentation_v1.md`](plan_final_gpu_presentation_v1.md) · PHYS-08 |
| **Books** | PHYS-08 (destructible) · presentation consumer of occupation (not a new sim authority) |

## Objective

When the CPU clears or damages world columns (demolish, blast crater, dam collapse), PixelWorld **hides** that geometry on the **same render frame**, without waiting for `CHUNKS_PER_FRAME` remesh.

## Draft charter corrections

| Rejected draft | This repo |
|:---|:---|
| `src/simulation/destruction/occupancy.rs` | No `src/simulation/` — CPU book under construction/strategic presentation mirror; GPU under `src/render/pixel_world/` |
| `src/render/pixel_world/pipeline.rs` | No such pipeline module today |
| `assets/shaders/pixel_world.wgsl` + `voxel_coord` | Terrain uses [`StandardMaterial`](../render/pixel_world/terrain.rs) + [`MeshBuf`](../render/pixel_world/terrain.rs) (pos / normal / uv / color only) |
| Sync GPU copy as sim authority | Mask is **presentation-only**; sim truth stays occupation / demolish / blast |

## Chosen design

### 1. CPU book — `DestructionMaskBook`

- World-size `u8` grid (tile XY): `0` intact · `1` damaged · `2` destroyed.
- **Not** save authority; rebuild/clear with field revision or explicit writers.
- **Writers (after** existing occupation clears): demolish / blast / dam collapse paths.
- Dirty rect list for tiny GPU uploads.

Suggested location: `src/render/pixel_world/destruction_mask.rs` (presentation) **or** thin resource in construction that render extracts — prefer **render-owned book** filled by systems that observe demolish/blast/dam events so construction funnel stays one writer for occupation.

### 2. GPU mirror — `DestructionMaskGpu`

- `Image` as `R8Uint` (preferred) or packed `R8Unorm`.
- Resource: `{ image: Handle<Image>, revision: u64, … }`.
- Upload dirty rects only (near-zero bandwidth vs full remesh).

### 3. Material — replace ground `StandardMaterial`

| Piece | Path |
|:---|:---|
| Rust | `src/render/pixel_world/ground_material.rs` — `PixelWorldGroundMaterial` (`AsBindGroup` / Bevy `Material`) |
| WGSL | `assets/shaders/pixel_world/ground_mask.wgsl` |

Bindings:

- Albedo / bake raster (same Image tactical + PixelWorld already share).
- Destruction mask texture.
- Uniform: world width/height (and origin if needed) so VS maps `world.xz` → tile.

Vertex logic (no `voxel_coord` attribute):

```wgsl
// Pseudocode — production sample must exist at file:line for ADOPTED
let tile = vec2<i32>(floor(world_x), floor(world_z_mapped));
let state = textureLoad(destruction_mask, tile, 0).r;
if (state >= 2u) {
    position.y = skirt_or_below; // collapse / bury destroyed column
}
```

Wire [`sync_pixel_world_terrain`](../render/pixel_world/terrain.rs) to spawn `MeshMaterial3d` with this material instead of `StandardMaterial`.

### 4. Scope for Accept (T2)

| In | Out |
|:---|:---|
| PixelWorld **ground** chunks | Site/massing columns ([`layers.rs`](../render/pixel_world/layers.rs)) → **T2.1** |
| Demolish + blast + dam collapse → mask `2` | STRUCT-ROLE / hydro pressure / staging ring |
| Remesh unchanged (`CHUNKS_PER_FRAME=48`) | Task-pool / tokio remesh |

### 5. Remesh vs mask clear rule (locked)

**Keep mask cell `destroyed` until** the terrain field revision rebuild (or the chunk remesh that regenerates that column from height/occupation that already excludes the geometry). Do **not** clear the mask on upload alone — that would resurrect ghost mesh before remesh finishes.

Optional **T2b** (separate slice): push destroyed chunk coords to the **front** of `TerrainLayerState.pending`.

### 6. Production callers (ADOPTED gate)

Must cite real `file:line` after impl:

| Event | Hook |
|:---|:---|
| Demolish | [`demolish.rs:233`](../construction/demolish.rs) `mark_destroyed_tiles` in `execute_demolish_at_tile` |
| Blast crater | [`blast.rs:175`](../construction/blast.rs) crater disk → `MASK_DESTROYED` |
| Dam collapse | [`dam_barrier.rs:295`](../construction/dam_barrier.rs) first Collapsed → footprint mask |
| Ground material | [`terrain.rs:270`](../render/pixel_world/terrain.rs) `PixelWorldGroundMaterial` + [`ground_mask.wgsl`](../../assets/shaders/pixel_world/ground_mask.wgsl) VS collapse |

## Accept

| Check | Pass |
|:---|:---|
| Lib | ✅ `--lib destroy_t2` 3 passed (book + demolish fixture + shader honesty) |
| Shader | ✅ `destruction_mask` + `state * 255.0 >= 1.5` → skirt Y |
| Eye | ⏳ playcheck / manual PixelWorld (see gpu-playcheck-eye) |
| Determinism | ✅ mask downstream of demolish/blast/dam; sim occupation unchanged |
| Ledger | ✅ ADOPTED with production `file:line` above |

## Exit predicates

```text
ADOPTED  = mask writers + ground material sample live + Accept above
UNWIRED  = types/shader exist but no production demolish/blast/dam caller
BLOCKED  = Bevy Material bind or Image format blocks same-frame upload (document API gap)
```

## File checklist (implementation)

| Action | Path |
|:---|:---|
| Add | `src/render/pixel_world/destruction_mask.rs` |
| Add | `src/render/pixel_world/ground_material.rs` |
| Add | `assets/shaders/pixel_world/ground_mask.wgsl` |
| Edit | `src/render/pixel_world/terrain.rs` — material swap + plugin register |
| Edit | `src/render/pixel_world/mod.rs` — mod exports |
| Edit | `src/construction/demolish.rs` / `blast.rs` / `dam_barrier.rs` — mask dirty after occupation clear |
| Test | `--lib` destroy_mask / demolish fixture; shader honesty test |

## Non-goals

- Custom mesh `voxel_coord` channel.
- Full-map BFS connectivity (STRUCT-CASCADE).
- Rigid-body debris.
- Folding water/fire into the land bake.
