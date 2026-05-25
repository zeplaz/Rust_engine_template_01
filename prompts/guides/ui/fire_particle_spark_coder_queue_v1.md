# Fire pinpoint sparks — dual @coder queue `v1`

> **Phase 2:** First-pass implementation is **done**. New work → [`src/dev/vfx_coder_phase2_queue_v1.md`](../../../src/dev/vfx_coder_phase2_queue_v1.md).

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@coder` ×2 · playbook [`render_pipeline_agent.md`](../../../tools/orchestrator/agents/render_pipeline_agent.md) |
| **Design gate** | **SIGNED** — [`fire_particle_spark_design_plan_v1.md`](../../../src/dev/fire_particle_spark_design_plan_v1.md) |
| **Master plan** | [`coder_execution_plan_v1.md`](../../../src/dev/coder_execution_plan_v1.md) |
| **Reference mock** | [`assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png`](../../../assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png) |
| **Legacy GLSL** | `C:\dev\razerz-master\shaderzglsl\elemental\compute_partical\` |

**Rule:** Two coders run **disjoint file sets** in parallel. One **primary** slice per session (≤3 files). Global regression after every merge.

---

## Status snapshot (2026-05-24)

| Phase | Slice | Status | Owner |
|:---|:---|:---|:---|
| Design | FX-FIRE-SPARK-DESIGN | ☑ **SIGNED** | designer |
| **FX-L1** | FX-FIRE-SPARK-001 Phase A look | ☑ **landed** — shaders + scatter + additive blend | either |
| **FX-L2** | FX-FIRE-SPARK-002 compute motion | ☐ **queued** | **Coder A** |
| **FX-L4** | FX-FIRE-SPARK-003 witness + caps | ☐ **queued** | **Coder B** |
| **FX-L5** | FX-FIRE-SPARK-004 smoke draw order | ☐ **queued** | **Coder A** |
| **FX-L6** | FX-FIRE-SPARK-005 Spark/Ember class split | ☐ **queued** | **Coder B** |
| **FX-L7** | FX-FIRE-SPARK-006 per-view cull | ☐ **queued** | **Coder B** |

**Phase A landed (do not redo):**

| File | Change |
|:---|:---|
| `assets/shaders/fire/fire_particle_draw.wgsl` | Sharp core, age/twinkle, zoom fade (D-F01/04/05/09) |
| `assets/shaders/fire/fire_particle.wgsl` | 0.5–2px half-edge expand |
| `src/render/gpu_particles.rs` | Scatter 1–8 sparks per hot cell (D-F07) |
| `src/render/gpu_fire_particle_raster.rs` | Additive-leaning blend (D-F08) |

---

## Two-coder assignment (parallel)

```text
┌─────────────────────────────────────────────────────────────────┐
│  CODER A — Render / GPU lane (shaders + compute + draw order)   │
│  FX-FIRE-SPARK-002 → 004 → optional point-list polish           │
│  Touch: assets/shaders/fire/*, gpu_fire_particle_raster.rs      │
│  Do NOT: gpu_particles scatter policy, extract, witness JSON      │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  CODER B — Policy / spine lane (Rust + witness + product)       │
│  FX-FIRE-SPARK-003 → 005 → 006 + parallel IND-E01               │
│  Touch: gpu_particles.rs, stage5 harness, visual_diagnostics    │
│  Do NOT: fire_particle_draw.wgsl, new compute WGSL                │
└─────────────────────────────────────────────────────────────────┘
```

**Safe parallel product lane for Coder B (disjoint files):** **IND-E01** — [`industrial_activation_pipeline.md`](../../../src/dev/industrial_activation_pipeline.md)

---

## Global regression (both coders)

```powershell
cargo test -p proc_A_dine01 --lib gpu_particles stage5
cargo test -p proc_A_dine01 --lib minimap_compositor simulation_shell_phase2
```

Visual when touching render:

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Coder A — copy-paste starters

### A0 — Verify Phase A (15 min, first session)

```
Lane: FX-FIRE-SPARK-001 verify
Agent: Coder A (render)
Read: fire_particle_spark_coder_queue_v1.md § A0
      assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png
First: cargo test gpu_particles; --test visual; compare spark read vs mock
Do NOT: rewrite Phase A shaders unless visual fails target mock
Exit: stage5 green; visual exit 0; sparks not blobs
```

| Check | Command / field |
|:---|:---|
| Unit tests | `cargo test -p proc_A_dine01 --lib gpu_particles` |
| Stage 5 | `cargo test -p proc_A_dine01 --lib stage5` |
| Visual | `--test visual` |
| Compare | Side-by-side `engine_blob_before.png` vs live frame |

---

### A1 — FX-FIRE-SPARK-002 · Phase B compute advection (PRIMARY for Coder A)

**Goal:** Port legacy `compute_expanse_BASE_A.glsl` — pos/vel buffers, 24 attractors from fire instances, lifetime respawn.

**Signed:** D-F02 **A→B**, D-F03 **A** (attractors = `FireVisualGpuInstance` centers).

```
Lane: FX-FIRE-SPARK-002 — fire spark compute advection
Agent: Coder A (render)
Read: fire_particle_spark_coder_queue_v1.md § A1
      prompts/guides/ui/fire_particle_legacy_elemental_reference_v1.md
      C:\dev\razerz-master\shaderzglsl\elemental\compute_partical\compute_expanse_BASE_A.glsl
First: assets/shaders/fire/fire_spark_compute.wgsl (new) — pos.w lifetime, attractor loop
Do NOT: second FireVisualFrame extract; scatter in gpu_particles.rs (Coder B)
Verify: cargo test -p proc_A_dine01 --lib gpu_particles stage5; --test visual
```

| Step | Task | Files (≤3) | Verify |
|:---:|:---|:---|:---|
| **B-1** | WGSL compute: pos/vel image buffers, `DeltaTime`, lifetime decay | `fire_spark_compute.wgsl` *(new)* | compiles |
| **B-2** | Attractor upload from top-N fire instance centers (max 24) | `gpu_particle_draw.rs` or new `gpu_spark_compute.rs` | attractor count in debug |
| **B-3** | Schedule: compute → expand → draw same frame | `gpu_particle_draw.rs`, `mod.rs` | motion visible |
| **B-4** | Respawn at fire core when `life ≤ 0` | `fire_spark_compute.wgsl` | shower not static |

**Legacy port notes:**

| GLSL | WGSL target |
|:---|:---|
| `pos.xyz -= val.xyz * dt` | Same advection |
| `pos.w -= 0.0016 * dt` | Lifetime channel |
| `if (pos.w <= 0) { pos = origin; val *= 0.9; pos.w += 3 }` | Respawn burst |
| Attractor loop `i < 24` | Bind fire centers from existing frame |

**Authority:** Attractors read **`WorldFireParticleFrame` instance origins** or deduped fire extract rows — **no new sim**.

---

### A2 — FX-FIRE-SPARK-004 · Sparks above smoke (D-F10)

**Goal:** Draw order + depth/blend so pinpoint sparks read **on top of** smoke field, not buried.

```
Lane: FX-FIRE-SPARK-004 — spark/smoke compositing
Agent: Coder A (render)
Read: fire_particle_spark_decision_worksheet_v1.md D-F10 A
First: render graph edge — fire particles after weather_fire_field / smoke pass
Do NOT: smoke sim changes; gpu_particles scatter
Verify: --test visual; sparks visible on smoky cells
```

| Step | Task | Files (≤3) |
|:---:|:---|:---|
| **S-1** | Audit render graph order vs `gpu_weather_fire_field.rs` | `gpu_fire_particle_raster.rs`, render graph install |
| **S-2** | Ensure particles render after smoke overlay when both visible | graph edges in `mod.rs` or raster register |
| **S-3** | Optional: depth bias tweak so sparks win z-fight | `gpu_fire_particle_raster.rs` |

---

### A3 — FX-FIRE-SPARK-007 · Point list primitive (stretch)

Only if quads still read too soft after A0 verify.

| Step | Task | Files |
|:---:|:---|:---|
| **P-1** | Evaluate `PointList` topology vs 1px quads | `gpu_fire_particle_raster.rs`, `fire_particle_draw.wgsl` |
| **P-2** | MSAA / alpha to coverage for crisp pins | pipeline descriptor |

---

## Coder B — copy-paste starters

### B0 — FX-FIRE-SPARK-003 · Witness + density caps (PRIMARY for Coder B)

**Goal:** Measurable proof in `stage5_full_app_live.json` + tune scatter without shader edits.

```
Lane: FX-FIRE-SPARK-003 — fire spark witness + policy caps
Agent: Coder B (policy)
Read: fire_particle_spark_coder_queue_v1.md § B0
      src/dev/fire_particle_spark_design_plan_v1.md FX-L4
First: stamp spark_rows, scatter_multiplier, zoom_alpha in stage5 / visual proof JSON
Do NOT: fire_particle_draw.wgsl, fire_spark_compute.wgsl
Verify: cargo test -p proc_A_dine01 --lib stage5; witness fields present
```

| Step | Task | Files (≤3) | Witness field |
|:---:|:---|:---|:---|
| **W-1**  | Add `fire_spark_rows`, `fire_spark_scatter_slots` to proof payload | `stage5_full_app_harness.rs`, `visual_diagnostics.rs` | JSON |
| **W-2** | Cap scatter when `instances.len()` nears budget | `gpu_particles.rs` | no OOM |
| **W-3** | Strategic zoom: zero scatter when `zoom_alpha < 0.35` | `gpu_particles.rs` | D-F09 |
| **W-4** | Unit test: hot cell → scatter count ≥ 3 | `gpu_particles.rs` tests | test green |

**Target witness shape (add under `readiness` or `fire_particles`):**

```json
{
  "fire_spark_phase": "A",
  "fire_spark_rows": 42,
  "fire_spark_scatter_max": 8,
  "fire_spark_zoom_alpha": 0.72,
  "fire_spark_additive_blend": true
}
```

---

### B1 — FX-FIRE-SPARK-005 · Spark vs Ember class split

**Goal:** Designer classes — **Spark** (0.5–2px, high twinkle) vs **Ember** (2–6px, softer α).

```
Lane: FX-FIRE-SPARK-005 — particle class split
Agent: Coder B (policy)
Read: fire_particle_spark_designer_brief_v1.md §3 (Spark / Ember / Haze)
First: extend ParticleClass or ember_class_radius_smoke.y encoding
Do NOT: fragment shader (Coder A owns D-F01 look)
Verify: gpu_particles tests; ember rows larger half-edge
```

| Step | Task | Files (≤3) |
|:---:|:---|:---|
| **C-1** | Add `ParticleClass::Spark` vs `Ember` (or reuse AtmosphereFx for haze) | `gpu_particles.rs` |
| **C-2** | Map LOD band → class: FullFlame=Spark, LowFlame=Ember, SmokeOnly=Haze | `shape_fire_row_for_particle_lod` |
| **C-3** | Different `fire_particle_quad_base_half_world` per class | `gpu_particles.rs` |

Fragment already scales by `class_id > 0.5` — align Rust ordinals with shader.

---

### B2 — FX-FIRE-SPARK-006 · Per-view particle cull (TRIAGE-PHASE-F-CULL)

**Goal:** Extend `PerViewRepresentationPolicy` so multiview / minimap do not duplicate spark soup.

```
Lane: FX-FIRE-SPARK-006 — per-view fire particle cull
Agent: Coder B (policy)
Read: stage5_triage_backlog.md TRIAGE-PHASE-F-CULL
First: filter WorldFireParticleFrame by active MapViewInstance / view policy
Do NOT: shader files
Verify: stage5 + vt_ci_matrix if touched
```

| Step | Task | Files (≤3) |
|:---:|:---|:---|
| **V-1** | Read view id from projection context | `gpu_particles.rs`, `fire_visual_extract.rs` |
| **V-2** | Skip emission for non-tactical / minimap-only views | `emit_world_fire_particles_from_projection` |
| **V-3** | Stamp `fire_particle_view_culled` in diagnostic JSON | `full_render_diagnostic.rs` |

---

### B3 — Parallel · IND-E01 industrial chain

**Disjoint from all FX-FIRE files** — run while Coder A on compute.

```
Lane: IND-E01 — concrete chain E2E
Agent: Coder B (product)
Read: src/dev/industrial_activation_pipeline.md
First: sim placement → industrial_activation_live.json production_green
Do NOT: gpu_particles, fire shaders, minimap_compositor
Verify: cargo test -p proc_A_dine01 --lib stage5
```

---

## Session schedule (suggested)

| Day | Coder A | Coder B |
|:---|:---|:---|
| **1** | A0 verify + A1 B-1/B-2 compute WGSL | B0 W-1/W-2 witness + scatter caps |
| **2** | A1 B-3/B-4 compute schedule + motion | B1 class split + B0 W-3 zoom gate |
| **3** | A2 smoke draw order | B2 per-view cull |
| **4** | A3 point list (if needed) | B3 IND-E01 or polish witness |

---

## Do-not-touch matrix

| Coder A working on '…' | Do not edit |
|:---|:---|
| FX-FIRE-SPARK-002/004/007 | `gpu_particles.rs` scatter, `stage5_full_app_harness.rs` witness |
| Any FX render slice | `economy/activation/`, `minimap_compositor/` |

| Coder B working on… | Do not edit |
|:---|:---|
| FX-FIRE-SPARK-003/005/006 | `fire_particle_draw.wgsl`, `fire_spark_compute.wgsl` |
| IND-E01 | fire shaders, `gpu_fire_particle_raster.rs` |

| Both | Do not add second `FireVisualFrame` extract |

---

## Accept criteria (lane close)

| Slice | Green when |
|:---|:---|
| **FX-FIRE-SPARK-001** | Visual matches `fire_spark_target_v1.png` read; stage5 green |
| **FX-FIRE-SPARK-002** | Sparks advect toward fire cores; respawn visible |
| **FX-FIRE-SPARK-003** | Witness JSON fields populated; scatter tests green |
| **FX-FIRE-SPARK-004** | Sparks visible on smoky fire cells |
| **FX-FIRE-SPARK-005** | Spark/Ember size ratio in unit test |
| **FX-FIRE-SPARK-006** | Non-tactical views emit 0 spark rows |

---

## Document index

| Doc | Role |
|:---|:---|
| [`fire_particle_spark_design_plan_v1.md`](../../../src/dev/fire_particle_spark_design_plan_v1.md) | Signed design authority |
| [`fire_particle_legacy_elemental_reference_v1.md`](fire_particle_legacy_elemental_reference_v1.md) | GLSL skim table |
| [`fire_ecology_f1_todos.md`](../../../src/dev/fire_ecology_f1_todos.md) | Sim fuel — orthogonal |
| [`stage5_triage_backlog.md`](../../../src/dev/stage5_triage_backlog.md) | TRIAGE-PHASE-F-CULL |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Dual-coder queue post-design SIGNED; Phase A landed |
