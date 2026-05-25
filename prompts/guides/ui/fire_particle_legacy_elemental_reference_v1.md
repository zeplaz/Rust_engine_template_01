# Legacy elemental particle reference `v1`

**Path (reference machine):**

```
C:\dev\razerz-master\shaderzglsl\elemental\compute_partical\
├── compute_expanse_BASE_A.glsl
├── vertex_partical_BASE_A.glsl
└── frag_partical_BASE_A.glsl
```

**Repo stills:** [`assets/vfx/reference/elemental_sparks/`](../../../assets/vfx/reference/elemental_sparks/)  
**Master plan:** [`fire_particle_spark_design_plan_v1.md`](../../../src/dev/fire_particle_spark_design_plan_v1.md) (**SIGNED**)

---

## GLSL skim summary (2026-05-24)

### `compute_expanse_BASE_A.glsl` — motion + lifetime

| Mechanism | Detail | Port target |
|:---|:---|:---|
| Workgroup | `512` threads, image buffers for pos/vel | Phase B `fire_spark_compute.wgsl` (optional) |
| Integration | `pos.xyz -= val.xyz * dt` | Advection step |
| Lifetime | `pos.w -= 0.0016 * dt`; respawn at `ubo.orgin` when `w ≤ 0`, `w += 3` | Per-instance life in expand or compute |
| Attractors | Loop `i < 24`: `val += normalize(dist) * mass / (dot(dist,dist)*500)` | `FireVisualGpuInstance` centers (D-F03 A) |
| Mass | `partc_attractor[i].w` | Heat / intensity at fire core |

**Do not** duplicate attractor sim in Rust — read existing fire instances.

### `vertex_partical_BASE_A.glsl` — varyings

| Output | Source | Engine |
|:---|:---|:---|
| `age_intensity` | `vert.w` | Normalized lifetime → fragment |
| `pos_intensity` | `vert.xy` | World xy or hash seed for twinkle |
| Position | `mvp[0]*mvp[1]*mvp[2] * vert.xyz` | `globals.view_proj` in draw VS |

### `frag_partical_BASE_A.glsl` — pinpoint look (authority)

| Mechanism | Code | Port (`fire_particle_draw.wgsl`) |
|:---|:---|:---|
| Age color | `mix(vec4(0.112,0.115,0.12,0.8), vec4(0.902,0.27,0.0,0.8), age_intensity)` | D-F04 A + palette tokens D-F06 B |
| Twinkle X | `mix(..., sin_intzy)` on orange channels | D-F05 A |
| Twinkle Y | `sin(pos.x)`, `cos(pos.y)` / tan variant | Simplify to `sin(pos.x)`, `cos(pos.y)` |
| Final | `mix(col_age, col_pos, 0.5)` | Replace `smoothstep` blob |
| Point sprite | Comments reference `gl_PointCoord` | D-F01 A — sharp radial or point list |

**Anti-pattern today:** `distance(uv, 0.5)` + `smoothstep(0.5, 0.08, d)` in `fire_particle_draw.wgsl` → soft blob.

---

## Reference stills (committed)

| File | Maps to |
|:---|:---|
| `legacy_frag_still_01.png` | Frag behavior — pinpoint field |
| `legacy_compute_still_01.png` | Compute — attractor trails |
| `engine_blob_before.png` | Current engine — replace this read |
| `fire_spark_target_v1.png` | **Before / after** design target |

---

## Engine files (Phase A touch list)

| File | Change |
|:---|:---|
| `assets/shaders/fire/fire_particle_draw.wgsl` | Fragment: age + twinkle; sharp falloff |
| `assets/shaders/fire/fire_particle.wgsl` | Reduce `half` expansion; optional life channel |
| `src/render/gpu_particles.rs` | Density / scatter caps (D-F07, D-F09) |

**Out of scope Phase A:** new extract, `fire_spark_compute.wgsl` (Phase B only if D-F02 B needed).

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-24 | GLSL skim table; stills; SIGNED linkage |
| v1.0.0 | 2026-05-24 | Path index only |
