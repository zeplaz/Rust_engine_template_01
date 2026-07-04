# City kit × palette variation charter `v1` — DES-CITY-PALETTE-VARIATION-001

| Field | Value |
|:---|:---|
| **ID** | **DES-CITY-PALETTE-VARIATION-001** |
| **Issue** | CITY-C5 |
| **Parent** | [`plan_city_grammar_upgrade_v1.md`](plan_city_grammar_upgrade_v1.md) § G2 · bevy_city **B4** |
| **Date** | 2026-07-03 |
| **Owner** | `@designer-mcp` (charter + critique) → `@coder` + `@coder-mcp` (index + bake) |
| **Status** | **SIGNED** — `CITY-G1-C3-001` done · charter + 3 palette catalogs on disk |
| **Queue** | [`city_grammar_queue.json`](../../tools/orchestrator/queues/city_grammar_queue.json) seq 10 |
| **Verdict** | **PASS** — designer-mcp sign-off 2026-07-03 |

```yaml
order_critique:
  request_summary: "Kit × palette variation axis — module_index + tile_atlas_index charter for G2"
  rules_audit:
    deterministic_seed_resolution: pass
    data_not_code: pass
    style_pack_authority: pass
    no_sim_gameplay_in_palette: pass
    g1_block_recipe_alignment: pass
  blocked: false
  proceed: yes
  foresight_flags:
    - "palette_catalog_v1.schema.json — @coder-mcp after sign-off"
    - "CITY-G2-C5-001 resolver in module_index.rs + tile_variant_resolver.rs"
    - "BQ-K4 defers implementation detail here — do not duplicate in BQ plan"
```

```text
DES-CITY-PALETTE-VARIATION-001
Kit × palette variation axis — combinatorial streetscape variety from small asset counts
```

---

## 0. Why this exists (gap today)

```text
today:   1 module_id ⇒ 1 visual (material_profile fixed on index row)
target:  module_id × palette_variation ⇒ N visuals from same massing kit (seeded)
```

| Lane | Authority today | After G2-C5 |
|:---|:---|:---|
| **GLB module kit** | `_module_index.ron` · single `material_profile` per row | + `palette_family` · resolver picks `variation_id` from `lot_seed` |
| **Tile atlas** | `variant_key` + UV in atlas meta | + `palette_column` suffix or lookup dimension |
| **Block recipes** | `district_style` on `lot_row` (C3) | inherits `style_pack` → default `palette_family` |
| **Building grammar** | `material_profiles` per slot in grammar RON | palette **overrides** slot map at resolve time (presentation) |

**bevy_city reference:** 5–12 meshes × 3–4 colormap palettes ⇒ 36+ distinct reads. We ship **1 visual per variant id** until this axis lands.

**Not in scope:** APS mandate tags, Variants panel layer merge, weathering simulation — those stay **building-tier** ([`design_aps_tag_tier2_v1.md`](design_aps_tag_tier2_v1.md)).

---

## 1. Design principles

| # | Rule |
|:---:|:---|
| P1 | **Deterministic** — same `lot_seed` + `module_id` + `palette_id` ⇒ same `variation_id` (G0c net preserved) |
| P2 | **Data not code** — palettes are RON/JSON catalogs; no `if industrial { rust }` in Rust |
| P3 | **Style-pack scoped** — each palette catalog declares `style_pack`; block `district_style` selects family |
| P4 | **Presentation only** — palette choice must not alter logistics, occupancy, or execute funnel |
| P5 | **Kit orthogonality** — massing kit (`module_id`) and palette (`variation_id`) are independent axes |
| P6 | **Teachable examples** — each catalog `_meta.teaches[]` ≥ 2 axes |

**Seed resolution (coder must implement):**

```text
world_seed → … → lot_seed(lot_idx) → building_grammar seed
                                    ↘ palette_pick = stable_u32(lot_seed, module_id, palette_family) % variation_count
```

**Visual identity string (v1):**

```text
visual_variant_id = "{module_id}::{palette_id}::{variation_id}"
```

---

## 2. Palette catalog vocabulary v1

Target schema: `tools/mcp/schemas/palette_catalog_v1.schema.json` (coder-mcp, after sign-off).

### 2.1 `PaletteCatalog` asset

| Field | Type | Required | Notes |
|:---|:---|:---:|:---|
| `schema` | `"palette_catalog_v1"` | ✓ | |
| `palette_id` | string | ✓ | e.g. `palette_industrial_west_v1` |
| `style_pack` | string | ✓ | Must match `module_index` style_pack ids |
| `label` | string | ✓ | Artist-facing |
| `variations` | list | ✓ | 2–6 entries in v1 |
| `_meta.teaches` | string[] | ✓ | Audit axes |

### 2.2 `PaletteVariation` entry

| Field | Type | Required | Notes |
|:---|:---|:---:|:---|
| `variation_id` | string | ✓ | Stable id — `iw_rust_clean` |
| `label` | string | ✓ | Plain language for APS/debug |
| `material_slots` | map slot → profile id | ✓ | Overrides grammar defaults |
| `atlas_tint` | `[r,g,b]` f32 | | Tile lane only — optional wash |
| `variant_tags` | string[] | | Optional APS mandate hints (presentation metadata) |

**Slot keys (v1 — align with grammar slots):** `wall_primary`, `wall_secondary`, `trim`, `roof`, `door`, `window`, `foundation`.

### 2.3 Module index extension (CITY-G2-C5-001)

Add optional fields on `_module_index` rows (backward compatible — default = legacy single-profile):

| Field | Type | Default | Notes |
|:---|:---|:---|:---|
| `palette_family` | string | `""` | Links to `palette_id` without version suffix |
| `palette_variation_count` | u8 | `1` | Cache for resolver; must match catalog len |
| `default_variation_id` | string | `""` | Fallback when seed disabled (editor preview) |

When `palette_family` empty → current behavior (row `material_profile` only).

### 2.4 Tile atlas extension (CITY-G2-C5-001)

| Mechanism | v1 choice |
|:---|:---|
| Variant naming | `{base_variant}__pal_{variation_id}` in atlas meta `lookups` |
| UV layout | Same footprint UVs; **palette column** = separate PNG strip or shared atlas row per [`tile-generation`](../../.cursor/skills/tile-generation/SKILL.md) state-machine |
| `style_pack_id` | Already on index row — palette must match |

**Rule:** Strategic zoom may collapse to default variation; tactical uses seeded pick.

---

## 3. Three v1 palette charters (aligned with G1 block recipes)

| Priority | Palette id | Style pack | Block recipe pairing | Variation count |
|:---:|:---|:---|:---|:---:|
| P0 | `palette_industrial_west_v1` | `style_industrial_west` | `block_recipe_industrial_yard_v1` | 3 |
| P0 | `palette_colonial_res_v1` | `style_colonial` | `block_recipe_low_density_res_v1` | 3 |
| P0 | `palette_rowhouse_urban_v1` | `style_victorian` | `block_recipe_medium_density_res_v1` | 4 |

### 3.1 `palette_industrial_west_v1`

**Read:** Corrugated steel yard — rust vs clean vs weathered paint.

| variation_id | Player read | Key slots |
|:---|:---|:---|
| `iw_rust_clean` | New galvanised hall | corrugated clean / grey roof |
| `iw_rust_weathered` | Working yard age | rust streak wall / dark roof |
| `iw_painted_blue` | Municipal utility read | blue paint trim / white roof |

### 3.2 `palette_colonial_res_v1`

**Read:** Detached colonial — brick vs clapboard vs stone foundation accent.

| variation_id | Player read | Key slots |
|:---|:---|:---|
| `cr_brick_red` | Red brick front | brick wall / white trim |
| `cr_clapboard_white` | Painted clapboard | white siding / dark shutters |
| `cr_stone_accent` | Stone base row | mixed stone foundation / wood upper |

### 3.3 `palette_rowhouse_urban_v1`

**Read:** Shared party-wall row — repeating façade with door colour variation.

| variation_id | Player read | Key slots |
|:---|:---|:---|
| `rh_brownstone` | Classic brownstone | dark stone / iron trim |
| `rh_painted_row` | Painted row | pastel wall / coloured doors |
| `rh_brick_party` | Red brick party wall | shared brick / subtle door shift |
| `rh_rehab_mixed` | Renovation mix | patched wall / new trim |

---

## 4. On-disk paths (v1)

| Palette | RON | JSON example |
|:---|:---|:---|
| Industrial west | `assets/configs/buildings/palettes/industrial_west_v1.ron` | `tools/mcp/schemas/examples/palette_industrial_west_v1.example.json` |
| Colonial res | `assets/configs/buildings/palettes/colonial_res_v1.ron` | `tools/mcp/schemas/examples/palette_colonial_res_v1.example.json` |
| Rowhouse urban | `assets/configs/buildings/palettes/rowhouse_urban_v1.ron` | `tools/mcp/schemas/examples/palette_rowhouse_urban_v1.example.json` |

**Index hook (coder — post charter):**

| File | Change |
|:---|:---|
| `assets/configs/buildings/_palette_catalog_index.ron` | Lists active palette catalogs (v1 optional manifest) |
| `_module_index.ron` | Add `palette_family` on pilot modules per style pack |
| `_tile_atlas_index.ron` | Add `__pal_*` variant keys on production atlases |

---

## 5. Block recipe ↔ palette handoff

Block recipes already declare `district_style` on `lot_row`:

```ron
district_style: "industrial_west",
```

**Mapping table (v1 — data in palette catalog, not block recipe):**

| `district_style` | Default `palette_id` |
|:---|:---|
| `industrial_west` | `palette_industrial_west_v1` |
| `colonial` | `palette_colonial_res_v1` |
| `victorian` / rowhouse bands | `palette_rowhouse_urban_v1` |

Evaluator / lot spawner passes `palette_id` into building grammar resolve — **does not** embed variation index in recipe (seed picks variation).

---

## 6. Acceptance (designer-mcp sign-off)

| # | Check |
|:---:|:---|
| C1 | §2 vocabulary — catalog + variation + module/tile extensions documented |
| C2 | Three palette charters §3 — industrial + colonial + rowhouse |
| C3 | RON shape §4 — `schema: palette_catalog_v1` with `variations[]` |
| C4 | Seed resolution §1 — explicit `lot_seed` modulo pick |
| C5 | Style pack ids match existing `_module_index` style_pack values |
| C6 | Presentation-only — no gameplay fields in palette catalog |
| C7 | Examples include `teaches[]` ≥ 2 axes each |
| C8 | **Critique pass** — rejects hard-coded colormap branches in Rust |

---

## 7. Coder exit (unblocks CITY-G2-C5-001)

After **PASS** on this charter + G1-C3 green:

| Deliverable | Owner |
|:---|:---|
| `palette_catalog_v1.schema.json` | coder-mcp |
| `resolve_palette_variation(lot_seed, module_id, palette_id)` | coder |
| `_palette_catalog_index.ron` + load in procedural init | coder |
| Pilot `palette_family` on 6–12 module rows | coder-mcp |
| Atlas `__pal_*` variant keys on 1 production atlas | coder-mcp |
| Unit test: fixed seed ⇒ stable `visual_variant_id` | coder |
| G2 witness: 3 palettes × 3 seeds ⇒ stable hashes | coder |

**Regression:** G0c + G1-C3 witnesses stay green; `cargo test -p proc_A_dine01 --lib construction::procedural`.

---

## 8. Relationship to adjacent programs

| Program | Relationship |
|:---|:---|
| [`design_city_block_recipe_v1.md`](design_city_block_recipe_v1.md) | Block `district_style` selects palette family |
| [`plan_building_quality_v1.md`](plan_building_quality_v1.md) BQ-K4 | Implementation owned here — BQ-K2 complete kits prerequisite |
| APS-G4 coverage | **Green 2026-07-03** — `pilot_hardcode` gate cleared for C5 |
| [`design_aps_tag_tier2_v1.md`](design_aps_tag_tier2_v1.md) | Optional `variant_tags` on palette rows — not required for G2 gate |

---

## 9. Deferred (v1.1)

| Item | Trigger |
|:---|:---|
| Weathering progression bands (age → palette lerp) | BQ-K3 |
| Per-slot normal-map variants | Production art batch |
| Minimap palette collapse rules | Designer after G2 witness |
| APS Variants panel palette picker | After resolver wired |

---

## Sign-off

| Role | Verdict | Date | Notes |
|:---|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-07-03 | Critique + 3 RON + JSON examples |
| `@planner-mcp` | **ACK** | 2026-07-03 | Aligned with `city_grammar_queue.json` seq 10 |
| `@coder` | — | | CITY-G2-C5-001 after schema |

**Exit predicate:** `debug_runs/city_palette_variation_charter_live.json` **green** → unblocks `CITY-G2-C5-001`.

**Witness:** `debug_runs/city_palette_variation_charter_live.json`  
**CLI:** `python -m rust_engine_mcp.cli dmcp-city-palette-variation-witness`

---

## Changelog

| Date | Notes |
|:---|:---|
| 2026-07-03 | **PASS** — G1-C3 done · 3 palette catalogs · witness green |
