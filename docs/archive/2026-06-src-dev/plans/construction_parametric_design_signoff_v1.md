# CONSTRUCTION-PARAM-DESIGN-001 — Design sign-off record `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **CONSTRUCTION-PARAM-DESIGN-001** |
| **Prereq** | **PLAN-CONSTRUCTION-PARAM-001** **SIGNED** — [`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) |
| **Product spec** | [`construction_parametric_placement_spec_v1.md`](construction_parametric_placement_spec_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** — design deliverables complete; sim witness rollup is coder gate |
| **Unblocks** | **CONSTRUCTION-PARAM-CODER-001** … **CONSTRUCTION-PARAM-CODER-006** |
| **Do not break** | `construction_mv_001.green`, `construction_r4_corridor_001.green`, construction invariants § preview/execute |

---

## Deliverables (this lane)

| # | Doc | Scope |
|:---:|:---|:---|
| 1 | [`construction_parametric_placement_design_v1.md`](construction_parametric_placement_design_v1.md) | **Master** — tray, staged list, hints, partial-alpha |
| 2 | [`construction_parametric_tray_mock_v1.md`](construction_parametric_tray_mock_v1.md) | Tray slice (toggle, readout, hints) |
| 3 | [`construction_parametric_staged_panel_v1.md`](construction_parametric_staged_panel_v1.md) | Staged list slice |
| 4 | [`construction_parametric_ghost_visual_v1.md`](construction_parametric_ghost_visual_v1.md) | Map visual slice — **CODER-005** |
| 5 | [`construction_parametric_staging_ux_v2.md`](construction_parametric_staging_ux_v2.md) | Staging polish v2 — **PASS** — **CODER-004** |
| 6 | [`construction_parametric_scale_hud_v1.md`](construction_parametric_scale_hud_v1.md) | Scale economy HUD — **PASS** — **CODER-006** |
| 7 | This record | Sign-off + witness pointers |

**No egui implementation** in designer lane — mocks + tokens only.

---

## Verdict rationale

| Criterion | Status |
|:---|:---|
| Planner authority SIGNED | ✓ |
| Tray + hints replace Shift+LMB copy | ✓ doc 1 |
| Staged panel columns + footer | ✓ doc 2 |
| MV-001 / R4 hue compatibility | ✓ doc 3 |
| Witness rollup on disk | ✗ — **coder** gate only |

**Policy:** Designer **PASS** recorded on mock/spec deliverables. Sim `construction_parametric_placement_001.green` does not block design sign-off.

---

## Unblocks matrix

| Coder ID | Primary design input |
|:---|:---|
| **CONSTRUCTION-PARAM-CODER-001** | Ghost visual § overlap red; staged validity column |
| **CONSTRUCTION-PARAM-CODER-002** | Tray mock § hints + toggle; scale/rotate input |
| **CONSTRUCTION-PARAM-CODER-003** | Staged panel § Build approved drain |
| **CONSTRUCTION-PARAM-CODER-004** | Staged panel full layout |
| **CONSTRUCTION-PARAM-CODER-005** | Ghost visual token table |
| **CONSTRUCTION-PARAM-CODER-006** | Tray readout economy lines |

---

## Witness pointers (future keys)

**File:** `debug_runs/construction_stage_live.json`  
**Block:** `construction_parametric_placement_001`

| Pointer | Flip designer to **PASS** when |
|:---|:---|
| `/construction_parametric_placement_001/gate` | `"CONSTRUCTION-PARAM-001"` |
| `/construction_parametric_placement_001/weighted_raster_tests_green` | `true` |
| `/construction_parametric_placement_001/shift_queue_building_removed` | `true` |
| `/construction_parametric_placement_001/enter_commits_single_ghost` | `true` |
| `/construction_parametric_placement_001/staging_toggle_wired` | `true` |
| `/construction_parametric_placement_001/build_approved_drains_staged` | `true` |
| `/construction_parametric_placement_001/overlap_blocks_commit` | `true` |
| `/construction_parametric_placement_001/commit_carries_scale_and_weights` | `true` |
| `/construction_parametric_placement_001/economy_scales_at_activation` | `true` |
| `/construction_parametric_placement_001/green` | `true` (rollup) |

**Do not break (regression guards):**

- `/construction_mv_001/green`
- `/construction_r4_corridor_001/green`

---

## Acceptance checklist (sign-off)

1. Stage placements defaults **OFF**; Enter commits single ghost (per planner decision).
2. Staging ON: LMB adds snapshot; footer buttons match spec strings exactly.
3. Tool hints never mention Shift+LMB blueprint queue.
4. Weighted tiles use alpha = weight; hue from MV-001 validity family.
5. Staged map ghosts 25% desaturated + dashed bound.
6. Overlap shows red tile weights and blocks commit.

---

## Sign-off table

| Role | Verdict | Date |
|:---|:---|:---|
| `@planner` | **PASS** (PLAN-CONSTRUCTION-PARAM-001) | 2026-05-26 |
| `@designer` | **PASS** | 2026-05-26 |
| `@coder` | **Unblocked** — implement 001…006 per plan phases | 2026-05-26 |
