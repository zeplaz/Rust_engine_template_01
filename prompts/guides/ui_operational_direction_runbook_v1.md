# UI operational direction runbook `v1`

**Purpose:** Translate **UI direction principles** into actionable UX targets: **strategic operations table** over classic RTS HUD clutter — **overlay-first**, **map-primary**, minimal permanent chrome.

**Parent:** [`experience_layer_orchestrator_v1.md`](experience_layer_orchestrator_v1.md) → [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

**Related:** [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md), [`gui_runbook_v1.md`](gui_runbook_v1.md), [`camera_map_navigation_runbook_v1.md`](camera_map_navigation_runbook_v1.md), [`strategic_overlay_runbook_v1.md`](strategic_overlay_runbook_v1.md)

**Source draft:** [`base_ui_direction_principls.md`](base_ui_direction_principls.md) *(historical filename: “principles” spelling)*

Version: `v1.0.0`

---

## 1. What to avoid

The game should **not** default to:

- Spreadsheet spam, RTS button carpets  
- Floating debug-style windows in **gameplay**  
- MMO hotbar clutter, Paradox-dense chrome on map  

The simulation already exposes **overlays, networks, logistics/ecology fields** — UI should **read** those, not duplicate them as menus.

---

## 2. Target posture: strategic operations table

**Intent:** UI acts like a **command table** more than a classic RTS HUD.

**Inspiration cues:** military GIS, satellite / ops-center planning, infrastructure dashboards.

**Layout pattern (conceptual):**

- **Top bar:** time, alerts, intel, economy, weather, power  
- **Left tool:** mode / layer (build, logistics, intel, military, ecology, utilities, regions)  
- **Center:** **map / world** with **operational overlays** (throughput, recon, fires, congestion, etc.)  
- **Bottom / tray:** **context** for selection (region, corridor, city) — throughput, shortages, construction, risk  

---

## 3. Mockup epics → implementation

| Draft mockup | Epic summary |
|:---|:---|
| **Operational Command Table** | Default strategic shell; top bar + left modes + overlay-heavy map + context tray |
| **Infrastructure Planner** | CAD/GIS-style corridor tools; transport/power/water/industry/defense tabs; tool context bar |
| **War Operations Overlay** | Threat / recon / EW / logistics / air / fires ribbon; front analysis strip |
| **Minimal Strategic UI** | Immersion mode: thin top line + fullscreen map + context-sensitive tray |
| **Geo-Strategic Systems UI** | Hybrid closest to engine direction: overlay strip + optional region panel + event feed |

**Recommended hybrid (from draft):**

| System | UI style |
|:---|:---|
| Gameplay HUD | Minimal native **Bevy** UI |
| Overlays | Fullscreen map-centric |
| Dev / debug | **egui** only |
| Construction | Contextual planners (corridors, splines) |
| Logistics | Inspector trays |
| Warfare | Operational overlays |

---

## 4. Key UX rules

1. **No giant permanent windows** — prefer slide-up inspectors, collapsible trays, radial actions, overlay toggles.  
2. **Map is primary** — reinforce reading the world, flows, and systems; **not** menu navigation.  
3. **Overlay-first** — prefer fields and toggles over window stacks.  
4. **Dev tools** — gated; follow [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md).

---

## 5. Interaction model (backlog drivers)

| Interaction | UX target |
|:---|:---|
| Inspect | Click / select → contextual inspector |
| Build corridor | Drag spline, route painting, preview costs |
| Artillery / fires | Zone paint / target sketch |
| Fortification | Line sketch |
| Logistics | Route / throughput overlay |
| Power | Network visualization |
| Ecology | Field overlays |

Bind inputs through [`InputBindings`](../../src/gui/input_bindings.rs) per [`camera_map_navigation_runbook_v1.md`](camera_map_navigation_runbook_v1.md) patterns.

---

## 6. Suggested visual language

| Element | Representation |
|:---|:---|
| Logistics | Flow arrows |
| Power | Glowing / grid network |
| Artillery | Danger heat |
| Recon | Soft visibility fields |
| EW | Noise / contour |
| Fire | Animated spread |
| Congestion | Bottleneck emphasis |

---

## 7. Technology split (must match boundary guide)

| Technology | Use for |
|:---|:---|
| **Bevy UI** | Game HUD, overlays chrome, notifications, contextual inspectors, radial menus |
| **egui** | Dev panels, tuning, editors, diagnostics |

---

## 8. Acceptance (v1 drafting)

- [ ] Every new gameplay panel decision cites **map-primary** or documents an explicit exception.  
- [ ] Overlay toggles **do not** spawn duplicate world-state panels (same data = one truth on map + thin inspector).  
- [ ] Feature UX reviewed against **anti-patterns** in §1 before merge.  

---

## 9. Long-term goal

Players **operate** infrastructure — war, ecology, logistics — **through** the map and fields, with **minimal** structural HUD weight.
