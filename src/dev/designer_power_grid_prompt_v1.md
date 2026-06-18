# @designer — power grid construction prompt `v1`

**Program:** [`plan_power_grid_construction_ux_v1.md`](plan_power_grid_construction_ux_v1.md)  
**Charter (read first):** [`design_power_line_construction_ux_v1.md`](design_power_line_construction_ux_v1.md)  
**Overlay colors:** [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md)  
**Tool sheet chrome:** [`plan_sim_hud_professional_polish_v1.md`](plan_sim_hud_professional_polish_v1.md)

---

## Situation

Power grid **sim exists** (transformers, overload, utility graph types) but players **cannot draw lines** — only place utility **buildings**. Roads/rails already have a **fun path-draw loop** with curved preview. Power needs the same craft: **curved vs 90°**, **voltage types**, **clear map read**, and UX for **cutting lines, knocking out transformers, islanding, and repair**.

**No Rust.** Specs + wireframes + copy only.

---

## P0 — Line construction (design first)

```
1. DES-POWER-LINE-TOOL-SHEET-001
   design_power_line_tool_sheet_v1.md
   Wire: mode toggle, voltage picker, snap toggles, commit/cancel
   Mirror road_tool_popup rhythm — UiPalette tokens
   Ref: design_power_line_construction_ux_v1.md §3

2. DES-POWER-ROUTING-MODE-001
   design_power_routing_mode_v1.md
   Curved (spline) vs Orthogonal (90° Manhattan):
     - when to default each
     - keybind cycle ( propose: O )
     - corner rules for 90° (no diagonal segments)
   Ref: road popup "Curved preview" — but power commits match mode

3. DES-POWER-VOLTAGE-PICKER-001
   design_power_voltage_picker_v1.md
   Low / Medium / High labels → VoltageClass
   Mismatch blocked copy for context strip
   Stroke weight table (overlay §4.1)

4. DES-POWER-SNAP-RULES-001
   design_power_snap_rules_v1.md
   Transformer, substation, junction tee, optional corridor
   Invalid reasons list for strip + red preview hatch
```

---

## P1 — Map read & islanding

```
5. DES-POWER-MAP-OVERLAY-002
   Extend design_infra_network_overlay_v1.md:
     live / preview / damaged / destroyed / island dim
   Auto-show power overlay while line tool active

6. DES-POWER-NODE-HOVER-001
   Transformer + substation hover card fields (human labels)

7. DES-POWER-ISLAND-UX-001
   Toast + ops strip + map boundary copy when graph splits
   Extend grid_overload toast pattern
```

---

## P2 — Combat & repair

```
8. DES-POWER-TARGETING-001
   Cut line / destroy transformer — preview "islands N consumers"

9. DES-POWER-REPAIR-PANEL-001
   Repair queue in context tray or dock — parts, priority 1–100
   Ref: power_damage_ui_persistence_v1.md
```

---

## @coder handoff (after P0 signed)

| Spec | Slice |
|:---|:---|
| tool_sheet + routing + voltage + snap | COD-POWER-LINE-DRAW-001 |
| commit to UtilityGraph | COD-POWER-LINE-COMMIT-001 |
| 90° router | COD-POWER-ORTHOGONAL-ROUTER-001 |
| curved router | COD-POWER-SPLINE-ROUTER-001 |
| Utilities rail entry | COD-POWER-TOOL-RAIL-001 |

---

## Rules

- **Reuse road input rhythm** — LMB add, RMB undo, Shift commit — do not invent Enter-only
- **Gold stroke family** — do not switch to random colors per mode
- **Graph authority** — UX previews must match UtilityGraph cuts (no fake island)
- **Fun = responsive** — preview updates same frame as cursor; no modal dead-ends
- **Strategic clarity** — player always sees *why* a factory went dark

```text
ΔWF→ DES-POWER-LINE-TOOL-SHEET-001 + DES-POWER-ROUTING-MODE-001 first
```
