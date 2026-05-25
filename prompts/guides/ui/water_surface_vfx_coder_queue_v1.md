# Water surface VFX — dual @coder queue `v1`

> **Active closure track:** [`src/dev/stages/water_vfx_closure_plan_v1.md`](../../../src/dev/stages/water_vfx_closure_plan_v1.md) — **NOT CLOSED** (first pass landed ≠ design done).  
> **Fire / shared proof:** [`src/dev/vfx_coder_phase2_queue_v1.md`](../../../src/dev/vfx_coder_phase2_queue_v1.md).

| Field | Value |
|:---|:---|
| **Version** | `1.2.0` |
| **Sign-off** | [`../../src/dev/stage_tracks_signoff_ledger_v1.md`](../../src/dev/stage_tracks_signoff_ledger_v1.md) · **WATER-DESIGN-001 SIGNED TUNE** |
| **Date** | 2026-05-24 |
| **Track plan** | [`src/dev/stages/water_vfx_closure_plan_v1.md`](../../../src/dev/stages/water_vfx_closure_plan_v1.md) |
| **Owner** | `@coder` ×2 · [`render_pipeline_agent.md`](../../../tools/orchestrator/agents/render_pipeline_agent.md) |
| **Design gate** | **SIGNED** — [`water_surface_vfx_design_plan_v1.md`](../../../src/dev/water_surface_vfx_design_plan_v1.md) |
| **Worksheet** | [`water_surface_vfx_decision_worksheet_v1.md`](water_surface_vfx_decision_worksheet_v1.md) |
| **Target mock** | [`assets/vfx/reference/water/water_surface_target_v1.png`](../../../assets/vfx/reference/water/water_surface_target_v1.png) |
| **Fire spine ref** | [`fire_particle_spark_coder_queue_v1.md`](fire_particle_spark_coder_queue_v1.md) (D-W05 A, D-W10 A) |

**Rule:** **W1 shaders first** (rivers visible), then **W2 particles**. Two coders use **disjoint files**. ≤3 files per step. **No second terrain extract. No Hanabi.**

---

## Signed decisions → coder work (your checklist)

| ID | Choice | Coder lane | Slice | Status |
|:---|:---:|:---|:---|:---|
| **D-W02** | **A** Lake ripple | A | W1 shader + CPU mirror | ☑ landed — verify |
| **D-W03** | **A** River UV scroll | A | W1 `water_overlay.wgsl` river branch | ☑ landed — verify |
| **D-W04** | **A** Ocean swell + haze | A | W1 ocean branch + haze mix | ☑ landed — verify |
| **D-W05** | **A** Pinpoint ≤2px | A | W2 `water_particle_draw.wgsl` | ☐ queued |
| **D-W06** | **B** Lake glints optional | B | W2 `LakeGlint` profile (low priority) | ☐ queued |
| **D-W07** | **A** River streaks + bend foam | A+B | W2 streak shader + foam emitters | ☐ queued |
| **D-W08** | **B** Ocean coast foam only | B | W2 coast-line emitters only | ☐ queued |
| **D-W09** | **A** Particles fade; shaders always on | B | emission caps only — **not** overlay α | ☐ queued |
| **D-W10** | **A** Custom WGSL spine | A | clone `fire_particle*.wgsl` pattern | ☐ queued |

**D-W01 A** (river polyline overlay) is W1 MVP — catalog + ribbon pass; same lane as D-W03.

---

## Status snapshot (2026-05-24)

| Phase | Slice | Status | Notes |
|:---|:---|:---|:---|
| Design | FX-WATER-DESIGN | ☑ **SIGNED** | D-W01…D-W10 on worksheet |
| **W1** | FX-WATER-SHADER-001/002 | ☑ **first pass** | `water_w1_green`, GPU hook in `engine_with_worldgen.rs` |
| **W1 gap** | Ocean + river **read** | ☑ | W1 slices landed; **WATER-WITNESS-001** / **WATER-STRATEGIC-001** dual-band JSON |
| **W2** | FX-WATER-PARTICLE-001/002 | ☑ **first pass** | `water_particle_rows: 96` tactical; **foam counts 0** |
| **Closure** | Track plan | ☐ **ACTIVE** | [`water_vfx_closure_plan_v1.md`](../../../src/dev/stages/water_vfx_closure_plan_v1.md) |

**Landed (do not redo from scratch):**

| File | D-W |
|:---|:---|
| `src/render/water_surface_visual.rs` | D-W01 catalog, D-W02–04 CPU mirror |
| `assets/shaders/water/water_overlay.wgsl` | D-W02 A, D-W03 A, D-W04 A |
| `src/render/gpu_water_surface_draw.rs` | W1 GPU pass |
| `assets/shaders/water/water_particle*.wgsl` | D-W05, D-W10 |
| `src/render/gpu_water_particles.rs` | D-W06–D-W09 emission |
| `src/engine/engine_with_worldgen.rs` | `register_water_surface_draw` |

---

## Two-coder assignment

```text
┌──────────────────────────────────────────────────────────────────┐
│  CODER A — Render / WGSL (W1 GPU hook + W2 particle shaders)     │
│  FX-WATER-SHADER-001 finish → FX-WATER-PARTICLE-001 shaders      │
│  Touch: water_overlay.wgsl, water_particle*.wgsl, gpu_*_draw.rs  │
│  Do NOT: HydrologyResult sim, particle scatter policy (Coder B)    │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│  CODER B — Policy / catalog / witness (W1 proof + W2 emission)   │
│  FX-WATER-SHADER-002 witness → FX-WATER-PARTICLE-002…004         │
│  Touch: water_surface_visual.rs, gpu_water_particles.rs (new)    │
│  Do NOT: water_overlay.wgsl fragment edits (Coder A)             │
└──────────────────────────────────────────────────────────────────┘
```

**Parallel (Coder B, disjoint):** IND-E01 · FX-FIRE-SPARK-003 witness — only when not on `gpu_particles.rs`.

---

## Global regression

```powershell
cargo test -p proc_A_dine01 --lib water_surface_visual stage5
cargo test -p proc_A_dine01 --lib gpu_particles
```

Visual / world-gen proof:

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
```

---

# W1 — Shaders (D-W02, D-W03, D-W04) · motion always on

## Coder A — FX-WATER-SHADER-001 finish

```
Lane: FX-WATER-SHADER-001 — W1 GPU overlay hook + shader verify
Agent: Coder A (render)
Read: water_surface_vfx_coder_queue_v1.md § W1-A
      assets/vfx/reference/water/water_surface_target_v1.png
First: register_water_surface_draw in engine render bootstrap (mod.rs / engine_with_worldgen.rs)
Do NOT: new hydrology compute; Hanabi; W2 particles yet
Verify: cargo test -p proc_A_dine01 --lib water_surface_visual stage5; --test visual
```

| Step | Task | D-W | Files (≤3) | Verify |
|:---:|:---|:---|:---|:---|
| **W1-A1** | Hook `register_water_surface_draw` into app startup | D-W01 | `engine_with_worldgen.rs`, `render/mod.rs` | GPU pass runs |
| **W1-A2** | Confirm river ribbon reads on live map (GPU path) | D-W03 A | `gpu_water_surface_draw.rs`, `water_overlay.wgsl` | rivers ≠ lakes |
| **W1-A3** | Lake ripple Hz ~0.03; ocean swell + horizon haze | D-W02/D-W04 A | `water_overlay.wgsl` fs_main | side-by-side mock |
| **W1-A4** | **Shader motion ignores zoom** — overlay α may use `zoom_alpha` for readability only; motion terms use `time_secs` only | D-W09 A (shader half) | `water_overlay.wgsl` | strategic zoom still animates |

**D-W02 A implementation reference** (`water_overlay.wgsl` lake branch):

```wgsl
// slow omnidirectional ripple (~0.03 Hz effective)
let ripple = sin(t * 0.6 + in.world_xy.x * 1.1 + in.world_xy.y * 0.9) * 0.5 + 0.5;
```

**D-W03 A** — river: `along + t * 0.35` scroll, center dark / edge light ribbon.

**D-W04 A** — ocean: swell sin/cos + `haze = smoothstep(0.6, 1.0, d)` toward `#060808` @ 35%.

---

## Coder B — FX-WATER-SHADER-002 witness

```
Lane: FX-WATER-SHADER-002 — W1 witness + catalog tests
Agent: Coder B (policy)
Read: water_surface_vfx_design_plan_v1.md §7
First: water_w1_green fields in stage5_full_app_live.json
Do NOT: water_overlay.wgsl
Verify: cargo test -p proc_A_dine01 --lib water_surface_visual
```

| Step | Task | Files (≤3) | Witness field |
|:---:|:---|:---|:---|
| **W1-B1** | Stamp `water_river_segments`, `water_river_tiles`, `water_w1_green` | `stage5_full_app_harness.rs` | JSON |
| **W1-B2** | Test: hydrology rect → `catalog.w1_green()` | `water_surface_visual.rs` tests | unit green |
| **W1-B3** | Confirm catalog built at world-gen (`world_generator_enhanced.rs`) | already partial — assert non-empty on fixture world | gen proof |

**W1 green when:** `water_w1_green: true` AND visual shows directional river strip distinct from lake teal.

**Unblocks W2.**

---

# W2 — Particles (D-W05–D-W09, D-W10) · fire spark spine

**W1 witness green** — W2 first pass landed. **Remaining:** foam, ocean fixture, designer PASS → [`water_vfx_closure_plan_v1.md`](../../../src/dev/stages/water_vfx_closure_plan_v1.md).

## Coder A — FX-WATER-PARTICLE-001 · WGSL spine (D-W05, D-W10) — landed

```
Lane: FX-WATER-PARTICLE-001 — water particle shaders (fire spine clone)
Agent: Coder A (render)
Read: water_surface_vfx_coder_queue_v1.md § W2-A
      assets/shaders/fire/fire_particle.wgsl + fire_particle_draw.wgsl
First: assets/shaders/water/water_particle.wgsl + water_particle_draw.wgsl
Do NOT: Hanabi; duplicate terrain extract
Verify: compiles; stage5 green
```

| Step | Task | D-W | Files (≤3) |
|:---:|:---|:---|:---|
| **W2-A1** | Expand pass: ≤2px half-edge (clone fire expand) | D-W05 A | `water_particle.wgsl` |
| **W2-A2** | Fragment: cyan/white glint + teal streak + foam archival colors §6 | D-W05/D-W07 | `water_particle_draw.wgsl` |
| **W2-A3** | River streak: elongated UV (stretch `uv.x` 3×) + flow_dir advect in expand | D-W07 A | `water_particle.wgsl` |
| **W2-A4** | Draw pass + raster (clone `gpu_fire_particle_raster.rs`) | D-W10 A | `gpu_water_particle_draw.rs`, `gpu_water_particle_raster.rs` |
| **W2-A5** | Additive-leaning blend (same as fire D-F08) for hot glints | D-W05 A | raster pipeline descriptor |

**Color tokens (§6 — map in fragment):**

| Profile | Token | Hex |
|:---|:---|:---|
| Lake glint | `water_glint_cyan` | `#5ee0dc` @ 20–40% α |
| River streak | `water_teal_edge` | `#4a7878` |
| Bend foam | `foam_archival` | `#c8b898` @ 50% α |
| Coast foam | `foam_archival` | D-W08 B only |

**Twinkle:** reuse fire D-F05 sin/cos on `world_xy` at **lower saturation** (design plan § particle profiles).

---

## Coder B — FX-WATER-PARTICLE-002 · Emission profiles (D-W06, D-W07, D-W08, D-W09)

```
Lane: FX-WATER-PARTICLE-002 — water particle emission + zoom caps
Agent: Coder B (policy)
Read: water_surface_vfx_design_plan_v1.md §7 density table
First: src/render/gpu_water_particles.rs (new) — WorldWaterParticleFrame
Do NOT: water_particle_draw.wgsl
Verify: cargo test gpu_water_particles; stage5
```

| Step | Task | D-W | Details |
|:---:|:---|:---|:---|
| **W2-B1** | `WorldWaterParticleFrame` from `WaterSurfaceVisualCatalog` | D-W10 A | single upload spine |
| **W2-B2** | **RiverStreak** — advect along `RiverPolylineSegment.flow_dir` | D-W07 A | centerline bias, higher density |
| **W2-B3** | **RiverFoam** — spawn at curvature peaks on path | D-W07 A | detect bend via tangent delta |
| **W2-B4** | **Ocean('OceanFoam')** — emit only at shallow/deep coast tiles | D-W08 B | **no** open-ocean spray in W2 |
| **W2-B5** | **LakeGlint** — sparse random twinkle | D-W06 B | **optional** — ship last; max 8/chunk tactical |
| **W2-B6** | **Zoom fade particles only** | D-W09 A | `zoom_alpha < 0.35` → **zero** particle rows; shaders unchanged |

**§7 caps (implement in Rust):**

| Zoom band | Lake glints | River streak+foam | Ocean foam |
|:---|:---:|:---:|:---:|
| Tactical | 100% (max 8/chunk) | 100% (24 streaks, 12 foam) | 100% (16 coast) |
| Operational | 40% | 60% | 50% |
| Strategic | **0%** | **0%** | **0%** |

Reuse `FireParticleCameraScale.zoom_alpha` hook — same as fire D-F09.

---

## Coder B — FX-WATER-PARTICLE-003 · Witness

| Field | Meaning |
|:---|:---|
| `water_particle_rows` | Total instances this frame |
| `water_particle_river_streaks` | D-W07 count |
| `water_particle_coast_foam` | D-W08 count |
| `water_particle_zoom_alpha` | D-W09 |
| `water_shader_motion_always_on` | `true` at all zooms |

---

# Session schedule (2 is days)

| Day | Coder A | Coder B |
|:---|:---|:---|
| **1** | W1-A1 GPU hook + W1-A2 river verify | W1-B1/B2 witness + tests |
| **2** | W2-A1/A2 water_particle shaders | W2-B1 frame struct + catalog read |
| **3** | W2-A3/A4 draw pass | W2-B2/B3 river streak + bend foam |
| **4** | W2-A5 blend polish | W2-B4 coast foam + B6 zoom gate + B5 glints optional |

---

## Do-not-touch matrix

| Working on… | Do not edit |
|:---|:---|
| **W1 shaders (A)** | `gpu_water_particles.rs`, hydrology sim |
| **W2 emission (B)** | `water_particle_draw.wgsl`, `water_overlay.wgsl` |
| **Any water slice** | Second `HydrologyResult` extract; Hanabi world fields |
| **Fire parallel (B)** | Same session as `gpu_particles.rs` unless coordinated |

---

## Accept criteria

| Slice | Green when |
|:---|:---|
| **FX-WATER-SHADER-001** | Rivers visually distinct; lake ripple + ocean swell animate at strategic zoom |
| **FX-WATER-SHADER-002** | `water_w1_green: true` in witness JSON |
| **FX-WATER-PARTICLE-001** | Pinpoint cyan/teal particles render; no Hanabi |
| **FX-WATER-PARTICLE-002** | River streaks + bend foam + coast foam only; lake glints optional |
| **D-W09 A** | Particles = 0 strategic; overlay motion still visible |

---

## Copy-paste — pick your lane

### Coder A today

```
Lane: FX-WATER-SHADER-001 finish (W1)
Read: prompts/guides/ui/water_surface_vfx_coder_queue_v1.md
First: register_water_surface_draw in engine_with_worldgen.rs
Decisions: D-W02 A, D-W03 A, D-W04 A shader motion; D-W09 shader always on
Verify: cargo test -p proc_A_dine01 --lib water_surface_visual stage5
```

### Coder B today

```
Lane: FX-WATER-SHADER-002 witness (W1)
Read: src/dev/water_surface_vfx_design_plan_v1.md §7
First: water_w1_green in stage5_full_app_live.json
Decisions: catalog + river tile counts only (no particles yet)
Verify: cargo test -p proc_A_dine01 --lib water_surface_visual
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.2.0 | 2026-05-24 | Aligned to sign-off ledger; WATER-DESIGN-001 SIGNED TUNE |
| v1.1.0 | 2026-05-24 | Honest partial status; link **FX-WATER** closure track |
| v1.0.0 | 2026-05-24 | Dual-coder queue; D-W02–D-W10 mapped to W1/W2 slices |
