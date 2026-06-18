# Power line tool sheet `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-POWER-LINE-TOOL-SHEET-001** |
| **Program** | PLAN-POWER-GRID-CONSTRUCTION-UX-001 · Track A |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Charter** | [`design_power_line_construction_ux_v1.md`](design_power_line_construction_ux_v1.md) |
| **Chrome** | [`design_sim_hud_cohesion_charter_v1.md`](design_sim_hud_cohesion_charter_v1.md) · [`design_sim_hud_popup_tiers_v1.md`](design_sim_hud_popup_tiers_v1.md) P0 anchored |
| **Handoff** | COD-POWER-LINE-DRAW-001 |
| **Verdict** | **PASS** |

```text
DES-POWER-LINE-TOOL-SHEET-001 Q✓
P0 anchored sheet — rail Utilities → Lines; mirrors road_tool_popup rhythm
```

---

## 0. Entry

| Path | Action |
|:---|:---|
| Build rail → **Utilities** | Build picker **Lines** tab (or submenu row **Draw power line**) |
| Active tool | `BuildTool::PowerLine` · sheet opens anchored to Utilities slot |

**Ban:** free-floating `egui::Window` at `(12, 200)` like road popup today.

---

## 1. Anchor & chrome

| Rule | Value |
|:---|:---|
| Gap rail → sheet | **8px** (`S8`) |
| Width | **300px** |
| Max height | `min(420px, viewport - ops_strip - 32)` |
| Background | `bg_vellum` header · `bg_elevated` body |
| Border | 1px `wire_magenta` · radius 4px · no shadow |
| Close | `✕` 36×36 · outside click · Esc step 1 |

Vertical align: Utilities rail slot top (clamp to viewport).

---

## 2. Layout wire

```text
┌ Power line — Medium voltage ─────────────┐
│ Mode   [ Curved ] [ 90° ]                  │
│ Type   ( ) Distribution (•) Medium ( ) HV  │
│ Snaps  [x] Transformers [x] Junctions     │
│        [ ] Corridors  [x] Grid (90° only) │
│ ─────────────────────────────────────────  │
│ Points: 4   Valid: 3   Est. cost: 120       │
│ LMB add · RMB undo · Shift+LMB commit     │
│ [ Build line ]  [ Cancel ]                 │
└────────────────────────────────────────────┘
```

**Section refs:** Mode → [`design_power_routing_mode_v1.md`](design_power_routing_mode_v1.md) · Type → [`design_power_voltage_picker_v1.md`](design_power_voltage_picker_v1.md) · Snaps → DES-POWER-SNAP-RULES-001 (coder P0 tail).

---

## 3. Controls

| Control | Behaviour |
|:---|:---|
| **Mode chips** | `Curved` \| `90°` — mutually exclusive · see routing spec |
| **Type radio** | `Distribution` / `Medium` / `Transmission` → `VoltageClass` |
| **Snap toggles** | Persist per session in tool state |
| **Grid snap** | Enabled only when `90°` mode active (disabled + muted in Curved) |
| **Build line** | `accent_action` · disabled when `valid_segments == 0` |
| **Cancel** | Clear control points · keep tool selected |

**Stats row:** mono `fg_data` — Points / Valid / Est. cost from placement resource.

---

## 4. Context strip (always on)

| State | Template |
|:---|:---|
| Drawing | `POWER · {voltage_label} · {mode} · LMB add · RMB undo · Shift commit` |
| Invalid | `POWER · blocked: {reason}` |
| Committed | `POWER · line queued · {n} segments` |
| Island alert | `POWER · island — {n} offline` (overlay spec) |

Registry keys: `power.strip.*` in [`design_power_grid_copy_v1.md`](design_power_grid_copy_v1.md).

---

## 5. Input (match road)

| Input | Result |
|:---|:---|
| LMB | Add control point |
| RMB | Undo last point |
| Shift+LMB | Commit path |
| Esc | Cancel path (sheet stays) |
| **O** | Cycle routing mode |
| **[** / **]** | Curved / 90° direct select |

---

## 6. Disabled-why

| Condition | Sheet + strip |
|:---|:---|
| No valid segments | `Build line` grey + `Blocked — no valid segments` |
| Voltage mismatch | `Blocked — voltage mismatch at {node}` |
| No anchor | `Blocked — snap to transformer or junction` |

Never color-only block.

---

## 7. Witness

```json
{
  "power_line_tool_sheet_open": true,
  "anchor_gap_px": 8,
  "sheet_width_px": 300,
  "routing_mode": "orthogonal90",
  "voltage_class": "Medium",
  "floating_window": false
}
```

---

## 8. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
