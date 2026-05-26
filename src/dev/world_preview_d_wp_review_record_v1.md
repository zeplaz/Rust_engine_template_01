# World Preview post-implementation review — `DESIGN-D-WP-REVIEW-001` `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-D-WP-REVIEW-001** |
| **Review ID** | **D-WP-REVIEW** (aliases: **D-WP-POST-IMPL**, **DESIGN-D-WP-REVIEW**) |
| **Version** | `1.1.0` |
| **Date** | 2026-05-25 |
| **Reviewer** | `@designer` |
| **Status** | **SIGNED — PASS** (operational chrome) |
| **Track rollup** | [`world_preview_d_wp_track_signoff_v1.md`](../prompts/guides/ui/world_preview_d_wp_track_signoff_v1.md) |
| **Worksheet** | [`world_preview_layout_decision_worksheet_v1.md`](../prompts/guides/ui/world_preview_layout_decision_worksheet_v1.md) |
| **Witness JSON** | [`debug_runs/wave_p_live.json`](../debug_runs/wave_p_live.json) · [`debug_runs/stage5_full_app_live.json`](../debug_runs/stage5_full_app_live.json) |
| **Witness refresh** | **COD-B-WP-WITNESS-001** **DONE** — lib refresh 2026-05-25 (`written_at_epoch_secs`: **1779725532**) |
| **Mock** | [`assets/ui/world_preview/layout_mock_v1.png`](../assets/ui/world_preview/layout_mock_v1.png) |

---

## Executive summary

**Designer post-implementation review** of **D-WP** after coder slices **UI-WP-LAYOUT-001**, **UI-WP-LAYOUT-002** (D-04), and **UI-WP-LAYOUT-D07** (D-07) landed with green witnesses.

**Verdict:** ☑ **SIGNED — PASS** for **operational World Preview chrome** — unified workspace, generator slide sheet + dim, corner minimap inset. **Does not** claim full mock parity (D-05, D-08…D-12, WP-L4 remain deferred).

**Not** simulation HUD — see [`ui_phase2_designer_signoff_v1.md`](../prompts/guides/ui/ui_phase2_designer_signoff_v1.md).

---

## Prerequisites

| Gate | Required | Observed | Met |
|:---|:---|:---|:---:|
| **UI4-DESIGN-001** | D-04 spec **SIGNED** | [`world_preview_d04_slide_sheet_spec_v1.md`](../prompts/guides/ui/world_preview_d04_slide_sheet_spec_v1.md) | ☑ |
| **UI-WP-LAYOUT-002** | `ui_wp_layout_002_green` | `true` in `wave_p_live.json` | ☑ |
| **UI-WP-LAYOUT-D07** | `ui_wp_layout_d07_green` | `true`; inset **140px** | ☑ |
| **D-01** | `d01_unified_workspace` | `true` in stage5 + d04 witness | ☑ |
| **wave_p** spine | `wave_p_green` | `true` | ☑ |
| **COD-B-WP-WITNESS-001** | `wave_p_live.json` refreshed | `written_at_epoch_secs` current; layout greens | ☑ |
| Stage 5 | `stage5_closure.passes` | orthogonal — not blocking WP chrome | ☑ |

**Prerequisite verdict:** ☑ **MET**

**Refresh command (lib):**

```powershell
cargo test -p proc_A_dine01 --lib ui_wp_layout_002_writes_wave_p_live_json
```

**Optional (sim):** WorldGen / unified workspace open → `write_wave_p_live_proof_system` updates same path every ~120 frames.

---

## §5 implementation review (D-01…D-12)

| ID | Design | Code (2026-05-25) | Review | Verdict |
|:---|:---:|:---|:---|:---:|
| **D-01** | A | ☑ unified workspace | Matches §9 flow; no second float | **PASS** |
| **D-02** | A | ◐ optional | Map hero; ratio clamp not required for PASS | **DEFERRED** |
| **D-03** | A | ☑ partial | Left sidebar stack in `window.rs` | **PASS** |
| **D-04** | A | ☑ | Sheet body wired; dim α **102** (~40%); width **520px** | **PASS** |
| **D-05** | B | ☐ | Layer strip on map top — not landed | **DEFERRED** |
| **D-06** | A | ☑ partial | Toolbar zoom/GPU in header | **PASS** |
| **D-07** | A | ☑ | Corner inset **140px**; sidebar thumb removed | **PASS** |
| **D-08** | A | ☐ | egui `Frame` only — WP-L1 paper deferred | **DEFERRED** |
| **D-09** | A | ☐ | Asymmetry offsets deferred | **DEFERRED** |
| **D-10** | A | ☐ | Registration ticks deferred | **DEFERRED** |
| **D-11** | B | ◐ partial | `MAP_PANEL_INSET_PX`; 12% margin not enforced | **TUNE** |
| **D-12** | A | ☐ | 400ms dissolve — **UI-WP-MOTION-001** | **DEFERRED** |

**Overrides:** none.

---

## Witness excerpt (`wave_p_live.json`)

```json
"ui_wp_layout_002_green": true,
"ui_wp_layout_d07_green": true,
"ui_wp_layout_002": {
  "d04_unified_workspace": true,
  "d04_map_dim_alpha": 102,
  "d04_sheet_body_wired": true,
  "d04_sheet_width_px": 520.0
},
"ui_wp_layout_d07": {
  "d07_corner_inset_on_map": true,
  "d07_inset_side_px": 140.0,
  "d07_sidebar_minimap_removed": true
}
```

| Check | Pass if | Result |
|:---|:---|:---:|
| D-04 operational | `d04_sheet_body_wired` + dim when sheet open | ☑ |
| D-07 operational | corner on map + sidebar removed | ☑ |
| Track honest | Does not claim D-05…D-12 done | ☑ |

---

## Mock parity notes (non-blocking)

| Element | Mock | Build | Ticket |
|:---|:---|:---|:---|
| Paper frames (D-08) | Torn vellum panels | Flat egui frames | **UI-WP-LAYOUT-003** / WP-L1 |
| Layer strip (D-05) | Map-top tracing strip | Not present | coder backlog |
| 12% archive margin (D-11) | Strong void | Partial inset only | **TUNE** optional |
| Map look (WP-L4) | capturez refs | Raster default | **UI4-DESIGN-003** + **UI-WP-L4-001** |

---

## Coder slice disposition

| Slice | Status | Designer review |
|:---|:---|:---|
| **UI-WP-LAYOUT-001** | **done** | ☑ **PASS** |
| **UI-WP-LAYOUT-002** | **done** | ☑ **PASS** |
| **UI-WP-LAYOUT-D07** | **done** | ☑ **PASS** |
| **UI-WP-LAYOUT-D02-OPT** | optional | No block |
| **UI-WP-LAYOUT-003** | deferred | WP-L1 |
| **UI-WP-MOTION-001** | deferred | D-12 |
| **UI-WP-L4-001** | deferred | WP-L4 |

**Recommended next coder order:** optional **D-02** → **UI-WP-LAYOUT-003** / motion / WP-L4 (no designer gate until WP-L4 refs signed).

---

## §11 Designer sign-off checklist

| # | Item | Done |
|:---|:---|:---:|
| 1 | Read **D-WP** track + worksheet | ☑ |
| 2 | Witness `wave_p_live.json` + stage5 `world_preview_layout` | ☑ |
| 3 | Landed slices D-01 / D-04 / D-07 **PASS** | ☑ |
| 4 | Gap table honest (D-05…D-12) | ☑ |
| 5 | Does **not** claim full mock parity | ☑ |
| 6 | Unblocks optional polish only — no regression on D-01 | ☑ |

**Verdict:** ☑ **SIGNED — PASS** (operational chrome)

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-25 | **SIGNED — PASS** |
| Designer (post **COD-B** refresh) | 2026-05-25 | **Reconfirmed — PASS** (no regression) |

---

## §12 Reconfirmation after **COD-B-WP-WITNESS-001** refresh

| Check | Result |
|:---|:---|
| `cargo test -p proc_A_dine01 --lib ui_wp_layout_002_writes_wave_p_live_json` | **ok** |
| `cargo test -p proc_A_dine01 --lib cod_b_wp_witness_001` | **ok** |
| `cod_b_wp_witness_001_green` | **true** |
| `ui_wp_layout_002_green` | **true** (D-04 sheet 520px, dim α102, unified workspace) |
| `ui_wp_layout_d07_green` | **true** (corner inset **140px**, sidebar minimap removed) |
| `wave_p_green` | **true** (`wave_p_readiness.passes`) |
| Stage 5 cross-check | `world_preview_layout.d01_unified_workspace`: **true** (orthogonal) |

**Verdict:** ☑ **PASS holds** — operational D-WP chrome unchanged; no worksheet reopen; no mock-parity claim.

---

## Unblocks

| Lane | Notes |
|:---|:---|
| **UI-WP-LAYOUT-D02-OPT** | Optional — designer **no gate** |
| **UI-WP-MOTION-001** | Deferred — no designer block |
| **UI-WP-L4-001** | Requires **UI4-DESIGN-003** color key (separate todo) |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **DESIGN-D-WP-REVIEW-001** post-impl review; D-04 + D-07 PASS |
| v1.1.0 | 2026-05-25 | **§12** reconfirmation after **COD-B-WP-WITNESS-001** lib witness refresh (`1779725532`) |
