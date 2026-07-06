# Remaining lane todos `v1`

**Date:** 2026-07-03 · **Branch:** `master` · **Migration:** CLOSED

**Authoritative coder picks:** [`coder_todos_v1.md`](coder_todos_v1.md)

Consolidated **open** work after BQ-C4 · APSR-S1/S2 · BQ-H1/H2/H3 · APSR-Q1 · SCH-W1-E1 closure.

---

## P0 — operator / display required

| ☐ | ID | Owner | Exit |
|:---:|:---|:---|:---|
| ☐ | **DR-RTT-VR16** | operator / @coder_b | `cargo run --release -- --test vfx` · refresh `stage5_full_app_live.json` |
| ☐ | **PERF-INSTR-VFX-002 accept** | operator | Lib ☑ — display run for `steady_wall_p50_ms ≤ 33` |

---

## P1 — coder / coder-mcp (parallel OK)

| ☐ | ID | Owner | Scope | Exit witness |
|:---:|:---|:---|:---|:---|
| ☑ | **APSR-A4-Q1-001** | @coder-mcp | QC strip ← BQ-A2 on Assembly tab | `apsr_a4_q1_001_live.json` · pytest 3/3 |
| ☑ | **SCH-W1-E1-001** | @coder | Ambiguity warn baseline (debug builds) | `sch_w1_e1_001_live.json` · `configure_schedules` wired |
| ☑ | **SCH-W1-T1-001** | @coder | Pause `dt_scale()` on hybrid/settlement ticks | `sch_w1_t1_001_live.json` |
| ☑ | **SCH-W1-E3/E4** | @coder | BuildProfiles fire edge · HybridSim logistics inject | `sch_w1_e3/e4_001_live.json` |
| ☑ | **CLN-P0-S8/P10** | @coder | spacial println! · EC-LOG frequency doc | `cln_p0_s8/p10_001_live.json` |
| ☑ | **BQ-Q3** | @coder | Golden-seed regression (12 seeds) | `bq_q3_golden_001_live.json` |
| ☑ | **CLN-P0-R4/R8/T4/T7/T6** | steward/operator | Phase 0 steward rows · `cln_p0_*_001_live.json` |
| ☐ | **RGR-M1-001** | @coder | `render/api.rs` + mod.rs pub use collapse | `render_gui_refactor_queue.json` · stage5 lib |

---

## P2 — Stream 4 tail (BQ-H/K/Q)

| ☐ | ID | Owner | Scope | Exit |
|:---:|:---|:---|:---|:---|
| ☑ | **BQ-H3-V0-RETIRE-001** | @coder | v0 grammar freeze shim | `bq_h3_v0_retire_001_live.json` |
| ☑ | **BQ-Q1-WITNESS-001** | @coder-mcp | APS QC strip reads `building_quality_live.json` | Assembly tab · `apsr_a4_q1_001_live.json` |
| ☐ | **BQ-Q2** | @operator | Screenshot QC · preview worker |
| ☑ | **BQ-K1/K2/K3** | designer-mcp | Kit charters · coverage — **SIGNED 2026-07-03** · `debug_runs/bq_k_lane_charters_live.json` |
| ☑ | **BQ-K1-KITFILL-001** | @designer-mcp | Brick/wood/concrete kit charters | 11 job specs → **@coder-mcp** bake |
| ☑ | **BQ-K2-COVERAGE-001** | @designer-mcp | 100% slot coverage audit | pytest green · purity gaps pending K1 wire |
| ☑ | **BQ-K3-GRAMMAR-001** | @designer-mcp | +massing strategies · FacadeRule tables | manifest → **@coder** RON merge |

---

## P2 — infra / perf (post-migration)

| ☐ | ID | Owner | Note |
|:---:|:---|:---|:---|
| ☑ | **GPU P0-C′-PRIME + P0-D/E partial** | @coder | PRIME + stamps + minimap source — [`gpu_todos_v1.md`](gpu_todos_v1.md) |
| ☑ | **SCH-W1-P1** | @sim-steward | Dormant production plugins deleted · `sch_w1_p1_001_live.json` |

---

## Blocked — do not pick

| DR-* | Item |
|:---|:---|
| **DR-MIG-TILEMAP** | `bevy_ecs_tilemap` 0.19 upstream |
| **DR-CITY-C6-BSN** | BSN charter after C6 visual |
| **DR-CLEANUP-P2** | Cleanup Phase 2+ until Phase 0 started |
| **DR-SCHED-W2** | Schedule Wave 2 until Wave 1 green |

---

## Done (2026-07-03) — do not re-pick

**RGR Phase 0 (@coder_b CHAIN-B):** CB-MIG-001..003 · CB-CLN-001 · CB-BQ-001/002 · CB-CITY-001/002 · CB-RGR-001 · witness `chain_b_witness_live.json`

BQ-C4 · BQ-A1/A2 · BQ-F1/F2/F3 · BQ-C1/C2/C3 · APSR-S1/S2 · BQ-H1/H2 · BQ-K1/K2/K3 charters · MIG program close · CITY G0–G3 · **RTT CHAIN-A lib** · `rtt_lane_witness_live.json`

---

## Session verify

```powershell
cargo test -p proc_A_dine01 --lib bq_h3 sch_w1 building_quality facade_propagation -q
cd tools/mcp/python && python -m pytest tests/test_building_quality_qc.py tests/test_dmcp_bq_k_lane.py tests/test_aps_panel_sync_characterization.py -q
```
