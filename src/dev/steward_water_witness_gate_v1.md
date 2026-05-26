# STEWARD-WATER-WITNESS-001 gate `v1`

| Field | Value |
|:---|:---|
| **Lane ID** | `STEWARD-WATER-WITNESS-001` |
| **Date** | 2026-05-25 (re-run) |
| **Prereq** | **WATER-W1-OCEAN-001** + **WATER-W2-FOAM-001** **done** |
| **Planner rollup** | [`water_vfx_track_closure_plan_v1.md`](water_vfx_track_closure_plan_v1.md) (**PLAN-WATER-TRACK-001**) |
| **Witness** | [`debug_runs/stage5_full_app_live.json`](../../debug_runs/stage5_full_app_live.json) (`written_at_epoch_secs` refreshed) |

## Verdict: **PASS**

Prereqs green in sim. Witness refreshed via `--test visual`. **No @coder blockers.**

| Gate | Required | Observed |
|:---|:---|:---:|
| `water_w1_green` | `true` | ✅ |
| `water_w1_ocean_green` / `water_w1_ocean_001_green` | ocean branch | ✅ |
| `water_ocean_tiles` | `> 0` | **1303** |
| `water_particle_rows` (tactical) | `> 0` | **216** |
| Strategic D-W09 | `rows == 0`, shader on | ✅ `strategic_band.rows: 0`, `shader_motion_always_on: true` |
| `water_particle_coast_foam` | `> 0` | **128** |
| `water_particle_river_foam` | `> 0` or no bends in fixture | **0** — `catalog_has_river_bend: false` (W2 gate waived) |
| `water_w2_foam_001_green` | `true` | ✅ |
| `water_strategic_001_green` | `true` | ✅ |
| `water_strategic_gates_green` | `true` | ✅ FULL_APP `tactical_vfx_witness` rollup (WATER-STRATEGIC-001) |
| `water_witness_001_green` | `true` | ✅ |
| `tactical_vfx_witness.all_green` | `true` | ✅ |
| `stage5_closure.passes` | `true` | ✅ |
| Lib tests | `gpu_water_particles` + `water_surface_visual` | **9/9** + **12/12** |

**Qualified (monitoring, not block):** `river_foam: 0` on this visual seed — acceptable per [`water_vfx_closure_plan_v1.md`](stages/water_vfx_closure_plan_v1.md) when no bend geometry in catalog. Lib test `water_w2_foam_001_river_bend_emits_foam` still proves bend path.

**FX-WATER coder slices:** **CLOSED** for witness exit. Designer **TUNE** optional ([`water_vfx_review_record_v1.md`](water_vfx_review_record_v1.md)).

```powershell
cargo test -p proc_A_dine01 --lib gpu_water_particles water_surface_visual
cargo test -p proc_A_dine01 --lib water_witness_001
cargo run -p proc_A_dine01 --release -- --test visual
```
