# World Preview D-02 — map dominance sign-off `v1` (optional implementation)

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` → `@coder` **UI-WP-LAYOUT-D02-OPT** (optional) |
| **Status** | **SIGNED** (design) — **implementation OPTIONAL** |
| **Parent gate** | [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) §5 |
| **Sibling** | [`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md) — **required** shell (done) |
| **Worksheet** | [`world_preview_layout_decision_worksheet_v1.md`](world_preview_layout_decision_worksheet_v1.md) |
| **Mock** | [`assets/ui/world_preview/layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png) — central map ≥62–65% |

---

## Executive summary

**D-02 A** — the **world map is the hero**: central panel should occupy **≥ 65%** of the workspace **client area** under normal World Preview chrome (field index visible, generator slide sheet **closed**).

**Optional lane:** this sign-off records the **signed design** and acceptance math. **Does not block** D-01 closure, **UI-WP-LAYOUT-002**, or Stage 5. Coders may implement **UI-WP-LAYOUT-D02-OPT** when layout polish is scheduled.

**Not** simulation HUD ([`ui_phase2_designer_signoff_v1.md`](ui_phase2_designer_signoff_v1.md)).

---

## §5 — Signed decision (D-02 only)

| ID | Question | **Choice** | Final spec |
|:---|:---|:---:|:---|
| **D-02** | Map dominance | **A** | Map ≥ **65%** of workspace area **always** (baseline chrome) |

**Rejected for v1:**

| Option | Why not |
|:---|:---|
| **B** Map ≥ 55% | Index competes with hero map; reads dashboard not archive table |
| **C** Full-bleed + hover index | Hides ecology/logistics context operators need at a glance |

**Overrides:** none.

---

## Measurement contract (when implementing)

| Term | Definition |
|:---|:---|
| **Workspace client area** | egui inner rect of **Operational Archive — World Index** after window chrome, excluding OS title bar |
| **Map area** | `CentralPanel` allocated rect at rest (includes `MAP_PANEL_INSET_PX` shrink) |
| **Baseline chrome** | Left field index (D-03 A) + top header + bottom status — **generator slide sheet closed** |
| **Pass** | `map_area / workspace_client_area ≥ 0.65` at **1920×1080** and at **min window** (1280×720) |

**Generator sheet open (D-04):** sheet may overlap map; measure **baseline** first. Optional follow-up: cap sheet `max_width` so map never drops below 65% **or** document “sheet open = temporary override” in witness — designer accepts **temporary** dip if sheet ≤ 35% width.

### Proposed constants (coder optional)

| Constant | Value | File hint |
|:---|:---|:---|
| `WORLD_PREVIEW_MAP_MIN_AREA_FRACTION` | `0.65` | `world_preview/mod.rs` |
| Field index default | `220px` (within §4 220–280) | `window.rs` sidebar |
| Field index max | `min(280, workspace_w * 0.22)` | clamp so map keeps 65% |
| Generator sheet max (when open) | `min(720, workspace_w * 0.35)` | optional coexistence with D-04 |

---

## Current code snapshot (honest)

| Item | Today | D-02 A target |
|:---|:---|:---|
| Central panel | `egui::CentralPanel` — fills remainder after side panels | ☑ structure OK |
| Sidebar | `default_width(180)`, range `160..=260` | May exceed 35% on narrow windows |
| Generator sheet | `default_width(520)`, range `400..=720` | Can shrink map below 65% when open |
| Right field notes | Not implemented (§4 optional v1) | N/A |
| Automated ratio test | None | Add `map_dominance_meets_d02` unit or dev overlay |

**Designer verdict (2026-05-24):** **SIGNED** on intent; **implementation not verified** — treat as **optional polish**, not a regression gate.

---

## §11 Designer sign-off checklist (D-02)

| # | Item | Done |
|:---|:---|:---:|
| 1 | **D-02** choice **A** on worksheet | ☑ |
| 2 | §4 wireframe central map **≥62–65%** width | ☑ |
| 3 | Mock hero map read ([`layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png)) | ☑ |
| 4 | Parent §11 SIGNED | ☑ |
| 5 | Optional coder slice scoped (no raster / motion) | ☑ |
| 6 | Ratio enforcement in code | ☑ **UI-WP-LAYOUT-D02-OPT** — `mod.rs` + `window.rs` + `wave_p_live_proof` |

**Verdict:** ☑ **SIGNED** (design) · implementation **OPTIONAL**

| Role | Date | Verdict | Notes |
|:---|:---|:---|:---|
| Designer | 2026-05-24 | **SIGNED** | D-02 A; does not block D-01 done |
| Coder | 2026-05-25 | **DONE** | Sidebar clamp + `d02_map_dominance_hd_baseline_sheet_closed` lib test |

---

## Optional coder slice — UI-WP-LAYOUT-D02-OPT

```
Lane: UI-WP-LAYOUT-D02-OPT (optional)
Read: world_preview_d02_map_dominance_signoff_v1.md + layout_mock_v1.png
Touch: window.rs (+ mod.rs for constant) — ≤2 files
Do: clamp sidebar/sheet widths; dev assert or unit test for 65% at 1920×1080 baseline
Do NOT: D-04 dim, motion §6, raster, new panels (right notes)
Verify: cargo test -p proc_A_dine01 --lib world_preview
```

| Step | Task | Exit |
|:---:|:---|:---|
| 1 | Export `WORLD_PREVIEW_MAP_MIN_AREA_FRACTION` | constant documented |
| 2 | Clamp `world_preview_sidebar` max width from workspace w | 1280×720 baseline ≥ 65% |
| 3 | Optional: cap generator sheet max to 35% w | sheet open does not violate policy (designer choice) |
| 4 | Unit test `d02_map_area_fraction_at_hd_baseline` | test green |

---

## Relationship to other decisions

| ID | Interaction |
|:---|:---|
| **D-01** | Single workspace — **done**; D-02 applies inside that window |
| **D-03** | Left index consumes ≤ ~22% width at HD if D-02 enforced |
| **D-04** | Slide sheet — may temporarily reduce map %; cap or accept per § Measurement |
| **D-11** | Negative space **12%** (B) — complementary; D-02 is **area**, D-11 is margin breathe |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | D-02 design SIGNED; implementation optional; sibling D-01 done |
