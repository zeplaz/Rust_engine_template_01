# DESIGN-R4-MV-PASS-001 — corridor overlay acceptance `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-R4-MV-PASS-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **DEFER** (designer ACCEPT; R4 MV witness refresh is gated on product board open) |
| **Unblocks** | `R4-MV-GHOST-001` |
| **Witness** | `debug_runs/construction_stage_live.json` → `/construction_mv_001/green` and `/construction_r4_prep_001/product_board_open` |
| **Witness alignment (expected)** | future: `debug_runs/construction_stage_live.json` → `/construction_r4_mv_ghost_001/mv_001_still_green` and `/construction_r4_mv_ghost_001/corridor_overlay_tokens_wired` |
| **Do not break** | `/construction_mv_001/green` and `/construction_r4_prep_001/green` |

---
## Scope
Acceptance record for **corridor phase overlay** on:
- `SimulationMap`
- `WorldMain` (map hole tactical view)

---
## Witness alignment: `construction_r4_mv_ghost_001` fields
The record must be considered green only when the MV witness rolls up:
1. `construction_r4_mv_ghost_001/gate == "DESIGN-R4-MV-001"`
2. `construction_r4_mv_ghost_001/green` rollup is true
3. `construction_r4_mv_ghost_001/corridor_overlay_tokens_wired` matches the token table in `R4-PLAN-002`
4. `construction_r4_mv_ghost_001/legend_wired` indicates tray shows corridor phase key
5. `construction_r4_mv_ghost_001/mv_001_still_green` remains true (no MV-001 regression)

---
## `mv_001_still_green` checklist
All must hold:
- Road valid/invalid polylines remain distinct (MV-001 baseline)
- Corridor overlay does not replace MV-001 tokens; it only adds R4 delta
- Completed corridor edges drop overlay (committed palette only)

---
## Acceptance matrix (designer)
| # | Pass | Fail |
|:---:|:---|:---|
| 1 | Planned edges show dashed amber overlay | Invisible overlay / wrong pattern |
| 2 | InProgress edges show partial fill direction | Binary on/off only |
| 3 | Completed edges show committed road palette only | Double-drawn thick overlay |
| 4 | `WorldMain` matches `SimulationMap` presentation path | Wrong surface |

---
## Optional capture (not required for DEFER)
- `assets/ui/construction/r4_corridor_phase_target_v1.png`

