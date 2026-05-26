# World Map Preview — decision worksheet `v1`

**One-page review pass** · **SIGNED 2026-05-24** — all **recommended** defaults accepted (no overrides).

| Field | Value |
|:---|:---|
| **Authority** | [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) |
| **Track sign-off** | [`world_preview_d_wp_track_signoff_v1.md`](world_preview_d_wp_track_signoff_v1.md) (**D-WP**) |
| **Designer** | Design pass (recommended defaults) · **Date** 2026-05-24 |
| **Mock path** | [`assets/ui/world_preview/layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png) (1920×1080) |

---

## §5 decisions — D-01…D-12 (final)

| ID | Question | **Choice** | Notes |
|:---|:---|:---:|:---|
| **D-01** | Shell model | **A** | Single workspace; generator slide sheet · [`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md) |
| **D-02** | Map dominance | **A** | Map ≥ **65%** · [`world_preview_d02_map_dominance_signoff_v1.md`](world_preview_d02_map_dominance_signoff_v1.md) (impl optional) |
| **D-03** | Field index placement | **A** | Left stack (§4 wireframe) |
| **D-04** | Generator params | **A** | Left slide-over (dimmed map) |
| **D-05** | Layer / overlay controls | **B** | Strip on map top edge (tracing overlay) |
| **D-06** | Zoom / pan / GPU | **A** | Header margin icons |
| **D-07** | Overview minimap | **A** | Corner inset 120–160px |
| **D-08** | Panel chrome tech | **A** | egui custom `Frame` + textures |
| **D-09** | Asymmetry rule | **A** | Fixed offsets: index +8px, notes −6px, header +4px |
| **D-10** | Magenta usage | **A** | Registration ticks + selected wire only |
| **D-11** | Negative space | **B** | Min **12%** map margin (archive silence) |
| **D-12** | Enter simulation | **A** | Chrome dissolve **400ms** |

**Overrides:** none.

---

## §11 sign-off checklist

| # | Item | Done |
|:---|:---|:---:|
| 1 | §5 **D-01…D-12** recorded above | ☑ |
| 2 | Mock **`assets/ui/world_preview/layout_mock_v1.png`** committed (1920×1080) | ☑ |
| 3 | Offsets / margins / negative space on mock (12% map margin, +8/−6/+4 asymmetry) | ☑ |
| 4 | Motion §6 accepted (no amend table) | ☑ |
| 5 | Magenta / hot-pink boundary confirmed (registration pigment; hot pink = active authority only) | ☑ |
| 6 | §9 operator journey walkthrough approved | ☑ |
| 7 | §3 **REMOVE** list acknowledged | ☑ |

**Verdict:** ☑ **SIGNED**

---

## Unblocks

| Slice | Agent | Status |
|:---|:---|:---|
| **UI-WP-DESIGN** | `@designer` | **done** |
| **UI-WP-LAYOUT-001** | `@coder` | **done** — D-01 shell · [`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md) |
| **UI-WP-LAYOUT-D02-OPT** | `@coder` | **optional** — D-02 map ≥65% · [`world_preview_d02_map_dominance_signoff_v1.md`](world_preview_d02_map_dominance_signoff_v1.md) |
| **UI4-DESIGN-001** | `@designer` | **done** — [`world_preview_d04_slide_sheet_spec_v1.md`](world_preview_d04_slide_sheet_spec_v1.md) |
| **UI-WP-LAYOUT-002** | `@coder` | **done** — D-04 dim + sheet ([`world_preview_d04_slide_sheet_spec_v1.md`](world_preview_d04_slide_sheet_spec_v1.md)) |

**Still deferred (WP-L3+):** motion §6 implementation, paper texture assets (WP-L1), terrain color key (WP-L4).

---

## Recommended column rationale (reference)

| ID | Why chosen |
|:---|:---|
| D-01 A | Eliminates dual-window hunt; matches §9 flow |
| D-02 A | Map as hero — archival table, not dashboard tiles |
| D-03 A | Ecology index without covering map center |
| D-04 A | Params without second window |
| D-05 B | Tracing-paper layer strip — not esports toolbar |
| D-06 A | Sparse header; zoom with index |
| D-07 A | Corner overview — not sidebar thumb |
| D-08 A | Fastest egui path v1 |
| D-09 A | Predictable coder offsets |
| D-10 A | Magenta = registration pigment only |
| D-11 B | Archive silence — Tarkovsky pacing |
| D-12 A | Dissolve matches PLAY-01 entry |
