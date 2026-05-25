# Palette v2 tokens

Version: `1.0.0` (2026-05-24)  
**Code authority:** [`src/gui/style/palette.rs`](../../../src/gui/style/palette.rs) (`UiPalette`)

Design names used in Phase 2 UI reviews. Map design → code when reading mocks or sign-off checklists.

| Design token | Hex / role | `UiPalette` field | Bevy helper |
|:---|:---|:---|:---|
| `ink_magenta` | `#d946ef` — strip wire, badge frame | `wire_magenta` | `bevy_wire_magenta()` |
| `ink_magenta_bright` | `#dc38b8` — hover emphasis (F-03) | `accent_hot` | `bevy_accent_hot()` |
| `vellum` | `#181612` — selected tab / rail wash | `bg_vellum` | `bevy_bg_vellum()` |
| `gold_bar` | `#e8c03a` — selected tab accent | `accent_gold` | `bevy_accent_gold()` |
| `terminal_ink` | `#5dca31` — telemetry OK / tray affordance | `accent_terminal` | `bevy_accent_terminal()` |
| `data_ink` | `#5dca31` — tick line mono (`T+00042`) | `fg_data` | `bevy_fg_data()` |
| `paper_field` | `#0a0c0a` — ops strip wash | `bg_paper` | `bevy_paper_fill()` |
| `label_primary` | `#5ee0dc` — primary labels | `fg_primary` | `bevy_primary_text()` |
| `label_muted` | `#4a7878` — inactive tabs / zones | `fg_muted` | `bevy_text_muted()` |
| `panel_elevated` | `#060808` — HUD panel fill | `bg_elevated` | `bevy_hud_panel_fill()` |

## Usage notes

- **Hover (F-03):** ops strip zone buttons use `accent_hot` stroke on `Interaction::Hovered`.
- **Selected tray tab (F-07/F-08):** Bevy tabs use `bg_vellum` + 2px `accent_gold` left bar; egui `flat_v2_tray_tab` mirrors the same tokens.
- **Peek body (F-06):** tray height uses `CONTEXT_TRAY_PEEK_BODY_H_PX` (48px) before full 96px expand.

## Related docs

- [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md)
- [`ui_phase2_designer_signoff_v1.md`](ui_phase2_designer_signoff_v1.md)
- [`ui_phase2_coder_queue_v1.md`](ui_phase2_coder_queue_v1.md)
