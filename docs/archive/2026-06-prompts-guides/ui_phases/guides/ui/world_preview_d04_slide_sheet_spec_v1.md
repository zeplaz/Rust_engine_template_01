# World Preview D-04 — generator slide sheet spec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **UI4-DESIGN-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` (Design pass) |
| **Status** | **SIGNED** (2026-05-24) |
| **Parent** | [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) **D-04 A** |
| **Track** | [`world_preview_d_wp_track_signoff_v1.md`](world_preview_d_wp_track_signoff_v1.md) |
| **Blocks** | ~~**UI-WP-LAYOUT-002**~~ **unblocked** |

---

## Signed choice (D-04)

| ID | Choice | Spec |
|:---|:---:|:---|
| **D-04** | **A** | **Left slide-over** sheet over map; map **dimmed** ~40%; focus trap in sheet |

---

## Layout (1080p reference)

| Element | Value | Notes |
|:---|:---|:---|
| Sheet edge | **Left** | Opens from unified workspace toolbar **Parameters ▸** |
| Sheet width | **35–40%** of workspace client width | Max **720px**; min **400px** |
| Sheet height | **100%** of content area below header | Full vertical sheet |
| Map dim | **40%** opacity overlay on central panel when sheet open | Per motion table §6 (240ms) |
| Entry control | Toolbar toggle **Parameters ▸ / ◂** | Wired in `window.rs` |
| Focus | Keyboard focus stays in sheet while open | Tab order: sheet → close toggle |
| Close | Same toggle; **Esc** optional v2 | No second float window |

**Visual mock:** [`assets/ui/world_preview/slide_sheet_spec_v1.png`](../../../assets/ui/world_preview/slide_sheet_spec_v1.png) (v1.0 interim — sheet-open layout from [`layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png); replace with annotated sheet-open art when available).

---

## Sheet body (coder scope for UI-WP-LAYOUT-002)

| Region | Content | Source today |
|:---|:---|:---|
| Header | **World Generator — Parameters** | `draw_world_gen_panel` |
| Body | Seed, sliders, presets, Generate | `world_gen_ui.rs` |
| Footer | Tuning I/O hint + generate actions | existing panel |

**Do not** change generate semantics or `GenerateWorldEvent` routing.

---

## Motion (signed §6)

| Interaction | Target |
|:---|:---|
| Open sheet | 240ms horizontal; map dim 40% ease-out |
| Close sheet | Reverse; map dim off |

---

## §11 Sign-off checklist

| # | Item | Done |
|:---|:---|:---:|
| 1 | D-04 **A** recorded | ☑ |
| 2 | Width / dim / height % in this doc | ☑ |
| 3 | `slide_sheet_spec_v1.png` committed | ☑ (interim from layout mock) |
| 4 | Designer **SIGNED** | ☑ |

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-24 | **SIGNED** |

**Unblocked:** **UI-WP-LAYOUT-002** — **DONE** (dim + sheet in `window.rs`; lib witness green).

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | **UI4-DESIGN-001** done; interim PNG; unblocks LAYOUT-002 |
| v0.1.0 | 2026-05-24 | DRAFT |
