# World Map Preview — product full plan `v1` (PLAN-WP-DECISION-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WP-DECISION-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — unblocks **DESIGN-D-WP-REVIEW-001** |
| **Summary decision** | [`world_preview_product_decision_v1.md`](world_preview_product_decision_v1.md) |
| **Layout authority** | [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) |
| **Designer review** | [`world_preview_d_wp_post_impl_review_v1.md`](world_preview_d_wp_post_impl_review_v1.md) → record [`world_preview_d_wp_review_record_v1.md`](../../../src/dev/world_preview_d_wp_review_record_v1.md) |
| **Track rollup** | [`world_preview_d_wp_track_signoff_v1.md`](world_preview_d_wp_track_signoff_v1.md) |
| **Witness** | [`debug_runs/wave_p_live.json`](../../../debug_runs/wave_p_live.json) |

**No Rust.** This is the **full planner plan** for World Preview product + layout + post-implementation designer review. The short product decision doc remains the ledger row; **this doc** is the gate chain designers and coders follow.

---

## What this plan unblocks

| Blocked work | Unblocked when |
|:---|:---|
| **DESIGN-D-WP-REVIEW-001** (`D-WP` post-impl review) | This plan **SIGNED** + layout **SIGNED** + **UI-WP-LAYOUT-002** + **D07** green + **COD-B-WP-WITNESS-001** |
| **UI-WP-LAYOUT-D02-OPT** (optional) | **Not** required for review PASS |
| **WP-L3 / WP-L4** (motion / map look) | **Not** required for review PASS |
| Simulation minimap work | **Never** — disjoint product ([`ui_phase3_minimap_track_naming_v1.md`](ui_phase3_minimap_track_naming_v1.md)) |

**Review scope (honest):** **Operational chrome PASS** — D-01, D-03, D-04, D-06, D-07. **Not** full mock parity (D-05, D-08…D-12).

---

## Gate chain (strict)

```text
PLAN-WP-DECISION-001 (this plan)     ☑ SIGNED 2026-05-25
        │
        ▼
world_map_preview_layout_decision   ☑ SIGNED 2026-05-24
world_preview_layout_worksheet      ☑ D-01…D-12 choices locked
        │
        ▼
UI4-DESIGN-001 + slide_sheet_spec   ☑ SIGNED (D-04)
        │
        ▼
UI-WP-LAYOUT-001 (D-01 shell)       ☑ DONE
UI-WP-LAYOUT-002 (D-04 sheet)        ☑ DONE → ui_wp_layout_002_green
UI-WP-LAYOUT-D07 (corner inset)      ☑ DONE → ui_wp_layout_d07_green
        │
        ▼
COD-B-WP-WITNESS-001                 ☑ DONE — wave_p_live.json refreshed
        │
        ▼
DESIGN-D-WP-REVIEW-001               ☑ SIGNED PASS 2026-05-25
        │
        ▼
Optional: D-02 · WP-L3 · WP-L4 · layer strip (D-05)
```

**Forbidden shortcuts:** Designer review **before** green witnesses; claiming **full mock parity** without D-05/D-08 evidence; reopening **P-WP-01** (single workspace) without planner amendment.

---

## Product decisions (P-WP) — review implications

| ID | Product choice | Coder slice | Designer review expects |
|:---|:---|:---|:---|
| **P-WP-01** | Single unified workspace | **UI-WP-LAYOUT-001** | No second floating World Generator |
| **P-WP-02** | Generator = slide sheet over dimmed map | **UI-WP-LAYOUT-002** | Sheet 40–55% height; dim ~40%; **Generator** entry |
| **P-WP-03** | Map ≥70% when sheet closed | optional **D-02** | Hero map readable — **DEFERRED** ok for PASS |
| **P-WP-04** | Preview read-only | Wave P contract | No gameplay mutation from chrome |
| **P-WP-05** | Terrain art deferred | **UI-WP-L4-001** | Out of review scope |
| **P-WP-06** | Motion deferred | **UI-WP-MOTION-001** | Out of review scope |
| **P-WP-07** | No sim HUD coupling | — | F8 WorldGen only; sim uses minimap |

---

## Layout decisions (D-01…D-12) — review verdict map

| ID | Design | Required for **DESIGN-D-WP-REVIEW PASS**? | Typical verdict |
|:---|:---:|:---:|:---|
| **D-01** | A unified shell | **Yes** | PASS |
| **D-02** | A map dominance | No | DEFERRED |
| **D-03** | A left sidebar | **Yes** | PASS |
| **D-04** | A slide sheet | **Yes** | PASS |
| **D-05** | B layer strip | No | DEFERRED |
| **D-06** | A toolbar | **Yes** | PASS |
| **D-07** | A corner inset | **Yes** | PASS |
| **D-08…D-12** | Paper / motion / ticks | No | DEFERRED |

Full worksheet: [`world_preview_layout_decision_worksheet_v1.md`](world_preview_layout_decision_worksheet_v1.md).

---

## Witness bundle (designer + planner)

| File | Fields | Required for review |
|:---|:---|:---:|
| `wave_p_live.json` | `wave_p_green`, `ui_wp_layout_002_green`, `ui_wp_layout_d07_green` | **Yes** |
| `wave_p_live.json` | `d04_sheet_body_wired`, `d04_map_dim_alpha`, `d07_corner_inset_on_map` | **Yes** |
| `stage5_full_app_live.json` | `world_preview_layout` / d01 flags | Supporting |
| Manual F8 | Sheet opens; map pans under dim; inset visible | **Yes** |

**Refresh (coder prereq for designer):**

```powershell
cargo test -p proc_A_dine01 --lib ui_wp_layout_002_writes_wave_p_live_json
# Optional sim: F8 WorldGen ~120 frames → write_wave_p_live_proof_system
```

---

## DESIGN-D-WP-REVIEW-001 — unblock checklist

Designer runs **after** this plan + witnesses — see [`world_preview_d_wp_post_impl_review_v1.md`](world_preview_d_wp_post_impl_review_v1.md).

| # | Prerequisite | Met (2026-05-25) |
|:---:|:---|:---:|
| 1 | **PLAN-WP-DECISION-001** full plan **SIGNED** | ☑ |
| 2 | Layout decision + worksheet **SIGNED** | ☑ |
| 3 | **UI4-DESIGN-001** + `slide_sheet_spec_v1.png` | ☑ |
| 4 | `ui_wp_layout_002_green: true` | ☑ |
| 5 | `ui_wp_layout_d07_green: true` | ☑ |
| 6 | **COD-B-WP-WITNESS-001** | ☑ |
| 7 | Review record lists deferred D-05…D-12 explicitly | ☑ |

**Deliverable:** [`world_preview_d_wp_review_record_v1.md`](../../../src/dev/world_preview_d_wp_review_record_v1.md) — **SIGNED — PASS**.

**Registry:** `DESIGN-D-WP-REVIEW-001` in [`designer_signoff_registry.json`](../../../tools/orchestrator/queues/designer_signoff_registry.json).

---

## Coder lanes (post-review — optional only)

| ID | Scope | Blocks review? |
|:---|:---|:---:|
| **UI-WP-LAYOUT-D02-OPT** | Map dominance ratio | No |
| **UI-WP-MOTION-001** | D-12 dissolve | No |
| **UI-WP-L4-001** | Terrain / map look | No |
| **UI-WP-LAYOUT-003+** | Queue 003–004 per [`ui_world_preview_coder_queue_v1.md`](ui_world_preview_coder_queue_v1.md) | No |

**Do not redo:** **UI-WP-LAYOUT-002**, **UI-WP-LAYOUT-D07**, **UI-WP-LAYOUT-001** without regression proof.

---

## Authority map (no drift)

| Layer | Owner | Review checks |
|:---|:---|:---|
| Preview raster | Wave P | Composite stable; pan/zoom under sheet |
| Chrome | `world_preview/window.rs` | Matches §5 + D-04/D-07 |
| Sim minimap | GPU compositor | **Absent** in WorldGen workspace |
| View pose | ViewManager / Wave P | No preview → `MapCameraDesired` for world main |

---

## Acceptance — PLAN-WP-DECISION-001 full plan

| # | Criterion |
|:---:|:---|
| P1 | Product decisions P-WP-01…07 documented with review mapping |
| P2 | Gate chain published; **DESIGN-D-WP-REVIEW** prerequisites explicit |
| P3 | Deferred vs blocking table prevents silent mock parity claims |
| P4 | Witness paths listed; COD-B-WP-WITNESS named |
| P5 | Linked from ledger + layout decision + post-impl brief |

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-WP-DECISION-001 full plan |
| Designer (layout) | 2026-05-24 | Layout **SIGNED** — prerequisite |
| Designer (review) | 2026-05-25 | **DESIGN-D-WP-REVIEW-001 PASS** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Full plan — unblocks DESIGN-D-WP-REVIEW-001 |
