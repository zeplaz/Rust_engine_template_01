# R4 corridor edge overlay UX `v1` (DESIGN-R4-CORRIDOR-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-R4-CORRIDOR-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** (witness on disk 2026-05-26) |
| **Unblocks** | `R4-CORRIDOR-001` (coder B — **done** on `construction_r4_corridor_001`) |
| **Witness** | `debug_runs/construction_stage_live.json` → `/construction_r4_corridor_001/green`, `/construction_r4_corridor_001/corridor_phase_visual_wired`, `/construction_r4_corridor_001/product_board_open` |
| **Planned witness extension** | `R4-MV-GHOST-001`: `/construction_r4_mv_ghost_001/corridor_overlay_tokens_wired` (not landed yet) |
| **Do not break** | `debug_runs/construction_stage_live.json` → `/construction_mv_001/green` and `/construction_r4_prep_001/green` |

---
## Purpose
Spec operator-visible **corridor phase edge overlay** on the **R4 sim map**.

Non-goals (explicit):
- No new execute/commit funnel.
- No egui-only painting.
- No minimap polylines (minimap heat dim only).

---
## Visual contract (map edge overlay)
Overlays apply on top of terrain and under tool ghosts (same projection as MV-001 corridor overlay family).

| Construction phase | Stroke | Color | Pattern / behavior |
|:---|:---:|:---|:---|
| **Planned** | 3px | `#E8B040` | dashed: 8px on / 4px off |
| **InProgress** | 4px | `#50A0E8` | solid; alpha multiplied by `progress` along the edge polyline |
| **Completed** | use committed palette | `road_committed_color()` | no duplicate overlay pass |

---
## Minimap heat dim when traffic_factor < 1
| Minimap requirement | Spec |
|:---|:---|
| Heat dim | When `traffic_factor < 1`, reduce minimap heat intensity |
| No polylines | Do not draw edge polylines on minimap (heat/markers only) |

---
## Map editor Save affordance
Operator must be able to visually confirm corridor phase state when using **Map editor Save affordance**.

Contract:
- Save occurs with corridor phase in sim/witness.
- Overlay reflects the saved phase (Planned/InProgress/Completed) immediately after commit pipeline refresh.

---
## Accessibility (color + pattern)
Rules:
- Phase mapping uses both hue and pattern (Planned dashed; InProgress solid with progress alpha).
- Do not rely on color-only to distinguish Planned vs InProgress.

---
## Acceptance checklist (designer)
1. Planned corridor edges are dashed amber and readable on forest + desert.
2. InProgress edges show partial fill direction (progress alpha grows along the edge).
3. Completed edges drop corridor overlay and show committed road palette only.
4. Minimap dims heat for `traffic_factor < 1` and never shows edge polylines.

