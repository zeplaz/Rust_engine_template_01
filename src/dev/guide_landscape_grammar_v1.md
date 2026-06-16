# LANDSCAPE-GRAMMAR — settlement-scale vegetation as landscape program `v1`

```text
⟦SYMLANG⟧⟐v1  ◈GUIDE
⟨ID⟩ GUIDE-LANDSCAPE-GRAMMAR-001
Date: 2026-06-13
Status: **SIGNED** (charter — exec LG-0 complete 2026-06-13)
Mirror: $ref:prompts/guides/build_grammer2_exman.md · $ref:src/dev/arch_build_grammar_v0_baseline_v1.md
Operator source: $ref:prompts/guides/olant_grammer.md
Pictorial ref: $ref:prompts/guides/low_res_forest_veg.png
Lexicon: $ref:prompts/guides/landscape_grammar_lexicon_v1.md
Exec plan: $ref:src/dev/plan_landscape_grammar_exec_001_v1.md
SYMLANG: $ref:prompts/SYMBOLIC_LANGUAGE.meta.md §2.13
Parent index: $ref:src/dev/construction_procedural_growth_index_v1.md (settlement lane)
Rule: Landscape program is authority — sprites are terminal extract only
```

**Headline:** Vegetation is not a render layer on ecology. It is the **visible surface** of a **landscape program** — pressures → topology → succession → assemblage → instances → sprites.

**Rejected spine (do not build):**

```text
Biome → Tree Density → Sprites     ❌  (same failure mode as Type → Shape)
Forest / Grassland / Wetland       ❌  (descriptive labels, weak generative power)
VegetationPopulation alone         ❌  (density map without topology or memory)
```

**Target spine (mirrors buildings):**

```text
SITE◈WORLD-DNA
      ▼
LAND-DNA
      ▼
LANDSCAPE-PRESSURE-FIELD (λ)
      ▼
LANDSCAPE-PROGRAM (human + natural drivers)
      ▼
VEGETATION-TOPOLOGY-GRAPH
      ▼
SUCCESSION + DISTURBANCE-HISTORY
      ▼
VEGETATION-ASSEMBLAGE (Cx class + layers)
      ▼
POPULATION-FIELDS (subcell / tile)
      ▼
DETERMINISTIC-INSTANCES
      ▼
SPRITES / OVERLAY EXTRACT   ← terminal only
```

---

## 1. Why the earlier plan was wrong

| Building anti-pattern | Vegetation equivalent (rejected) |
|:---|:---|
| `Warehouse → Rect` | `Forest → green blob` |
| Era/style picker | Biome picker |
| One shape per type | One density per biome |
| Chunk = one massing | Chunk = uniform canopy |

**Same fix as ARCH-DNA:** function and **pressure** precede form. For landscape, **land use + hydrology + disturbance** precede **tree lines and woodlots**.

---

## 2. Repo today (honest map)

| Exists | Role | Gap |
|:---|:---|:---|
| `ChunkEcology`, `VegetationField` | CPU field sim (canopy, understory, old_growth, burn) | **Input** to grammar — not generative authority |
| `VegetationStructure`, `EcologicalSuccessionStage` | Enum labels | **States without graph or age** |
| `FloraType`, `FlowerType` | Taxonomy | No topology, no assemblage |
| `estimate_ecological_suitability()` | Biome → density scalar | **The rejected spine** |
| Ecology preview tint | Chunk color | No topology, no patch structure |
| Fire / construction | Disturbance **effects** | No **DisturbanceHistory** resource |
| Settlement / construction grammar | Site programs | **No landscape program coupling** |

**Do not** extend ecology tick alone and call it done. **Add** landscape grammar authority upstream of population fields.

---

## 3. LAND-DNA (equivalent of ARCH-DNA)

| Key | Name | Example values |
|:---:|:---|:---|
| **H** | Hydrology | Riparian · Upland · Seasonal-flood · Drainage-ditch · Pond-margin |
| **S** | Soil | Deep-alluvial · Thin-ridge · Organic-muck · Rocky · Compacted-fill |
| **E** | Exposure | Sheltered · Wind-exposed · Coastal-spray · Frost-pocket |
| **T** | Temperature | (from climate chunk — weak alone) |
| **D** | Disturbance | Fire-return · Grazing · Industrial-spill · Seasonal-flood |
| **L** | LandUse | Wild · Agricultural · Industrial · Military · Residential · Utility |
| **A** | Age | Pioneer · Young · Mature · Legacy |
| **M** | Management | Wild · Maintained · Cleared · Abandoned · Intensively-managed |

**Example assemblage (not a generator name):**

```text
H=Riparian S=Deep-alluvial E=Sheltered D=Seasonal-flood L=Agricultural A=Mature M=Maintained
  → pressures require: TreeLines · Windbreaks · Hedgerows · Riparian-corridor (topology nodes)
```

No `TreeLineGenerator` enum. Topology **emerges from** LAND-DNA + λ + operator stack.

---

## 4. Landscape pressure field (λ)

Analog of building **β pressure field**. Eight keys minimum (v0); extend in vNext.

| λ key | Drives |
|:---|:---|
| λ_moisture | Riparian bias, wetland assemblage |
| λ_slope | Ridgeline spine, gully fan |
| λ_exposure | Windbreak demand, stunted canopy |
| λ_disturbance | Clearance, regrowth rate |
| λ_access | Roadside / railside corridors |
| λ_security | Concealment belts, clearings |
| λ_productivity | Field geometry, edge trees |
| λ_legibility | Observation clearings, maintained lines |

**Source inputs (read-only):** hydrology graph, transport edges, construction sites, military zones, weather/soil fields, fire history.

---

## 5. Landscape program (missing layer)

Not a biome tag. A **spatial program** on a site or chunk partition — same conceptual tier as building **ProgramGraph** (vNext for buildings; **required v0 for landscape** at coarser grain).

| Program class | Examples |
|:---|:---|
| **Natural** | Riparian corridor · Ridgeline spine · Floodplain fan · Old-growth core |
| **Agricultural** | Field-edge trees · Windbreak · Farm woodlot · Drainage ditch margin |
| **Industrial** | Cleared zone · Utility corridor · Storm basin · Fence regrowth · Buffer strip |
| **Military** | Concealment belt · Observation clearing · Defensive tree line · Vehicle exclusion |
| **Settlement** | Street tree line · Estate ring · Park patch · Abandoned lot succession |

Programs attach to **settlement districts** and **infrastructure corridors** — not to whole chunks uniformly.

---

## 6. Vegetation topology (first-class grammar nodes)

**Law:** `VegetationTopology ≠ VegetationShape`. Topologies are **graph nodes with flow**, not noise blobs or biome labels.

Each topology carries:

```text
Origin · GrowthPressure · FlowDirection · DensityGradient · AgeGradient
     · DisturbanceHistory · Connectivity
```

**Chunk question (correct):** *What vegetation topologies intersect me?* — not *What forest type am I?*

### 6.1 Library V2 (20 base kinds)

| Kind | Chart hint | Typical anchor |
|:---|:---|:---|
| **Corridor** | `═` / `≈` flow band | River, road, rail, pipe |
| **Patch** | core `█` → edge `▒` gradient | Woodlot, park, regrowth island |
| **Ring** | `██  ██` enclosure | Shelter belt, estate, defensive perimeter |
| **Fan** | widening `▼` | Floodplain, alluvial spread, gully mouth |
| **Spine** | ridge/river axis | Ridgeline, levee crown |
| **Cluster** | irregular `◊` nuclei | Regeneration, burn mosaic |
| **Network** | `◊═◊═◊` linked patches | Ecological / fire / concealment connectivity |
| **Mosaic** | mixed `█▒▓` tiles | Real landscapes — not uniform forest |
| **Fringe** | sharp `\|` edge | Field–forest transition |
| **Pocket** | `█` in open `▒` | Internal woodlot in cleared matrix |
| **Barrier** | thick/thin block | Movement / visibility block |
| **Archipelago** | islands in open matrix | Ag steppe, dryland savanna |
| **Delta** | branching `≈╬≈` | Distributary wet spread |
| **Convergence** | many inflows → basin | Wetland collection |
| **Divergence** | radial spread from point | Disturbance / seed source |
| **Veins** | capillary fine net | Micro drainage / seed networks |
| **Basin** | bowl `▼` | Wetland expansion |
| **Crown** | hilltop cap | Exposure-limited stunted growth |
| **Ladder** | terraced bands | Agroforestry steps |
| **Nested** | topology inside topology | Region → Network → Patch → Cluster → Gap |

Full preset catalog (30 named variants with param ranges): $ref:prompts/guides/landscape_grammar_lexicon_v1.md §5 · pictorial sheet $ref:prompts/guides/low_res_forest_veg.png.

### 6.2 Patch internals (never uniform fill)

```text
Core (dense █) · Edge (▓▒ gradient) · Gap (○) · Regrowth (◇▒) · Deadfall
```

Old-growth patch uses **●** anchor nuclei inside canopy mass.

### 6.3 Per-node metadata (v0 required)

```text
topology_kind · preset_id · anchor_ref · operator_stack_id · succession_age · cx_class
  · width · curvature · branching · age · species_bias · continuity · flow_direction · scale_band
```

Ring adds pressure mode: Containment · Protection · Boundary · Visibility · Concealment.

Cluster adds: NucleusCount · Connectivity · ExpansionBias · SpeciesDiversity · AgeDiversity.

---

## 6b. Nested composition (mandatory)

Real landscapes nest topologies — same tier as building site composition:

```text
Region
 └─ Network
     └─ Patch
         └─ Cluster
             └─ Gap (○)
                 └─ Regrowth (▒)
```

**Example — agricultural mosaic (`AGRI-LANDSCAPE-Δ9`):**

```text
▒▒▒ fields · █ woodlots · ═ road spine · RING shelterbelt on field edge
```

**Example — defensive belt (`DEFENSIVE-VEGETATION-Ω13`):**

```text
▒ concealment fringe · █ canopy · ○ observation gaps · □ facility core · ⊖ suppression toward enemy vector
```

Chart catalog (planner citations): $ref:prompts/guides/landscape_grammar_lexicon_v1.md §7 · $ref:prompts/guides/olant_grammer.md.

---

## 6c. Dual symbolic lexicon (planning vs extract)

Two glyph sets — **never mixed in one layer**:

| Layer | Glyphs | Use |
|:---|:---|:---|
| **Planning / charts** (§1) | `█ ▓ ▒ ● ○ ◊ ═ ≈ ⊕ ⊖ ⊗` | Plans, solutions, SYMLANG handoffs, debug overlays |
| **Extract / encoder** (§2) | `@ # % * . , ~ ^ v = + x` | Population fields, MCP specs, deterministic tile encode |

Mapping table: $ref:prompts/guides/landscape_grammar_lexicon_v1.md §3.

---

## 6d. Field overlay layers (multi-scale)

A partition belongs to **8–20 overlapping fields** simultaneously — closer to circuit schematics than terrain paint:

```text
Canopy + Age + Hydrology + Wind + Disturbance + Species + Visibility + Fire + HumanPressure
```

| Overlay | Glyphs | Authority |
|:---|:---|:---|
| Wind domain | ⚶ | Weather / exposure λ |
| Expansion front | ⊕ | Succession graph |
| Suppression | ⊖ | Urban / clear / maintain |
| Disturbance | ⊗ | Fire / harvest events |
| Hydrology | ≈ ~ | Hydro graph |
| Habitat attractor | ⊙ | Ecology (derived) |

Reference charts: `ECOLOGICAL-PRESSURE-Ω27` · `FIRE-CORRIDOR-Ω51` · `URBAN-FRACTURE-Σ63` · `MEGA-BIOSPHERE-Ω200` — $ref:prompts/guides/olant_grammer.md.

**Scale bands:** S (1–4) micro · M (5–16) meso · L (17–64) macro · XL (65+) mega — $ref:prompts/guides/low_res_forest_veg.png.

**Context tags:** `#` biome · `@` moisture · `^` elevation · `[` land_use `]` · `!` disturbance · `=` infrastructure.

---

## 6e. Composite symbols (connecting primitives into complex charts)

Complex charts are **composed**, not invented as new Unicode. Three modes:

```text
JUXTAPOSE  mass meets mass (█▓▒░ gradients, ⇡⇣ fronts)
CONNECT    bones link nodes (◊═◊ · ≈╬≈ · ║ ╱ ╲)
STACK      fields over structure (⚶{█▓▒} · ⊕→{▒▒▒})
```

**7-step derivation:** Anchor → Bone → Mass → Field → Vector → Frame → Metadata — full rules in $ref:prompts/guides/landscape_grammar_lexicon_v1.md §1.17.

**Reusable macros:** `MACRO-RIPARIAN-AXIS` · `MACRO-WIND-ALLEY` · `MACRO-REGROWTH-CHAIN` · `MACRO-AG-PARCEL` · etc. (§1.17.4).

**Nest operators:** `A ⊃ B` contain · `A ∩ B` intersect · `A ⊣ B` block · `A → B` succession flow.

**Complexity budget:** S=2 overlays · M=4 · L=8 · XL=12+ — split charts when exceeded (§1.17.7).

**New chart protocol:** decompose → name (`◉NAME-Ωn`) → register macro if reused → preset if sim-ready (§1.19).

---

Primitives (v0 set):

| Op | Meaning |
|:---|:---|
| **⌂** Anchor | Bind to river / road / site edge |
| **═** Corridor | Extrude along polyline |
| **⊕** SeedPatch | Initialize patch interior |
| **⊖** Clear | Remove vegetation authority |
| **⇉** Expand | Grow patch edge (succession) |
| **⇇** Contract | Shrink / maintain |
| **≈** Drift | Meander / fuzzy edge (riparian) |
| **◊** Cluster | Irregular interior fill |
| **□** Ring | Closed perimeter band |
| **⟁** Branch | Fork along secondary anchors |

**Example — riparian corridor (no special-case generator):**

```text
Stack: ⌂ River · ≈ Drift · ═ Corridor · ⇉ Expand · ◊ Cluster
```

Result: tree band following water with irregular interior — **topology graph + ops**, not `if riparian`.

Operator history is stored (like building `OperatorHistory`) for replay and disturbance rewind.

---

## 8. Succession graph (memory, not enum)

Replace static `DenseForest` with **directed graph + age**.

**Natural track:**

```text
Grass → Shrub → YoungForest → MatureForest → OldGrowth
```

**Fire:**

```text
OldGrowth → BurnScar → Grass → Shrub → …
```

**Construction:**

```text
Forest → Cleared → IndustrialLot
```

**Abandonment:**

```text
IndustrialLot → Grass → Brush → Forest
```

**Authority:**

| Resource | Stores |
|:---|:---|
| `SuccessionState` | Current stage + **SuccessionAge** (time in stage) |
| `DisturbanceHistory` | Ring buffer: fire, clear, build, harvest events with tick + severity |
| `LandUseInfluence` | District program weights per partition |

`VegetationField.canopy_density` becomes a **derived display/sim scalar** from assemblage + succession — not the generative root.

---

## 9. Vegetation complexity classes (Cx0–Cx5)

Structure classes — not sprite counts.

| Class | Name | Layers |
|:---:|:---|:---|
| **Cx0** | Open ground | Ground cover only |
| **Cx1** | Scattered trees | Sparse canopy, minimal understory |
| **Cx2** | Patch forest | Closed patch, single layer |
| **Cx3** | Connected forest network | Patches linked by corridors |
| **Cx4** | Mixed forest network | Multi-species, edge + core |
| **Cx5** | Multi-layer old growth | Canopy + subcanopy + shrub + ground |

Cx5 is **vertical structure** in population fields:

```text
Canopy      ████████████
Subcanopy   ▓▓▓▓▓▓▓▓▓▓▓▓
Shrubs      ▒▒▒▒▒▒▒▒▒▒▒▒
Ground      ............
```

Sprites **sample** these layers at extract time — they do not define them.

---

## 10. Human landscape grammar (land use, not biome)

| District | Required topologies (program outputs) |
|:---|:---|
| **Agricultural** | Field-edge lines · windbreaks · drainage ditches · farm woodlot patches |
| **Industrial** | Cleared zones · utility corridors · storm basins · fence regrowth · buffer strips |
| **Military** | Concealment belts · observation clearings · defensive tree lines · exclusion zones |
| **Residential** | Street lines · estate rings · park patches · yard management |
| **Wild / reserve** | Old-growth core · regrowth cluster · riparian corridor |

Coupling: `LandUseInfluence` reads **settlement hierarchy** + **construction sites** + **transport graph** — same spine as organic growth, inverted (vegetation responds to human program).

---

## 11. World scale — one chunk, many patches

**Wrong:**

```text
Chunk → Forest → ████████████████████  (one blob)
```

**Right:**

```text
Chunk partition:
  · Forest patch A (Patch)
  · Forest patch B (Patch)
  · Drainage corridor (Corridor)
  · Clearing (Clearing)
  · Rock outcrop (Cx0)
  · Shrub band (Line)
  · Tree line (Line)
  · Old-growth core (Patch, Cx5)
```

**10× perceived detail without 10× global entities** — topology graph subdivides chunk authority (tile or subcell partitions), same as site composition subdivides building footprint.

---

## 12. Authority layers (separate writers — do not collapse)

| Layer | Single writer | Mutated by |
|:---|:---|:---|
| `LandscapeProgram` | world-gen + settlement growth | district change, zoning |
| `VegetationTopologyGraph` | grammar evaluator | program + λ |
| `SuccessionState` | ecology tick | time, disturbance |
| `DisturbanceHistory` | event bus (SimEffect / fire / construction) | fire, build, harvest, clear |
| `LandUseInfluence` | settlement + construction | commits, abandonment |
| `VegetationPopulation` | derived from above | recompute on graph change |
| `VegetationField` (existing) | ecology integrator | sim tick — **reads** population |
| Sprite instances | render extract | read-only |

⛔ ecology tick writing topology · ⛔ render writing succession · ⛔ biome tag writing program

---

## 13. Pipeline parity with buildings

| Building spine | Landscape spine |
|:---|:---|
| Economy → infrastructure | Hydrology → transport → land use |
| Site | Chunk partition / district cell |
| ARCH-DNA | LAND-DNA |
| β pressure | λ pressure |
| ProgramGraph | LandscapeProgram |
| Topology (massing strategies) | VegetationTopology nodes |
| FootprintMatrix | Population partition grid |
| Module runs | Assemblage species mix |
| WRK / ATL | Sprite atlas extract |
| Weathering | Burn / regrowth / seasonal |

---

## 14. Phasing (no empty stubs)

| Phase | Deliverable | Owner | Not |
|:---:|:---|:---|:---|
| **LG-0** | This guide + lexicon + LAND-DNA schema + T01–T20 enum + 30 preset IDs + operator catalog | @planner-mcp | Rust |
| **LG-1** | `LandscapeProgram` + topology graph on **one** agricultural test partition | @coder | Sprites |
| **LG-2** | Succession graph + `DisturbanceHistory` wired to fire + construction | @coder | Global ECS trees |
| **LG-3** | Human district programs (ag + industrial) tied to settlement | @coder + @designer | Biome rewrite |
| **LG-4** | Population field derive + ecology preview shows **patches** not blobs | @coder | Atlas |
| **LG-5** | Iso sprite extract (minimal atlas) | @designer-mcp → @coder-mcp | Per-tree sim |
| **LG-6** | Flowers aesthetic layer | @designer-mcp | defer |

**v0 pilot:** one agricultural chunk with riparian corridor + field-edge lines + windbreak ring — witness JSON, no art gate.

**vNext defer:** full operator history replay, 3D canopy, per-tree combat ECS except local tactical bubble.

---

## 15. Witness / acceptance (v0)

| Probe | Pass |
|:---|:---|
| One chunk contains ≥3 topology node kinds | patch + line + corridor |
| SuccessionAge increases on undisturbed partition | graph tick |
| Construction clear writes DisturbanceHistory + topology ⊖ | event linked |
| Fire transitions OldGrowth → BurnScar → Grass in graph | not density-only |
| Ecology preview shows **heterogeneous** partition tints | not uniform green |
| No global Tree entity count scales with map area | instance extract bounded |

---

## 16. Queue hook (when signed)

Program ID: **PLAN-LANDSCAPE-GRAMMAR-001**  
Exec doc: [`plan_landscape_grammar_exec_001_v1.md`](plan_landscape_grammar_exec_001_v1.md)  
Lexicon: [`prompts/guides/landscape_grammar_lexicon_v1.md`](../prompts/guides/landscape_grammar_lexicon_v1.md)  
Sources: [`olant_grammer.md`](../prompts/guides/olant_grammer.md) · [`low_res_forest_veg.png`](../prompts/guides/low_res_forest_veg.png)  
Index row: `construction_procedural_growth_index_v1.md` + `development_plan_index.md`

**Do not queue:** "implement 13 empty stub rs files" or "VegetationPopulation-only" slice.

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-13 | Charter from operator critique — landscape program over ecology render |
| v1.1.0 | 2026-06-13 | Integrated olant_grammer + pictorial v2.3 — V2 topology library, dual lexicon, nested + field overlays |
| v1.2.0 | 2026-06-13 | Schema v1.2 sync — field_overlay_stack · composition_macros · chart_id enum · ◊A…I · dual-layer tags |
| v1.2.0 | 2026-06-13 | Complete olant_grammer symbol audit — §1.0–§1.10, all 15 chart IDs, framing glyphs, disambiguation |
| v1.3.0 | 2026-06-13 | §1.11–§1.16 semantic operator deep ref (⚶☍⊙⊕⊖⊗⌂◇), composition recipes, field matrix |
| v1.4.0 | 2026-06-13 | §1.17–§1.19 composite & connection grammar — macros, stacks, derivation pipeline |

```text
⟦/GUIDE-LANDSCAPE-GRAMMAR-001⟧  ΔWF→@planner-mcp LG-0 schema · @planner sign exec
```
