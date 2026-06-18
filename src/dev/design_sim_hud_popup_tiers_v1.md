# Sim HUD popup tiers & migration map `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-SIM-HUD-POPUP-TIERS-001** |
| **Program** | PLAN-SIM-HUD-PROFESSIONAL-POLISH-001 · Track 2 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Depends** | [`design_sim_hud_cohesion_charter_v1.md`](design_sim_hud_cohesion_charter_v1.md) · [`design_sim_hud_build_picker_v1.md`](design_sim_hud_build_picker_v1.md) · [`design_sim_hud_tray_build_v1.md`](design_sim_hud_tray_build_v1.md) |
| **Audit** | [`design_sim_hud_reflection_audit_v1.md`](design_sim_hud_reflection_audit_v1.md) §4 |
| **Handoff** | COD-SIM-HUD-POPUP-MIGRATE-001 |
| **Verdict** | **PASS** |

```text
DES-SIM-HUD-POPUP-TIERS-001 Q✓
Every floating UI has a tier — sim build path: 0 ad-hoc corner popups
```

---

## 0. Tier system (authoritative)

| Tier | ID | Behaviour | Close | Sim build examples |
|:---|:---|:---|:---|:---|
| **P0 Anchored** | `anchored` | Attached to rail slot or initiating control | Outside click · Esc (step 1) | Build picker sheet · Road tool sheet |
| **P0 Map** | `map_attached` | Follows ghost / tile — max 1 | N/A (transient) | Footprint validity chip |
| **P1 Tray** | `tray` | Context tray tab body | Tray collapse | Site legend · staging · queue |
| **P2 Docked** | `docked` | Product shell widget — persisted | Widget close | Construction queue · Overlays |
| **P3 Modal** | `modal` | Blocks map input | Explicit dismiss · Esc final | Pause menu · confirm destructive |

**Sim build path rule:** at most **one** `anchored` sheet + **one** `map_attached` chip + tray peek line. **Ban** second corner `egui::Area` / draggable `Window` in default sim layout.

---

## 1. Anti-pattern → fix map

| ID | Anti-pattern | File today | Target tier | Migration |
|:---|:---|:---|:---|:---|
| POP-1 | `RIGHT_BOTTOM` anchor | `staged_ghost_panel.rs` | **P1 tray** | Remove sim draw · tray §4 |
| POP-1 | Floating submenu area | `build_toolbox.rs` `sim_build_rail_submenus` | **P0 anchored** | Replace with picker sheet |
| POP-2 | Drag title meta | `build_toolbox.rs` editor window | **N/A sim** | Gated off sim (existing) |
| POP-3 | Unstyled heading | `staged_ghost_panel.rs` | **P1 tray** | Token title in tray |
| POP-4 | Multi corner stack | staged + road window + hints | **tier split** | See §2 |
| POP-5 | Engineer IDs | `industrial_menu.rs` | **picker cards** | Copy registry |
| POP-6 | Color-only invalid | `build_footprint_overlay.rs` | **P0 map + strip** | Text label required |

---

## 2. Surface migration matrix

| Surface | Renderer | Current anchor | Target | Priority | Coder slice |
|:---|:---:|:---|:---|:---:|:---|
| **Build rail submenus** | egui Area | fixed near rail | **P0 picker sheet** | P0 | COD-SIM-HUD-BUILD-PICKER-001 |
| **Staged parametric panel** | egui Area | `RIGHT_BOTTOM` | **P1 tray Build** | P0 | COD-SIM-HUD-TRAY-BUILD-001 |
| **Road / rail tool popup** | egui Window | `default_pos(12,200)` | **P0 Road tool sheet** | P0 | COD-SIM-HUD-POPUP-MIGRATE-001 |
| **Tool hints overlay** | egui Area | `LEFT_BOTTOM` | **Context strip + tray peek** | P1 | COD-SIM-HUD-POPUP-MIGRATE-001 |
| **Footprint validity hint** | egui Area | ghost tile offset | **P0 map_attached** (keep, restyle) | P1 | COD-SIM-HUD-EGUI-THEME-001 |
| **Build toolbox window** | egui Window | floating | **off in sim** | — | already gated |
| **Placement debug** | egui Window | floating | **dev only** | — | no sim change |
| **Phase labels** | map | world | **map overlay** | P2 | defer |
| **Docked shells** | shell | dock framework | **P2** | — | keep |

---

## 3. P0 — Road tool sheet (new)

Replaces `draw_road_tool_popup_egui` draggable window.

### Anchor
Same contract as build picker ([`design_sim_hud_build_picker_v1.md`](design_sim_hud_build_picker_v1.md) §2):
- Rail **Roads** slot active → sheet at rail + 8px
- Width **280px** · vellum header · `wire_magenta` border

### Content (from existing popup — presentation only)

```text
┌─ Road — Street ─────────────────────────────┐
│ Control points: {n} · Valid segments: {v}   │
│ Estimated cost: {cost} · Width: {w}         │
│ [✓] Grid snap  [✓] Node snap  [ ] Curved    │
│ LMB add · RMB undo · Shift+LMB commit       │
│ [ Build ]  [ Cancel ]  [ Upgrade nearest ]  │
└─────────────────────────────────────────────┘
```

| Element | Token |
|:---|:---|
| Primary | `Build` — `accent_action` |
| Secondary | `Cancel` — muted |
| Data lines | mono `fg_data` |

**Ban:** `default_pos` free-floating window · raw `Color32` in labels.

### Copy keys (add to registry)

| Key | String |
|:---|:---|
| `road.sheet.title.street` | `Road — Street` |
| `road.sheet.title.highway` | `Road — Highway` |
| `road.sheet.title.rail` | `Rail — Standard` |
| `road.sheet.hint.input` | `LMB add · RMB undo · Shift+LMB commit` |
| `road.sheet.build` | `Build` |
| `road.sheet.cancel` | `Cancel` |
| `road.sheet.upgrade` | `Upgrade nearest segment` |

---

## 4. P1 — Tool hints retirement

**Problem:** `tool_hints.rs` duplicates and **contradicts** two-click building charter (`Shift+LMB: queue blueprint`).

### Target

| Tool | Primary hints surface | Secondary |
|:---|:---|:---|
| **Building** | Context strip ([`sim_hud_copy_registry_v1.md`](sim_hud_copy_registry_v1.md) §1) | Tray peek modifiers |
| **Zone** | Strip one-liner + picker Roads/Zone tab footnote | — |
| **Road / Rail** | Road tool sheet §3 hint row | — |
| **Demolish** | Strip | — |

### Migration steps

1. **Sim:** disable `draw_tool_hints_egui` when `BaseState::Simulation`
2. **Editor:** keep hints OR move to diagnostics — out of Phase 2 scope
3. Wire zone/road strings to registry keys `hints.zone.*` / `hints.demolish.*` (P1 appendix)

**Success:** no `LEFT_BOTTOM` hint area in sim witness.

---

## 5. P0 map_attached — footprint validity chip

Keep map-following chip — **restyle only**:

| Rule | Spec |
|:---|:---|
| Background | `bg_elevated` @ 90% alpha — not hard-coded RGB |
| Text | `fg_primary` + invalid reason line |
| Position | ghost anchor offset (unchanged geometry) |
| Duplicate | If strip shows `blocked: {reason}`, chip may shorten to icon + word |

**Tier:** `map_attached` — does not count toward anchored sheet limit.

---

## 6. Esc cascade (single handler)

Align [`sim_hud_copy_registry_v1.md`](sim_hud_copy_registry_v1.md) §6:

```text
Esc #1 → close anchored sheet (picker OR road sheet)
Esc #2 → collapse context tray
Esc #3 → pause menu (if no other modal)
```

Road sheet and build picker are **mutually exclusive** — only one open.

---

## 7. Implementation order (@coder)

```text
1. COD-SIM-HUD-BUILD-PICKER-001     — retires sim_build_rail_submenus
2. COD-SIM-HUD-TRAY-BUILD-001       — retires staged_ghost_panel in sim
3. COD-SIM-HUD-POPUP-MIGRATE-001    — road sheet + disable tool_hints sim
4. COD-SIM-HUD-EGUI-THEME-001       — footprint chip tokens + palette audit
```

Each step: witness JSON + `ad_hoc_corner_popups == 0` in sim.

---

## 8. Witness contract

`debug_runs/sim_hud_popup_tiers_live.json`:

```json
{
  "program_id": "DES-SIM-HUD-POPUP-TIERS-001",
  "green": true,
  "sim_build_path": {
    "anchored_sheets_max": 1,
    "staged_panel_right_bottom_sim": false,
    "tool_hints_left_bottom_sim": false,
    "road_popup_floating_window": false,
    "tray_staging_wired": true
  }
}
```

---

## 9. Out of scope (Phase 2)

| Surface | Note |
|:---|:---|
| Pause menu | DES-SIM-HUD-PAUSE-002 |
| Info / overlay dock IA | DES-SIM-HUD-OVERLAY-002 |
| Minimap legend dock | DES-SIM-HUD-MINIMAP-002 |
| Editor construction window | stays floating |
| F4 HUD dev overlay | dev only |

---

## 10. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |

**Unblocks:** COD-SIM-HUD-POPUP-MIGRATE-001

**ΔWF complete for P0 designer lane** — next per prompt: P1 OPS-002 / OVERLAY-002 / MINIMAP-002
