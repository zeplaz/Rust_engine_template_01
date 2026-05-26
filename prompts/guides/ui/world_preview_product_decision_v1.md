# World Map Preview — product decision `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WP-DECISION-001** |
| **Version** | `1.2.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` (product) · layout authority: `@designer` |
| **Status** | **SIGNED** — closes ledger **World Preview product** row |
| **Full plan** | [`world_preview_product_full_plan_v1.md`](world_preview_product_full_plan_v1.md) — **unblocks DESIGN-D-WP-REVIEW-001** |
| **Layout (chrome)** | [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) **SIGNED** |
| **Designer review** | [`world_preview_d_wp_post_impl_review_v1.md`](world_preview_d_wp_post_impl_review_v1.md) — **PASS** 2026-05-25 |
| **D-WP track rollup** | [`world_preview_d_wp_track_signoff_v1.md`](world_preview_d_wp_track_signoff_v1.md) |
| **Coder queue** | [`ui_world_preview_coder_queue_v1.md`](ui_world_preview_coder_queue_v1.md) |
| **Ledger** | [`stage_tracks_signoff_ledger_v1.md`](../../../src/dev/stage_tracks_signoff_ledger_v1.md) |

**No Rust.** This doc is the **product** contract: what World Preview is for, who owns pixels, and what ships next.

---

## Product north star

World Map Preview is the **editor / WorldGen operations surface** — a living archival table for planning terrain and logistics **before** simulation play. It is **not** the in-game minimap (Bevy HUD, [`ui_phase3_minimap_compositor_plan_v1.md`](ui_phase3_minimap_compositor_plan_v1.md)).

| Session | Surface | Pixel owner |
|:---|:---|:---|
| `AppState::WorldGen` | Unified workspace (D-01 **done**) | Wave P / preview composite + egui chrome |
| `BaseState::Simulation` | Minimap chrome only | GPU minimap compositor — **no** preview workspace |

---

## Signed product decisions

| ID | Decision | Choice | Blocks |
|:---|:---|:---|:---|
| **P-WP-01** | Single workspace vs dual float | **A** — one window (D-01 landed) | — |
| **P-WP-02** | Generator placement | **A** — slide sheet over dimmed map (D-04) | **DONE** — UI4-DESIGN-001 + **UI-WP-LAYOUT-002** |
| **P-WP-03** | Map dominance | **A** — map ≥70% usable area when sheet closed | D-02 optional polish |
| **P-WP-04** | Preview mutates gameplay | **No** — read-only composite; Wave P contract | Wave P audit |
| **P-WP-05** | Terrain art pass | **Deferred** — WP-L4 after D-04 shell | **UI-WP-L4-001** |
| **P-WP-06** | Motion / paper chrome | **Deferred** — WP-L3 after LAYOUT-002 | motion table §6 |
| **P-WP-07** | Coupling to sim HUD | **None** — disjoint files from `simulation_shell_phase2` | parallel coder lanes OK |

---

## Delivery phases (product view)

```text
DONE     D-01 unified shell · D-04 slide sheet (002) · D-07 corner inset
OPTIONAL WP-L3 motion · WP-L4 map look · D-02 map dominance polish
OPS      wave_p_live.json refresh after Wave P / chrome edits
NEVER    Duplicate minimap extract · gameplay mutation from preview UI
```

---

## Authority map

| Layer | Writer | Reader |
|:---|:---|:---|
| World-gen / preview raster | Wave P + `PreviewPathAuthority` | Preview panel |
| Chrome layout | egui in `world_preview/window.rs` | Designer §5 |
| Sim minimap | `MinimapCompositorPlugin` | `MinimapGpuImageNode` |

**Forbidden:** preview panel writing `MapCameraDesired` for world main; construction commits from preview chrome without construction funnel.

---

## Witness / exit (product lane)

| Milestone | Proof |
|:---|:---|
| D-01 shell | [`world_preview_d01_shell_signoff_v1.md`](../../../src/dev/world_preview_d01_shell_signoff_v1.md) |
| D-04 sheet | `slide_sheet_spec_v1.png` + `wave_p_live.json` → `ui_wp_layout_002_green: true` |
| D-07 inset | `wave_p_live.json` → `ui_wp_layout_d07_green: true` |
| Wave P spine | `wave_p_live.json` → `wave_p_green: true` |
| Regression | `cargo test -p proc_A_dine01 --lib stage5` after each coder slice |

**Product lane CLOSED** (2026-05-25): D-04 **SIGNED** + **UI-WP-LAYOUT-002** landed — map look (WP-L4) is **enhancement**, not product blocker.

---

## Designer WP review (unblocked by full plan)

**Queue:** **DESIGN-D-WP-REVIEW-001** — **SIGNED PASS** (operational chrome).

| Prerequisite | Doc |
|:---|:---|
| This product row + **full plan** | [`world_preview_product_full_plan_v1.md`](world_preview_product_full_plan_v1.md) |
| Post-impl workflow | [`world_preview_d_wp_post_impl_review_v1.md`](world_preview_d_wp_post_impl_review_v1.md) |
| Review record | [`world_preview_d_wp_review_record_v1.md`](../../../src/dev/world_preview_d_wp_review_record_v1.md) |

**Not required for PASS:** full mock parity (D-05, D-08…D-12, WP-L4).

---

## Handoffs

| To | When | Doc |
|:---|:---|:---|
| **@designer** | Post-impl review | [`world_preview_d_wp_post_impl_review_v1.md`](world_preview_d_wp_post_impl_review_v1.md) |
| **@designer** | D-04 spec | [`ui_phase4_handoff_plan_v1.md`](ui_phase4_handoff_plan_v1.md) § UI4-DESIGN-001 |
| **@coder A** | Shell body | **UI-WP-LAYOUT-002** |
| **@coder B** | Disjoint parallel | industrial / water / infra — **not** `window.rs` same session |
| **@planner** | Queue 003–004 | [`ui_world_preview_coder_queue_v1.md`](ui_world_preview_coder_queue_v1.md) |

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-24 | **SIGNED** — product row |
| Designer | 2026-05-24 | Layout **SIGNED** (see layout decision v1.2) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.2.0 | 2026-05-25 | Link full plan; DESIGN-D-WP-REVIEW unblocked |
| v1.1.0 | 2026-05-25 | Witness alignment: D-04/D-07 green; optional WP-L3/L4 only |
| v1.0.0 | 2026-05-24 | PLAN-WP-DECISION-001 — ledger world preview product |
