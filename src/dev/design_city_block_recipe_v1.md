# City block recipe charter `v1` — DES-CITY-BLOCK-RECIPE-001

| Field | Value |
|:---|:---|
| **ID** | **DES-CITY-BLOCK-RECIPE-001** |
| **Issue** | CITY-C3 |
| **Parent** | [`plan_city_grammar_upgrade_v1.md`](plan_city_grammar_upgrade_v1.md) § G1 |
| **Date** | 2026-07-03 |
| **Owner** | `@designer-mcp` (charter + critique) → `@coder` (evaluator) |
| **Status** | **SIGNED** — `CITY-G0-WIT-001` done · charter + 3 RON examples on disk |
| **Queue** | [`city_grammar_queue.json`](../../tools/orchestrator/queues/city_grammar_queue.json) seq 8 |
| **Verdict** | **PASS** — designer-mcp sign-off 2026-07-03 |

```yaml
order_critique:
  request_summary: "BlockRecipe vocabulary v1 + 3 archetype recipe charters for G1 evaluator"
  rules_audit:
    data_not_code: pass
    deterministic_seed_chain: pass
    catalog_archetype_ids_only: pass
    sim_authority_on_visual_steps: pass
    no_bpy_in_charter: pass
  blocked: false
  proceed: yes
  foresight_flags:
    - "block_recipe_v1.schema.json — @coder-mcp after sign-off"
    - "CITY-G1-C3-001 evaluator still blocked on CITY-G1-C2 BlockFrame"
    - "v1 edge steps are teach-tier single pass — full perimeter deferred v1.1"
```

```text
DES-CITY-BLOCK-RECIPE-001
BlockRecipe vocabulary v1 + 3 archetype recipe charters
Town-scale grammar between district sim and per-lot building grammar
```

---

## 0. Why this exists (gap today)

```text
today:   GrowthProposal → single-lot building commit → incoherent streetscapes
target:  DistrictBook → BlockArchetype → BlockRecipe → lots → building grammar per lot
```

| Layer | Authority today | After G1 |
|:---|:---|:---|
| District / town sim | `TownBook` · `DistrictBook` · `DevelopmentPressure` | unchanged |
| Block clustering | `BlockBook` / `BlockRecord` (tiles + site_ids) | + `BlockArchetype` per block (C1) |
| Block composition | **none** (implicit) | **BlockRecipe** (this charter) |
| Building massing | `building_grammar` + ARCH-DNA presets | per-lot, seeded from `lot_seed` |

**Not in scope:** APS tier chips, tag presets, or assembly panel copy — those remain **building-tier** specs ([`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md)).

---

## 1. Design principles (MCP + sim grade)

| # | Rule |
|:---:|:---|
| R1 | **Data not code** — no hand-authored `spawn_low_density()` Rust; recipes are RON/JSON assets |
| R2 | **Deterministic** — `block_seed` + recipe id ⇒ identical lot layout across runs (G0c witness net) |
| R3 | **Relative to BlockFrame** — all placements are offsets from block anchor + street facing (C2) |
| R4 | **Catalog authority** — building archetype ids reference existing grammar ids (`IndustrialWarehouse`, `CivicBlock`, …), not new inline meshes |
| R5 | **Presentation vs sim** — edge scatter / street furniture may be visual-only until logistics authority exists; flag `sim_authority: false` on steps |
| R6 | **Teachable examples** — each shipped example declares `_meta.teaches[]` axes (see §7) |

**Seed chain (C4 — evaluator must use):**

```text
world_seed → town_seed(town_id) → block_seed(block_id) → lot_seed(lot_idx) → building_grammar seed
```

---

## 2. BlockArchetype enum (C1 alignment)

Coder owns threshold RON (CITY-G1-C1-001). Designer charters **recipe content** per archetype band.

| `BlockArchetype` | Player read | Primary building grammar families | Recipe id (v1) |
|:---|:---|:---|:---|
| `ForestPark` | Green buffer / park edge | none · optional scatter only | `block_recipe_forest_park_v1` |
| `LowDensityRes` | Detached / duplex row | `CivicBlock` (residential zoning) | `block_recipe_low_density_res_v1` |
| `MediumDensityRes` | Rowhouses / small apartments | `CivicBlock` · `RailEdge` (infill) | `block_recipe_medium_density_res_v1` |
| `HighDensityCommercial` | Mid-rise street wall | `CivicBlock` · `FactoryCluster` (ground retail) | `block_recipe_high_density_commercial_v1` |
| `Industrial` | Yard + hall + rail spur | `IndustrialWarehouse` · `FactoryCluster` | `block_recipe_industrial_yard_v1` |
| `Civic` | Plaza + civic frontage | `CivicBlock` | `block_recipe_civic_plaza_v1` |

**v1 charter deliverable:** fully author **3** recipes (bold rows) — enough to prove vocabulary + evaluator without blocking G1 gate on all six.

| Priority | Recipe id | Why |
|:---:|:---|:---|
| P0 | `block_recipe_industrial_yard_v1` | Aligns with existing G4 grammar pilots (logistics / manufacturing F-axis) |
| P0 | `block_recipe_low_density_res_v1` | Contrasts industrial — tests street-facing row primitive |
| P0 | `block_recipe_medium_density_res_v1` | Exercises multi-row depth + shared edge |

Defer `ForestPark`, `HighDensityCommercial`, `Civic` to **v1.1** after G1 gate witness.

---

## 3. Recipe vocabulary v1 (primitives)

Five primitives in v1 — extend only via versioned schema bump (`block_recipe_v2`).

### 3.1 `lot_row`

Repeated building lots along a block edge.

| Field | Type | Required | Notes |
|:---|:---|:---:|:---|
| `kind` | `"lot_row"` | ✓ | |
| `count` | u8 (1–8) | ✓ | Lots along edge |
| `depth` | u8 (1–4) | ✓ | Tiles deep from street |
| `facing` | `street` \| `interior` \| `alley` | ✓ | Relative to `BlockFrame.street_side` |
| `setback` | u8 | | Tiles from street edge (default 0) |
| `building_archetype` | grammar id string | ✓ | e.g. `CivicBlock` |
| `district_style` | string | | e.g. `industrial_west` — inherits block default if omitted |
| `lot_width` | u8 | | Default 1 tile; monotonic with footprint rules |

### 3.2 `edge`

Street-edge furniture / enclosure (may be visual-only).

| Field | Type | Required | Notes |
|:---|:---|:---:|:---|
| `kind` | `"edge"` | ✓ | |
| `asset` | `fence` \| `hedge` \| `trees` \| `lamp_row` \| `none` | ✓ | |
| `spacing` | f32 | | Tiles between repeats (default 1) |
| `offset` | u8 | | Tiles from street curb |
| `sim_authority` | bool | | default `false` |

### 3.3 `scatter`

Interior yard / park fill.

| Field | Type | Required | Notes |
|:---|:---|:---:|:---|
| `kind` | `"scatter"` | ✓ | |
| `asset` | `tree_small` \| `tree_large` \| `shrub` \| `parking_strip` | ✓ | |
| `density` | f32 0–1 | ✓ | Seeded placement count |
| `jitter` | f32 0–1 | | Tie-break noise amplitude (uses block_seed) |
| `zone` | `interior` \| `rear` | | Default `interior` |

### 3.4 `park_fill`

Open ground — no building lots.

| Field | Type | Required | Notes |
|:---|:---|:---:|:---|
| `kind` | `"park_fill"` | ✓ | |
| `coverage` | f32 0–1 | ✓ | Fraction of interior tiles |
| `surface` | `grass` \| `gravel` \| `paved` | | |

### 3.5 `plaza`

Hard-scaped civic void at street corner or mid-block.

| Field | Type | Required | Notes |
|:---|:---|:---:|:---|
| `kind` | `"plaza"` | ✓ | |
| `extent` | `[w, d]` u8 | ✓ | Tiles |
| `anchor` | `corner` \| `mid_block` \| `street_mouth` | ✓ | Relative to frame |
| `furniture` | list of `edge` assets | | Optional lamp/bench scatter |

**Evaluation order:** steps run **top to bottom**; later steps may not overlap earlier lot footprints (evaluator error).

---

## 4. BlockRecipe asset shape (RON)

Target schema: `tools/mcp/schemas/block_recipe_v1.schema.json` (coder-mcp, after sign-off).

```ron
(
    schema: "block_recipe_v1",
    version: "1.0.0",
    recipe_id: "block_recipe_industrial_yard_v1",
    block_archetype: Industrial,
    label: "Industrial yard block",
    default_district_style: "industrial_west",
    _meta: (
        teaches: ["lot_row", "edge", "scatter", "block_archetype:Industrial"],
    ),
    steps: [
        (
            kind: "lot_row",
            count: 2,
            depth: 2,
            facing: street,
            setback: 1,
            building_archetype: "IndustrialWarehouse",
            district_style: "industrial_west",
        ),
        (
            kind: "edge",
            asset: fence,
            spacing: 1.0,
            offset: 0,
            sim_authority: false,
        ),
        (
            kind: "scatter",
            asset: tree_small,
            density: 0.15,
            jitter: 0.2,
            zone: rear,
        ),
    ],
)
```

**On-disk paths (v1):**

| Recipe | Path |
|:---|:---|
| Industrial yard | `assets/configs/settlement/block_recipes/industrial_yard_v1.ron` |
| Low density res | `assets/configs/settlement/block_recipes/low_density_res_v1.ron` |
| Medium density res | `assets/configs/settlement/block_recipes/medium_density_res_v1.ron` |

JSON twins allowed under `tools/mcp/schemas/examples/block_recipe_*_v1.example.json` for MCP validation fixtures.

---

## 5. Three v1 recipe charters (designer-mcp authoring targets)

### 5.1 `block_recipe_industrial_yard_v1`

**Read:** Rail-adjacent warehouse row + rear utility scatter + perimeter fence.

| Step | Summary |
|:---|:---|
| `lot_row` | 2× depth-2 lots, street-facing, `IndustrialWarehouse` / `industrial_west`, setback 1 |
| `edge` | `fence` along street + alley sides |
| `scatter` | `tree_small` rear yard, density 0.15 |

**Pairs with:** existing logistics/manufacturing grammar pilots (G4 building set).

### 5.2 `block_recipe_low_density_res_v1`

**Read:** Single-family row — one deep lot per street frontage, hedge edge, no rear scatter.

| Step | Summary |
|:---|:---|
| `lot_row` | 2× depth-1 lots, `CivicBlock` / `colonial` district |
| `edge` | `hedge` street-facing |
| `park_fill` | rear 30% `grass` (driveway read) |

### 5.3 `block_recipe_medium_density_res_v1`

**Read:** Rowhouse strip — deeper lots, lamps on street edge.

| Step | Summary |
|:---|:---|
| `lot_row` | 4× depth-2, `CivicBlock`, shared party-wall implication (evaluator merges adjacent same archetype) |
| `edge` | `lamp_row` spacing 2.0 |
| `scatter` | `shrub` interior density 0.1 |

---

## 6. BlockFrame integration (C2 — coder; designer documents constraints)

Designer does **not** author `BlockFrame` math — only document placement semantics:

| `BlockFrame` field | Recipe consumer rule |
|:---|:---|
| `street_side` | `facing: street` aligns lot frontage here |
| `anchor` | Origin for lot grid; (0,0) = anchor tile |
| `extent` | Recipe must not emit lots outside `extent` |
| `orientation` | Clockwise from +X; alley = opposite street |

**Debug overlay (CITY-G1-C2-001):** color lots by `building_archetype`, edges green, scatter brown.

---

## 7. Example teachability (`_meta.teaches`)

Each example JSON/RON must include teach axes (per [`plan_mcp_grammar_build_set_guards_v1.md`](plan_mcp_grammar_build_set_guards_v1.md)):

```json
"_meta": {
  "teaches": ["lot_row", "edge", "scatter", "block_archetype:Industrial"]
}
```

Minimum **2** axes per example file for `example_teachable_audit` green.

---

## 8. Acceptance (designer-mcp sign-off)

| # | Check |
|:---:|:---|
| B1 | Vocabulary §3 — five primitives documented with required fields |
| B2 | Three recipe charters §5 — industrial + low + medium density |
| B3 | RON shape §4 — `schema: block_recipe_v1` with `steps[]` |
| B4 | Seed chain §1 — explicit `block_seed` → `lot_seed` handoff |
| B5 | Building archetype ids — only catalog ids (no new grammar families in v1) |
| B6 | `sim_authority` flagged on visual-only edge/scatter steps |
| B7 | Target asset paths listed; examples include `teaches[]` |
| B8 | **Critique pass** — designer-mcp rejects hand-coded layout in Rust for these three bands |

---

## 9. Coder exit (unblocks CITY-G1-C3-001)

After **PASS** on this charter + G0c green:

| Deliverable | Owner |
|:---|:---|
| `block_recipe_v1.schema.json` | coder-mcp |
| `block_recipe_evaluator` (deterministic) | coder |
| 3 RON files on disk per §4 | designer-mcp promotes from charter → `@coder-mcp` validate |
| Unit test: fixed `block_seed` ⇒ stable lot list hash | coder |
| G1 gate witness: one district · fixed seed · coherent blocks | coder |

**Regression:** G0c determinism witness stays green; `cargo test -p proc_A_dine01 --lib construction` + settlement tests.

---

## 10. Deferred (v1.1 — do not block G1)

| Item | Owner | Trigger |
|:---|:---|:---|
| `ForestPark` · `Civic` · `HighDensityCommercial` recipes | designer-mcp | G1 gate witness green |
| [`design_city_palette_variation_v1.md`](design_city_palette_variation_v1.md) (CITY-C5) | designer-mcp | **PASS** 2026-07-03 |
| APS block-tier UI / debug overlay copy | designer | **PASS** — [`design_city_block_debug_read_v1.md`](design_city_block_debug_read_v1.md) |
| BSN street furniture (C6 BSN part) | designer-mcp | **DR-CITY-C6-BSN** — after C6 visual charter |

---

## 11. Relationship to existing designer specs

| Existing spec | Relationship |
|:---|:---|
| [`design_civic_block_concept_v1.md`](design_civic_block_concept_v1.md) | **Building** archetype for lots inside `LowDensityRes` / `MediumDensityRes` blocks |
| [`design_aps_tag_tier2_v1.md`](design_aps_tag_tier2_v1.md) | Unrelated — per-building tags in APS |
| [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) | **Building** maturity G0–G4; block tier is new layer above |

---

## Sign-off

| Role | Verdict | Date | Notes |
|:---|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-07-03 | Critique + 3 RON + JSON examples on disk |
| `@planner-mcp` | **ACK** | 2026-07-03 | Aligned with `city_grammar_queue.json` seq 8 |
| `@coder` | — | | Evaluator after `CITY-G1-C2` + schema |

**Exit predicate:** `debug_runs/city_block_recipe_charter_live.json` **green** → unblocks `CITY-G1-C3-001` (evaluator; still needs `CITY-G1-C2`).

**Witness:** `debug_runs/city_block_recipe_charter_live.json`  
**CLI:** `python -m rust_engine_mcp.cli dmcp-city-block-recipe-witness`

---

## Changelog

| Date | Notes |
|:---|:---|
| 2026-07-03 | DRAFT skeleton — vocabulary + 3 recipe charters; blocked on G0c |
| 2026-07-03 | **PASS** — G0c done · 3 RON + JSON examples · witness green |
