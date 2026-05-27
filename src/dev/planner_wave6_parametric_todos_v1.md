# Planner wave 6 — parametric placement `v1`

| Field | Value |
|:---|:---|
| **Parent** | [`planner_wave6_todos_v1.md`](planner_wave6_todos_v1.md) (**PLAN-WAVE-6-001**) |
| **Date** | 2026-05-26 |
| **Rule** | Planner sign-off + queue hygiene only (no Rust) |

---

## Master board

| ☐/☑ | # | ID | Deliverable | Status | Unblocks |
|:---:|---:|:---|:---|:---|:---|
| ☑ | 1 | **PLAN-CONSTRUCTION-PARAM-001** | [`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) **SIGNED** | **DONE** 2026-05-26 | **CONSTRUCTION-PARAM-DESIGN-001** |
| ☐ | 2 | **CONSTRUCTION-PARAM-DESIGN-001** | Tray toggle, staged list, hint strings, partial-alpha ghost spec | @designer **OPEN** | **CONSTRUCTION-PARAM-CODER-001…006** |
| ☐ | 3 | **CONSTRUCTION-PARAM-CODER-001** | `weighted_footprint.rs` + tests | @coder B **blocked** | Phase 1 witness |
| ☐ | 4 | **CONSTRUCTION-PARAM-CODER-002** | Ghost scale/rotate input; deprecate Shift queue (buildings) | @coder B **blocked** | Phase 2 |
| ☐ | 5 | **CONSTRUCTION-PARAM-CODER-003** | Commit path + `TileOccupationBook` + `SiteWeightedFootprint` | @coder B **blocked** | Phase 1 exit |
| ☐ | 6 | **CONSTRUCTION-PARAM-CODER-004** | Staged panel egui | @coder B **blocked** | Phase 3 |
| ☐ | 7 | **CONSTRUCTION-PARAM-CODER-005** | Visual authority partial-alpha tiles | @coder B **blocked** | Phase 2 |
| ☐ | 8 | **CONSTRUCTION-PARAM-CODER-006** | Economy scale activation | @coder B **blocked** | Phase 4 |

**Exec slices:** [`plan_construction_param_exec_phases_v1.md`](plan_construction_param_exec_phases_v1.md)

**Witness:** `debug_runs/construction_stage_live.json` → `construction_parametric_placement_001`

**Priority vs Round 4:** Parametric lane runs **after** **R4-MV-GHOST-001** if in flight (`coder_active_queue.json` → `parametric_placement`).

**Do not reopen:** Round 4 corridor witness-only slice, F7 exit gates, steward preflights, dual-queue closure rows.
