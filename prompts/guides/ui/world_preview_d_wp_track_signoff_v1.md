# World Preview layout track — `D-WP` sign-off `v1`

| Field | Value |
|:---|:---|
| **Review ID** | **D-WP** (World Preview chrome track) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` → `@coder` **UI-WP-*** slices |
| **Status** | **SIGNED** (design) — **partial implementation** |
| **Parent gate** | [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) §5 (D-01…D-12) |
| **Worksheet** | [`world_preview_layout_decision_worksheet_v1.md`](world_preview_layout_decision_worksheet_v1.md) |
| **Coder queue** | [`ui_world_preview_coder_queue_v1.md`](ui_world_preview_coder_queue_v1.md) |
| **Mock** | [`assets/ui/world_preview/layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png) |
| **Witness** | `debug_runs/stage5_full_app_live.json` → `world_preview_layout` |

---

## Executive summary

**D-WP** rolls up signed layout decisions **D-01…D-12** into one track verdict. **W0 design is SIGNED**; **D-01 shell is coded and done**; **D-02…D-12** are signed on paper with **selective** coder landing.

**Not** simulation HUD ([`ui_phase2_designer_signoff_v1.md`](ui_phase2_designer_signoff_v1.md) P1–P4).

---

## §5 — Decision rollup

| ID | Choice | Design | Code | Focused sign-off |
|:---|:---:|:---|:---|:---|
| **D-01** | A | ☑ | ☑ **done** | [`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md) |
| **D-02** | A | ☑ | ◐ optional | [`world_preview_d02_map_dominance_signoff_v1.md`](world_preview_d02_map_dominance_signoff_v1.md) |
| **D-03** | A | ☑ | ☑ partial | Left sidebar stack in `window.rs` |
| **D-04** | A | ☑ | ◐ stub | Slide sheet hook; body/dim → **UI-WP-LAYOUT-002** |
| **D-05** | B | ☑ | ☐ | Layer strip on map top — not landed |
| **D-06** | A | ☑ | ☑ partial | Toolbar zoom/GPU in header |
| **D-07** | A | ☑ | ☐ | **Gap:** sidebar thumb today; mock = **corner inset** 120–160px |
| **D-08** | A | ☑ | ☐ | egui `Frame` only — WP-L1 paper textures deferred |
| **D-09** | A | ☑ | ☐ | Fixed asymmetry offsets deferred |
| **D-10** | A | ☑ | ☐ | Registration ticks — deferred |
| **D-11** | B | ☑ | ◐ partial | `MAP_PANEL_INSET_PX`; 12% margin not enforced |
| **D-12** | A | ☑ | ☐ | 400ms dissolve on enter sim — **UI-WP-MOTION-001** |

**Overrides:** none.

---

## Witness (2026-05-24)

From [`debug_runs/stage5_full_app_live.json`](../../../debug_runs/stage5_full_app_live.json):

```json
"world_preview_layout": {
  "d01_unified_workspace": true,
  "ui_wp_layout_001": "signed"
}
```

| Check | Pass if |
|:---|:---|
| Unified workspace | `d01_unified_workspace: true` | ☑ |
| D-01 coder slice | `ui_wp_layout_001: "signed"` | ☑ |
| Stage 5 spine | `stage5_closure.passes: true` | ☑ (orthogonal) |

---

## Coder slice map

| Slice | Decisions | Status |
|:---|:---|:---|
| **UI-WP-DESIGN** | D-01…D-12 worksheet + mock | **done** |
| **UI-WP-LAYOUT-001** | D-01 | **done** |
| **UI-WP-LAYOUT-D02-OPT** | D-02 | **optional** |
| **UI-WP-LAYOUT-002** | D-04 | **queued** |
| **UI-WP-LAYOUT-003** | D-08, D-09, D-10 | deferred (WP-L1) |
| **UI-WP-MOTION-001** | §6 + D-12 | deferred |
| **WP-L4** | map look (capturez) | deferred |
| *(new)* **UI-WP-LAYOUT-D07** | D-07 corner inset | **queued** — move minimap off sidebar |

**Recommended next coder order:** **UI-WP-LAYOUT-002** (D-04) → **UI-WP-LAYOUT-D07** (D-07) → optional D-02 → motion.

---

## §11 Designer sign-off checklist (D-WP)

| # | Item | Done |
|:---|:---|:---:|
| 1 | Worksheet D-01…D-12 **SIGNED** | ☑ |
| 2 | Mock committed | ☑ |
| 3 | D-01 focused sign-off + code | ☑ |
| 4 | D-02 optional sign-off filed | ☑ |
| 5 | Honest gap table (D-05…D-12) | ☑ |
| 6 | Track does **not** claim full mock parity | ☑ |

**Verdict:** ☑ **SIGNED** (design complete; implementation **in progress**)

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-24 | **SIGNED** |
| Coder | 2026-05-24 | **D-01 done**; remainder per slice table |

---

## Do-not-touch (track-wide)

| Rule | Reason |
|:---|:---|
| No `render_raster.rs` in layout-only slices | Presentation chrome only |
| No gameplay mutation from preview UI | [`world_preview_runbook_v1.md`](../world_preview_runbook_v1.md) |
| D-01 must not regress | Single workspace invariant |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | **D-WP** track rollup; D-01 done; D-07 gap documented |
