# Fire spark VFX track — closure sign-off `v1` (PLAN-FIRE-VFX-CLOSURE-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-FIRE-VFX-CLOSURE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` (sign-off only) |
| **Track** | **FX-FIRE** / **VFX-P2** (fire channel) |
| **Status** | **CLOSED** — **do not re-queue FX-FIRE-SPARK / P2-FIRE slices** |
| **Doc type** | **Closure sign-off only** — not an implementation queue |
| **Designer (post tune)** | [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) — **DESIGN-D-VFX-POST-001** / **D-VFX PASS** |
| **Steward** | [`steward_spark_vfx_gate_v1.md`](steward_spark_vfx_gate_v1.md) — **GO (qualified)** |
| **Phase 2 hub** | [`stages/vfx_phase2_closure_plan_v1.md`](stages/vfx_phase2_closure_plan_v1.md) |
| **Water (separate)** | [`water_vfx_track_closure_plan_v1.md`](water_vfx_track_closure_plan_v1.md) — **CLOSED** |

**No new Rust.** Records **fire spark** track closure after **P2-FIRE-SPARK-011** tune and **D-VFX** post-implementation review. Maintain regression only.

---

## Designer gate (after spark tune)

```text
FX-FIRE-SPARK-DESIGN (§11 SIGNED)              ☑
        │
        ▼
FX-FIRE-SPARK-001 … 006 (shader + compute)     ☑
        │
        ▼
P2-VFX-VISUAL-001 (tactical harness)           ☑
P2-FIRE-SPARK-010 (sparks above smoke)         ☑
        │
        ▼
P2-FIRE-SPARK-011 (spark tune @ zoom 0.85)      ☑  ← tune before designer POST
        │
        ▼
DESIGN-D-VFX-POST-001 / D-VFX                  ☑ PASS 2026-05-25
        │
        ▼
PLAN-FIRE-VFX-CLOSURE-001 (this doc)           ☑ CLOSED
```

**Brief:** [`prompts/guides/ui/vfx_post_implementation_review_v1.md`](../prompts/guides/ui/vfx_post_implementation_review_v1.md)  
**Mock:** [`assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png`](../assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png)

**Prerequisite rule:** **D-VFX** runs **after** **P2-FIRE-SPARK-011** — not before tactical tune (@coder A, then @designer).

---

## Do not re-queue (hard rule)

| ID | Layer | Status | Policy |
|:---|:---|:---:|:---|
| **FX-FIRE-SPARK-DESIGN** | Design §11 | **DONE** | No redesign without new product brief |
| **FX-FIRE-SPARK-001** | Phase A draw | **DONE** | `fire_particle_draw.wgsl` / `fire_particle.wgsl` |
| **FX-FIRE-SPARK-002** | Compute advection | **DONE** | `fire_spark_compute.wgsl` |
| **FX-FIRE-SPARK-003** | Witness rows | **DONE** | harness fields landed |
| **FX-FIRE-SPARK-004** | Smoke draw order | **DONE** | → **P2-FIRE-SPARK-010** |
| **FX-FIRE-SPARK-005** | Spark/Ember class | **DONE** | |
| **FX-FIRE-SPARK-006** | Per-view cull | **DONE** | D-F09 strategic cull **keep** |
| **P2-VFX-VISUAL-001** | Tactical proof | **DONE** | `fire_spark_rows: 308` |
| **P2-FIRE-SPARK-010** | Above smoke | **DONE** | `fire_sparks_above_smoke: true` |
| **P2-FIRE-SPARK-011** | Tune @ **0.85** | **DONE** | unblocked **D-VFX** |
| **STEWARD-SPARK-VFX-001** | Steward gate | **PASS** | Refresh only if fire render code changes |

**Do not** disable strategic spark cull globally to green witness — breaks **D-F09** and water **D-W09** parity ([`steward_spark_vfx_gate_v1.md`](steward_spark_vfx_gate_v1.md)).

---

## Closure record (completed)

```text
DONE  FX-FIRE-SPARK-001 … 006
DONE  P2-VFX-VISUAL-001 · P2-FIRE-SPARK-010 · P2-FIRE-SPARK-011
PASS  DESIGN-D-VFX-POST-001 (D-VFX fire channel)
PASS  STEWARD-SPARK-VFX-001 (harness + VX-P0-01)
      │
      ▼
CLOSED  FX-FIRE spark track / PLAN-FIRE-VFX-CLOSURE-001
```

**Water** remains under [`water_vfx_track_closure_plan_v1.md`](water_vfx_track_closure_plan_v1.md) — **not** reopened by this sign-off.

---

## Witness snapshot (fleet truth — use for audits)

**File:** [`debug_runs/stage5_full_app_live.json`](../../debug_runs/stage5_full_app_live.json)

| Field | Value | Meaning |
|:---|:---|:---|
| `particle_routing.fire_spark_rows` | **308** | Tactical emit — D-F07/D-F09 |
| `particle_routing.fire_spark_zoom_alpha` | **0.85** | Post **P2-FIRE-SPARK-011** tune |
| `fire_sparks_above_smoke` | `true` | D-F10 — **P2-FIRE-SPARK-010** |
| `fire_spark_compute_enabled` | `true` | Phase A+B |
| `fire_spark_phase` | `A+B` | draw + compute |
| `fire_spark_additive_blend` | `true` | D-F08 |
| `tactical_vfx_witness.all_green` | `true` | P2 harness |
| `fire_spark_rows_gt_0` | `true` | Stage 5 rollup |

**Strategic zoom `fire_spark_rows: 0` is correct** — intentional cull, not a defect.

### Non-blocking (do not reopen track)

| Field | Follow-up | Owner |
|:---|:---|:---|
| `fire_instance_buffer_rows: 0` | **VX-P2-01** — projection graph native fire | coder (future) |
| `fire_spark_projection_view: overlay_bootstrap` | Same | coder (future) |
| PNG **ACCEPTED** vs interim captures | **VX-P0-04** / VFX-CAPTURE-001 | operator/designer optional |

---

## Two columns (steward — do not collapse)

| Column | Proves | Status |
|:---|:---|:---:|
| **A — Harness / lib** | `--test visual`, tactical zoom, witness JSON | **PASS** |
| **B — Operator sim** | No map-wide pink wash; sparks when zoomed in | **VX-P0-01 done** · monitor **VX-P0-02** |

Details: [`steward_spark_vfx_gate_v1.md`](steward_spark_vfx_gate_v1.md).

---

## Authority map (archive)

| Writer | Files | Closed rule |
|:---|:---|:---|
| Coder A | `fire_particle*.wgsl`, `fire_spark_compute.wgsl`, `gpu_particles.rs` | No second fire extract; D-F09 cull preserved |
| Coder B | witness rollup in `stage5` harness | No WGSL unless test proves bug |
| Designer | **D-VFX** record + optional captures | **PASS** — tune tickets **F-T03** closed at witness |
| Steward | witness refresh | No feature work in closure pass |

**Design authority:** [`fire_particle_spark_design_plan_v1.md`](fire_particle_spark_design_plan_v1.md) · [`fire_particle_spark_designer_brief_v1.md`](../prompts/guides/ui/fire_particle_spark_designer_brief_v1.md).

---

## Maintenance only (after fire spark render edits)

```powershell
cargo test -p proc_A_dine01 --lib fire_spark gpu_particles stage5 tactical_vfx_witness
cargo test -p proc_A_dine01 --lib steward_spark_vfx_001_lib_bundle
cargo run -p proc_A_dine01 --release -- --test visual
```

Re-run **STEWARD-SPARK-VFX-001** if `gpu_particles.rs`, `fire_spark_compute.wgsl`, `fire_particle_draw.wgsl`, or harness rollup changes.

---

## Optional polish (not closure)

| ID | Owner | Notes |
|:---|:---|:---|
| **VX-P0-02** | operator | Zoom tactical to see sparks (D-F09) |
| **VX-P0-04** | designer | In-sim PNGs → `review_captures/` |
| **VX-P2-01** | coder | Graph-native fire instance buffer |
| **P2-FIRE-SPARK-010** | coder | Only if operator reproduces smoke-over-spark regression |

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-FIRE-VFX-CLOSURE-001 |
| Designer | 2026-05-25 | **D-VFX PASS** — after **P2-FIRE-SPARK-011** |
| Sim-steward | 2026-05-25 | **STEWARD-SPARK-VFX-001 GO (qualified)** |
| Coder | 2026-05-25 | FX-FIRE + P2 fire slices **DONE** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Fire spark track closure; D-VFX POST after tune |
