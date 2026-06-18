# Sim HUD context tray — Build tab body `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-SIM-HUD-TRAY-BUILD-001** |
| **Program** | PLAN-SIM-HUD-PROFESSIONAL-POLISH-001 · Track 2 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Depends** | [`sim_hud_copy_registry_v1.md`](sim_hud_copy_registry_v1.md) · [`design_sim_hud_cohesion_charter_v1.md`](design_sim_hud_cohesion_charter_v1.md) |
| **Prior slice** | [`design_sim_hud_build_v1.md`](../docs/archive/2026-06-src-dev/plans/design_sim_hud_build_v1.md) |
| **Handoff** | COD-SIM-HUD-TRAY-BUILD-001 |
| **Verdict** | **PASS** |

```text
DES-SIM-HUD-TRAY-BUILD-001 Q✓
Legend · staging · queue live in tray Build tab — not floating RIGHT_BOTTOM
```

---

## 0. Purpose

Move **site legend**, **staged parametric rows**, and **pending queue summary** into the context tray **Build** tab body. Retire `staged_ghost_panel` as default sim floater ([`design_sim_hud_reflection_audit_v1.md`](design_sim_hud_reflection_audit_v1.md) B6).

**Popup tier:** **P1 Tray** — not `egui::Area` anchored to viewport corner.

---

## 1. Tray states (unchanged shell)

| State | Tab row | Build tab body |
|:---|:---|:---|
| **Collapsed** (sim enter default) | visible | hidden |
| **Peek** (48px) | visible | **one line** `tray.build.peek.modifiers` only |
| **Expanded** | visible | full §2 layout |

Constants retained: `CONTEXT_TRAY_TAB_H_PX = 32`, body **96px min** expanded (scroll if content exceeds).

**Default tab on expand:** **Alerts** (unchanged). User switches to **Build** tab for construction detail.

---

## 2. Build tab body layout (expanded)

```text
┌ [Alerts] [Logistics] [Build] ─────────────────────────────────────┐
│ ┌ Site stub ────────────────────────────────────────────────────┐ │
│ │ Green — building footprint                                    │ │
│ │ Dashed — yard / rail / park                                   │ │
│ │ Yard · Rail · Svc · Park · Load                               │ │
│ └───────────────────────────────────────────────────────────────┘ │
│ ┌ Staged placement ─────────────────────────────────────────────┐ │
│ │ Row: {label} · {x},{z} · Ctrl/Shift hints          [×]        │ │
│ │ … scroll max 3 rows …                                         │ │
│ └───────────────────────────────────────────────────────────────┘ │
│ ┌ Pending queue ────────────────────────────────────────────────┐ │
│ │ 2 pending · Cement kiln                                       │ │
│ └───────────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────┘
```

**Section order fixed:** Legend → Staging → Queue.

---

## 3. Site stub legend

| Condition | Show |
|:---|:---|
| Site overlay **off** | legend section **hidden** (zero height) |
| Site overlay **on** | full §2 legend block |

Copy: registry `tray.build.legend.*` — from [`design_build_toolbox_hud_v1.md`](design_build_toolbox_hud_v1.md) §4.

**Not on map center** · not floating.

---

## 4. Staged placement (migrated from `staged_ghost_panel`)

| Element | Spec |
|:---|:---|
| Title | `tray.build.staging.title` |
| Row | human label · tile `x,z` · modifier reminder caption |
| Remove | `×` per row — same authority as today |
| Empty | `tray.build.staging.empty` |
| Max visible rows | **3** — vertical scroll inside section |
| Max section height | **120px** |

**Ban:** `egui::Area::RIGHT_BOTTOM` · unstyled `ui.heading` · meta drag title.

**Parametric scale readout:** append to row caption when PARAM active — `scale {factor}` mono `fg_data`.

---

## 5. Pending queue summary

| Element | Spec |
|:---|:---|
| Title | `tray.build.queue.title` |
| Summary line | `tray.build.queue.summary` with `{n}`, `{first_label}` |
| Empty | `tray.build.queue.empty` |
| Click summary | *(P2)* expand docked construction queue widget |

v1: read-only summary — no inline edit in tray.

---

## 6. Peek mode (48px body)

When tray peek and build tool active:

```text
Ctrl rotate · Shift scale
```

Single line `FONT_CAPTION` · `fg_muted` — modifiers always visible even when tray not expanded ([`design_sim_hud_reflection_audit_v1.md`](design_sim_hud_reflection_audit_v1.md) B7).

Context strip still carries full mode line — peek is **redundant safety**, not sole source.

---

## 7. Interaction with picker sheet

| Situation | Behaviour |
|:---|:---|
| Picker open + tray expanded | Both visible — sheet does not cover tray |
| Invalid placement | strip + tray staging row show `blocked: {reason}` |
| Place success | staging row clears · queue summary updates |

---

## 8. Typography & tokens

| Section | Style |
|:---|:---|
| Section headers | Title 13px `fg_primary` |
| Legend lines | Body 11px |
| Staging coords | Data mono `fg_data` |
| Queue count | Data mono `fg_data` |

Background: `bg_elevated` inside vellum tray body wash.

---

## 9. Migration map

| From | To |
|:---|:---|
| `staged_ghost_panel.rs` floating area | tray §4 |
| Map-centered legend (if any) | tray §3 |
| Build toolbox staging (editor) | unchanged — editor only |

**COD-SIM-HUD-POPUP-MIGRATE-001** removes POP-1 anchor after tray wire green.

---

## 10. Witness fields

```json
{
  "context_tray_build_tab_wired": true,
  "staged_panel_floating_sim": false,
  "site_legend_in_tray": true,
  "peek_shows_modifiers": true
}
```

Path: `debug_runs/sim_hud_tray_build_live.json`

---

## 11. Verification

| Check | Method |
|:---|:---|
| Sim enter tray collapsed | existing witness |
| Expand Build → legend when overlay on | manual |
| Staged row after parametric queue | integration |
| No RIGHT_BOTTOM panel in sim | grep + witness |
| Esc collapses tray (step 2) | input |

---

## 12. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |

**Unblocks:** COD-SIM-HUD-TRAY-BUILD-001 · COD-SIM-HUD-POPUP-MIGRATE-001
