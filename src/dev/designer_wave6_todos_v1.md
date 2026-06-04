# Designer wave 6 todos `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.2.0` |
| **Date** | 2026-05-27 |
| **Trigger** | Wave 5 contracts delivered; coders open next visual slices |
| **Workboard** | [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/designer_active_queue.json`](../tools/orchestrator/queues/designer_active_queue.json) |

**Rule:** Design / review / sign-off records only. No Rust.

---
## Already SIGNED / ready-for-signoff (P1 coder slices)

| ID | Verdict | Record |
|:---|:---|:---|
| **DESIGN-UI-P6-MULTIVIEW-001** | PASS (qualified) | [`ui_phase6_multiview_readability_v1.md`](ui_phase6_multiview_readability_v1.md) |
| **DESIGN-THEME-COLLAGE-001** | PASS (qualified) | [`ui_theme_collage_delta_v1.md`](ui_theme_collage_delta_v1.md) |
| **DESIGN-WP-QUALIFIED-UPGRADE-001** | PASS (qualified) | [`world_preview_visual_upgrade_checklist_v1.md`](world_preview_visual_upgrade_checklist_v1.md) |
| **DESIGN-VFX-CAPTURE-WAVE5-001** | PASS (qualified) | [`vfx_capture_status_wave5.md`](../assets/vfx/reference/review_captures/vfx_capture_status_wave5.md) |
| **DESIGN-OPERATOR-VISUAL-BUNDLE-001** | PASS (qualified) | [`operator_visual_signoff_design_checklist_v1.md`](operator_visual_signoff_design_checklist_v1.md) |
| **DESIGN-ATMOS-CLIPMAP-READ-001** | PASS | [`wss_contamination_visual_language_v1.md`](wss_contamination_visual_language_v1.md) |

---
## Master board (wave 6)

| ☐ | # | ID | Unblocks | Verdict |
|:---:|:---:|:---|:---|:---|
| ☑ | 10 | **DESIGN-UI-P6-MULTIVIEW-001** | `UI-W3-P6-001` | PASS (qualified) |
| ☑ | 11 | **DESIGN-THEME-COLLAGE-001** | `PLAN-UI-THEME-MERGE-001` | PASS (qualified) |
| ☑ | 12 | **DESIGN-WP-QUALIFIED-UPGRADE-001** | `UI-WP-VISUAL-001` | PASS (qualified) |
| ☑ | 13 | **DESIGN-VFX-CAPTURE-WAVE5-001** | `VFX-CAPTURE-WAVE5` (operator capture lane) | PASS (qualified) |
| ☑ | 14 | **DESIGN-OPERATOR-VISUAL-BUNDLE-001** | `PLAN-OPERATOR-VISUAL-BUNDLE-001` | PASS (qualified) |
| ☑ | 15 | **DESIGN-ATMOS-CLIPMAP-READ-001** | `A-W2`, `WSS-ATMOS-CLIPMAP-001` | PASS |

---
## Handoff note: spec references for current coder A/B rows
| Coder row lane (planner wave 6) | Blocked by designer spec(s) |
|:---|:---|
| **Coder A row 1 (Fire wake/sleep UX)** | `DESIGN-F7-STREAM-001`, `DESIGN-F7-DEBUG-PASS-001` |
| **Coder A row 2 (VT-5 spread triage policy)** | `DESIGN-VT-SPREAD-001` |
| **Coder B deferred row (R4 MV acceptance)** | `DESIGN-R4-MV-PASS-001` |
| **Coder B active row 1-2 (M3 depth + replay live)** | `DESIGN-M3-DEPTH-001`, `DESIGN-REPLAY-LIVE-001` |
| **Coder B row 6 (M2 tray bridge)** | `DESIGN-M3-TRAY-001` |
| **Coder A/B row 7 (UI multiview readability)** | `DESIGN-UI-P6-MULTIVIEW-001` |
| **Coder A row 8 (theme collage tokens)** | `DESIGN-THEME-COLLAGE-001` |
| **Coder A row 9 (world preview upgrade checklist)** | `DESIGN-WP-QUALIFIED-UPGRADE-001` |
| **Operator capture lane (VFX wave 5)** | `DESIGN-VFX-CAPTURE-WAVE5-001` |
| **Operator visual bundle lane** | `DESIGN-OPERATOR-VISUAL-BUNDLE-001` |

---
## Witness sync + coder policy (2026-05-26)

- **DEFER** on a designer row means **witness pending implementation**, not “wait for designer.” Specs are sufficient to code.
- **Wave 6 P1 promoted 2026-05-26:** `DESIGN-UI-P6-MULTIVIEW-001` and `DESIGN-VFX-CAPTURE-WAVE5-001` → **PASS (qualified)** (checklists complete; witness/PNG rollup optional).
- **Parallel coder lanes (active):** `@coder B` **M3-UNITS-DEPTH-001**, **REPLAY-RING-LIVE-001**, **UI-P3-M2-TRAY-OPT**; `R4-MV-GHOST-001` deferred.

*** End of wave 6 todo deck ***

