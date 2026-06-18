# HUD power icons `v1.1` — build rail + tool sheet

| Field | Value |
|:---|:---|
| **ID** | **DES-ART-HUD-POWER-ICONS-001** |
| **Program** | PLAN-POWER-GRID-ART-ASSETS-001 · Lane D |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Depends** | [`design_power_line_tool_sheet_v1.md`](design_power_line_tool_sheet_v1.md) · [`design_power_voltage_picker_v1.md`](design_power_voltage_picker_v1.md) · [`design_power_routing_mode_v1.md`](design_power_routing_mode_v1.md) · [`design_utility_industrial_style_v1.md`](design_utility_industrial_style_v1.md) |
| **Related** | [`design_plant_card_gauges_v1.md`](design_plant_card_gauges_v1.md) (DES-ART-PLANT-CARD-001) |
| **Handoff** | COD-ART-HUD-ICON-ATLAS-001 |
| **Verdict** | **PASS** |

```text
DES-ART-HUD-POWER-ICONS-001 Q✓ v1.1
Line tool · L/M/H voltage · curved/90° routing — pixel wire + atlas layout
```

---

## 0. Atlas contract

| Field | Value |
|:---|:---|
| Grid cell | **20×20** px (build rail) · **16×16** (sheet chips) |
| Safe margin | **2px** inset — stroke never touches cell edge |
| Color | 1-bit glyph + **tint** via `UiPalette` — cyan idle, gold selected |
| Voltage tint | Distribution `#e8c040` · MV `#f0d050` · HV `#ffd878` (chip fill @ 40% when active) |
| Style | Wireframe CRT — match build rail industrial icons |
| File | `assets/ui/icons/power_hud_atlas.png` + `power_hud_atlas.ron` layout |

**Stroke:** 1px @ 1x · anti-alias off · no gradients inside glyph.

---

## 1. Line tool — `icon_power_line_tool`

**Use:** Build rail **Utilities → Lines** · sheet header adjunct.

### 20×20 rail wire

```text
····················
····················
······●────────●····  ← pole caps (2px dots)
······│╲      ╱│····
······│ ╲    ╱ │····  ← curved span (MV default tint)
······│  ╲  ╱  │····
······│   ╲╱   │····
······●────────●····
····················
```

| State | Treatment |
|:---|:---|
| Idle | `fg_primary` stroke |
| Selected (rail) | `accent_gold` border on cell + glyph unchanged |
| Tool active | glyph + 1px gold underline on rail slot |

**Read @ 20px:** two anchors + one arc — distinct from road tool (no lane width ticks).

---

## 2. Voltage tiers — `icon_voltage_low` / `_medium` / `_high`

**Use:** Tool sheet **Type** row · 16×16 chips beside radio labels.

### Shared grammar

| Tier | Id | Visual | Bar count |
|:---|:---|:---|:---:|
| Distribution | `icon_voltage_low` | Single horizontal bar | 1 |
| Medium | `icon_voltage_medium` | Two stacked bars | 2 |
| Transmission | `icon_voltage_high` | Three bars + crown tick | 3 |

### 16×16 wires

```text
low (1 bar):          medium (2 bars):       high (3 + tick):
················      ················       ················
················      ················       ·······┌┐·····
····████████████··    ····████████████··     ······┌┘└┐····
················      ····████████████··     ····████████████
················      ················       ····████████████
················      ················       ····████████████
```

| State | Treatment |
|:---|:---|
| Unselected chip | `fg_muted` outline bars |
| Selected chip | Tier tint fill @ 40% + `accent_gold` 1px chip border |
| Disabled (compatibility) | `fg_muted` @ 50% + strikethrough caption in sheet (not on icon) |

**Rule:** bar count = tier — never recolor same shape for L/M/H.

---

## 3. Routing mode — `icon_route_curved` / `icon_route_90`

**Use:** Tool sheet **Mode** row · mutually exclusive chips.

### 16×16 wires

```text
curved (~):                    90° (⊞):
················               ················
················               ················
······●───────●···             ····████████████
······│╲     ╱│···             ····█········█··
······│ ╲   ╱ │···             ····█········█··
······│  ╲ ╱  │···             ····█········█··
······│   ╳   │···             ····████████████
················               ················
```

| State | Treatment |
|:---|:---|
| Inactive chip | `fg_muted` glyph |
| Active chip | `fg_primary` glyph + gold chip border |
| Grid snap locked off (Curved) | `icon_route_90` stays selectable; no lock icon in v1 |

**Keybind hint:** sheet caption `O cycle · [ curved · ] 90°` — no key glyphs in atlas.

---

## 4. Secondary icons (inventory)

| Id | Label | Use |
|:---|:---|:---|
| `icon_snap_transformer` | ▣⌇ | snap toggle |
| `icon_snap_junction` | ●⌇ | snap toggle |
| `icon_substation_place` | ▣▣ | place substation |
| `icon_transformer_place` | ▣ | place transformer |
| `icon_diesel` | ▣~ | nuclear diesel status |
| `icon_scram` | ▼ | SCRAM alert |
| `icon_island` | ⚠ | grid island |
| `icon_repair` | ≡ | repair queue |

---

## 5. Rail slot mapping

| Build picker tab | Icon |
|:---|:---|
| Utilities → **Lines** | `icon_power_line_tool` |
| Utilities → **Substation** | `icon_substation_place` |
| Utilities → **Transformer** | `icon_transformer_place` |

Selected slot: **gold** border per [`design_sim_hud_build_picker_v1.md`](design_sim_hud_build_picker_v1.md).

---

## 6. Tool sheet chip row (layout)

```text
Mode   [~ Curved] [⊞ 90°]     ← icons 16×16 left of 4px gap to label
Type   ( ) ▬ Dist  (•) ▬▬ MV  ( ) ▬▬▬ HV
```

Chip size: **min 44×28** touch target · icon vertically centered.

---

## 7. Ops strip badges

| Alert | Icon | Strip prefix |
|:---|:---|:---|
| Island | `icon_island` | `⚠ Island` |
| Overload | spark adjunct | existing PWR row |
| SCRAM | `icon_scram` | `▼ SCRAM` |
| Diesel running | `icon_diesel` | `~ Diesel` |

---

## 8. Atlas layout (`power_hud_atlas.ron`)

| Row | Col 0 | Col 1 | Col 2 | Col 3 |
|:---:|:---|:---|:---|:---|
| 0 | `icon_power_line_tool` | `icon_substation_place` | `icon_transformer_place` | `icon_repair` |
| 1 | `icon_voltage_low` | `icon_voltage_medium` | `icon_voltage_high` | — |
| 2 | `icon_route_curved` | `icon_route_90` | `icon_snap_transformer` | `icon_snap_junction` |
| 3 | `icon_diesel` | `icon_scram` | `icon_island` | — |

Tile size **20** · rows **4** · cols **4**.

---

## 9. COD registration

```ron
// icon_atlas.rs — power_hud section
// ids per §8 · tint rules §0 · selected = gold border in widget, not atlas variant
```

**Witness:** `debug_runs/sim_hud_power_icons_live.json`

---

## 10. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** v1.1 | 2026-06-18 |
