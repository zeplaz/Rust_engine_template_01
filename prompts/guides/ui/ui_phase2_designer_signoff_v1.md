# UI Phase 2 designer sign-off `v1`

| Field | Value |
|:---|:---|
| **Version** | `2.2.0` |
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
| **P4** | Left context rail **48px** + build **52px** | `LeftContextRail`, `BuildRailRoot` | **PASS** | **2C-B** dual column — mock + witness aligned (2026-05-24) |

**Overall verdict:** **SIGNED** (2026-05-24) — 4.3 visual gate green; P4 **CONDITIONAL** on 2C overlap only (defer OK).

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
| Phase 4 icons | Atlas rail icons | `IconAtlasPlugin` · **placeholder PNG kept** · witness `icon_atlas_loaded` | ◐ |

### Phase 2C — deferred layout lane (`LeftContextStack` + `BuildRailRoot` vs P4 **48px** mock)

**Status:** **CLOSED** — **2C-B** (2026-05-24) — mock § P4 amended; witness `phase2c` in live JSON.

#### Problem

[`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) § P4 describes a **single 48px** collapsed rail (`CONTEXT_RAIL_W_PX`) that expands into a left narrative stack. Phase **2B** added a **second always-visible column** — `BuildRailRoot` — beside that rail. Both live in an **absolute overlay** that does **not** shrink the map viewport hole.

#### Implementation tree (authoritative)

Spawn host: `CommandLeftStackOverlay` · [`in_game_hud.rs`](../../../src/gui/in_game_hud.rs) · `ZIndex(900)` · `position: absolute`

```text
CommandLeftStackOverlay          ← overlays map; NOT in flex measure for SimulationMapViewportFill
├── LeftContextRail              ← 48px (CONTEXT_RAIL_W_PX); Button; glyph column ⏱ ⛭ ◎ ☰
├── BuildRailRoot                ← 52px fixed width; 7 ToolContext slots; Phase 4 icons row-0
└── LeftContextStackBody         ← 400px (LEFT_CONTEXT_STACK_W_PX); objectives / narrative / logistics lines
```

Map hole: `SimulationMapViewportFill` is **full-window** under the overlay (`ZIndex(100)`). Comment at spawn: *"Left stack overlays the map — does not participate in viewport measure."*

#### Width budget (horizontal chrome on map)

| `CommandLeftStackState` | `LeftContextRail` | `BuildRailRoot` | `LeftContextStackBody` | `column_gap` | **Overlay footprint** |
|:---|:---:|:---:|:---:|:---:|:---:|
| **Collapsed** (default, PLAY-01) | 48px visible | 52px visible | hidden (`Display::None`) | 6px | **~106px** |
| **Expanded** (`toggle_command_left_stack` / rail press) | hidden | 52px visible | 400px visible | 6px | **~458px** |

Edge inset: overlay `left: CENTER_ROW_EDGE_PAD_PX` (**8px**) from window edge — not counted above.

**Mock delta:** Phase 0 P4 documents **48px only**. Code ships **48 + 52** collapsed (~106px) and never narrows the tactical map rect when the stack opens/closes (`reset_simulation_map_viewport_on_left_stack_toggle` resets settle latch; hole geometry unchanged).

#### Visibility rules (`sync_command_left_stack_visibility`)

| State | LeftContextRail | BuildRailRoot | LeftContextStackBody |
|:---|:---|:---|:---|
| Collapsed | **Visible** | **Always visible** | Hidden + `Display::None` |
| Expanded | Hidden | **Always visible** | Visible |

**UX consequence:** Build tools stay on-screen when the narrative stack is open — good for construction flow, bad for mock fidelity and left-edge map occlusion (~458px).

#### Why Phase 2 accepted the gap

| Mitigation | Detail |
|:---|:---|
| PLAY-01 default | `CommandLeftStackState.collapsed = true` on sim enter — most sessions see ~106px, not 458px |
| Build rail authority | Single sim tool rail (2B) — `BuildStripState` writes from `BuildRailRoot`; cannot hide without product regression |
| Viewport spine | Absolute overlay preserves Stage 5 viewport measure contract; no flex reflow drift |
| Witness green | `UI_SHELL_MIGRATION_2B` does not assert left-chrome width — layout debt tracked here |

#### Resolution options (designer picks one in Phase 2C)

| ID | Approach | `@designer` | `@coder` | Tradeoff |
|:---|:---|:---|:---|:---|
| **2C-A** | **Merge** build tools into **48px** P4 rail (stacked icon cells; expand for labels) | Update P4 mock + rail diagram | Collapse `BuildRailRoot` into `LeftContextRail`; one column | Matches mock width; denser rail |
| **2C-B** | **Amend mock** — canonical **dual column** 48 + 52 (+ gap) collapsed | Revise § P4 in `ui_phase0_panel_mocks_v1.md` | Doc-only unless 52→48 normalization wanted | Honest spec; map occlusion documented |
| **2C-C** | **Inset map hole** when left chrome present | Mock shows map rect inset by chrome width | Move overlay into flex row OR drive `SimulationMapViewportFill` margins from chrome measure | Best occlusion fix; touches viewport authority — sim-steward review |
| **2C-D** | **Hide build rail** when stack expanded; relocate tools into stack footer | Expanded-state mock frame | Toggle `BuildRailRoot` visibility with stack; tool strip in stack body | Reduces expanded footprint; mode switch UX |

**Recommendation:** **2C-B** (mock amendment) or **2C-A** (merge) for lowest viewport risk; **2C-C** only if product requires zero map underlay occlusion.

#### Phase 2C exit criteria

- [ ] Designer records chosen option (**2C-A/B/C/D**) in [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) § P4 (+ diagram if needed).
- [ ] `@coder` implements chosen layout; no parallel left chrome spawn.
- [ ] Optional witness: `left_chrome_width_px_collapsed` / `_expanded` in `UiShellMigrationWitness`.
- [ ] If **2C-C**: viewport drift witness ≤2px after toggle (`viewport_drift.json` / sim-steward gate).
- [ ] Designer re-signs P4 row in this doc (**PASS** or **CONDITIONAL** cleared).

**Panel verdict (Phase 2):** **CONDITIONAL** on layout mock delta only — functionally shippable; **2C** closes the P4 row.

### Optional — traced atlas PNG (Phase 4 art)

**Disposition:** Keep [`icon_atlas_phase4_v1.png`](../../../assets/textures/ui/icon_atlas_phase4_v1.png) **placeholder** until designer delivers traced silhouettes per [`ui_phase4_icon_atlas_brief_v1.md`](ui_phase4_icon_atlas_brief_v1.md) §4–§7. **Not blocking** implementation or Phase 2 **SIGNED** — swap PNG only when art drops (same path, same RON grid).

---

## Accessibility (cross-panel)

| Check | Mock / rule | Implementation | Match |
|:---|:---|:---|:---:|
| Alert non-color cue | ◆ + number | `format_ops_strip_alert_badge` | ☑ |
| Focus order | Documented | [`ui_layout_agent.md`](../../../tools/orchestrator/agents/ui_layout_agent.md) § F-10 | ☑ |
| Hover token | `ink_magenta_bright` | Ops zones use `bevy_accent_hot()` border (F-03) | ✅ |

---

## Overall verdict

### **SIGNED**

| Criterion | Status |
|:---|:---|
| P1–P3 match Phase 0 mocks | ✅ |
| P4 material complete; 2C overlap flagged, defer OK | ✅ |
| Witness `UI_SHELL_MIGRATION_2B` green (2A + 2B) | ✅ |
| Sprint **4.3** visual / Stage 5 regression | ✅ (2026-05-24 — `--test visual` exit 0, `stage5` 22/22, `stage5_full_app_live.json` refreshed) |

Signed after **@coder item 1** (4.3 visual gate) without spine regression.

---

## @coder follow-ups (numbered)

1. ~~**4.3 visual gate**~~ — **done** (2026-05-24).
2. ~~**F-03 (optional)**~~ — **done** (`ops_zone_hover_token: true`).
3. **P4-ART-01 (optional)** — Traced atlas PNG when ready; **keep placeholder** until drop.
4. ~~**2C layout**~~ — **CLOSED** — **2C-B** signed 2026-05-24; mock § P4 + witness `phase2c` aligned. No coder layout change required.

---

## Sign-off record

| Role | Date | Verdict | Notes |
|:---|:---|:---|:---|
| Coder | 2026-05-24 | **4.3 + F-03** | Visual gate green; ops strip hover `bevy_accent_hot()` |
| Designer | 2026-05-24 | **SIGNED** | v2.2 — 2C-B closed; P4 **PASS**; P4-ART optional |
| Designer | 2026-05-23 | **P4-ART hold** | Placeholder atlas kept — traced PNG not delivered |

**Out of scope (Phase 2):** sim-side `BuildToolbox` catalog (PLAY-01); F3 editor egui removal; Phase 4 traced atlas (**designer-owned**, placeholder acceptable).

**Separate lane — World Preview D-01 (not P1–P4):** [`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md) · **D-01 A** single workspace · **UI-WP-LAYOUT-001** coder **done** (2026-05-24). Do not conflate with this **UI-P2-DESIGN** sign-off.

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-23 | Initial checklist |
| v1.6.0 | 2026-05-23 | UI-P2-DESIGN witness reconcile |
| v2.2.1 | 2026-05-24 | @coder item 4 — designer picks **2C-A/B/C/D** first; coder implements after mock § P4 |
| v2.2.0 | 2026-05-23 | Expanded Phase 2C layout defer — options 2C-A/B/C/D, width budget, exit criteria |
| v2.1.1 | 2026-05-23 | P4-ART blocked on designer traced atlas; placeholder kept |
| v2.1.0 | 2026-05-24 | **SIGNED** — 4.3 visual gate; F-03 hover token |
| v2.0.0 | 2026-05-23 | Phase 1 mirror format; panel verdict table P1–P4; 2C flag; numbered @coder items |
