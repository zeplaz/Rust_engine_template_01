# SIM-HUD-SLICE-MINIMAP — Minimap sim overlays `v1`

| Field | Value |
|:---|:---|
| **Slice ID** | **SIM-HUD-SLICE-MINIMAP** |
| **Program** | SIM-HUD-PRODUCT-001 |
| **Owner** | `@designer` → `@coder` |
| **Verdict** | **PASS (qualified)** |
| **Date** | 2026-06-03 |
| **Code** | `minimap_shell.rs`, `simulation_session.rs`, `dock_shell.rs`, minimap compositor |
| **Prereq** | SIM-HUD-SLICE-PLAY01 |

---

## Problem

Minimap overlay defaults differ between **editor** session and **Simulation** enter. Operators need M2/M3 play-read layers on without editor overlay-tray chrome bleeding into sim, and without ambient **fire heat** washing the full map at strategic zoom.

---

## Target — Simulation enter defaults

Source of truth: [`simulation_minimap_overlay_defaults()`](../../src/gui/minimap_shell.rs)

| Layer | Sim default | Rationale |
|:---|:---:|:---|
| logistics_heat | **on** | M2 corridor stress read |
| construction_heat | **on** | Site phase read |
| ecology_heat | **on** | M2 macro band |
| fow | **on** | M3 FoW |
| ew | **on** | M3 EW |
| units | **on** | M3 aggregation |
| replay_scrub | **on** | M3 replay ring when active |
| fire_heat | **off** | Avoid pink full-map wash; enable via tray when needed |

**Presentation:** minimap `visible = true`, `minimized = false`, GPU compositor when env enabled.

---

## Requirements (@coder)

| # | Requirement | Acceptance |
|:---:|:---|:---|
| 1 | `apply_simulation_map_presentation_defaults` applies mask to **both** `map_views.minimap.overlays` and `HudOverlayTrayState` | Tray toggles match minimap on enter |
| 2 | Simulation map (`MapViewInstanceId::SimulationMap`) keeps `fire_heat = false` on enter (non-test) | Already in session.rs — verify no regression |
| 3 | Overlay tray panel **collapsed** on sim enter — toggles still reflect defaults when opened | No auto-expand tray |
| 4 | Editor-only minimap egui texture dock **gated** in sim (`product_egui_shell_active` / dock registry) | No duplicate minimap chrome |
| 5 | Witness seeds M2/M3 snapshots before first composite (`seed_minimap_m2_*`, `seed_minimap_m3_*`) | FoW/EW visible in tactical capture |

---

## Wireframe — minimap chrome (embedded)

```text
┌─ Minimap ─────────────┐
│ [GPU/CPU raster]      │
│  + FoW veil           │
│  + unit glyphs        │
│  (fire heat OFF)      │
└───────────────────────┘
  corner inset per MAP_FRAME_INSET_PX
```

---

## Overlay tray copy (when user expands TRAY)

Toggle labels must match mask fields — prefix text, not color-only:

| Toggle | Label |
|:---|:---|
| fire_heat | Fire heat |
| logistics_heat | Logistics stress |
| construction_heat | Construction |
| ecology_heat | Ecology |
| fow | Fog of war |
| ew | EW denial |
| units | Units |
| replay_scrub | Replay scrub |

---

## PLAY-01 checklist

| Check | Pass |
|:---|:---:|
| Enter Sim → minimap visible | ✓ |
| Fire heat off until tray toggle | ✓ |
| FoW + units on in default sim | ✓ |
| Exit to editor → editor tray may differ (OK) | ✓ |

---

## Witness

`debug_runs/sim_hud_slice_minimap_live.json`:

```json
{
  "program_id": "SIM-HUD-SLICE-MINIMAP",
  "green": true,
  "defaults_match_simulation_minimap_overlay_defaults": true,
  "fire_heat_default_false": true,
  "fow_ew_units_default_true": true,
  "minimap_visible_on_sim_enter": true
}
```

Regression: `minimap_compositor_live.json` keys unchanged unless intentional.

---

## Out of scope

- New overlay channels · minimap multiview isolation (VM-* infra)
- Tactical fire VFX on main map (separate from minimap heat toggle)
