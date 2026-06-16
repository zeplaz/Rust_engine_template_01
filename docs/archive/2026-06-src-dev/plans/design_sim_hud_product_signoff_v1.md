# SIM-HUD-PRODUCT-001 — Program close sign-off `v1.2`

| Field | Value |
|:---|:---|
| **Program** | **SIM-HUD-PRODUCT-001** / **SIM-HUD-PRODUCT-CLOSE-001** |
| **Owner** | `@designer` (sign-off) · `@coder` (slice witnesses) |
| **Date** | 2026-06-03 |
| **Verdict** | **PASS (full)** |
| **Prior** | v1.1 **PASS (qualified)** — 3/5 slices |
| **Brief** | [`sim_hud_product_brief_v1.md`](../prompts/designer_questions/sim_hud_product_brief_v1.md) |
| **Orders** | [`bevy_hud_lanes_agent_orders_v1.md`](bevy_hud_lanes_agent_orders_v1.md) |
| **Rollup witness** | [`sim_hud_product_close_001_live.json`](../../debug_runs/sim_hud_product_close_001_live.json) |

---

## Witness rollup (program close)

**Gate:** all **5** slice witnesses green + lane-4 boundary confirmed — **met**.

| Gate | Witness | Green | Key fields |
|:---:|:---|:---:|:---|
| G1 PLAY-01 | [`sim_hud_play01_live.json`](../../debug_runs/sim_hud_play01_live.json) | ✓ | `sim_hud_slice_play01.green`, `enter_hidden`, `exit_restore_wired` |
| G2 DOCK | [`sim_hud_slice_dock_live.json`](../../debug_runs/sim_hud_slice_dock_live.json) | ✓ | `command_tray_collapsed_on_sim_enter`, `overlay_tray_collapsed_on_sim_enter` |
| G3 OPS | [`sim_hud_slice_ops_live.json`](../../debug_runs/sim_hud_slice_ops_live.json) | ✓ | `ops_strip_font_min_px: 11`, `alerts_text_pairing: true` |
| G4 MINIMAP | [`sim_hud_slice_minimap_live.json`](../../debug_runs/sim_hud_slice_minimap_live.json) | ✓ | `fire_heat_default_false`, `minimap_visible_on_sim_enter` |
| G5 BUILD | [`sim_hud_slice_build_live.json`](../../debug_runs/sim_hud_slice_build_live.json) | ✓ | `build_rail_width_px: 52`, `context_tray_collapsed_on_sim_enter` |

**Close set:** G1–G5 (5/5). Rollup: `sim_hud_product_close_001_live.json` → `green: true`, `slices_green_count: 5`.

---

## 1080p polish review (Phase 3 close)

Designer playtest against slice specs at **1920×1080** (map viewport ≈ 1280×720 after ops strip + left stack):

| Surface | Read at 1080p | Notes |
|:---|:---|:---|
| **Ops strip** | ✓ | All zones legible at 11px floor; WX/PWR not clipped; ALERTS count paired with label |
| **Command dock** | ✓ | Trays collapsed on enter — map not obscured by editor command table |
| **Minimap** | ✓ | Corner inset readable; FoW/EW/units on; fire heat off avoids pink wash |
| **Build rail** | ✓ | 52px icons + gold selected border; ghost contrast meets construction read spec |
| **Context tray** | ✓ | Collapsed default; tab row discoverable via ▼ TRAY affordance |

**Deferred (non-blocking):** operator visual capture under `assets/ui/` · minimap legend dynamic wx wash (weather lane).

---

## Designer review checklist

### PLAY-01 regression

- [x] `apply_simulation_hud_defaults` wired — witness `sim_hud_slice_play01.modules`
- [x] Floating egui shells gated — `product_egui_shell_in_simulation: false`
- [x] Enter Sim hides editor chrome — `enter_hidden: true`
- [x] Exit restore wired — `exit_restore_wired: true`

### Slices verified (witness-backed)

- [x] DOCK — collapsed command + overlay trays on sim enter
- [x] OPS — font ≥11px; alerts text pairing green
- [x] MINIMAP — sim overlay defaults + fire_heat off; witness green
- [x] BUILD — 52px rail + collapsed context tray; ghost readability wired

### Boundary (do not merge)

- [x] Assembly snapshot QC stays egui dev panel (Ctrl+Shift+Q) — lane 4 separate
- [x] No Tk APS chrome in Bevy HUD
- [x] `lane4_egui_qc_separate: true` in rollup witness

---

## Lane map (closed)

| Lane | Program | Status |
|:---|:---|:---|
| **5** | SIM-HUD-PRODUCT-001 | **CLOSED** — full PASS |
| **4** | APS-BEVY-QC-HUD-001 | Shipped separately — not merged into product HUD |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (full)** | 2026-06-03 |

```text
SIM-HUD-PRODUCT-CLOSE-001 complete
Verdict: PASS (full)
Rollup: debug_runs/sim_hud_product_close_001_live.json
Program SIM-HUD-PRODUCT-001: CLOSED
```
