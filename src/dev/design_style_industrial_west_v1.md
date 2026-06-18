# Industrial West style bible `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-STYLE-INDUSTRIAL-WEST-001** |
| **Program** | Style concepts · Track C1 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Authority** | [`style_industrial_west.ron`](../../assets/configs/buildings/style_packs/style_industrial_west.ron) · [`design_kit_production_002_concept_v1.md`](design_kit_production_002_concept_v1.md) |
| **Unblocks** | DMCP-MODULE-KIT002-001 G4 · `IndustrialWarehouse` · `FactoryCluster` · utility bible |
| **Verdict** | **PASS** |

```text
DES-STYLE-INDUSTRIAL-WEST-001 Q✓
Massing · roof · bay rhythm · palette · weathering — concept sheet + 3 refs
```

---

## 0. Lineage

**Industrial West** = North American / European **early–mid 20th c.** factory + warehouse read — corrugated steel, sawtooth clerestories, roll-up doors, honest weathering.

**Grammars:** `IndustrialWarehouse` · `FactoryCluster` · `RailEdge` share one kit.  
**Utility grid** borrows palette only — less ornament ([`design_utility_industrial_style_v1.md`](design_utility_industrial_style_v1.md)).

---

## 1. Massing

| Strategy | Silhouette | Width:depth | Roof |
|:---|:---|:---:|:---|
| `long_hall` | Single bar + yard | 1.6–2.0 | sawtooth or gable |
| `double_hall` | Twin parallel halls | 1.2–1.5 | dual sawtooth |
| `yard_complex` | U or L around yard | — | mixed shed |
| `l_shape` | Office wing + hall | — | lower office flat roof |

**Height:** 1–2 floors · hall ≤ 12m equivalent · office wing ≤ 8m.

**@ 64px iso:** long low mass + **roof signature** must read before wall detail.

---

## 2. Roof language

| Type | Module | Read |
|:---|:---|:---|
| **Sawtooth** | `roof_industrial_shed_2u` | Primary signature — north-facing glazed slope |
| **Low metal** | `roof_metal_low` | Office / annex |
| **Flat tar** | rare — service wings only | |

**Rule:** ≥60% roof area uses sawtooth on warehouse/factory massing strategies.

---

## 3. Door / window rhythm

| Element | Module | Rhythm |
|:---|:---|:---:|
| Roll-up door | `door_warehouse` | every **3–4** bays on ground |
| Clerestory band | `win_industrial_3u` | every **2** bays · mid-wall |
| Steel bay wall | `wall_steel_1u` | **1u** repeat — defines width grid |
| Corner | `corner_L` | every footprint turn |

```text
┌──┬──┬──┬──┬──┐
│▓▓│░░│▓▓│░░│▓▓│  ▓ = 3u window band
│  │██│  │██│  │  ██ = roll-up door
└──┴──┴──┴──┴──┘
```

**Ban:** residential bay proportions · arched entries · brick primary façade (brick = accent band only).

---

## 4. Palette

| Token | Hex | Use |
|:---|:---|:---|
| `steel_cool` | `#8a9098` | corrugated wall |
| `steel_warm` | `#7a7570` | weathered panel |
| `roof_galvanized` | `#a8b0b8` | sawtooth metal |
| `glass_clerestory` | `#6a8aa0` @ 40% | window band |
| `door_charcoal` | `#3a3a3a` | roll-up |
| `concrete_base` | `#8a8884` | footing / yard |
| `rust_accent` | `#8a4a30` | **≤10%** surface — streaks only |
| `night_glow` | `#e8c878` | window emissive night_on |

**Material profiles:** `steel_panel_01` · `roof_metal_01` · `glass_panel_01` — no new ids in G1.

---

## 5. Weathering

| Tier | Surface | APS variant |
|:---|:---|:---|
| **Clean** | Factory-new steel, crisp edges | `clean` |
| **Production default** | Light grime, edge wear | `dirty` |
| **Damaged** | Dent, panel replacement, broken glass | `damaged` |
| **Ruined** | Collapsed bay, open frame | `ruined` |

**Weathering never obscures** bay rhythm @ tactical zoom.

---

## 6. Factory vs warehouse (same kit)

| Grammar | Emphasis | Clutter |
|:---|:---|:---|
| `IndustrialWarehouse` | Single hall + yard door | low |
| `FactoryCluster` | `double_hall` + stacks | `stack_chimney_1u` ×2–4 |
| `RailEdge` | Loading dock + rail spur | `prop_vent` · signs |

---

## 7. Concept sheet + references

**Concept sheet:** `assets/reference/style/industrial_west_concept_sheet_v1.png` (layout brief — coder-mcp export from staging stills)

| Ref # | Subject | Source |
|:---:|:---|:---|
| **R1** | Long sawtooth hall + roll-up rhythm | kit002 `long_hall` keyframe |
| **R2** | Twin-bay factory + chimney clutter | `FactoryCluster` grammar still |
| **R3** | Night windows + yard flood | `clean_night_on` variant cell |

**Until G4 stills ship:** use [`design_kit_production_002_concept_v1.md`](design_kit_production_002_concept_v1.md) §1 wire + roof bpy witness.

---

## 8. Iso readability (@ 64px)

| Must read | Test |
|:---|:---|
| Sawtooth teeth | 3+ teeth visible on default warehouse |
| Door bay | ≥1 dark door slot on ground row |
| Stack (factory) | vertical punctuation above roofline |

---

## 9. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
