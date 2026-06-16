# VEG/LAND — landscape grammar symbolic lexicon `v1`

```text
⟦SYMLANG⟧⟐v1  ◈LEXICON
⟨ID⟩ VEG-LAND-LEXICON-001
Version: v1.4.0 (composite & connection grammar — building complex symbols)
Sources: $ref:prompts/guides/olant_grammer.md · $ref:prompts/guides/low_res_forest_veg.png
Charter: $ref:src/dev/guide_landscape_grammar_v1.md
SYMLANG hook: $ref:prompts/SYMBOLIC_LANGUAGE.meta.md §2.13
Rule: Planning charts use §1 glyphs · extract/encoder uses §2 glyphs · never mix in one layer
```

**Headline:** Two complementary glyph sets serve the same topology grammar — **operational chart symbols** (plans, solutions, agent handoffs) and **extract encoder symbols** (deterministic tile/subcell population). Topology is authority; glyphs are views.

---

## §0 Core law

```text
VegetationTopology ≠ VegetationShape

Topology carries: Origin · GrowthPressure · FlowDirection · DensityGradient · AgeGradient
                 · DisturbanceHistory · Connectivity

Rendered forest = visible consequence of intersecting topologies + field overlays
```

**Chunk question (correct):**

```text
"What vegetation topologies intersect me?"
  NOT "What forest type am I?"
```

Real landscapes = overlapping systems (riparian corridor + old-growth patch + regrowth cluster + windbreak + roadside spine + drainage fan).

---

## §1 Planning / chart glyphs (operational maps)

From `olant_grammer.md` — **complete authoritative set**. Use in plans, solutions, SYMLANG charts, witness sketches.

### §1.0 Authoritative legend (verbatim semantics from operator source)

```text
█ = mature canopy          ▓ = secondary canopy       ▒ = shrub                 ░ = grass
● = old growth anchor      ○ = disturbance scar       ◊ = topology node         □ = protected space
◇ = regeneration nucleus   ≈ = hydrology              ═ = transport             ╬ = convergence
▲ = elevation source       ▼ = flow sink              ⊕ = expansion pressure    ⊖ = suppression pressure
⊗ = disturbance event      ⊙ = ecological attractor   ☍ = ecological barrier    ⚶ = wind influence
⌂ = human management
```

**Disambiguation (same glyph, context-dependent):**

| Glyph | Primary | Alternate context |
|:---:|:---|:---|
| **▲** | Elevation source / ridge head | Succession ladder: sapling stage (only inside `NESTED-SUCCESSION-Ψ18` cascade blocks) |
| **~** | Wind-domain wave front (planning charts) | Extract layer: water/wet — **never** use planning `~` in encoder fields |
| **═** | Heavy transport spine (road / rail) | — |
| **─** | Thin boundary / observation line / weak edge | Not transport — see §1.5 |
| **●** | Old-growth anchor (canopy) | SYMLANG §2.11: confidence `●` = certain — **do not** mix VEG/LAND charts with confidence clusters on same line |

### §1.1 Canopy & structure

| Glyph | Meaning | Layer |
|:---:|:---|:---|
| **█** | Mature canopy mass | Canopy |
| **▓** | Secondary canopy / mid-story | Subcanopy |
| **▒** | Shrub / brush | Understory |
| **░** | Grass / open ground | Ground |
| **●** | Old-growth anchor nucleus | Canopy (legacy) |
| **○** | Disturbance scar / gap | Void + regrowth seed |
| **◇** | Regeneration nucleus | Succession front |
| **□** | Protected / cleared human space | Land-use void |

**Patch density zones (always model all three — never uniform █ block):**

```text
Core  ████████████   (dense)
Mid   ▓▓▓▓▓▓▓▓▓▓▓▓   (secondary — between core and edge)
Edge  ▒▒▒▒▒▒▒▒▒▒▒▒   (sparse fringe)
```

**Patch internals (structural states — no single glyph for deadfall; tag as `deadfall` metadata):**

```text
Core · Edge · Gap · Regrowth · Deadfall
```

### §1.2 Topology & flow

| Glyph | Meaning |
|:---:|:---|
| **◊** | Topology anchor node (may suffix: `◊A`…`◊I` — see §1.8) |
| **═** | Heavy transport corridor (road / rail / pipe spine) |
| **─** | Thin line — observation line, weak fence, non-transport boundary |
| **≈** | Hydrology axis (river / wet / marsh / riparian) |
| **╬** | Convergence / junction / distributary split |
| **▲** | Elevation source (ridge / mountain head) |
| **▼** | Flow sink (basin mouth / delta / drainage terminus) |
| **│** | Vertical flow connector (source→sink in corridor charts) |
| **║** | Network linkage (vertical arms between corridor nodes) |
| **⌂** | Human management anchor |
| **⚶** | Wind influence domain / wind pressure front |
| **~** | Wind-domain wave front (planning only — see §1.0) |
| **☍** | Ecological barrier field |

**Corridor subtypes (same `═`/`─`/`≈` bones, different metadata):**

```text
Riparian   ≈≈≈≈≈ + ███▓▓▓▒▒▒▒▒▒▓▓▓█████  (moist bias, drift)
Rail       ═══════ + ███▓▓▓███▓▓▓███▓▓▓███  (edge species, disturbance high)
Roadside   ────── + ▒▒▓▓████▓▓▒▒▒▓▓████▓▓  (hardy edge, thin fringe)
```

**Corridor metadata fields:** Width · Curvature · Branching · Age · SpeciesBias · Continuity · FlowDirection

### §1.3 Pressure, disturbance & species fronts

| Glyph | Meaning |
|:---:|:---|
| **⊕** | Expansion pressure (succession front / regrowth advance) |
| **⊖** | Suppression pressure (urban / clear / maintain / observation) |
| **⊗** | Disturbance event (fire / harvest / clear scar) |
| **⊙** | Ecological attractor / habitat nucleus |
| **⇡** | Species or succession advance vector (north / upslope / invading) |
| **⇣** | Species or succession retreat vector (south / downslope / losing) |

**Stacked intensity (repeat glyph = stronger field):**

```text
▲▲▲▲▲   elevation head intensity
⚶⚶⚶⚶   wind pressure front width
⊕⊕⊕⊕   expansion front strength
⊖⊖⊖⊖   suppression field strength
⊗⊗⊗⊗   disturbance cluster / fire vector
```

**Directional annotation (prose suffix on charts):**

```text
Wind →          enemy vector ▼          upstream ▲          fire vector ▲
```

### §1.4 Chart framing & graph edges

Box and junction drawing used in operational maps (`AGRI-LANDSCAPE-Δ9`, `URBAN-FRACTURE-Σ63`, `ECOLOGICAL-NETWORK-Ω91`):

| Glyph | Meaning |
|:---:|:---|
| **╔ ╗ ╚ ╝** | Chart / parcel frame corners |
| **║** | Frame vertical edge (also network arm — context) |
| **╦ ╩** | T-junction — urban grid fracture, transport crossing |
| **╱ ╲** | Diagonal ecological graph edge |
| **│** | Tree / flow vertical in graph layouts |

**Land-use shorthand (AGRI-LANDSCAPE-Δ9 legend):**

```text
FIELDS = ▒     WOODLOTS = █     TRANSPORT = ═
```

### §1.5 Chart section markers & ID suffixes

| Pattern | Meaning |
|:---|:---|
| **◉** | Chart / section header prefix (e.g. `◉VEG-NETWORK-Ω7`) |
| **Ω** | Network / interaction / warfare chart family |
| **Σ** | Complex / fracture / composite chart family |
| **Δ** | Drainage / delta / agricultural mosaic family |
| **Λ** | Delta / distributary fan family |
| **Ψ** | Regeneration / succession cascade family |
| **Ξ** | Hierarchy / nesting chart family |

**Named chart IDs (complete catalog from operator source):**

| Chart ID | Subject |
|:---|:---|
| `VEG-NETWORK-Ω7` | Network + ◊ anchors + ● nuclei + ○ gaps + ╬ junctions |
| `OLD-GROWTH-COMPLEX-Σ4` | ▲ ridge spine + patch + regrowth fan + ≈ riparian |
| `AGRI-LANDSCAPE-Δ9` | ▒ fields + █ woodlots + ═ transport (framed parcel) |
| `VEGETATION-HIERARCHY-Ξ12` | Region → Network → Patch → Gap → regrowth |
| `DELTA-FOREST-Λ5` | Upstream ≈ + ╬ split + FAN-A/B/C + ◊ nodes |
| `DEFENSIVE-VEGETATION-Ω13` | ▒ concealment + █ canopy + ○ gaps + ─ observation + □ facility |
| `RIPARIAN-REGENERATION-Ψ8` | ⚶ wind + ≈ hydrology + ◇ regrowth + ⊕ expansion |
| `ECOLOGICAL-PRESSURE-Ω27` | ⚶ wind front + ● canopy + ⊕ regrowth + ≈ hydrology + ▒ regrowth front |
| `NESTED-SUCCESSION-Ψ18` | Region → Network → Patch → Cluster → Gap → Regrowth → ▓ shrub → ▲ sapling → █ canopy |
| `MOUNTAIN-FOREST-DRAINAGE-Δ44` | ▲▲▲ head + ⚶ + spine descent + ╬ stages + ≈ wetland basin ▼ |
| `FIRE-CORRIDOR-Ω51` | Canopy mass + ╬ fire corridor + ⊗ vector ▲ |
| `URBAN-FOREST-FRACTURE-Σ63` | Forest grid + ╦╩ urban blocks + □□ + ⊖ suppression field |
| `ECOLOGICAL-NETWORK-Ω91` | ◊A…I weighted graph + ╱╲ edges + ╬ junctions |
| `FOREST-WARFARE-Ω113` | Competing canopy blocks + ⇣⇣ retreat + ⇡⇡ advance front |
| `MEGA-BIOSPHERE-Ω200` | ⚶ wind domain ~~~~ + ◊ network + ═ transport + □ occupation + field stack legend |

### §1.6 ◊ node semantic weights (`ECOLOGICAL-NETWORK-Ω91`)

When suffixing anchor nodes, these are the canonical semantics:

| Node | Semantics |
|:---:|:---|
| **◊A** | Old growth |
| **◊B** | Wetland |
| **◊C** | Regrowth |
| **◊D** | Shelter belt |
| **◊E** | Corridor junction |
| **◊F** | Disturbance patch |
| **◊G** | River edge |
| **◊H** | Core habitat |
| **◊I** | Succession front |

### §1.7 Succession cascade ladder (`NESTED-SUCCESSION-Ψ18`)

Vertical stage markers inside nested charts:

```text
GAP          ○
REGENERATION ▒▒▒
SHRUB        ▓
SAPLING      ▲        ← succession stage ONLY in this ladder context
CANOPY       █
```

### §1.8 Overlapping field layers (8–20 simultaneous)

A partition is **not** one forest blob — it is the sum of:

```text
CanopyField + AgeField + HydrologyField + WindField + DisturbanceField
  + SpeciesField + VisibilityField + FireField + HumanPressureField
```

| Field | Primary glyphs |
|:---|:---|
| Canopy | █ ▓ |
| Age | ● ◇ gradient |
| Hydrology | ≈ |
| Wind | ⚶ ~ |
| Disturbance | ⊗ ○ |
| Species | ⇡ ⇣ |
| Visibility | ⊖ ─ (clear sight lines) |
| Fire | ⊗ + corridor ╬ |
| Human pressure | ⌂ ⊖ □ |

**Mega-scale overlay legend (`MEGA-BIOSPHERE-Ω200`):**

```text
⊕ Succession expansion    ⊖ Urban suppression    ⊗ Fire disturbance
⊙ Habitat attractor       ☍ Barrier field        ⚶ Wind domain
≈ Hydrology network       ◊ Ecological node      □ Human occupation
```

### §1.9 Mosaic composition (percentages, not single type)

Single chunk may mix (example from operator source):

```text
40% Mature · 20% Young · 15% Shrub · 15% Wetland · 10% Open
```

Encode as `MOSAIC_MIXED` preset + `%` breakdown metadata — not a biome tag.

### §1.10 Density shorthand (ASCII mass blocks)

```text
Dense   ██████████   (~100%)
Medium  ▓▓▓▓▓▓▓▓▓▓   (~75%)
Sparse  ▒▒▒▒▒▒▒▒▒▒   (~50%)
Open    ............   (~10% — also ░ for grass ground)
```

Fan/spine density decreases with distance from origin: Dense █ → Medium ▓ → Sparse ▒.

### §1.11 Semantic operators — deep reference (⚶ ☍ ⊙ ⊕ ⊖ ⊗ ⌂ ◇)

These glyphs are **field semantics**, not canopy mass. They answer *why* vegetation sits where it does and *what forces* act on it. Each has a distinct role — do not interchange.

**Quick disambiguation (common confusions):**

| Glyph | IS | IS NOT |
|:---:|:---|:---|
| **●** | Old-growth structural anchor | ◊ graph node · ⊙ habitat attractor |
| **◊** | Topology graph anchor (place in network) | ● legacy tree · geographic coordinate |
| **⊙** | Ecological attractor (species/habitat pull) | ● single veteran tree · ◊ junction |
| **◇** | Regeneration nucleus (succession seed) | ○ disturbance void · ⊕ expansion front |
| **○** | Disturbance scar / gap (absence) | ◇ active regrowth seed |
| **⊕** (field) | Expansion / succession pressure front | Operator-stack seed (see §9 note) |
| **⊖** (field) | Suppression / exclusion field | ☍ hard ecological barrier · ⊖ operator clear |
| **☍** | Ecological barrier (movement block) | ⊖ human/urban suppression |
| **⚶** | Wind influence / exposure domain | ≈ hydrology · weather emoji |
| **⌂** | Human management anchor | □ cleared footprint · built `[` |
| **⊗** | Disturbance event (fire/harvest scar) | Fire corridor `═╬` alone without event |

#### ⚶ — wind influence / wind domain

| Property | Value |
|:---|:---|
| **Unicode** | U+26B6 (Mercury sign — reused as wind marker in this grammar) |
| **Reads as** | Wind exposure, fetch direction, shelter demand, stunting pressure |
| **Single `⚶`** | Local wind-bias point — edge exposure, gap venturi, ridge slot |
| **Row `⚶⚶⚶⚶…`** | Wind pressure **front** — full-width domain band (`ECOLOGICAL-PRESSURE-Ω27`, `MOUNTAIN-FOREST-DRAINAGE-Δ44`) |
| **Corridor pair `⚶═══╬═══⚶`** | Wind channeled along transport/hydro axis (`RIPARIAN-REGENERATION-Ψ8`) |
| **Label `⚶ WIND DOMAIN`** | Named mega-field header + optional `~~~~` wave underline (`MEGA-BIOSPHERE-Ω200`) |
| **Drives in sim** | Shelterbelt demand · crown stunting · edge-hardy species · fire spread vector · snow drift · seed dispersal bias |
| **Pairs with** | `▲▲▲` exposure head · `RING_SHELTERBELT` · `CROWN_HILLTOP` · `BARRIER_THIN` · `⇣` retreat downslope |
| **Encode metadata** | `wind_exposure: 0..1` · `fetch_direction: deg` · `shelter_demand: low\|med\|high` |
| **Extract layer** | No single char — derive from elevation + weather field; tag `!wind` modifier |

#### ☍ — ecological barrier field

| Property | Value |
|:---|:---|
| **Unicode** | U+261D + combining (standard **☍** conflict/sign; in SYMLANG §2.4 also = authority conflict — **scope by chart block**) |
| **Reads as** | Hard ecological edge — species/wildlife movement blocked |
| **Use when** | Cliff ecotone · salinity barrier · industrial dead zone · wide water gap · dense impervious edge |
| **Not** | Urban clear (`⊖`) · thin fence (`─`) · transport (`═`) |
| **Drives in sim** | Corridor termination · genetic isolation · fire break (if combined with low fuel) · visibility block |
| **Pairs with** | `BARRIER_THICK` · `☍` row flanking `□` · dead `░░` strip |
| **SYMLANG note** | In agent routing charts ☍ = conflict; in VEG/LAND charts ☍ = **barrier field** — disambiguate via `⟨VEG⟩` block header |

#### ⊙ — ecological attractor / habitat nucleus

| Property | Value |
|:---|:---|
| **Unicode** | U+2299 (circled dot operator) |
| **Reads as** | Biotic pull — preferred habitat, nesting core, seed source pool |
| **Use when** | Old-growth interior (habitat function) · wetland core · protected reserve focus · riparian oxbow |
| **vs ●** | ● = **structural** veteran anchor visible in canopy; ⊙ = **functional** attractor (may have no single hero tree) |
| **Drives in sim** | Wildlife density · rare species weight · regrowth seed rain · fire refugia priority |
| **Pairs with** | `◊H` Core Habitat · `BASIN_WETLAND` · `POCKET_INTERNAL` · `⊙` in MEGA legend |
| **Encode metadata** | `habitat_weight: 0..1` · `refugia: bool` · `seed_rain_radius: cells` |

#### ⊕ — expansion pressure (succession front)

| Property | Value |
|:---|:---|
| **Unicode** | U+2295 (circled plus) |
| **Reads as** | Active outward push — regrowth, invasion, succession advance |
| **Single / row `⊕⊕⊕⊕`** | Front strength / width (`ECOLOGICAL-PRESSURE-Ω27`, `RIPARIAN-REGENERATION-Ψ8`) |
| **Drives in sim** | Shrub→forest edge advance · post-fire regrowth rate · species invasion · grassland encroachment on field |
| **Pairs with** | `◇` nuclei · `⇡` advance vectors · `▒` regrowth mass · `⊖` front collision (stall line) |
| **Operator-stack collision** | Building grammar uses **⊕ AddVolume**; landscape **operator stack §9** uses ⊕ as **SeedPatch**. In **field charts** ⊕ always means **expansion pressure**. Tag context: `field:⊕` vs `op:⊕` |
| **Extract mapping** | `>` spread direction |

#### ⊖ — suppression pressure

| Property | Value |
|:---|:---|
| **Unicode** | U+2296 (circled minus) |
| **Reads as** | Active inward push against vegetation — maintain, exclude, urban erase |
| **Row `⊖⊖⊖⊖…`** | Suppression **field** (`URBAN-FOREST-FRACTURE-Σ63` urban strip) |
| **Use when** | Mowing · grazing pressure · urban heat · maintenance · military clear zone · observation requirement |
| **Drives in sim** | Caps succession · prevents regrowth · lowers fuel · increases visibility · pushes forest edge back |
| **Pairs with** | `□` occupation · `─` observation line · `⇣` losing species · transport grid `╦╩` |
| **Operator-stack collision** | §9 ⊖ = **Clear** operator — same glyph, different layer: `field:⊖` vs `op:⊖` |
| **Extract mapping** | `<` suppression gradient |

#### ⊗ — disturbance event

| Property | Value |
|:---|:---|
| **Unicode** | U+2297 (circled times) |
| **Reads as** | Discrete scar event — fire pass, harvest, blowdown, construction scrape |
| **Cluster `⊗⊗⊗⊗`** | Fire vector / repeated disturbance (`FIRE-CORRIDOR-Ω51`) |
| **Drives in sim** | Resets succession · creates ○ gaps · opens fire corridor if aligned with `╬` · triggers regrowth ◇ |
| **Pairs with** | `○` scar · `⇡` pioneer advance into scar · `⊕` regrowth front trailing · `!` disturbance tag |
| **Operator-stack collision** | Building grammar **⊗ Merge** — never merge meanings in one chart line |
| **Extract mapping** | `x` + `!fire` / `!harvest` modifier |

#### ⌂ — human management anchor

| Property | Value |
|:---|:---|
| **Unicode** | U+2302 (house / home outline) |
| **Reads as** | Active human intent at a point — farm, forestry office, park maintenance, military land mgmt |
| **Use when** | Deciding maintained vs wild edge · scheduled harvest · planting campaign · fire suppression policy |
| **Drives in sim** | Overrides natural succession locally · schedules ⊖ strips · places `□` · sets `M` in LAND-DNA |
| **Pairs with** | `⌂` on `═` roadside · ag parcel frame · `RING_SHELTERBELT` · `LADDER_TERRACE` |
| **Not** | Built footprint (`□` or `[`) — ⌂ is the **management authority**, not the building |

#### ◇ — regeneration nucleus

| Property | Value |
|:---|:---|
| **Unicode** | U+25C7 (white diamond) |
| **Reads as** | Active regrowth seed cluster — younger than surrounding mass |
| **Pattern `◇▒▒▒◇`** | String of nuclei along regrowth front (`RIPARIAN-REGENERATION-Ψ8`) |
| **Drives in sim** | Patch reformation after ○ · edge infill · post-⊗ pioneer islands |
| **Pairs with** | `⊕` expansion · `▒`/`▓`/`█` succession ladder · `CLUSTER_NATURAL` |
| **vs ○** | ○ = empty; ◇ = occupied regrowth starter |

---

### §1.12 Composition grammar — how to combine glyphs

**Overlapping systems (use `+` in prose, not in encoder):**

```text
Riparian Corridor + Old Growth Patch + Regrowth Cluster + Agricultural Windbreak + Roadside Spine + Drainage Fan
```

**Reading order (top → bottom of chart):**

```text
1. Domain headers (⚶ WIND DOMAIN · ≈ Hydrology Network · ═ TRANSPORT)
2. Frame / parcel (╔╗╚╝)
3. Heavy structure (█ ▓ ▒ ░ mass)
4. Topology bones (◊ ═ ≈ ╬ ║)
5. Anchors & nuclei (● ◇ ○ □)
6. Pressure fronts (⊕ ⊖ ⊗ rows)
7. Vector annotations (⇡ ⇣ ▲ ▼ →)
8. Legend block
```

**Common composition recipes:**

| Scenario | Glyph stack (planning) |
|:---|:---|
| **Ag riparian woodlot** | `≈` axis · `⌂` farm · `█` woodlot · `▒` field · `═` road · `RING` via `□` frame |
| **Shelterbelt** | `Wind →` · `⚶` front · `▒` or `█` ring band · `▒` field interior |
| **Military concealment** | `enemy ▼` · `▒` fringe · `█` belt · `○` gaps · `─` observation · `□` core |
| **Post-fire regrowth** | `⊗` scar row · `○` gaps · `◇▒▒▒` nuclei · `⊕⊕` front · trailing `▓`/`█` |
| **Old-growth complex** | `▲RIDGE-SPINE▲` · `●` anchors in `█` · `◊A` node · `≈` riparian `▼` · regrowth fan `▒` |
| **Urban fracture** | `█` grid · `╦╩` blocks · `□□□` · `⊖⊖` suppression row |
| **Species warfare** | two `█` blocks · `⇣⇣` retreat · `══` stalemate line · `⇡⇡` advance into `▒` |

**Intensity scaling (repeat count → metadata):**

| Repeat | Interpretation |
|:---|:---|
| ×1 | Local / weak |
| ×2–4 | Moderate front |
| ×5–8 | Strong domain |
| ×9+ | Dominant mega-field (XL scale) |

---

### §1.13 Field interaction matrix (what amplifies / cancels)

| Field A | Field B | Interaction |
|:---|:---|:---|
| **⚶** wind | **▲** ridge | Amplifies exposure → `CROWN_HILLTOP` stunting |
| **⚶** wind | **RING** shelterbelt | Barrier reduces ⚶ to `▒` leeward zone |
| **≈** hydrology | **⊕** expansion | Riparian ⊕ faster regrowth (`CORRIDOR_RIPARIAN`) |
| **≈** hydrology | **⊗** fire | Wet brake — unless drought modifier |
| **⊕** expansion | **⊖** suppression | Front stalls at collision line (`FOREST-WARFARE-Ω113` stalemate `══`) |
| **⊗** disturbance | **◇** nucleus | Scar seeds ◇ — regrowth in 1–3 succession ticks |
| **☍** barrier | **NETWORK** | Cuts connectivity — isolates `◊` nodes |
| **⊙** attractor | **☍** barrier | Refugia trapped — high conservation value |
| **⌂** management | **⊖** suppression | Maintained clear — permanent `▒` or `,` |
| **⌂** management | **⊕** expansion | Planted windbreak — forced `RING` |
| **═** transport | **CORRIDOR** roadside | Edge hardy `▓`/`▒` — disturbance-tolerant |
| **□** urban | **⊖** field | Urban fracture (`URBAN-FOREST-FRACTURE-Σ63`) |
| **⇡** species | **⇣** species | Warfare front — mosaic boundary |
| **●** old growth | **⊗** fire | High severity — long recovery; may leave `○` |

---

### §1.14 Network topology outputs (why NETWORK matters)

When `NETWORK_CONNECTED` or `NETWORK_DENDRITIC` resolves, these **functional products** emerge (tag in metadata):

```text
Animal Corridors · Fire Corridors · Concealment Routes · Ecological Connectivity
```

Glyph signature: `████══╬══████` with `║` arms — see `VEG-NETWORK-Ω7`.

---

### §1.15 Relation & hierarchy notation (non-tile)

| Symbol | Role |
|:---:|:---|
| **≠** | Topology ≠ Shape (core law) |
| **+** | Overlapping system composition (prose) |
| **→** | Direction annotation (wind, enemy, flow) |
| **├ └ │** | Tree hierarchy in topology library listings |
| **▼** (alone on line) | "read below" / cascade to next nested level (`VEGETATION-HIERARCHY-Ξ12`) |

---

### §1.16 Master symbol index (planning layer — copy reference)

| Glyph | Name | § |
|:---:|:---|:---:|
| █ ▓ ▒ ░ | Canopy mass bands | §1.1 |
| ● ○ ◇ □ | Anchors & voids | §1.1 |
| ◊ | Topology node | §1.2 · §1.6 |
| ═ ─ ≈ | Transport / weak / hydro | §1.2 |
| ╬ ║ │ ╦ ╩ ╱ ╲ | Junctions & graph | §1.2 · §1.4 |
| ▲ ▼ | Source / sink | §1.2 |
| ⚶ ~ | Wind domain | §1.2 · §1.11 |
| ⊕ ⊖ ⊗ | Pressure & disturbance | §1.3 · §1.11 |
| ⊙ ☍ ⌂ | Attractor / barrier / management | §1.11 |
| ⇡ ⇣ | Species fronts | §1.3 |
| ◉ Ω Σ Δ Λ Ψ Ξ | Chart IDs | §1.5 |
| ╔ ╗ ╚ ╝ | Parcel frame | §1.4 |
| → + ≠ ├ └ | Composition notation | §1.15 |
| **composite** | Built from §1.17 rules | §1.17–§1.19 |

---

### §1.17 Composite & connection grammar — building complex symbols

Complex landscape charts are **not new glyphs** — they are **composed expressions** over the primitive alphabet (§1.0). Three composition modes:

```text
MODE-1  JUXTAPOSE   place glyphs adjacent in space (mass meets mass)
MODE-2  CONNECT     join glyphs with edge bones (═ ≈ ║ │ ╬ ╱ ╲)
MODE-3  STACK       overlay field operators on structure (⚶ ⊕ ⊖ ⊗ on top of █ ◊)
```

**Law:** A composite is valid when each glyph keeps its **layer** (structure · bone · field · annotation). Invalid when layers collapse (e.g. `⊕` drawn inside `█` as if it were canopy texture).

#### §1.17.1 Connection edges (bone grammar)

Edges carry **flow** between nodes. The edge glyph tells you *what kind of linkage*:

| Edge pattern | Reads as | Example composite |
|:---|:---|:---|
| **A═B** | Transport link | `◊A═══════◊B` roadside network arm |
| **A≈B** | Hydrology link | `≈≈≈≈╬≈≈≈≈` riparian split |
| **A╬B** | Convergence / merge | `████══╬══████` patch junction |
| **A║B** | Vertical network arm | `◊` down `║` to lower corridor |
| **A│B** | Source-feed vertical | `▲│▼` elevation→sink feed |
| **A╱B · A╲B** | Diagonal ecological tie | `◊A ╱│╲ ◊B` (Ω-91 graph) |
| **A─B** | Weak / sight / fence tie | `○ ─ ─ □` observation gap |
| **A╦B · A╩B** | Grid crossing | urban `╦` over forest `═` |

**Guarded connection (optional condition on edge):**

Borrow SYMLANG edge guards — attach `[cond]` on the bone when linkage is conditional:

```text
◊A ═[wet]≈ ◊B        corridor exists only when λ_moisture > 0.6
◊C ═[maintained]─ ◊D   weak edge only under ⌂ management
◊E ╬[fire_active] ◊F   fire corridor open only during ⊗ event
```

Encode as metadata: `"edge": { "from": "A", "to": "B", "kind": "hydro", "guard": "wet" }`.

#### §1.17.2 Juxtaposition rules (mass meets mass)

When canopy mass glyphs touch, read **gradient transitions** left→right or core→edge:

| Pattern | Composite meaning |
|:---|:---|
| **█▓▒░** | Single patch density gradient (core→edge→open) |
| **█▓▒▒▓█** | Patch with mid-story dip — gully or trail |
| **███●███** | Mature mass with old-growth anchor embedded |
| **███○███** | Canopy with disturbance gap — regrowth pending |
| **▒▒██▒▒** | Field with woodlot island (mosaic without explicit frame) |
| **█⇣⇣▒** | Retreat front — losing canopy → shrub |
| **▒⇡⇡█** | Advance front — shrub → invading canopy |

**Invalid juxtaposition:** `█⊕█` — do not embed field operator inside mass; stack as two layers (§1.17.3).

#### §1.17.3 Stack notation (field over structure)

Stack fields **above** structure using a vertical layer block (top = dominant overlay):

```text
⟨STACK⟩  top → bottom
  ⚶⚶⚶⚶          wind domain
  ⊕⊕⊕⊕          expansion front
  ████████        canopy structure
  ≈≈≈≈≈≈          hydrology axis
  ◊───◊           topology bones
```

**Stack shorthand in inline charts:**

```text
⚶{█●█}     wind acting on anchored canopy
⊖{▒▒▒}     suppression over open field
⊗→{██}     disturbance vector into canopy
⌂⟨═⟩       management bound to transport
◇⊂{▒▒▒}    regrowth nuclei inside shrub field
```

**Reading rule:** `{inner}` = structural substrate · prefix operator = active field applied to that substrate.

#### §1.17.4 Compound macros (reusable composites)

Register recurring composites as **named macros** — cite in plans instead of redrawing:

| Macro ID | Expansion | Use |
|:---|:---|:---|
| **MACRO-RIPARIAN-AXIS** | `≈≈≈╬≈≈≈` + `█▓▒` levee bands | River spine charts |
| **MACRO-WIND-ALLEY** | `⚶═══╬═══⚶` + `▲` upstream | Wind channeled corridor |
| **MACRO-FIRE-SADDLE** | `██══╬══██` + `⊗⊗` + `▲` vector | Fire corridor through network |
| **MACRO-REGROWTH-CHAIN** | `○ → ◇▒▒▒ → ⊕⊕ → ▓ → █` | Post-disturbance sequence |
| **MACRO-AG-PARCEL** | `╔` + `▒` field + `═` road + `█` woodlot + `╚` | Framed farm unit |
| **MACRO-DEFENSE-IN-DEPTH** | `▒` + `█` + `○` gaps + `─` + `□` | Military belt |
| **MACRO-SHELTER-LEE** | `Wind→` + `⚶` + `█` ring + `▒` lee field | Shelterbelt cross-section |
| **MACRO-DELTA-FORK** | `≈╬≈` + `FAN-A/B/C` + `◊` nodes | Distributary split |
| **MACRO-URBAN-CELL** | `╦` + `□□□` + `⊖` row | City block in forest grid |
| **MACRO-SPECIES-FRONTLINE** | `█⇣⇣` + `══` + `⇡⇡▒` | Competition stalemate |

Macros compose: `MACRO-AG-PARCEL + MACRO-RIPARIAN-AXIS` = farm with river edge.

#### §1.17.5 Nesting grammar (topology inside topology)

Nest with **depth markers** — each level adds a scale band:

```text
L0 REGION    ⟨XL⟩  ████████████████████
  L1 NETWORK ⟨L⟩   ◊═══════╬═══════◊
    L2 PATCH   ⟨M⟩   █████●████
      L3 CLUSTER ⟨S⟩  ◊ ◇ ◇
        L4 GAP   ⟨S⟩  ○
          L5 REGROWTH ⟨S⟩ ▒▒▒
```

**Nesting operators:**

| Notation | Meaning |
|:---|:---|
| **A ⊃ B** | Topology B fully inside A's boundary |
| **A ∩ B** | Overlap — both topologies active (answer: intersect) |
| **A ⊣ B** | A blocks B (barrier / ☍) |
| **A → B** | Succession / flow from stage A to B |
| **A + B** | Coexistent without containment |

**Valid nest depth:** 2–6 levels typical · 7+ requires `NESTED_HIERARCHY` preset · each level must have distinct `topology_kind`.

#### §1.17.6 Derivation pipeline (primitive → complex chart)

```text
Step 1  ANCHOR     pick ◊ ⌂ ▲ ▼ (where)
Step 2  BONE       connect anchors with ═ ≈ ║ ╬ (how linked)
Step 3  MASS       fill regions with █▓▒░ + ●○◇ (what grows)
Step 4  FIELD      overlay ⚶ ⊕ ⊖ ⊗ ⊙ ☍ (what forces act)
Step 5  VECTOR     add ⇡⇣ → ▲ (direction of change)
Step 6  FRAME      optional ╔╗╚╝ + ◉ chart ID
Step 7  METADATA   guards, scale_band, preset_id, macro refs
```

**Worked derivation — riparian woodlot with wind and regrowth:**

```text
Step 1  ◊A anchor on hydro:river_12
Step 2  ◊A ≈≈≈≈╬≈≈≈≈ ◊B
Step 3  fill: ███●███  (patch + old-growth anchor)
Step 4  stack: ⚶{█▓▒}  (wind on gradient edge)
Step 5  trailing: ○ → ◇▒▒▒ → ⊕⊕  (downstream regrowth)
Step 6  ◉RIPARIAN-REGENERATION-Ψ8
```

#### §1.17.7 Complexity budget (when to stop adding glyphs)

| Scale | Max field overlays | Max topology nodes | Max nest depth |
|:---:|:---:|:---:|:---:|
| **S** | 2 | 3 | 2 |
| **M** | 4 | 8 | 4 |
| **L** | 8 | 20 | 5 |
| **XL** | 12+ | 50+ | 6 |

Beyond budget → split into linked charts with cross-refs (`◊A` in chart-1 = `◊G` in chart-2).

---

### §1.18 Composite pattern catalog (learn by example)

**Pattern A — Network junction (from primitives):**

```text
◊A═══════◊B
   ║       ║
   ║   █   ║
   ◊C══╬══◊D
       ║
       ◊E

Parse: 4 nodes · 2 transport bones · 1 ╬ convergence · 1 central patch █
Macro:  NETWORK_CONNECTED fragment
```

**Pattern B — Pressure sandwich:**

```text
⚶⚶⚶⚶⚶⚶⚶⚶
────────────  ← domain separator (optional prose line)
████████████
⊕⊕⊕⊕⊕⊕⊕⊕
────────────
▒▒▒▒▒▒▒▒▒▒

Parse: wind domain → canopy → expansion front → regrowth substrate
Chart:  ECOLOGICAL-PRESSURE-Ω27 family
```

**Pattern C — Stalemate front:**

```text
████████⇣⇣⇣
════════════  ← neither ⇡ nor ⇣ crosses
▒▒▒▒▒▒⇡⇡⇡

Parse: species competition · ⊕ stalled against ⊖
Chart:  FOREST-WARFARE-Ω113
Metadata: { "front": "stalled", "north_species": "…", "south_species": "…" }
```

**Pattern D — Nested cascade (vertical read):**

```text
REGION ████████████
  ▼
NETWORK ◊═══╬═══◊
  ▼
PATCH █████●████
  ▼
GAP ○
  ▼
REGROWTH ▒▒▒ → ▓ → ▲ → █

Parse: 5-level nest · succession time runs downward
Chart:  NESTED-SUCCESSION-Ψ18
```

**Pattern E — Multi-field tile (one cell, many layers):**

```text
Cell(12,7) stacks:
  [visibility: ⊖]  [wind: ⚶]  [canopy: █]  [age: ●]  [hydro: ≈]  [disturbance: ∅]
→ JSON: { "overlays": ["suppress","wind","canopy","old_growth","hydro"] }
```

---

### §1.19 Authoring new composites (extension protocol)

When inventing a composite not in the catalog:

```text
1. DECOMPOSE  — list primitives used (must all exist in §1.0)
2. NAME       — ◉YOUR-NAME-Ωn following suffix family (§1.5)
3. MACRO      — if reused ≥3 times, register MACRO-* row in §1.17.4
4. PRESET     — if sim-ready, add topology_preset_id to §5 (planner-mcp)
5. WITNESS    — one ASCII chart + metadata JSON in debug_runs/ or schema example
6. COLLISION  — check SYMLANG §2.13 collision table · tag ⟨VEG⟩ block
7. LAYER TAG  — mark each glyph field: / op: / struct: in JSON sidecar
```

**Naming suffix guide for new charts:**

| Suffix | Use when composite is… |
|:---:|:---|
| **Ω** | Network, interaction, multi-field |
| **Σ** | Fracture, composite, multi-patch |
| **Δ** | Drainage, ag mosaic, land-use |
| **Λ** | Fan, delta, distributary |
| **Ψ** | Succession, regeneration time series |
| **Ξ** | Deep nest, hierarchy |

**Anti-patterns (do not compose):**

```text
⛔ New Unicode glyph without §1.19 review
⛔ Same glyph, two layers, one line (except documented stacks §1.17.3)
⛔ Composite with no anchor ◊/⌂/▲/▼
⛔ Field overlay without structural substrate below it
⛔ Chart ID without parseable bone or mass
⛔ Biome label substituting for composition ("temperate_forest" alone)
```

**Complexity lift examples (simple → advanced):**

```text
L1  ████                    single patch (invalid alone — add edge gradient)
L2  █▓▒ + ◊                 patch + node
L3  ◊═◊ + █▓▒               connected patch pair
L4  ◊═◊ + █▓▒ + ⚶           network + wind field
L5  ◊≈◊ + █●█ + ⚶ + ⊕▒      riparian + regrowth front
L6  nest(L5) + MACRO-AG-PARCEL + ⊖ urban strip   settlement-scale program
L7  MEGA-BIOSPHERE-Ω200     8–20 field layers · XL chart family
```

---

## §2 Extract / encoder glyphs (deterministic placement)

From `low_res_forest_veg.png` v2.3 — use in population fields, MCP job specs, atlas extract staging.

### §2.1 Vegetation layers

| Glyph | Role |
|:---:|:---|
| **@** | Old growth / canopy core |
| **#** | Mature forest |
| **%** | Mid-story / dense understory |
| ***** | Young forest / regrowth |
| **.** | Shrub / brush |
| **,** | Grass / open |

### §2.2 Terrain & features

| Glyph | Role |
|:---:|:---|
| **~** | Water / wet / marsh |
| **^** | Ridgeline / high |
| **v** | Gully / low / flow |
| **=** | Road / path / rail |
| **\|** | Edge / boundary |
| **+** | Focal point / node |
| **x** | Disturbance / fire / gap |
| **:** | Sparse / scattered |

### §2.3 Built & land-use

| Glyph | Role |
|:---:|:---|
| **[** | Built / urban / structure |
| **]** | Clearance / field / farm |

### §2.4 Direction & structure

| Glyph | Role |
|:---:|:---|
| **<** | Flow direction / gradient in |
| **>** | Spread direction / expansion out |
| **/** **\\** | Slope direction |
| **()** | Ring / enclosure |
| **{}** | Compound / nested |
| **<>** | Convergence / divergence |

### §2.5 Density bands (encoder)

| Band | Coverage |
|:---|:---|
| ████ block | ~100% |
| ▓▓▓▓ | ~75% |
| ▒▒▒▒ | ~50% |
| ░░░░ | ~25% |
| .... | ~10% |

---

## §3 Glyph mapping (planning ↔ extract)

| Planning (§1) | Extract (§2) | Notes |
|:---:|:---:|:---|
| ● | @ | Old-growth nucleus |
| █ | # | Mature canopy |
| ▓ | % | Mid-story |
| * (regrowth) | * | Young / regrowth |
| ▒ | . | Shrub |
| ░ | , | Grass / open |
| ○ / x | x | Disturbance gap |
| ◊ / + | + | Topology node |
| ≈ | ~ | Hydrology |
| ▲ / ^ | ^ | Ridge |
| ▼ / v | v | Gully / sink |
| ═ | = | Transport |
| □ | [ | Built / protected |
| ] (field) | ] | Cleared / farm |
| ⊕ | > | Expansion front |
| ⊖ | < | Suppression front |
| ⊗ | x + modifier | Disturbance overlay |
| ⚶ | (derive) | Wind — from weather+elevation; tag `!wind` |
| ☍ | (no twin) | Barrier — metadata `barrier:hard` |
| ⊙ | + refugia tag | Attractor — `habitat_weight` in metadata |
| ⌂ | [ + mgmt tag | Management anchor — `land_dna.M` |
| ◇ | * cluster | Regeneration nucleus |
| ─ | (no extract twin) | Observation / weak boundary — planning only |
| ~ | (no extract twin) | Wind wave front — planning only; extract `~` = water |
| │ ║ | \| | Vertical flow / network arm |
| ╱ ╲ | / \\ | Diagonal graph edge |
| ◉ | (header only) | Chart section marker — not encoded in tiles |
| → | > or dir: | Direction annotation |

---

## §4 Topology library V2 (20 base kinds)

Canonical node kinds for `VegetationTopologyGraph`:

| ID | Kind | Role |
|:---:|:---|:---|
| T01 | **Corridor** | Flow-aligned band (not a mere line) |
| T02 | **Patch** | Density-gradient blob (core → edge → gap) |
| T03 | **Ring** | Enclosure / shelter / perimeter |
| T04 | **Fan** | Widening spread from source (alluvial / flood) |
| T05 | **Spine** | Ridgeline or river-axis linear feature |
| T06 | **Cluster** | Regeneration nuclei (most common regrowth) |
| T07 | **Network** | Linked patches via corridors |
| T08 | **Mosaic** | Multi-type tile mix within one partition |
| T09 | **Fringe** | Sharp or gradual edge transition |
| T10 | **Pocket** | Internal vegetation in open matrix |
| T11 | **Barrier** | Movement / visibility block (thick or thin) |
| T12 | **Archipelago** | Forest islands in open matrix |
| T13 | **Delta** | Distributary / branching wet spread |
| T14 | **Convergence** | Many inflows → basin / node |
| T15 | **Divergence** | Single origin → radial spread |
| T16 | **Veins** | Capillary fine network |
| T17 | **Basin** | Bowl / wetland collection |
| T18 | **Crown** | Hilltop / exposure-limited cap |
| T19 | **Ladder** | Terraced / stepped bands |
| T20 | **Nested** | Topology inside topology (see §6) |

**Per-node metadata (required v0):**

```text
width · curvature · branching · age · species_bias · continuity · flow_direction
  + anchor_ref · operator_stack_id · succession_age · cx_class · scale_band
```

**Patch internals (always modeled, never uniform fill):**

```text
Core · Edge · Gap · Regrowth · Deadfall
```

**Ring pressure modes:**

```text
Containment · Protection · Boundary · Visibility · Concealment
```

**Cluster properties:**

```text
NucleusCount · Connectivity · ExpansionBias · SpeciesDiversity · AgeDiversity
```

---

## §5 Preset catalog (30 named variants — pictorial v2.3)

Machine IDs for LG-0 schema `topology_preset_id`:

| # | Preset ID | Base | Key params |
|:---:|:---|:---:|:---|
| 01 | `CORRIDOR_RIPARIAN` | Corridor | w:3–6, d:80–100, dir:<>, cont:high, bias:moist |
| 02 | `CORRIDOR_ROADSIDE` | Corridor | w:2–4, d:40–70, bias:edge, species:hardy |
| 03 | `CORRIDOR_RAILSIDE` | Corridor | w:2–3, d:30–60, disturbance:high |
| 04 | `PATCH_ROUNDED` | Patch | core80-edge20, r:5–12, shape:blob |
| 05 | `PATCH_IRREGULAR` | Patch | core80-edge20, r:6–15, gaps:yes |
| 06 | `PATCH_SPLIT` | Patch | cores:2–4, gap:2–8, connect:low |
| 07 | `RING_SHELTERBELT` | Ring | w:2–4, d:70–90, use:windbreak |
| 08 | `RING_FORTIFIED` | Ring | w:3–6, d:80–100, use:defensive |
| 09 | `FAN_FLOODPLAIN` | Fan | spread:fan, angle:60–120°, bias:moist |
| 10 | `FAN_ALLUVIAL` | Fan | origin:high, gradient:steep→flat |
| 11 | `SPINE_RIDGELINE` | Spine | w:1–3, follow:ridge, exposure:high |
| 12 | `SPINE_RIVER` | Spine | follow:river, w:2–6, moisture:high |
| 13 | `CLUSTER_NATURAL` | Cluster | nuclei:3–8, connect:low, spread:organic |
| 14 | `CLUSTER_DENSE` | Cluster | nuclei:5–12, d:high, gaps:low |
| 15 | `ARCHIPELAGO_FOREST` | Archipelago | islands:many, matrix:open |
| 16 | `MOSAIC_MIXED` | Mosaic | mix:5–8 types, transition:gradual |
| 17 | `NETWORK_CONNECTED` | Network | nodes:5–20, connectivity:high |
| 18 | `NETWORK_DENDRITIC` | Network | branch:many, order:1–4, flow:directed |
| 19 | `POCKET_INTERNAL` | Pocket | inside:open, size:small |
| 20 | `FRINGE_EDGE` | Fringe | edge:sharp, width:1–4 |
| 21 | `BARRIER_THICK` | Barrier | w:4–12, d:80–100, visibility:low |
| 22 | `BARRIER_THIN` | Barrier | w:1–2, d:40–70 |
| 23 | `DELTA_DISTRIBUTARY` | Delta | channels:many, sediment:high |
| 24 | `CONVERGENCE_BASIN` | Convergence | inflow:many, moisture:high |
| 25 | `DIVERGENCE_FLOW` | Divergence | outflow:many, spread:radial |
| 26 | `VEINS_CAPILLARY` | Veins | w:1, network:high |
| 27 | `BASIN_WETLAND` | Basin | bowl:yes, vegetation:hydrophilic |
| 28 | `CROWN_HILLTOP` | Crown | exposure:high, growth:stunted |
| 29 | `LADDER_TERRACE` | Ladder | steps:many, use:agroforestry |
| 30 | `NESTED_HIERARCHY` | Nested | levels:5+, complexity:high |

---

## §6 Nested hierarchy (mandatory composition model)

```text
Region
 └─ Network
     └─ Patch
         └─ Cluster
             └─ Gap
                 └─ Regrowth
```

**Example stack (chart ID `VEGETATION-HIERARCHY-Ξ12`):**

```text
REGION ══════════════════════════════
  NETWORK-01  ◊═══════◊ ║ ◊═══╬═══◊
    PATCH-04  ███████████ / █████●████ / ███○██████
      GAP-02  ○ → ▒▒▒▒▒▒ regrowth
```

Nested nodes carry `parent_topology_id` + `depth` + `scale_band`.

---

## §7 Field overlay layers (multi-scale)

A tile/partition simultaneously belongs to **8–20 overlapping field layers** — not one biome tag. See also §1.8 for the complete field stack and mega-scale legend.

| Field | Glyph hint | Drives |
|:---|:---:|:---|
| Canopy | █ ▓ | Fuel, shade, visibility |
| Age | ● ◇ ▒ gradient | Succession stage |
| Hydrology | ≈ | Species bias, wetland |
| Wind | ⚶ ~ | Shelterbelts, stunting |
| Disturbance | ⊗ ○ | Gaps, regrowth fronts |
| Species | ⇡ ⇣ | Competitive fronts |
| Visibility | ⊖ ─ □ | Military / urban sight lines |
| Fire | ⊗ + ╬ corridor | Fire corridors |
| Human pressure | ⌂ ⊖ | Cleared / maintained |
| Habitat | ⊙ | Attractor nodes |

**Mega-scale chart IDs (complete catalog — art in `olant_grammer.md`):**

| Chart ID | Contents |
|:---|:---|
| `VEG-NETWORK-Ω7` | Network + ◊ anchors + ● nuclei + ○ gaps + ╬ junctions |
| `OLD-GROWTH-COMPLEX-Σ4` | ▲ ridge spine + patch + regrowth fan + ≈ riparian |
| `AGRI-LANDSCAPE-Δ9` | ▒ fields + █ woodlots + ═ transport (╔ frame) |
| `VEGETATION-HIERARCHY-Ξ12` | Region → Network → Patch → Gap → regrowth |
| `DELTA-FOREST-Λ5` | Upstream ≈ + distributary fans + ◊ nodes |
| `DEFENSIVE-VEGETATION-Ω13` | ▒ concealment + █ canopy + ─ observation + □ facility |
| `RIPARIAN-REGENERATION-Ψ8` | ⚶ wind + ≈ hydrology + ◇ regrowth + ⊕ expansion |
| `ECOLOGICAL-PRESSURE-Ω27` | ⚶ wind front + ⊕ regrowth + ≈ hydrology + ▒ regrowth front |
| `NESTED-SUCCESSION-Ψ18` | Full cascade ladder ○→▒→▓→▲→█ |
| `MOUNTAIN-FOREST-DRAINAGE-Δ44` | ▲▲▲ head + ⚶ + ╬ drainage stages + ≈ basin ▼ |
| `ECOLOGICAL-NETWORK-Ω91` | ◊A…I weighted graph + ╱╲ diagonal edges |
| `FOREST-WARFARE-Ω113` | Competing canopy + ⇡⇣ species fronts |
| `FIRE-CORRIDOR-Ω51` | Canopy network + ╬ fire corridor + ⊗ vector |
| `URBAN-FOREST-FRACTURE-Σ63` | Forest grid + ╦╩ urban fracture + ⊖ suppression |
| `MEGA-BIOSPHERE-Ω200` | Full biospheric overlay + ~~~~ wind domain |

---

## §8 Modifiers, scale, context tags

### §8.1 Environmental modifiers (overlay)

```text
fire · harvest · drought · wind · night_fire_risk
```

### §8.2 Age layers (overlay)

```text
old_growth · mid_story · regrowth · open
```

### §8.3 Scale bands

| Band | Cells | Use |
|:---:|:---:|:---|
| **S** | 1–4 | Micro — gap, single cluster |
| **M** | 5–16 | Meso — patch, corridor segment |
| **L** | 17–64 | Macro — network, mosaic region |
| **XL** | 65+ | Mega — biospheric overlay |

### §8.4 Context tags (attach to any node)

```text
# biome   @ moisture   ^ elevation   [ land_use ]   ! disturbance   = infrastructure
```

---

## §9 Operator stack (shared with building grammar)

**Layer rule:** Operators below mutate **topology graph** structure. Field glyphs (**§1.11** ⊕⊖⊗⚶☍⊙⌂◇) describe **forces on** that structure — never swap layers in one expression.

| Op | Graph meaning | Field glyph collision |
|:---:|:---|:---|
| **⌂** | Anchor to river / road / site edge | Same glyph as management anchor — `op:⌂` binds anchor; `field:⌂` sets policy |
| **═** | Corridor extrude | — |
| **⊕** | Seed patch (init interior) | **Not** expansion front — use `field:⊕` in overlay for succession |
| **⊖** | Clear / suppress volume | **Not** urban field — use `field:⊖` for suppression front |
| **⇉** | Expand edge (succession tick) | Pairs with `field:⊕` front |
| **⇇** | Contract / maintain | Pairs with `field:⊖` |
| **≈** | Drift / meander | Same glyph as hydrology axis |
| **◊** | Cluster fill | Same glyph as topology node |
| **□** | Ring band | Same glyph as cleared space |
| **⟁** | Branch to secondary anchor | — |

**Succession tick operators vs species vectors:**

| Symbol | Layer | Meaning |
|:---:|:---|:---|
| **⇉ ⇇** | Operator | Graph edge expand/contract per sim tick |
| **⇡ ⇣** | Field | Species/competition advance direction in charts |

**Example — riparian corridor:**

```text
Stack: ⌂ River · ≈ Drift · ═ Corridor · ⇉ Expand · ◊ Cluster
Chart: ≈≈≈≈╬≈≈≈≈ / ███●███ / ▒▒▒▒▒ regrowth fan ▼
```

---

## §10 Cross-reference index

| Artifact | Role |
|:---|:---|
| [`olant_grammer.md`](olant_grammer.md) | Operator source — topology charts + field warfare maps |
| [`low_res_forest_veg.png`](low_res_forest_veg.png) | Pictorial preset library v2.3 (30 variants + legend) |
| [`guide_landscape_grammar_v1.md`](../../src/dev/guide_landscape_grammar_v1.md) | Architectural charter |
| [`plan_landscape_grammar_exec_001_v1.md`](../../src/dev/plan_landscape_grammar_exec_001_v1.md) | LG-0…LG-6 execution |
| [`arch_build_grammar_v0_baseline_v1.md`](../../src/dev/arch_build_grammar_v0_baseline_v1.md) | Building grammar mirror |
| [`SYMBOLIC_LANGUAGE.meta.md`](../SYMBOLIC_LANGUAGE.meta.md) §2.13 | SYMLANG registration |
| `tools/mcp/schemas/landscape_grammar_v0.schema.json` | LG-0 JSON schema (**shipped** v1.2 — lexicon v1.4.0) |
| `tools/mcp/schemas/examples/landscape_dna_agri_riparian_v0.json` | Pilot preset — ag riparian nested topology |

```text
⟦/VEG-LAND-LEXICON-001⟧  LG-0 schema ✅ · NEXT: LG-1 @coder evaluator stub
```
