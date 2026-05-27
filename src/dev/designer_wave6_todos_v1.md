# Designer wave 6 todos `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Trigger** | Wave 5 contracts delivered; coders open next visual slices |
| **Workboard** | [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/designer_active_queue.json`](../tools/orchestrator/queues/designer_active_queue.json) |

**Rule:** Design / review / sign-off records only. No Rust.

---
## Already SIGNED / ready-for-signoff (P1 coder slices)

| ID | Verdict | Record |
|:---|:---|:---|
| **DESIGN-UI-P6-MULTIVIEW-001** | DEFER | [`ui_phase6_multiview_readability_v1.md`](ui_phase6_multiview_readability_v1.md) |
| **DESIGN-THEME-COLLAGE-001** | PASS (qualified) | [`ui_theme_collage_delta_v1.md`](ui_theme_collage_delta_v1.md) |
| **DESIGN-WP-QUALIFIED-UPGRADE-001** | PASS (qualified) | [`world_preview_visual_upgrade_checklist_v1.md`](world_preview_visual_upgrade_checklist_v1.md) |
| **DESIGN-VFX-CAPTURE-WAVE5-001** | DEFER | `assets/vfx/reference/review_captures/vfx_capture_status_wave5.md` |
| **DESIGN-OPERATOR-VISUAL-BUNDLE-001** | PASS (qualified) | [`operator_visual_signoff_design_checklist_v1.md`](operator_visual_signoff_design_checklist_v1.md) |

---
## Master board (wave 6)

| ☐ | # | ID | Unblocks | Verdict |
|:---:|:---:|:---|:---|:---|
| ☑ | 10 | **DESIGN-UI-P6-MULTIVIEW-001** | `UI-W3-P6-001` | DEFER |
| ☑ | 11 | **DESIGN-THEME-COLLAGE-001** | `PLAN-UI-THEME-MERGE-001` | PASS (qualified) |
| ☑ | 12 | **DESIGN-WP-QUALIFIED-UPGRADE-001** | `UI-WP-VISUAL-001` | PASS (qualified) |
| ☑ | 13 | **DESIGN-VFX-CAPTURE-WAVE5-001** | `VFX-CAPTURE-WAVE5` (operator capture lane) | DEFER |
| ☑ | 14 | **DESIGN-OPERATOR-VISUAL-BUNDLE-001** | `PLAN-OPERATOR-VISUAL-BUNDLE-001` | PASS (qualified) |

---
## Handoff note: which designer specs block which coder A/B rows
| Coder row lane (planner wave 6) | Blocked by designer spec(s) |
|:---|:---|
| **Coder A row 1 (Fire wake/sleep UX)** | `DESIGN-F7-STREAM-001`, `DESIGN-F7-DEBUG-PASS-001` |
| **Coder A row 2 (VT-5 spread triage policy)** | `DESIGN-VT-SPREAD-001` |
| **Coder B row 1-2 (R4 corridor edge overlay)** | `DESIGN-R4-CORRIDOR-001`, `DESIGN-R4-TRAY-001` |
| **Coder B row 3 (R4 MV acceptance)** | `DESIGN-R4-MV-PASS-001` |
| **Coder B row 4-5 (M3 depth + replay live)** | `DESIGN-M3-DEPTH-001`, `DESIGN-REPLAY-LIVE-001` |
| **Coder B row 6 (M2 tray bridge)** | `DESIGN-M3-TRAY-001` |
| **Coder A/B row 7 (UI multiview readability)** | `DESIGN-UI-P6-MULTIVIEW-001` |
| **Coder A row 8 (theme collage tokens)** | `DESIGN-THEME-COLLAGE-001` |
| **Coder A row 9 (world preview upgrade checklist)** | `DESIGN-WP-QUALIFIED-UPGRADE-001` |
| **Operator capture lane (VFX wave 5)** | `DESIGN-VFX-CAPTURE-WAVE5-001` |
| **Operator visual bundle lane** | `DESIGN-OPERATOR-VISUAL-BUNDLE-001` |

---
## Witness sync + coder policy (2026-05-26)

- **DEFER** on a designer row means **witness pending implementation**, not “wait for designer.” Specs are sufficient to code.
- **Still DEFER (wave 6):** `DESIGN-UI-P6-MULTIVIEW-001` (`ui_w3_p6_001.green` false), `DESIGN-VFX-CAPTURE-WAVE5-001` (operator PNGs).
- **Parallel coder lanes (active):** `@coder B` **R4-MV-GHOST-001**, **M3-UNITS-DEPTH-001**, **REPLAY-RING-LIVE-001**; `@coder A` wave-6 fire/UI depth per planner queue.

*** End of wave 6 todo deck ***

