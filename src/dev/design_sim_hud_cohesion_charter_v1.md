# Sim HUD cohesion charter `v1` — Bevy / egui parity

| Field | Value |
|:---|:---|
| **ID** | **DES-SIM-HUD-COHESION-001** |
| **Program** | PLAN-SIM-HUD-PROFESSIONAL-POLISH-001 · Track 1 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Authority** | [`design_sim_hud_reflection_audit_v1.md`](design_sim_hud_reflection_audit_v1.md) §1 · [`UiPalette`](../../src/gui/style/palette.rs) |
| **Handoff** | COD-SIM-HUD-EGUI-THEME-001 |
| **Verdict** | **PASS** |

```text
DES-SIM-HUD-COHESION-001 Q✓
One instrument — same tokens, spacing, selection, focus — Bevy shell + egui satellites
```

---

## 0. North star

Simulation HUD reads as **one professional tool**, not Bevy chrome beside default-gray egui popups. **Keep the colour story** (black field, cyan labels, green data, gold selection, vellum tray). **Unify craft:** spacing, borders, typography roles, disabled-why, focus.

**Rejected:** new palette · APS Tk widgets · raw `Color32` in new sim UI · default egui gray windows.

---

## 1. Renderer split (acknowledged)

| Layer | Renderer | Examples |
|:---|:---|:---|
| **Shell** | Bevy UI | Ops strip, build rail, context tray chrome, minimap frame |
| **Satellite** | egui | Build picker sheet, road tool sheet, docked product shells |
| **Map** | Bevy world + overlays | Ghost, site stub, phase labels |

**Rule:** satellites **must** call `palette.to_egui_visuals()` + shell chrome helpers — never `egui::Visuals::dark()` alone.

---

## 2. Token mapping (egui ↔ Bevy)

| Role | `UiPalette` field | egui use |
|:---|:---|:---|
| App background | `bg_app` | `panel_fill`, `window_fill` |
| Elevated panel | `bg_elevated` | picker sheet body, card bg |
| Vellum wash | `bg_vellum` | picker sheet header, tray selected tab |
| Paper wash | `bg_paper` | ops strip |
| Primary label | `fg_primary` | body text, tab labels |
| Muted | `fg_muted` | captions, disabled secondary |
| Data / telemetry | `fg_data` | mono tick lines, OK values |
| Terminal accent | `accent_terminal` | valid highlights |
| Primary action | `accent_action` | Place, Confirm (amber) |
| Selection | `accent_gold` | selected rail slot, selected card border |
| Danger | `danger` | invalid, blocked |
| Warn | `warn` | risky overlap |
| Panel wire | `wire_magenta` | 1px window stroke |
| Attention wire | `wire_red` | focus ring (sparse) |

**Ban:** hard-coded `Color32::GRAY`, `ui.heading()` default white, engineer hex in new panels.

---

## 3. Spacing scale

| Token | px | Use |
|:---|:---:|:---|
| `S4` | 4 | icon inset, tight row gap |
| `S8` | 8 | rail-to-sheet gap, card padding |
| `S12` | 12 | section gap |
| `S16` | 16 | panel outer margin |

**Ban:** arbitrary 6px, 10px, 14px gaps in new specs.

---

## 4. Typography roles

| Role | Font | Colour | Example |
|:---|:---|:---|:---|
| **Title** | Segoe semibold 13px | `fg_primary` | `Build picker` |
| **Body** | Segoe 12px | `fg_primary` | card name |
| **Data** | mono 12px | `fg_data` | `T+0042`, tile coords |
| **Caption** | Segoe 11px | `fg_muted` | power tier footnote |
| **Action** | Segoe semibold 12px | `accent_action` | `Place` |

Min body **11px** in sim (Phase 1 floor retained).

---

## 5. Selection & focus

| Surface | Selected | Hover | Focus |
|:---|:---|:---|:---|
| Build rail slot | `accent_gold` 2px border | brighten border + label | gold ring |
| Picker card | gold left bar 3px | `bg_interactive` | `wire_red` 1px outer |
| Tray tab | `bg_vellum` fill | muted underline | cyan underline |
| egui button | `selection_bg` | `bg_interactive` | same as picker card |

**Keyboard:** picker sheet traps focus while open; Tab cycles cards; Esc closes sheet (before tray — see copy registry `esc.cascade`).

---

## 6. Borders & chrome

| Element | Spec |
|:---|:---|
| Picker sheet | 1px `wire_magenta` · corner radius 4px · **no shadow** |
| Cards | 1px muted stroke · radius 4px |
| Separator | 1px `wire_magenta` @ 70% alpha |
| Scrollbars | thin · `fg_muted` thumb |

Match Bevy rail: flat, wireframe — no skeuomorphic drop shadows.

---

## 7. Disabled-why

| Pattern | Rule |
|:---|:---|
| Grayed button | adjacent caption `Blocked — {reason}` |
| Invalid place | strip + toast — never color-only |
| Empty category | `○ No items in this category` |

Ref: construction invariants · [`sim_hud_copy_registry_v1.md`](sim_hud_copy_registry_v1.md).

---

## 8. Density ladder

| State | Chrome visible |
|:---|:---|
| Sim enter | Ops strip · rail · tray **collapsed** (tabs only) · context strip peek |
| Build active | + picker sheet when category opened |
| Tray expanded | + tray body (Build tab content per tray spec) |
| Pause | modal blocks all |

**Max simultaneous egui popups in build path:** **1** anchored sheet (P0 tier).

---

## 9. Enforcement (coder)

| Check | Method |
|:---|:---|
| Palette on all sim egui passes | `COD-SIM-HUD-EGUI-THEME-001` audit grep |
| No default visuals | deny `Visuals::dark()` in `src/construction/*_menu.rs` after migrate |
| Spacing lint | code review against §3 |
| Witness | `debug_runs/sim_hud_egui_theme_live.json` |

---

## 10. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |

**Unblocks:** DES-SIM-HUD-BUILD-PICKER-001 · COD-SIM-HUD-EGUI-THEME-001
