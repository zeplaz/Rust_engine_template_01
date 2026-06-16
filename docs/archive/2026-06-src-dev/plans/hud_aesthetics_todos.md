# HUD aesthetics lane

**Scope:** egui product shell + shared theme — not Bevy operations strip (see `in_game_hud.rs`).

---

## Done (2026-05-23)

| Id | Change |
|----|--------|
| **HUD-01** | Split `status_side_panel_state` vs `command_tray_state` (no more one key toggling rail + window) |
| **HUD-02** | `hud_chrome.rs` — icon buttons, rail frame, stat chips, section rules |
| **HUD-03** | Side panel: collapsed icon rail, shortcuts in collapsible, cleaner header |
| **HUD-04** | Theme: 4px corners, looser density, softer panel strokes |

---

## Keys (default)

| Key | Panel |
|-----|--------|
| `=` | Status side rail |
| `` ` `` | Command **window** + Bevy left stack visibility (dock registry) |

---

## Next (optional polish)

| Id | Work |
|----|------|
| **HUD-05** | Bevy left stack rail — match egui icon style (shared glyphs / palette) |
| **HUD-06** | Minimap / overlay tray — same `hud_chrome` header + icons |
| **HUD-07** | SVG or texture icon atlas (replace unicode glyphs) |
| **HUD-08** | Operations strip — shorten tool hint strings; chip buttons instead of wall of text |

---

## Verify

Enter simulation → side rail collapsed by default → press `=` → rail expands with stat chips, no duplicate command window unless opened separately.
