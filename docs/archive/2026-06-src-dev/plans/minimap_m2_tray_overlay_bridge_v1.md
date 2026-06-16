# DESIGN-M3-TRAY-001 — M2 overlay tray → MinimapOverlayMask bridge `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-M3-TRAY-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS (qualified)** (bridge wiring is already green in current minimap compositor witness; this doc specifies the coder-facing mapping) |
| **Unblocks** | `UI-P3-M2-TRAY-OPT` (coder polish lane) |
| **Witness** | `debug_runs/minimap_compositor_live.json` → `/ui_p3_m2_tray_opt_green` and `/overlay_tray_minimap_mask` |
| **Do not break** | `/ui_p3_m2_tray_opt_green` |

---
## Scope
UI-P3-M2-TRAY-OPT contract: the **overlay tray selection** must drive `MinimapOverlayMask` for the minimap compositor.

---
## Bridge mapping (Minimap overlay mask bits)
`overlay_tray_minimap_mask` must reflect the tray selections for:
- `construction_heat` (true/false)
- `ecology_heat` (true/false)
- `fire_heat` (true/false)
- `logistics_heat` (true/false)

The bridge must keep the mask consistent across:
- sim map overlays
- minimap overlays

---
## Acceptance checklist (designer)
1. Toggling overlay tray updates minimap overlay mask bits (no desync).
2. Mask drives visibility of each overlay layer without changing extraction authority.
3. Bridge does not introduce new minimap extract passes (presentation-only).

