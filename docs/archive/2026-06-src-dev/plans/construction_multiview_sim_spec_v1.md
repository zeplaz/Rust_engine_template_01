# PLAN-CONSTRUCTION-MV-001 — construction multiview sim spec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-CONSTRUCTION-MV-001** |
| **Coder lane** | **CONSTRUCTION-MV-SIM-001** (alias **CONSTRUCTION-MV-001**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — sim writer **CLOSED** on lib + disk (2026-05-26) |
| **Infra context** | [`ui_phase6_shell_perf_multiview_plan_v1.md`](ui_phase6_shell_perf_multiview_plan_v1.md) · **DQ-POST-04** |
| **Designer** | [`construction_multiview_ghost_readability_v1.md`](construction_multiview_ghost_readability_v1.md) **SIGNED** (DESIGN-CONSTRUCTION-MV-001) |
| **Witness** | `debug_runs/construction_stage_live.json` |
| **Invariants** | [`construction_invariants.md`](construction_invariants.md) |

**No Rust in this deliverable.**

---

## Executive summary

| Track | Verdict |
|:---|:---|
| **Module spine** | **PASS** — `map_egui_projection`, `visual_authority`, road/zone ghosts |
| **View authority hook** | **PASS** — `ViewProjectionAuthority` + `SimulationMap` in sim |
| **Sim live writer** | **PASS** — `write_construction_live_proof_system` in **Simulation** |
| **Witness rollup** | **PASS** — `construction_mv_001.green: true` on disk |
| **Full multiview product** | **PARTIAL** — qualified; not full `--test visual` per-view ghost audit |

**Distinction:** **CONSTRUCTION-MV-001** proves **sim-session** JSON + wiring. It does **not** close all VM-06…11 isolation audits or Round 4 construction product boards.

---

## Naming

| ID | Meaning |
|:---|:---|
| **CONSTRUCTION-MV-001** | Witness gate id in `construction_stage_live.json` |
| **CONSTRUCTION-MV-SIM-001** | Coder queue row — **same** exit criteria |
| **DQ-POST-04** | Policy: ghosts use `ViewManager` / map projection, not egui-only authority |

---

## Authority map

```text
Build tool input (construction/)
  → build_ghost / roads/ghost / zones/ghost
  → visual_authority + map_egui_projection
  → ViewProjectionAuthority (SimulationMap)
  → presentation extract (no second execute funnel)
```

| Domain | Sole writer | Forbidden |
|:---|:---|:---|
| Site commit | `CommitConstructionSiteEvent` → construction pipeline | Preview / WorldGen execute |
| Ghost pose | construction visual authority + projection | Raw `MapCameraDesired` as ghost authority |
| Live proof JSON | `write_construction_live_proof_system` | Hand-edited greens |

---

## Witness contract — `construction_stage_live.json`

| Path | Type | Green when |
|:---|:---|:---:|
| `construction_mv_001.gate` | string | `"CONSTRUCTION-MV-001"` |
| `construction_mv_001.green` | bool | rollup below |
| `construction_mv_001.multiview_ghosts_wired` | bool | `true` |
| `operational_green` | bool | `true` (construction operational spine) |

**Rollup formula:**

```text
construction_mv_001.green :=
  multiview_ghosts_wired
  AND ghost_commit_isolated   (ConstructionStageWitness)
  AND road_ghost_draw
```

**Source of truth:** [`ConstructionStageWitness::refresh_construction_stage_witness`](../../src/construction/construction_stage_witness.rs) — `multiview_ghosts_wired` requires modules + sim `ViewProjectionAuthority` commit.

---

## PASS gates

| # | Criterion | Evidence |
|:---:|:---|:---|
| MV-1 | Sim-only writer | `write_construction_live_proof_system` runs when `BaseState::Simulation` |
| MV-2 | MV modules present | `map_egui_projection.rs`, `visual_authority.rs`, ghost paths |
| MV-3 | Authority in sim | `ViewSurfaceId::SimulationMap` committed in proof harness |
| MV-4 | JSON rollup | `construction_mv_001.green: true` |
| MV-5 | Lib regression | `simulation_writes_construction_stage_live_json_operational_green` |
| MV-6 | Bundle refresh | `refresh_construction_mv_001_live_witness()` optional |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json_operational_green
cargo test -p proc_A_dine01 --lib coder_b_s7p_construction_mv_proof
```

**Operator (optional):** enter **Simulation** with construction tool — confirm ghosts on map hole, not editor preview surface.

---

## Out of scope

| Item | Lane |
|:---|:---|
| Round 4 catalog reconcile | **CONSTRUCTION-R4-PREP-001** |
| Per-view ghost pixels in WorldPreview | Wave P / editor |
| VM-08 fire overlay isolation | infra witness separate |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **PLAN-CONSTRUCTION-MV-001** signed — sim path green |
