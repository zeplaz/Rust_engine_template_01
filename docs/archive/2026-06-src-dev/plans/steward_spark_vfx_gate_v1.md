# STEWARD-SPARK-VFX-001 gate `v1`

| Field | Value |
|:---|:---|
| **Lane ID** | `STEWARD-SPARK-VFX-001` |
| **Date** | 2026-05-25 |
| **Owner** | `@sim-steward` |
| **Track** | **VFX-P2** / fire sparks |
| **Triage** | [`vfx_triage_v1.md`](vfx_triage_v1.md) |
| **Designer** | [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) **D-VFX PASS** |
| **Closure** | [`fire_spark_track_closure_plan_v1.md`](fire_spark_track_closure_plan_v1.md) (**PLAN-FIRE-VFX-CLOSURE-001**) |
| **Witness** | [`debug_runs/stage5_full_app_live.json`](../../debug_runs/stage5_full_app_live.json) |

## Verdict: **GO (qualified)**

**Harness / lib:** fire spark tactical proof **green**. **Operator sim** presentation is a **separate column** — not a Stage 5 gate failure.

---

## Two columns (do not collapse)

| Column | What it proves | Verdict |
|:---|:---|:---:|
| **A — Harness** | `--test visual` tactical zoom, `fire_spark_rows > 0`, D-F09 cull at strategic | ✅ **PASS** |
| **B — Operator sim** | Normal `Simulation` without map-wide pink wash; pinpoint sparks when zoomed in | ✅ **VX-P0-01 done** · monitor **VX-P0-02** |

Witness JSON proves **column A only**. Column B is code policy + operator replay.

---

## Shift A — Observe

| Gate | Required | Observed (`stage5_full_app_live.json`) |
|:---|:---|:---|
| `tactical_vfx_witness.all_green` | `true` | ✅ |
| `particle_routing.fire_spark_rows` | `> 0` | ✅ **308** |
| `particle_routing.fire_spark_zoom_alpha` | tactical (~0.65+) | ✅ **0.85** |
| `fire_sparks_above_smoke` | `true` | ✅ |
| `fire_spark_compute_enabled` | on | ✅ |
| `fire_spark_phase` | `A+B` | ✅ |
| `stage5_closure.passes` | `true` | ✅ |

| Code policy (**VX-P0-01**) | Evidence |
|:---|:---|
| `simulation_minimap_overlay_defaults().fire_heat: false` | ✅ `vx_p0_01_operator_simulation_fire_heat_off_by_default` |
| Minimap raster `fire_boost` capped at **1.0** | ✅ `tile_world_fallback.rs` |
| WorldMain spark zoom via ViewManager | ✅ slice 1 VM-09 (`gpu_particles`) |

| Lib tests | Result |
|:---|:---:|
| `p2_fire_spark_011_at_tactical_proof_zoom` | ✅ |
| `p2_tactical_zoom_alpha_08_fire_spark_rows_positive` | ✅ |
| `strategic_zoom_culls_fire_spark_rows` | ✅ |
| `tactical_vfx_witness_gates_green_at_tactical_zoom` | ✅ |
| `vx_p0_01_operator_simulation_fire_heat_off_by_default` | ✅ |

**Non-blocking:** `fire_instance_buffer_rows: 0`, `fire_spark_projection_view: overlay_bootstrap` — **VX-P2-01** (graph-native projection later).

---

## Shift B — Decide

```yaml
shift: B
issue:
  id: STEWARD-SPARK-VFX-001
  severity: LOW
route:
  pass: close spark steward; harness + VX-P0-01 signed
  monitor:
    - VX-P0-02: operator zoom in for GPU sparks (D-F09 intentional cull)
    - VX-P0-04: in-sim PNGs under review_captures/ (designer ACCEPTED optional)
  delegate:
    - P2-FIRE-SPARK-010: "@coder" only if operator reproduces smoke-over-spark
    - ~~VX-P0-03~~: background precip @ strategic zoom — **done** (`weather_visual.rs`, `vx_p0_03` tests)
block: none for spine
```

**Do not** disable strategic spark cull globally to green witness — breaks D-F09 / water D-W09 parity.

---

## Shift C — Act

```powershell
cargo test -p proc_A_dine01 --lib fire_spark
cargo test -p proc_A_dine01 --lib vx_p0_01 tactical_vfx_witness
cargo test -p proc_A_dine01 --lib steward_spark_vfx_001_lib_bundle
# Optional witness timestamp refresh:
cargo run -p proc_A_dine01 --release -- --test visual
```

| Action | Result |
|:---|:---|
| Lib fire spark suite | ✅ **5/5** |
| Bundle JSON gate test | ✅ |
| **@coder blockers** | **0** |

---

## Route to @coder / @operator

| # | ID | When |
|:---:|:---|:---|
| 1 | **P2-FIRE-SPARK-010** | Operator sees sparks hidden under smoke layer |
| ~~2~~ | ~~**VX-P0-03**~~ | ~~Background weather missing when zoomed out~~ — **done** |
| 3 | **VX-P2-01** | Restore `fire_instance_buffer_rows` via projection graph |
| — | **VX-P0-02** | **@operator** — zoom tactical; compare [`fire_spark_target_v1.png`](../assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png) |
| — | **VX-P0-04** | **@operator** — save tactical PNGs to `assets/vfx/reference/review_captures/` |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | STEWARD-SPARK-VFX-001 **GO (qualified)** |
