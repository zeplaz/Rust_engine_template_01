# PLAN-LANDSCAPE-GRAMMAR-001 — execution plan `v1`

```text
⟦SYMLANG⟧⟐v1  ◈EXEC
⟨ID⟩ PLAN-LANDSCAPE-GRAMMAR-001
Date: 2026-06-13
Status: **SIGNED** (@planner 2026-06-13)
Charter: $ref:src/dev/guide_landscape_grammar_v1.md
Lexicon: $ref:prompts/guides/landscape_grammar_lexicon_v1.md
Operator source: $ref:prompts/guides/olant_grammer.md
Pictorial ref: $ref:prompts/guides/low_res_forest_veg.png
Schema: $ref:tools/mcp/schemas/landscape_grammar_v0.schema.json
Example: $ref:tools/mcp/schemas/examples/landscape_dna_agri_riparian_v0.json
Mirror: $ref:src/dev/plan_operator_build_readability_exec_001_v1.md (building grammar exec pattern)
Parent index: $ref:src/dev/construction_procedural_growth_index_v1.md
```

**Goal:** Ship landscape program authority (topology graph + succession memory + human land-use coupling) **before** any iso sprite atlas work. Sprites are terminal extract only.

**Rejected (explicit):** Biome→Density→Sprites · global Tree ECS · empty stub modules · ecology-only tick extension.

---

## 0. Evidence reviewed (2026-06-13)

| Source | Key additions folded into plan |
|:---|:---|
| **`olant_grammer.md`** | Topology≠Shape · 20-kind library · nested hierarchy · field overlay maps · chart IDs (Ω/Σ/Δ/Λ/Ψ series) · dual pressure glyphs ⊕/⊖ |
| **`low_res_forest_veg.png` v2.3** | 30 named topology presets with param ranges · extract glyph set (@#%*.,~^v=…) · scale bands S/M/L/XL · context tags · environmental modifiers |

Both sources are canonical; reconciled in [`landscape_grammar_lexicon_v1.md`](../prompts/guides/landscape_grammar_lexicon_v1.md).

---

## 1. Architecture spine (unchanged intent, enriched detail)

```text
SITE◈WORLD-DNA
      ▼
LAND-DNA { H,S,E,T,D,L,A,M }
      ▼
λ PRESSURE-FIELD { moisture,slope,exposure,disturbance,access,security,productivity,legibility }
      ▼
LANDSCAPE-PROGRAM (natural · ag · industrial · military · settlement)
      ▼
VEGETATION-TOPOLOGY-GRAPH  ← 20 base kinds · 30 presets · nested composition
      ▼
SUCCESSION-GRAPH + DISTURBANCE-HISTORY + LAND-USE-INFLUENCE
      ▼
POPULATION-FIELDS (planning glyphs → extract glyphs via §3 mapping)
      ▼
DETERMINISTIC-INSTANCES
      ▼
SPRITES / OVERLAY EXTRACT
```

**Chunk authority question:**

```text
∀ partition P:  topologies(P) = { T₁…Tₙ }   where n ≥ 1 typically 3–8
```

---

## 2. LG-0 deliverables (@planner-mcp)

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| **LG-0-001** | `landscape_grammar_v0.schema.json` | **DONE** — lexicon v1.4.0 aligned · validates teaching preset |
| **LG-0-002** | `landscape_dna_agri_riparian_v0.json` example | [`tools/mcp/schemas/examples/landscape_dna_agri_riparian_v0.json`](../../tools/mcp/schemas/examples/landscape_dna_agri_riparian_v0.json) — riparian + field-edge + shelterbelt ring |
| **LG-0-003** | Lexicon registered in SYMLANG §2.13 | $ref in meta header |
| **LG-0-004** | Topology preset table in repo | Same 30 rows as pictorial v2.3 (lexicon §5) |
| **LG-0-005** | `@planner` sign charter + exec | This doc → **SIGNED** 2026-06-13 |

**Schema fields (topology node v0):**

```json
{
  "topology_kind": "Corridor",
  "preset_id": "CORRIDOR_RIPARIAN",
  "anchor_ref": "hydro:river_edge_12",
  "scale_band": "M",
  "metadata": {
    "width": [3, 6],
    "density": [80, 100],
    "flow_direction": "downstream",
    "continuity": "high",
    "species_bias": "moist_mix"
  },
  "operator_stack": ["anchor_river", "drift", "corridor", "expand", "cluster"],
  "cx_class": "Cx3",
  "parent_topology_id": null
}
```

**Nested example (NESTED_HIERARCHY):**

```text
Region: AGRI-LANDSCAPE-Δ9
  ├─ NETWORK_CONNECTED (woodlot links)
  ├─ CORRIDOR_RIPARIAN (≈ axis)
  ├─ RING_SHELTERBELT (field windbreak)
  └─ PATCH_IRREGULAR (farm woodlot)
       └─ CLUSTER_NATURAL (regrowth nuclei)
```

---

## 3. LG-1 — pilot partition (@coder) **DONE**

| ID | Task | Witness |
|:---|:---|:---|
| **LG-1-001** | `LandscapeGrammarCatalog` + `evaluate_landscape_program` (preset load, flatten, λ blend) | `src/systems/ecology/landscape_grammar.rs` |
| **LG-1-002** | Flattened topology graph ≥4 preset kinds (parent-chain depth) | 6 kinds on `agri_riparian_v0` |
| **LG-1-003** | Read-only λ blend from `ChunkEcology` + `VegetationField` + `ChunkWeather` | no ecology writeback |
| **LG-1-004** | Planning glyph overlays per node (machine witness; UI overlay LG-4) | `debug_runs/landscape_grammar_lg1_live.json` |

**Runtime:** `LandscapeProgramOnChunk` attaches on pilot chunk `(12, 0)` via `EcologyPlugin`.

**v0 pilot site:** agricultural riparian strip — `assets/configs/landscape/presets/agri_riparian_v0.json`.

**Verify:** `cargo test -p proc_A_dine01 --lib landscape_grammar`

---

## 4. LG-2 — succession + disturbance (@coder)

| ID | Task |
|:---|:---|
| **LG-2-001** | `SuccessionState` + `SuccessionAge` per partition cell |
| **LG-2-002** | `DisturbanceHistory` ring buffer (fire, clear, build, harvest) |
| **LG-2-003** | Fire: OldGrowth → BurnScar → Grass path on graph (not density-only) |
| **LG-2-004** | Construction clear → ⊖ operator + history event |

---

## 5. LG-3 — human district programs (@coder + @designer)

| District | Required topology presets |
|:---|:---|
| Agricultural | CORRIDOR_RIPARIAN · PATCH_IRREGULAR · RING_SHELTERBELT · FRINGE_EDGE |
| Industrial | BARRIER_THICK · POCKET_INTERNAL · FRINGE_EDGE · MOSAIC_MIXED |
| Military | RING_FORTIFIED · BARRIER_THICK · DEFENSIVE chart pattern · POCKET_INTERNAL (clearing) |

Coupling: `LandUseInfluence` ← settlement hierarchy + construction sites + transport graph.

---

## 6. LG-4 — population derive + preview (@coder)

| ID | Task |
|:---|:---|
| **LG-4-001** | Derive `VegetationPopulation` from graph + succession (not biome scalar) |
| **LG-4-002** | Map planning → extract glyphs per lexicon §3 |
| **LG-4-003** | Ecology preview tints show patch heterogeneity (≥3 topology kinds visible) |
| **LG-4-004** | Field overlays optional debug layer (wind, fire corridor, suppression) |

---

## 7. LG-5 / LG-6 — art (defer until LG-4 green)

| Phase | Owner | Gate |
|:---:|:---|:---|
| LG-5 minimal iso atlas | @designer-mcp → @coder-mcp | LG-4 witness |
| LG-6 flowers aesthetic | @designer-mcp | LG-5 |

### §7.1 Burn · succession · extract extension (**PLAN-VEG-BURN-EXTRACT-001**)

**Signed:** 2026-06-14 · Exec: [`plan_veg_burn_extract_001_v1.md`](plan_veg_burn_extract_001_v1.md)

Burn and regrowth ship **before** LG-5 atlas pixels — sim + extract first, sprites last.

| Layer | Mechanism | Owner |
|:---|:---|:---|
| **Sim** | `SuccessionState` + `LandscapeDisturbanceQueue` + **ActiveBurn** overlay | @coder A |
| **Planning** | ⊗ / ○ / **MACRO-REGROWTH-CHAIN** (lexicon §1.17) | schema + evaluator |
| **Extract** | `VegetationExtractFrame` — glyphs + modifiers → `variant_key` | @coder A |
| **Art** | LG-5 catalog `veg_burn_00..07` — construction PT-4 pattern | @designer-mcp → @coder-mcp |

**Principles:** P-001 plants from grammar · P-002 burn via disturbance · P-003 sprites terminal · P-004 ActiveBurn transient · P-005 variant_key lookup.

**Authority:** `.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md` — extract after `FireVisualFrameSet::BuildProfiles`; no render write to succession.

**Coder pick order:** VEG-BURN-OVERLAY-001 → SM-002 → SUCCESSION-003 → EXTRACT-004 → GLYPH-005 → FULLAPP-006.

**Do not:** LG-5 atlas before extract frame green · per-tree global ECS · density-only burn.

#### §7.1.1 Teach exception — 3-tile pilot atlas (PLAN-LG-EXEC-ANNOT-001)

The LG-5 **pilot** (`tile_landscape_lg5_pilot_v1`, 3 variants) is an intentional **teach** batch:

| Field | Pilot value | Full expansion |
|:---|:---|:---|
| `ship` | `false` | `true` only after G-ART-SHIP |
| `development_tier` | `pilot` | `production` |
| Variant count | 3 (`topology_*` clean only) | full matrix per `plan_landscape_atlas_budget_v1.md` |
| Burn rows | **absent** — extract uses tint | `veg_burn_00..07` in catalog + batch |

**Rule:** Do not cite pilot bake green as burn/scar coverage. `_meta.teaches: landscape_lg5` examples are **not** ship targets. Expansion job: `assets/staging/specs/tile_batch_landscape_expanded_v1.example.json`.

---

## 8. Witness probes (v0 acceptance)

| Probe | Pass |
|:---|:---|
| One chunk ≥3 topology kinds | patch + corridor + ring |
| Nested depth ≥2 | region → patch → cluster |
| Preset IDs resolve to graph nodes | 30/30 in schema enum |
| SuccessionAge ticks on undisturbed cell | graph time |
| Fire + construction write DisturbanceHistory | linked events |
| Extract layer uses §2 glyphs deterministically | same seed → same field |
| No global Tree entity ∝ map area | bounded instances |

---

## 9. Queue placement

| Queue | Row |
|:---|:---|
| `post_drain_phase5_queue.json` | Add **PLAN-LANDSCAPE-GRAMMAR-001** after G-PLAY tail (P2) |
| `construction_procedural_growth_index_v1.md` | Deliverables map row |
| `development_plan_index.md` | Landscape lane link |

**Do not queue:** empty `.rs` stubs · VegetationPopulation-only · biome rewrite.

---

## 10. Symbolic chart examples (planner handoff)

Operators may cite these chart IDs in plans/solutions (full art in `olant_grammer.md`; registry in lexicon §1.5):

```text
VEG-NETWORK-Ω7 · OLD-GROWTH-COMPLEX-Σ4 · AGRI-LANDSCAPE-Δ9 · VEGETATION-HIERARCHY-Ξ12
DELTA-FOREST-Λ5 · DEFENSIVE-VEGETATION-Ω13 · RIPARIAN-REGENERATION-Ψ8
ECOLOGICAL-PRESSURE-Ω27 · NESTED-SUCCESSION-Ψ18 · MOUNTAIN-FOREST-DRAINAGE-Δ44
FIRE-CORRIDOR-Ω51 · URBAN-FOREST-FRACTURE-Σ63 · ECOLOGICAL-NETWORK-Ω91
FOREST-WARFARE-Ω113 · MEGA-BIOSPHERE-Ω200
```

---

## 11. Symbol retention audit (`olant_grammer.md` → schema v1.2)

**Lexicon:** v1.4.0 (§1.0–§1.19). **SYMLANG:** §2.13 expanded. **Verdict:** all 21 semantic glyphs + structural/chart glyphs registered — JSON carries them in tagged fields, never flattened.

| Symbol class | Lexicon section | Schema field |
|:---|:---|:---|
| **§1.0 authoritative legend** | 21 semantic glyphs | `symbolic_field_stack` · `field_overlay_stack` |
| **Thin ─ vs heavy ═** | §1.2 · §1.4 | `corridor_subtype` · `metadata.glyph_planning` |
| **│ ║ ╱ ╲ framing ╔╗╚╝╦╩** | §1.4 | `_meta.symbolic_sketch_planning` · chart framing in sketches |
| **~ wind vs extract ~ water** | §1.0 disambiguation | `layer_tag: field` + extract only in `glyph_extract` |
| **▲▲▲ ⊕⊕⊕ intensity stacks** | §1.3 | `tagged_glyph_entry.intensity` |
| **◉ + Ω Σ Δ Λ Ψ Ξ chart IDs** | §1.5 (15 IDs) | `chart_id` enum · `chart_family` |
| **◊A…I node weights** | §1.6 | `anchor_node_label` |
| **Succession ladder ○→█** | §1.7 | `metadata.succession_stage` |
| **9-field overlay stack** | §1.8 | `field_overlay_stack[]` |
| **Mosaic % breakdown** | §1.9 | `metadata.mosaic_percent` |
| **Core/Mid/Edge density** | §1.1 | `core_density` · `mid_density` · `edge_density` |
| **§1.11 semantic deep ref** | ⚶☍⊙⊕⊖⊗⌂◇ | `symbolic_operator_glyph` enum + `symbolic_field_stack` |
| **§1.17 macros** | MACRO-* | `composition_macro` · `composition_macros[]` |
| **§9 field: vs op:** | Layer split | `field_overlay_entry.layer_tag` · `tagged_glyph_entry.layer` |
| **FIELDS=▒ legend** | §1.4 | `land_use_legend` · `_meta.land_use_shorthand` |
| **30 topology presets** | §5 | `topology_preset_id` enum |
| **20 base kinds T01–T20** | §4 | `topology_kind` + `topology_kind_id` |

**Rule (unchanged):** planning glyphs (§1) and extract glyphs (§2) **never mix in one layer**.

**Teaching preset** demonstrates v1.4 retention:

```text
lexicon_version: 1.4.0
chart_id: AGRI-LANDSCAPE-Δ9 · chart_family: Δ
land_use_shorthand: FIELDS=▒ WOODLOTS=█ TRANSPORT=═
composition_macros: MACRO-AG-PARCEL + MACRO-RIPARIAN-AXIS + MACRO-SHELTER-LEE
corr_riparian: anchor_node_label G · corridor_subtype riparian · mid_density zone
ring_shelter: anchor_node_label D · symbolic_field_stack ⚶{wind} + █{mass}
```

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-13 | Exec plan — integrates olant_grammer + pictorial v2.3 into LG phases |
| v1.1.0 | 2026-06-13 | LG-0-001 schema shipped · @planner **SIGNED** · §11 symbol retention audit |
| v1.2.0 | 2026-06-13 | Schema aligned to lexicon v1.4.0 — chart enum · field overlays · macros · ◊A…I · dual-layer tags |
| v1.3.0 | 2026-06-14 | §7.1 burn/extract extension — PLAN-VEG-BURN-EXTRACT-001 SIGNED |
| v1.4.0 | 2026-06-16 | §7.1.1 pilot teach exception — PLAN-LG-EXEC-ANNOT-001 |

```text
⟦/PLAN-LANDSCAPE-GRAMMAR-001⟧  LG-0 ✅ · ΔWF→@coder LG-1-001 evaluator stub
```
