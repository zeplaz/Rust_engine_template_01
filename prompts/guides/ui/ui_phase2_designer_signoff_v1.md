# UI Phase 2 designer sign-off `v1`

| Field | Value |
|:---|:---|
| **Version** | `2.0.0` |
| **Date** | 2026-05-23 |
| **Reviewer** | Designer — **UI-P2-DESIGN** |
| **Format** | Mirrors [`ui_phase1_designer_signoff_v1.md`](ui_phase1_designer_signoff_v1.md) |
| **Mocks** | [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) |
| **Master plan** | [`src/dev/ui_overhaul_plan.md`](../../../src/dev/ui_overhaul_plan.md) |
| **Tokens** | [`palette_v2_tokens.md`](palette_v2_tokens.md) · [`src/gui/style/palette.rs`](../../../src/gui/style/palette.rs) |
| **Witness** | `debug_runs/ui_shell_migration_live.json` — profile **`UI_SHELL_MIGRATION_2B`** |
| **Coder queue** | [`ui_phase2_coder_queue_v1.md`](ui_phase2_coder_queue_v1.md) |

---

## Panel verdict summary

Compared to [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) § Panel index.

| Panel | Mock host | Code root | Verdict | Summary |
|:---|:---|:---|:---:|:---|
| **P1** | Ops strip — alerts zone | `OperationsStripRoot` | **PASS** | Zones wired; ◆ badge; 2px top; magenta bottom stroke; witness interaction flags true |
| **P2** | Bottom context tray | `ContextTrayRoot` | **PASS*** | Peek/expand, gold/vellum tabs, Escape; *32px tab bar vs mock 28px handle (F-05 accepted) |
| **P3** | Map inset + minimap chrome | `MapViewportFrameInset`, `MinimapChromeRoot` | **PASS** | 4px inset; chrome ≤2px delta; egui texture + Bevy stroke; movable minimap (DQ-POST-06) |
| **P4** | Left context rail **48px** | `LeftContextRail`, `BuildRailRoot` | **CONDITIONAL** | Rail + build column landed; **LeftContextStack** overlay overlap — **2C defer OK** |

**Overall verdict:** **CONDITIONAL** — all four panels materially match Phase 0 mocks; **SIGNED** blocked on numbered **@coder** item **1** only.

---

## P1 — Ops strip

**Mock:** [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) § P1 — 22×22 badge, `ALERTS`, mission count, click → tray Alerts.

| Check | Mock | Implementation | Match |
|:---|:---|:---|:---:|
| Zone row | TIME · ALERTS · … · EW/DENY | `OpsStripZone` buttons; alerts centered flex | ☑ |
| Alert badge | **◆** + count, cap `99+` | `format_ops_strip_alert_badge` | ☑ |
| Alert click | Tray expanded, Alerts tab | `ops_strip_zone_click_system` | ☑ |
| Intel click | Map focus request | `OpsStripIntelFocusRequest` | ☑ |
| Top offset | — | `OPS_STRIP_TOP_OFFSET_PX = 2` | ☑ |
| Bottom stroke | `ink_magenta` 1px bottom | `bevy_wire_magenta()` bottom only | ☑ |

**Panel verdict:** **PASS**

---

## P2 — Context tray

**Mock:** [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) § P2 — 32px tabs, 96px body, Escape collapse.

| Check | Mock | Implementation | Match |
|:---|:---|:---|:---:|
| Tab bar | 32px — Alerts / Intel / Logistics / Diag | `CONTEXT_TRAY_TAB_H_PX`; `ContextTrayTab` | ☑ |
| Peek cycle | Tab select → preview body | `on_tab_pressed` → `Peek` (48px) → `Expanded` | ☑ |
| Body height | 96px expanded | `CONTEXT_TRAY_BODY_H_PX` + `HudInfoLiveData` | ☑ |
| Selected chrome | gold + vellum | `sync_context_tray_tab_chrome_system` | ☑ |
| Escape | Collapse unpinned | `collapse_context_tray_on_escape` | ☑ |
| Ops affordance | — | **▼ TRAY** on ops strip (F-05) | ☑ (extra) |

**Panel verdict:** **PASS** *(mock amended: no separate 28px drag handle — designer accepted F-05)*

---

## P3 — Map frame + minimap chrome

**Mock:** [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) § P3 — 4px map inset; minimap chrome tracks egui rect.

| Check | Mock | Implementation | Match |
|:---|:---|:---|:---:|
| Map inset | 4px inner wire | `MAP_FRAME_INSET_PX = 4` · `MapViewportFrameInset` | ☑ |
| Minimap chrome | Stroke-only frame | `MinimapChromeRoot` · no fill slab | ☑ |
| Rect sync | Match egui window | `sync_minimap_chrome_root_system` | ☑ |
| Delta | ≤2px | witness `last_minimap_rect_delta_px: 1.0` | ☑ |
| Movable panel | — | egui `Minimap` window drag + `HudLayoutStore` (DQ-POST-06) | ☑ (product) |

**Panel verdict:** **PASS**

---

## P4 — Left context rail + build rail

**Mock:** [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) § P4 — **48px** collapsed rail; `« HUD`; expand → left stack.

| Check | Mock | Implementation | Match |
|:---|:---|:---|:---:|
| Rail width | **48px** | `CONTEXT_RAIL_W_PX = 48` · `LeftContextRail` | ☑ |
| Collapsed default | Rail visible; stack closed | `CommandLeftStackState::collapsed = true` | ☑ |
| Icon affordance | Tactical glyph column | `⏱` `⛭` `◎` `☰` on rail | ☑ |
| Expand stack | Rail click / toggle key | `command_left_stack_rail_interaction` | ☑ |
| Build tools | — | `BuildRailRoot` **52px** + `BuildStripState` sync | ◐ |
| Phase 4 icons | Atlas rail icons | `IconAtlasPlugin` · placeholder PNG · witness `icon_atlas_loaded` | ◐ |

### Flag — `LeftContextStack` overlap with P4 (**2C defer OK**)

**Observation:** `CommandLeftStackOverlay` is **absolute** over the map hole (`in_game_hud.rs`). When expanded, `LeftContextStackBody` sits beside `LeftContextRail` + `BuildRailRoot` (~**100px** horizontal chrome vs mock **48px** single rail). Map viewport measure is **not** shrunk — overlay stacks on the tactical view.

| Issue | Severity | Disposition |
|:---|:---|:---|
| P4 mock assumes single **48px** rail | Layout | **2C** — unify rail + build column vs mock diagram (defer OK) |
| Expanded stack occludes map edge | UX | Accept for Phase 2; collapse default (PLAY-01) mitigates |
| `LeftContextRail` hidden when stack expanded | Behavior | Intentional — rail visible only when collapsed |

**Panel verdict:** **CONDITIONAL** — functionally shippable for Phase 2; **2C** layout unification is non-blocking.

---

## Accessibility (cross-panel)

| Check | Mock / rule | Implementation | Match |
|:---|:---|:---|:---:|
| Alert non-color cue | ◆ + number | `format_ops_strip_alert_badge` | ☑ |
| Focus order | Documented | [`ui_layout_agent.md`](../../../tools/orchestrator/agents/ui_layout_agent.md) § F-10 | ☑ |
| Hover token | `ink_magenta_bright` | Ops zones use `accent_terminal` border (F-03 deviation) | ◐ |

---

## Overall verdict

### **CONDITIONAL**

| Criterion | Status |
|:---|:---|
| P1–P3 match Phase 0 mocks | ✅ |
| P4 material complete; 2C overlap flagged, defer OK | ✅ |
| Witness `UI_SHELL_MIGRATION_2B` green (2A + 2B) | ✅ |
| Sprint **4.3** visual / Stage 5 regression | ☐ |

Flip to **SIGNED** when **@coder item 1** completes without spine regression.

---

## @coder follow-ups (numbered)

1. **4.3 visual gate** — Run `cargo run -p proc_A_dine01 --release -- --test visual` and `cargo test -p proc_A_dine01 --lib stage5`; refresh `stage5_full_app_live.json`. *Blocks **SIGNED**.*
2. **F-03 (optional)** — Ops strip zone hover: swap border from `accent_terminal` to `bevy_accent_hot()` per [`palette_v2_tokens.md`](palette_v2_tokens.md) `ink_magenta_bright`.
3. **P4-ART-01 (optional)** — Replace placeholder `icon_atlas_phase4_v1.png` with traced silhouettes per [`ui_phase4_icon_atlas_brief_v1.md`](ui_phase4_icon_atlas_brief_v1.md).
4. **2C layout (defer OK)** — Reconcile `LeftContextStack` + `BuildRailRoot` width with P4 **48px** mock; consider single merged rail column or mock update. *Not blocking Phase 2 **SIGNED**.*

---

## Sign-off record

| Role | Date | Verdict | Notes |
|:---|:---|:---|:---|
| Coder | 2026-05-24 | Phase 2A+2B wired | Lib tests green |
| Designer | 2026-05-23 | **CONDITIONAL** | UI-P2-DESIGN v2.0 — panel table; 2C overlap flagged; @coder **1** blocks SIGNED |

**Out of scope (Phase 2):** sim-side `BuildToolbox` catalog (PLAY-01); F3 editor egui removal; Phase 4 art bake.

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-23 | Initial checklist |
| v1.6.0 | 2026-05-23 | UI-P2-DESIGN witness reconcile |
| v2.0.0 | 2026-05-23 | Phase 1 mirror format; panel verdict table P1–P4; 2C flag; numbered @coder items |
