# Design — GPU RTT bake + 3D water VS displace + pipeline practices

| Field | Value |
|:---|:---|
| **Id** | **DESIGN-GPU-RTT-BAKE-WATER-v1** |
| **Status** | **ACTIVE · authority for GPU-A / GPU-D + Part 3 practices** |
| **Date** | 2026-11-17 |
| **Parent** | [`plan_gpu_stubs_kill_v1.md`](plan_gpu_stubs_kill_v1.md) · [`plan_final_gpu_presentation_v1.md`](plan_final_gpu_presentation_v1.md) |
| **Implements** | GPU-A / GS-01–02 · GPU-D · GPU-B · GPU-C · GS-09 · GPU-MIPS · GPU-CULL |

**Principle:** Decouple simulation authoring (ECS / logic grid) from rendering (VRAM textures). CPU writes **tiny control / field uploads** only; GPU owns bake pixels and water vertex Y.

---

## 1. Architectural layout — GPU RTT terrain bake

### A. Data layout & buffers

| Resource | Format | Role |
|:---|:---|:---|
| **Control texture** | `R16Uint` (preferred) or `R8Uint` if biome fits elsewhere | One texel / world tile — packed metadata (below) |
| **Dirty-rect buffer** | Uniform or storage: array of `[min_x, min_y, max_x, max_y]` | Active dirty bounds; updated only when chunks change |
| **Material atlas** | `RGBA8Unorm` | Master tile sub-textures; loaded once into VRAM |
| **World bake Image** | `RGBA8Unorm` (+ mips) | RenderTarget output; sole terrain display authority |

**Control texel pack (`R16Uint`):**

```text
bits [0..5]   Material ID     (6)  → atlas cell index (0..63)
bits [6..7]   Road mask       (2)  → 0 = none … blend/tint mode
bits [8..15]  Biome variant   (8)  → atlas / tint variant
```

`TerrainGpuBakeIndexMap` / stamp indices → **control texels**, never host-sampled atlas RGBA into the bake.

### B. Render pipeline flow

```text
[ECS Simulation / Editor Logic]
       │  Writes only tiny dirty subregions of Control Texture
       │  + updates Dirty Rect Uniform/Storage
       ▼
[GPU Control Texture in VRAM]
       │
       ▼
[Custom Bevy Render Node / RenderPass]
       │  Fullscreen quad or instanced rects over dirty bounds
       ├── Bind Group 0: control_texture (u32)
       ├── Bind Group 1: material_atlas + sampler
       ├── Bind Group 2: dirty_rects / atlas layout uniforms
       ▼
[World Bake Image (RenderTarget)]
       │
       ├──► Flat display shader (sprite / Core2d)
       ├──► PixelWorld / voxel ground sample
       └──► Minimap compositor (zero-copy, GS-09)
              └──► GPU-MIPS after bake node
```

Ship: `assets/shaders/terrain/terrain_bake_rtt.wgsl` + RenderApp graph node in bake plugin (replaces host `write_atlas_to_world_bake` authority).

### C. WGSL — GPU RTT bake pass (authority sketch)

```wgsl
@group(0) @binding(0) var control_texture: texture_2d<u32>;
@group(0) @binding(1) var material_atlas: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) control_data: u32,
};

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Prefer flat-interpolated control from VS when drawing dirty rects;
    // textureLoad path shown for compute/fullscreen tile-coord mapping.
    let packed_val = textureLoad(control_texture, vec2<u32>(in.position.xy), 0).r;
    let material_id = packed_val & 0x3Fu;
    let road_mask = (packed_val >> 6u) & 0x3u;
    // let biome_variant = (packed_val >> 8u) & 0xFFu;

    let atlas_uv = compute_atlas_uv(material_id, in.uv);
    var color = textureSample(material_atlas, texture_sampler, atlas_uv);

    if (road_mask > 0u) {
        color = mix(color, ROAD_TINT_COLOR, 0.5);
    }
    return color;
}
```

### Honesty (bake)

| Forbidden | Required |
|:---|:---|
| `rtt_pass_wired = true` after host `write_atlas_to_world_bake` | Flag only when Render-graph bake node ran |
| CPU atlas cell sample into bake Image | Control upload + GPU sample only |
| Sim land RGBA paint when bake consumers ready | `tile_world_fallback` land off in Simulation |

---

## 2. Architectural layout — GPU vertex displacement (3D water)

### A. Mesh & buffer layout

| Item | Spec |
|:---|:---|
| **Static chunk grid** | Once at chunk init — e.g. 16×16 or 32×32 verts |
| **Attr 0** | Local position `vec3<f32>` — `(X, 0, Z)` |
| **Attr 1** | UV `vec2<f32>` — `0..1` across chunk |
| **Water field** | Existing DF-3 `RGBA8Unorm` Image — fast upload; bound to water material |
| **Bed source** | Separate GPU bind (see pack note) |

Do **not** rebuild vertex buffers when depth changes.

### Live pack (DF-3 — do not casually break 2D)

[`gpu_field.rs`](../systems/water/gpu_field.rs) shared with `water_field.wgsl`:

| Channel | Live meaning |
|:---|:---|
| **R** | depth01 |
| **GB** | signed flow |
| **A** | foam01 |

Bed is **not** in this tex today (CPU reads [`PixelWorldTerrainField`](../render/pixel_world/field.rs)).

**Pack decision for GPU-D (explicit):**

| Option | Approach | When |
|:---|:---|:---|
| **D0 (default)** | Keep DF-3; bind companion bed/height texture for VS | Prefer — no 2D migration |
| **D1** | Migrate pack to R=bed, G=depth, B=flow (breaks foam/A or folds foam) | Only with coordinated 2D+3D+minimap cutover |

Conceptual sketches that assume R=bed/G=depth describe **intent**; ship code must follow **D0** unless D1 is signed.

### B. Vertex shader layout (`water_field_3d.wgsl`)

```wgsl
#import bevy_pbr::mesh_view_bindings::view
// + model / mesh bindings per Bevy 0.20 Material vertex path

@group(2) @binding(0) var water_field_texture: texture_2d<f32>;
@group(2) @binding(1) var water_sampler: sampler;
// D0: @group(2) @binding(2) var bed_texture: texture_2d<f32>;
// D0: @group(2) @binding(3) var bed_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) depth: f32,
};

@vertex
fn vertex(mesh: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let field_data = textureSampleLevel(water_field_texture, water_sampler, mesh.uv, 0.0);
    // D0: R = depth01 (DF-3)
    let water_depth_m = field_data.r * DEPTH_SCALE_M;
    let bed_height = sample_bed_height(/* bed bind */, mesh.uv);

    var vertex_pos = mesh.position;
    vertex_pos.y = bed_height
        + (water_depth_m / TILE_METRES) * PIXEL_WORLD_HEIGHT_SCALE
        + lift;

    out.world_position = model.model * vec4<f32>(vertex_pos, 1.0);
    out.clip_position = view.view_proj * out.world_position;
    out.uv = mesh.uv;
    out.depth = field_data.r;
    return out;
}
```

Scales must match current CPU path: `DEPTH_SCALE_M`, `TILE_METRES_GAMEPLAY`, `PIXEL_WORLD_HEIGHT_SCALE`.

### Fragment (Part 3.4)

FS owns look: screen-space foam + world-space normal perturbation; VS only displaced position + depth.

### Exit (GPU-D)

- Mesh handle stable across depth updates; only field/bed textures + uniforms change.
- Zero per-frame water vertex realloc/upload in PixelWorld.

---

## 3. Comparative architecture trade-offs

| Pipeline component | Old (CPU hybrid) | Proposed GPU-native | Perf gain | Visual / architectural benefit |
|:---|:---|:---|:---|:---|
| **Terrain land pixels** | CPU atlas sample + host scissor writes into `Image` | GPU fragment/compute RTT from compact control texture | ~95% less CPU tile-paint | Unifies Editor + Sim; eliminates Sim `tile_world_fallback` land |
| **Fire scorch decals** | Paint into terrain raster → chunk re-uploads | Low-res `R8Unorm` scorch accumulation sampled in terrain shaders | Zero land re-upload on fire spread | Smooth continuous scorch, no stutter |
| **3D water geometry** | Rebuild CPU `MeshBuf` every update | Static grid + texture-driven VS displace | Zero per-frame CPU vertex alloc/upload | Dense water grids at 60+ FPS |

---

## 4. Part 3 practices (cross-walk)

| # | Category | Best practice | Todo |
|:---|:---|:---|:---|
| **1** | Bake mips | Hardware/compute mip-chain after GPU RTT | **GPU-MIPS** |
| **2** | Scorch | `R8Unorm` accumulation; sample in display shaders | **GPU-B** |
| **3** | Vegetation | Static quad + instance buffer (± compute indirect) | **GPU-C** |
| **4** | Water shading | VS displace + FS foam/normals | **GPU-D** |
| **5** | Minimap | Bake + water → fullscreen GPU composite | **GS-09** |
| **6** | Cull | AABB storage + compute → `draw_indexed_indirect` | **GPU-CULL** |

---

## 5. Implementation slices (coder)

1. **GPU-A.1** — `R16Uint` control texture + pack helpers + dirty subregion upload.
2. **GPU-A.2** — Dirty-rect uniform/storage + `terrain_bake_rtt.wgsl` RenderNode; retire host write authority.
3. **GPU-A.3** — Sim: land paint off when bake ready; Editor writes control only.
4. **GPU-MIPS** — mipgen after bake node.
5. **GPU-D.1** — Static chunk water grids; stop Y rebuild; choose **D0** bed bind (default).
6. **GPU-D.2** — Custom/Material VS sample in `water_field_3d.wgsl`; FS foam/normals follow.
7. **GPU-B / C / GS-09 / GPU-CULL** — per kill ladder in parent plan.
