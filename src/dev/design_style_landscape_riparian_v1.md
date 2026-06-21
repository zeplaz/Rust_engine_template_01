# Riparian / agricultural landscape style bible `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-STYLE-LANDSCAPE-RIparian-001** |
| **Program** | Style concepts · Track C1 |
| **Date** | 2026-06-02 |
| **Owner** | `@designer-mcp` |
| **Authority** | [`agri_riparian_v0.json`](../../assets/configs/landscape/presets/agri_riparian_v0.json) · [`landscape_dna_agri_riparian_v0.json`](../../tools/mcp/schemas/examples/landscape_dna_agri_riparian_v0.json) · [`landscape_grammar_lexicon_v1.md`](../../prompts/guides/landscape_grammar_lexicon_v1.md) |
| **Burn cross-ref** | [`design_veg_burn_visual_language_v1.md`](design_veg_burn_visual_language_v1.md) |
| **Feeds** | LG-5 landscape presets · `CORRIDOR_RIPARIAN` atlas rows · district hydro bias (VEG-DISTRICT-HYDRO-001) |
| **Verdict** | **PASS** |

```text
DES-STYLE-LANDSCAPE-RIparian-001 Q✓
Canopy mass · edge softness · burn read @ 64px iso — ag riparian parcel language
```

---

## 0. Lineage

**Ag riparian** = temperate farm parcel with **water-axis tree band**, **field mosaic**, **shelterbelt ring**, and **soft field fringe** — not wilderness old-growth.

**Chart:** `AGRI-LANDSCAPE-Δ9` · **Preset:** `agri_riparian_v0`  
**Macros:** `MACRO-AG-PARCEL` · `MACRO-RIPARIAN-AXIS` · `MACRO-SHELTER-LEE`

**Required topologies:** `CORRIDOR_RIPARIAN` · `RING_SHELTERBELT` · `PATCH_IRREGULAR` · `FRINGE_EDGE`

---

## 1. Canopy mass (silhouette @ 64px)

| Topology | Mass read | Height band | Density cue |
|:---|:---|:---|:---|
| **Corridor riparian** | Linear band following `≈` hydrology axis | Mid–tall (M band) | Core 80 · edge 20 — **spine must read before fill** |
| **Patch woodlot** | Irregular blob with interior gaps | Mid (M band) | Core 80 · gaps true · regrowth cluster child |
| **Ring shelterbelt** | Closed windbreak frame | Low–mid (M band) | 70–90 · lee-side protection read |
| **Fringe field edge** | Low scatter at ag boundary | Low (S band) | Gradual width 1–4 · no hard wall |

**Rule @ iso zoom:** player identifies **corridor vs patch vs ring** without opening diagnostics — corridor = **elongated spine**, patch = **closed mass**, ring = **frame around field**.

```text
Planning sketch (AGRI-LANDSCAPE-Δ9):
╔ parcel ╗ · FIELDS=▒ · WOODLOTS=█ · TRANSPORT=═ · () RING shelterbelt
≈≈≈╬≈≈≈  riparian corridor spine along water
```

---

## 2. Edge softness

| Edge type | Visual | Operator stack | Ban |
|:---|:---|:---|:---|
| **Riparian drift** | Meander / fuzzy interior along `≈` axis | `drift` · `expand` · `cluster` | Sharp rectangular tree box on water |
| **Field fringe** | `▒` scatter tapering to open | `fringe` · gradual metadata | Hard `#000` void between field and wood |
| **Woodlot interior** | Gaps + regrowth nuclei | `cluster` child on patch | Solid filled circle — no gap read |
| **Shelterbelt lee** | Dense band on windward, thin leeward | `MACRO-SHELTER-LEE` | Uniform donut thickness |

**λ_moisture ≥ 0.85** (preset default 0.88): riparian corridor gets **moist species bias** — slightly darker green core, lighter edge feather.

---

## 3. Burn / scar / recovery read

Inherits [`design_veg_burn_visual_language_v1.md`](design_veg_burn_visual_language_v1.md) with **topology-specific overrides:**

| Topology | Burn reads as… | Scar reads as… | Regrowth reads as… |
|:---|:---|:---|:---|
| **Corridor riparian** | Linear fire along water spine — **keep corridor axis visible** | Ash void following meander — not filled rectangle | Grass/shrub scatter **along** spine, not blob |
| **Patch woodlot** | Crown gap in blob mass | Flat void with sharp woodlot edge | Cluster nuclei returning inside patch |
| **Ring shelterbelt** | Segment breaks in frame | Broken ring segments | Partial ring refill — gap in frame OK |
| **Fringe field edge** | Edge creep — low contrast | Thin ash line at ag boundary | Fine grass scatter — lowest height |

**Fail @ 64px:** burn and scar same hue on corridor spine; regrowth reads as mature canopy on scar base.

---

## 4. Palette (landscape extract)

| Token | Hex | Use |
|:---|:---|:---|
| `canopy_riparian_core` | `#2d7038` | Corridor / woodlot core |
| `canopy_riparian_edge` | `#4a8040` | Feather edge · shelterbelt |
| `field_stubble` | `#b8a878` | Agricultural open · `▒` fields |
| `field_fallow` | `#9a9070` | Disturbed / bare soil fringe |
| `hydrology_glint` | `#6a8aa0` @ 30% | Water adjacency hint — not building glass |
| `burn_core` | `#e87830` | Active fire — per burn bible |
| `scar_ash` | `#3a3a3a` | Post-fire void |
| `regrowth_grass` | `#8aaa58` | Fine scatter regrowth |
| `regrowth_shrub` | `#4a8040` | Mid regrowth nuclei |
| `road_dust` | `#8a8884` | Transport spine `═` |

**Material profiles (pilot):** reuse `grass_canopy_pilot_02` · `soil_brown_pilot_02` from mat pilot 002 — no new ids until merge.

---

## 5. Transport + land-use overlay

| Layer | Glyph (planning) | Extract read |
|:---|:---|:---|
| Fields | `▒` | Low texture scatter |
| Woodlots | `█` | Closed canopy mass |
| Transport | `═` | Hard spine — **legibility over softness** |
| Hydrology field | `≈` | Moist bias overlay — not standalone geometry |

**Rule:** transport spine stays **crisp**; riparian softness applies to **vegetation mass only**.

---

## 6. Concept refs (3)

| Ref | Subject | Takeaway |
|:---|:---|:---|
| **R1** | Meandering riparian band beside rectilinear field | Corridor spine + soft drift interior |
| **R2** | Farm shelterbelt windbreak | Ring frame · lee-side thin read |
| **R3** | Post-fire ag edge with regrowth fringe | Scar line + fine grass return — not closed canopy |

*(Refs are art-direction anchors — not generated assets.)*

---

## 7. Handoff

| Owner | Next |
|:---|:---|
| **@coder-mcp** | LG-5 atlas rows for `topology_corridor` + regrowth variants · VEG-DISTRICT-HYDRO bias |
| **@designer-mcp** | G4 manual `topology_corridor_regrowth_grass` per [`dmcp_veg_atlas_ship_v1.md`](dmcp_veg_atlas_ship_v1.md) |
| **Operator** | Preset walk on `agri_riparian_v0` partition — corridor vs patch vs ring @ tactical zoom |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-02 |

```text
DES-STYLE-LANDSCAPE-RIparian-001 Q✓
```
