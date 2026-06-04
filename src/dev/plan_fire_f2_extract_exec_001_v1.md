# PLAN-FIRE-F2-EXEC-001 — F2 hot-cell / projection-graph fire instances `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-FIRE-F2-EXEC-001** |
| **Slice ID** | **FIRE-F2-EXTRACT-001** |
| **Coder lane** | **A-V2** (primary) · fallback **A-V1** F7-debug |
| **Prior** | **FIRE7-PLAN-001 SIGNED** · [`vfx_triage_v1.md`](vfx_triage_v1.md) **VX-P2-01** / **F-T02** |
| **Parent** | [`planner_elemental_vfx_domain_charter_v1.md`](planner_elemental_vfx_domain_charter_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **READY** — coder may pick up **A-V2** |
| **Blocks** | **A-W3** smoke bridge (prefer F2 rows green first) |

**Do not re-open:** WSS-PLAN-001..004 · FIRE7 architecture baselines.

**No Rust in this deliverable.**

---

## Summary

Close **VX-P2-01** / **F-T02**: tactical harness and live witness must show **`fire_instance_buffer_rows > 0`** on the **projection-graph path** (`RenderProjectionGraph.fire.instance_buffer`), not only sparks seeded via **`overlay_bootstrap`** in `gpu_particles.rs`. Per-view fire extract (**FIRE7**) stays authoritative; this slice wires **instance projection** through `fire_frame_for_projection_graph` → `FireProjectionNode::evaluate` → `project_fire_instances` with correct `RepresentationResult.extract_plan.fire_instances` and stamp alignment.

---

## Problem statement (VX-P2-01)

| Symptom | Witness / harness | Root cause class |
|:---|:---|:---|
| `fire_instance_buffer_rows: 0` | `stage5_full_app_live.json` | Graph evaluates but buffer empty OR extract plan off |
| `fire_spark_projection_view: "overlay_bootstrap"` | same + `gpu_particles` witness | Sparks use **chunk_heat bootstrap** when `instance_buffer` empty |
| Sparks visible, instances zero | `fire_spark_rows > 0` | **Not** a spark shader bug — projection path starved |
| Designer non-blocking | [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) | Accept tactical sparks; **F2** is graph-native completeness |

**Acceptance headline:** At tactical zoom with fire heat overlay enabled on `WorldMain` or `SimulationMap`, **`graph.fire.instance_buffer.len() > 0`** when `FireVisualFramesByView` has instances for source view.

---

## Authority map

| Domain | Sole writer | Consumers | Notes |
|:---|:---|:---|:---|
| Sim fire / heat | `ChunkSurfaceFire`, `FireSimulationSnapshot` | extract | **unchanged** — F1 ecology truth |
| Per-view frames | `build_fire_visual_frames_by_view` | `fire_frame_for_projection_graph` | **FIRE7** single writer |
| Projection graph | `run_render_projection_graph` | `FireProjectionNode::evaluate` | fills `instance_buffer` |
| `FireProjectionNode.instance_buffer` | `project_fire_instances(ctx.fire, ctx.policy)` | `gpu_particles`, indirect draw, harness | **target of this slice** |
| Spark particles | `emit_world_fire_particles_from_projection` | GPU raster | reads **graph** rows; bootstrap is **fallback only** |
| Overlay bootstrap path | `gpu_particles` chunk_heat loop | sparks when buffer empty | **retire as primary** — see § Hybrid retirement |

**Forbidden:** Second global fire extract; minimap ECS fire query; writing `instance_buffer` outside `FireProjectionNode::evaluate`.

```text
ChunkSurfaceFire  →  extract_fire_simulation_snapshot  →  FireVisualFramesByView
       →  fire_frame_for_projection_graph  →  RenderProjectionContext.fire
       →  FireProjectionNode.evaluate  →  instance_buffer
       →  emit_world_fire_particles_from_projection (projection_view != overlay_bootstrap)
```

---

## PR plan (≤3 files per PR)

### F2-PR-1 — Policy + stamp alignment

**Goal:** `extract_plan.fire_instances == true` and `ctx.fire.stamp == ctx.committed_stamp` on tactical FULL_APP fixture.

| File | Change |
|:---|:---|
| `src/gui/view_representation.rs` | Ensure tactical band sets `extract_plan.fire_instances` when fire overlay on |
| `src/render/extraction/render_projection_graph.rs` | Guard: log/stamp mismatch witness hook (dev only) |
| `src/render/view_fire_projection.rs` | Assert source view has instances before clear |

**Witness delta:** `fire_projection_stamp_aligned: true` (new optional field)

**Tests:** extend `render_projection_graph` lib tests — non-empty buffer when frame has instances + plan on

---

### F2-PR-2 — Populate `project_fire_instances`

**Goal:** `project_fire_instances` copies/shapes `FireVisualFrame.instances` into `FireVisualGpuInstance` rows under LOD cap.

| File | Change |
|:---|:---|
| `src/render/extraction/render_projection_graph.rs` | `project_fire_instances` body + capacity from `gpu_budget.fire_instance_cap` |
| `src/render/extraction/fire_visual_extract.rs` | Ensure `FireVisualFrameSet::ProjectGpu` runs after frames + committed stamp |
| `src/render/stage5_full_app_harness.rs` | Harness asserts `fire_instance_buffer_rows > 0` in tactical scenario |

**Witness delta:** `fire_instance_buffer_rows > 0`, `fire_projection_graph_native: true`

---

### F2-PR-3 — Retire overlay_bootstrap primary path

**Goal:** Sparks only use chunk_heat bootstrap when **explicit** degraded mode — not default tactical path.

| File | Change |
|:---|:---|
| `src/render/gpu_particles.rs` | Gate bootstrap: only if `instance_buffer.is_empty() && policy.degraded_fire_spark_fallback` |
| `src/render/stage5_full_app_harness.rs` | `fire_spark_projection_view` expects graph view id label |
| `src/dev/visual_run_blockers.md` | Close **VX-P2-01** row when green |

**Witness delta:** `fire_spark_projection_view` ∉ `{overlay_bootstrap}` at tactical; `fire_spark_overlay_bootstrap_fallback: false`

---

## Hybrid `overlay_bootstrap` retirement plan

| Phase | Behavior | Witness |
|:---|:---|:---|
| **H0 (today)** | Empty `instance_buffer` → bootstrap from `chunk_heat` | `projection_view: overlay_bootstrap` |
| **H1 (F2-PR-1/2)** | Buffer populated from graph; bootstrap rarely runs | `fire_instance_buffer_rows > 0` |
| **H2 (F2-PR-3)** | Bootstrap only when `degraded_fire_spark_fallback` + buffer empty | `fire_spark_overlay_bootstrap_fallback: false` in tactical harness |
| **H3 (future)** | Remove bootstrap loop or dev-only | planner **PLAN-FIRE-F2-02** after smoke bridge |

**Do not** delete bootstrap in F2-PR-2 — strategic zoom may still need heat-only read without instances.

---

## Witness JSON schema extensions

**Primary file:** `debug_runs/stage5_full_app_live.json`

| Pointer | Type | Green when |
|:---|:---|:---|
| `/tactical_vfx_witness/fire_instance_buffer_rows_gt_0` | bool | `true` (new rollup) |
| `/fire_instance_buffer_rows` | number | `> 0` tactical harness |
| `/fire_spark_projection_view` | string | `WorldMain` or `SimulationMap` (not `overlay_bootstrap`) |
| `/fire_projection_graph_native` | bool | `true` |
| `/fire_projection_stamp_aligned` | bool | `true` |
| `/fire_spark_overlay_bootstrap_fallback` | bool | `false` at tactical |

**Regression (unchanged):**

| File | Guard |
|:---|:---|
| `fire_ecology_live.json` | `f1_green: true` |
| `infrastructure_view_isolation_live.json` | F7-A exit green |
| `fire_streaming_live.json` | F7-B green |

---

## Lib test predicates

```powershell
cargo test -p proc_A_dine01 --lib render_projection_graph
cargo test -p proc_A_dine01 --lib fire_visual_extract
cargo test -p proc_A_dine01 --lib tactical_vfx
cargo test -p proc_A_dine01 --lib stage5_full_app_harness
```

| Test name (add or extend) | Predicate |
|:---|:---|
| `strategic_band_keeps_full_frame_but_projection_drops_gpu_instances` | existing — preserve |
| `tactical_projection_fills_fire_instance_buffer` | `graph.fire.instance_buffer.len() > 0` |
| `projection_view_not_overlay_bootstrap_when_buffer_populated` | witness label ≠ bootstrap |
| `stamp_mismatch_clears_instance_buffer` | existing evaluate guard |

---

## Schedule placement

```text
FireVisualFrameSet::BuildProfiles
  → build_fire_visual_frames_by_view
  → commit_fire_visual_snapshot
  → FireVisualFrameSet::ProjectGpu
       → run_render_projection_graph    [instance_buffer WRITER]
  → FireVisualFrameSet::EmitParticles
       → emit_world_fire_particles_from_projection   [READ graph]
```

**Do not** run projection graph before `CommittedVisualSnapshotFence` tick alignment.

---

## Edge cases

- **Strategic band:** `extract_plan.fire_instances` may be false — buffer empty **OK**; sparks use policy cull (D-F09)
- **Operational band:** cap via `fire_cap_for_world_band` — buffer may be `<` frame instances but **> 0**
- **Overlay off:** `fire_heat` false → empty buffer OK; do not fail witness
- **Multiview:** source view from `projection_fire_source_view` only — not per-secondary-view rows in v1
- **VT-5 flicker:** orthogonal — do not block F2 on VT-5 intermittent ([`visual_run_blockers.md`](visual_run_blockers.md))

---

## Anti-patterns

| Forbidden | Why |
|:---|:---|
| Witness green via hand-edited JSON | authority |
| Seed `instance_buffer` in harness only without graph evaluate | fake F2 |
| Disable strategic cull globally | breaks D-F09 |
| Second fire extract for “quick green” | violates FIRE7-PLAN-001 |
| Re-open WSS-PLAN-001..004 | planner policy |

---

## Designer checkpoint

**DESIGN-F2-EXTRACT-READ-001** ([`designer_parallel_workboard_v1.md`](designer_parallel_workboard_v1.md)) — tactical readability when rows > 0; run after F2-PR-2.

---

## Rollback trigger

- `f1_green` false or F7 isolation witness red → revert PR-3 first, then PR-2
- FULL_APP tactical witness red with `fire_spark_rows == 0` → stop; do not ship PR-3 bootstrap gate

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Exec plan for A-V2 / VX-P2-01 |
