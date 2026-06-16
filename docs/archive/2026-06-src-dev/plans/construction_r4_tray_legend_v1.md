# R4 tray/legend UX `v1` (DESIGN-R4-TRAY-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-R4-TRAY-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **DEFER** (designer ACCEPT; product board gate blocks R4 MV witness refresh) |
| **Unblocks** | `R4-TRAY-001` (coder B1/B2 bridge for corridor phase legend) |
| **Witness** | `debug_runs/construction_stage_live.json` → `/construction_r4_prep_001/green` and `/construction_r4_prep_001/product_board_open` |
| **Planned witness extension** | expected future: `debug_runs/construction_stage_live.json` → `/construction_r4_mv_ghost_001/legend_wired` |
| **Do not break** | `debug_runs/construction_stage_live.json` → `/construction_r4_prep_001/green` and `/construction_mv_001/green` |

---
## Scope
Expand the R4 Round-4 tray/legend footer for corridor phases.

Pairs with `R4-PLAN-002` tray/legend baseline; this doc specifies the R4 **legend** delta for corridor edges.

---
## Legend swatches and labels
Footer layout:
- **48+52 layout**: left rail swatch column (48px region) + right label column (52px region)
- 3 swatches visible at all times; labels change with visibility rules below.

Swatches:
| Phase | Swatch token | Label |
|:---|:---|:---|
| **Planned** | `#E8B040` | `Planned` |
| **InProgress** | `#50A0E8` | `Building` |
| **Open/Idle** | UI neutral (not a construction color) | `Open` |

---
## Visibility and interaction rules
Legend visibility:
- Show legend when **any** corridor row is not Completed.
- Also show when the **road tool** is active.

Interaction with road tool + existing build rail:
- Legend must not occlude tool controls.
- Swatch labels must remain stable while panning/zooming.

---
## Acceptance checklist (designer)
1. Swatches match the corridor overlay phase mapping (Planned dashed, InProgress solid).
2. Legend appears only when road tool/corridor-in-progress is relevant.
3. Open/Idle label uses neutral styling and does not mimic phase colors.

