# Power infrastructure glyphs `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-ART-POWER-GLYPHS-001** |
| **Program** | PLAN-POWER-GRID-ART-ASSETS-001 · Lane C |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Depends** | [`design_utility_industrial_style_v1.md`](../../../src/dev/design_utility_industrial_style_v1.md) |
| **Overlay** | [`design_power_map_overlay_v1.md`](../../../src/dev/design_power_map_overlay_v1.md) |
| **Verdict** | **PASS** |

```text
DES-ART-POWER-GLYPHS-001 Q✓
Map node + state glyphs — 24×24 base grid, 1px stroke @ 1x
```

---

## 0. Canvas

| Field | Value |
|:---|:---|
| Base size | **24×24** px (map overlay) |
| Large (hover) | **32×32** px |
| Format | PNG keyframes + SVG source (coder export) |
| Stroke | 1px @ 1x · `fg_primary` cyan on dark field |
| Fill | flat — no gradients in glyph core |

**Folder:** `assets/ui/infrastructure/glyphs/` (PNG deliverables from art pipeline).

---

## 1. Node glyphs

| Id | Glyph | Description | Use |
|:---|:---|:---|:---|
| `node_transformer` | ▣ + 3 dots top | Coil pad + bushings | distribution transformer |
| `node_substation` | ▣▣ + bus line | Wider yard + horizontal bar | grid substation |
| `node_tee` | ●— | Junction tee | line split |
| `node_plant_coal` | ▣ + stack | Hall + chimney | coal plant |
| `node_plant_nuclear` | ◠ dome | Containment dome | PWR |
| `node_diesel` | ▣ + exhaust | Aux diesel gen | nuclear backup |

### ASCII wire (design reference)

```text
transformer:     substation:       tee:
  · · ·            ═══════           ●──
 ┌─────┐          ┌──┬──┐
 │     │          │  │  │
 └─────┘          └──┴──┘
```

---

## 2. State adjuncts (overlay corner 8×8)

| Id | Mark | Color token |
|:---|:---|:---|
| `state_live` | — (none) | — |
| `state_preview` | dashed ring | gold @ 60% |
| `state_damaged` | ◆ spark | `warn` |
| `state_destroyed` | × | `danger` |
| `state_island` | dim wash | muted gray |
| `state_overload` | ⟳ pulse | `warn` |
| `state_scram` | ▼ amber | `#e9c46a` |
| `state_meltdown` | ▲ column | `danger` |
| `state_diesel_run` | ~ exhaust | `fg_data` green |

**Rule:** state adjunct **bottom-right** of node glyph — never replace base shape.

---

## 3. Line endpoint glyphs

| Id | Use |
|:---|:---|
| `span_pole_mv` | MV pole at control point |
| `span_tower_hv` | HV lattice (P1) |
| `corner_90` | ⊞ orthogonal bend |
| `span_curve` | ~ midpoint on spline |

---

## 4. Voltage stroke (paired — not duplicate HUD icons)

| Class | Dash | Weight |
|:---|:---|:---:|
| Distribution | solid | 2px |
| Medium | solid | 3px |
| High | solid + glow 1px | 4px |

Colors per [`design_power_voltage_picker_v1.md`](../../../src/dev/design_power_voltage_picker_v1.md).

---

## 5. Keyframe list (PNG export)

| File | Content |
|:---|:---|
| `node_transformer_live.png` | base |
| `node_transformer_overload.png` | + overload adjunct |
| `node_substation_live.png` | base |
| `node_substation_destroyed.png` | + × |
| `node_tee_live.png` | base |
| `node_plant_nuclear_scram.png` | dome + SCRAM |
| `node_plant_nuclear_meltdown.png` | dome + meltdown |
| `state_island_boundary.png` | gold edge segment |

**Witness:** `debug_runs/art_pipeline/power_glyphs_keyframes_live.json`

---

## 6. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
