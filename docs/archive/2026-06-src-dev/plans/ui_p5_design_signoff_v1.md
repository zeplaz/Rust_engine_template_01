# UI Phase 5 pause menu — `UI-P5-DESIGN-001` `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **UI-P5-DESIGN-001** |
| **Review ID** | **UI-P5-DESIGN** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Reviewer** | `@designer` |
| **Status** | **SIGNED — PASS** (Bevy polish landed) |
| **Plan** | [`ui_phase5_pause_menu_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md) |
| **Implementation** | [`src/gui/pause_menu_bevy.rs`](../src/gui/pause_menu_bevy.rs) |
| **Tokens** | [`palette_v2_tokens.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/palette_v2_tokens.md) |

---

## Executive summary

**Designer polish** for Phase 5 pause overlay — vellum card, magenta wire, gold primary **Resume**, hot hover (F-03 parity), stub styling for Save/Load, confirm modal for destructive exits.

**Verdict:** ☑ **SIGNED — PASS** — matches Phase 2 HUD language; does **not** wire Save spine (**P5-SAVE-001** deferred).

---

## Token map

| Element | Token | Application |
|:---|:---|:---|
| Backdrop | `bg_app` @ 88% | Full-screen dim |
| Card | `panel_elevated` + `ink_magenta` wire | Centered menu |
| Title | `label_primary` | `SIMULATION PAUSED` |
| Hint | `label_muted` | Tick **P** vs menu pause |
| **Resume** (primary) | `gold_bar` left bar + `vellum` fill | Default emphasis |
| Normal actions | `panel_elevated` button + subtle border | Load / WorldGen |
| Stub actions | `label_muted` copy | Save (stub) |
| Destructive | `ink_magenta` border; confirm uses `accent_hot` on confirm | Main menu / Exit |
| Hover | `ink_magenta_bright` | All buttons (F-03) |

---

## Layout (2C-style card)

```text
┌─ pause card (min 320px) ─────────────┐
│ SIMULATION PAUSED                    │
│ Menu pause — sim tick (P) separate   │
│ ─────────────────────────────────    │
│ ▌ Resume                             │  ← gold left bar
│   Save game (stub)                   │  ← muted
│   Load game                          │
│ ─────────────────────────────────    │
│   World Generator…                   │
│   Return to Main Menu                │
│   Exit program                       │
│ Esc — resume when tray collapsed     │
└──────────────────────────────────────┘
```

---

## Sign-off checklist

| # | Item | Done |
|:---|:---|:---:|
| 1 | Centered card + backdrop | ☑ |
| 2 | Button order per plan | ☑ |
| 3 | Confirm modal copy | ☑ |
| 4 | PLAY-01: pause does not expand tray | ☑ (Escape ordering) |
| 5 | Hover tokens (F-03 parity) | ☑ |

**Verdict:** ☑ **SIGNED — PASS**

| Role | Date |
|:---|:---|
| Designer | 2026-05-25 |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **UI-P5-DESIGN-001** polish spec + implementation sign-off |
