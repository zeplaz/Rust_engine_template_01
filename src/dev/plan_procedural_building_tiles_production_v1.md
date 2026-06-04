# PLAN-PROC-TILE-PROD-001 — Procedural building iso tiles (production) `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-PROC-TILE-PROD-001** |
| **Owner** | `@planner` → `@designer-mcp` (bar + matrix) → `@coder-mcp` (bake) → `@coder` (runtime) |
| **Date** | 2026-06-03 |
| **Status** | **ACTIVE** (PT-0 **SIGNED** 2026-06-03) |
| **North star** | **Iso map tiles are the authoritative building read** at strategic/tactical zoom; **production-tier** assembled bakes; sim drives **variant_key** (lights, damage, construction, fire frames). |
| **Parents** | [`design_assembly_to_tile_coupling_v1.md`](design_assembly_to_tile_coupling_v1.md) · [`design_art_pipeline_suite_v1.md`](design_art_pipeline_suite_v1.md) · [`plan_tile_pipeline_automation_exec_v1.md`](plan_tile_pipeline_automation_exec_v1.md) · [`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md) |
| **Supersedes (scope)** | Treat **lod0 pilot tiles** as **spine proof only** — not ship art. PG-2 GLB extract is **witness/staging**, not product facade. |

---

## Problem (why this plan)

| Today | Gap |
|:---|:---|
| `TileAtlasRegistry` + map stamp **shipped** | **4 lod0 pilots**; runtime picks **2 keys** from coarse `SiteConstructionPhase` |
| `variant_set_v1` + APS Variants tab | Rich axes (lighting, damage, fill) **authored** but **not bound to sim** |
| PG-2 module GLBs at tactical LOD | Reads as “3D buildings” in planning; **product is isometric map** |
| Fire on map | **Terrain heat tint** only — no **building** fire sprite / frame cycle |
| `kit_production_*` | **Frozen** — tile bake still allowed on **lod0** (`development_tier: lod0`) |
| Legacy `assets/tiled/*.tmx` | Roads/rails tests — **not** procedural building loader |

**User directive:** solution at **final production level** — Object-Plus iso fidelity, deterministic sim→visual, no manual Blender primary path.

---

## Product rules (hard)

| Rule | Meaning |
|:---|:---|
| **Iso primary** | If atlas exists for `assembly_id`, **map stamp wins** over PG-2 mesh at all bands where map raster is visible. |
| **3D secondary** | `procedural_module_meshes` may remain for **tactical debug / assembly witness** until production tiles cover a style pack; default **off** when production atlas present (PT-4). |
| **No smoke tiles** | `DevelopmentTier::Smoke` never indexes, never bakes, never stamps. |
| **Bake from assembly** | Building iso PNGs from **assembled scene** only ([`design_assembly_to_tile_coupling_v1.md`](design_assembly_to_tile_coupling_v1.md)). |
| **Ship tier** | `_tile_atlas_index.ron` rows for player-facing art use `development_tier: production` + designer G4. |
| **Tri-mode** | Authoring: APS = MCP = CLI (`rust_engine_mcp`); engine reads **RON index + meta JSON** only. |

---

## End-state architecture

```text
MODULE KIT (kit_production_*)
    → STYLE PACK + FOOTPRINT GRAMMAR
    → ASSEMBLY SNAPSHOT (production tier)
    → VARIANT SET (canonical keys + sim_tags)
    → tile_batch_run (all variant_keys in matrix)
    → ATLAS PNG + atlas_meta.json
    → _tile_atlas_index.ron (production rows)
    → TileAtlasRegistry (startup)
    → TileVariantResolver (sim → variant_key)
    → map_tile_atlas_stamp → tile_world_fallback raster
    → minimap / tactical map (player read)

Optional overlay (same footprint):
    → fire_frame_* OR FireVisualFramesByView tint (PT-5 policy)
```

**Not in scope (this program):** TMX layer loader for buildings; per-module iso tiles; runtime procedural mesh generation.

---

## Production tier bar (Object-Plus)

Aligned with [`design_assembly_to_tile_coupling_v1.md`](design_assembly_to_tile_coupling_v1.md) § Module tier.

| Gate | Production module (`kit_production_*`) | Production tile bake |
|:---|:---|:---|
| Mesh | PBR materials shipped; ≤2k tris/slot; real-world proportion | Ortho bake from **production** assembly collection |
| References | `reference_tags` on assembly snapshot | Designer G4 sign-off YAML per archetype × style |
| Index | `_module_index.ron` tier `production` | `_tile_atlas_index.ron` tier `production` |
| Validator | `validate_asset_report` + tier filter | `tile_batch_validate` rejects `source_tier: lod0` for **ship** batches |
| Witness | `debug_runs/art_pipeline/kit_production_*_live.json` | `debug_runs/art_pipeline/tile_*_production_live.json` |

**lod0** remains for PG-2 spine tests and APS automation proofs — **excluded** from `TileVariantResolver` ship path and from FULL_APP “production tiles green” gate.

---

## Canonical variant catalog (PT-0)

Single cross-team vocabulary. Bakes **must** use these keys (or subset documented per archetype).

| Key pattern | Sim drivers (PT-4) | Visual intent |
|:---|:---|:---|
| `clean_day` | operational, day, power any | Default facade |
| `clean_night_off` | night, power off | Dark windows |
| `clean_night_on` | night, power on | Lit windows |
| `damaged_day` | damage > 0.25, day | Wear, no glow |
| `damaged_night_on` | damage + night + power on | Damaged + lit |
| `under_construction_01` … `_03` | `Planned` / `Surveying` / `Clearing` → `_01`; `Foundation` / `UnderConstruction` → `_02`; `Provisioning` → `_03` | Scaffold read |
| `abandoned` | `SiteConstructionPhase::Abandoned` | Collapsed / boarded |
| `burning_00` … `burning_07` | fire intensity at footprint | **8-frame** iso cycle (2×4 or 1×8 atlas row) |
| `ruined` | destroyed / removed prep | Rubble |

**Per-archetype matrix:** designer-mcp publishes `debug_runs/art_pipeline/variant_matrix_<archetype>_v1.yaml` — which keys are **required** vs **optional** for G4.

**Schema deliverable:** `tools/mcp/schemas/variant_catalog_v1.schema.json` + `assets/configs/buildings/_variant_catalog.ron` (global defaults + per-style overrides).

---

## Sim → variant resolver (PT-4 contract)

**New:** `src/construction/procedural/tile_variant_resolver.rs` (name flexible).

### Inputs

| Source | Fields used |
|:---|:---|
| `ConstructionSite` | `phase` |
| `PlannedSite` / catalog | `catalog_id`, footprint |
| `ProceduralBuildingSpec` | `style`, `seed`, `floors` |
| `TileAtlasEntry` | available `variants` keys |
| `VariantCatalog` (RON) | priority rules, fallbacks |
| **Power** (when wired) | site or grid `power_available` |
| **Time** (when wired) | sim clock / `FireAtmosphereAggregate` day fraction → day/night |
| **Damage** (when wired) | site damage scalar or building health component |
| **Fire** (when wired) | max heat in footprint tiles from fire sim |

### Output

```rust
pub struct ResolvedTileVariant {
    pub variant_key: String,
    pub animation_frame: Option<u8>, // Some(0..7) for burning_* prefix
}
```

### Resolution order (deterministic)

1. If footprint fire heat ≥ `burn_threshold` → `burning_{frame}` where `frame = (sim_tick / fire_frame_ms) % 8`.
2. Else match **construction phase** → `under_construction_*` or `abandoned` / `ruined`.
3. Else match **damage band** + **day/night** + **power** → `clean_*` / `damaged_*`.
4. Fallback: nearest key present in atlas `variants` map (documented in catalog); never smoke; never missing-key panic in release — log + `clean_day` if present.

### Wire point

Replace [`variant_key_for_site_phase`](../../src/gui/map_tile_atlas_stamp.rs) stub with resolver; keep function as thin delegate for tests.

**Tests:** `cargo test -p proc_A_dine01 tile_variant_resolver --lib` — table-driven cases for night+power, fire frames, missing key fallback.

---

## Fire on buildings (PT-5)

Two allowed implementations — **pick one in PT-5 spike** (planner default: **A**).

| Option | Mechanism | Pros | Cons |
|:---|:---|:---|:---|:---|
| **A — Atlas frames** | `burning_00`…`07` in same atlas; resolver advances frame | Matches Republic static+swap; works on CPU raster | 8 bakes per building; atlas wider |
| **B — Map overlay** | Reuse fire heat tint **masked to footprint** from building tile alpha | Fewer bakes | Less facade-specific; harder at iso angle |

**Program acceptance:** Option A for **production** rowhouses/industrial pilots; Option B only as **fallback** when atlas has no fire keys.

**Not:** 3D particle fire on building mesh as primary read (existing `FireVisualFramesByView` stays terrain/VFX lane).

---

## PG-2 / 3D demotion (PT-4)

| Band | Policy after PT-4 |
|:---|:---|
| Strategic / Macro | **No** procedural module meshes (unchanged); iso stamp if atlas registered |
| Tactical / Operational | If `TileAtlasRegistry::atlas_for_assembly` hits **production** entry → set `procedural_module_meshes: false` in `RepresentationResult` |
| Debug env | `RUST_ENGINE_FORCE_PG2_MESHES=1` overrides for assembly witness |

Document in [`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md) § PG-2 mesh authority addendum.

---

## Phase map

### PT-0 — Catalog + plan sign-off `@planner` — **SIGNED**

| ID | Deliverable | Acceptance | Status |
|:---|:---|:---|:---:|
| PT-0-001 | This doc + index links | Linked from growth index + development_plan_index | **PASS** |
| PT-0-002 | `variant_catalog_v1.schema.json` + `_variant_catalog.ron` | jsonschema pass on `variant_catalog_v1.example.json`; pytest `test_variant_catalog` | **PASS** |
| PT-0-003 | Witness spec | [`procedural_tiles_production_witness_v1.md`](procedural_tiles_production_witness_v1.md) | **PASS** |
| PT-0-004 | PT-0 closure witness | [`debug_runs/art_pipeline/plan_pt0_procedural_tiles_live.json`](../../debug_runs/art_pipeline/plan_pt0_procedural_tiles_live.json) | **PASS** |
| PT-0-005 | Sign-off record | [`plan_pt0_signoff_procedural_tiles_v1.md`](plan_pt0_signoff_procedural_tiles_v1.md) | **PASS** |

**Unblocks:** **PT-1** (`@designer-mcp`) — variant matrices must use `canonical_variant_keys` + `ship_minimum_keys` from [`_variant_catalog.ron`](../../assets/configs/buildings/_variant_catalog.ron).

**Phase keys:** `construction_phase_keys` in catalog align with `SiteConstructionPhase` (`Planned` … `Abandoned` in `src/strategic/site/resources.rs`) — not legacy `ConstructionStates`.

---

### PT-1 — Production art bar + variant matrix `@designer-mcp`

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| PT-1-001 | `design_procedural_tile_production_bar_v1.md` | Object-Plus rubric: silhouette, PBR, entourage, light rig |
| PT-1-002 | Variant matrix YAML × **7 style packs** × **4 archetypes** (rowhouse, warehouse, shopfront, bunker) | Files under `debug_runs/art_pipeline/variant_matrix_*` |
| PT-1-003 | G4 sign-off template | Extends PG-2 sign-off; per-atlas `*_production_signoff.yaml` |
| PT-1-004 | Reference tags required on production assembly snapshots | Validator rule in MCP G0 rules |

**Blocks:** PT-2 production bakes without PT-1-002 matrix for that archetype.

---

### PT-2 — Production module + tile bake `@coder-mcp`

**Bake spine (mandatory):** [`design_tile_bake_spine_convergence_v1.md`](design_tile_bake_spine_convergence_v1.md) — `keyframe_render` → PNG folder → `tile-atlas-pack`; `bake_source: keyframe_pack` on ship batches.

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| PT-2-001 | `kit_production_001` wave plan (40 modules, 8×5) | [`plan_kit_lod0_roadmap_v1.md`](plan_kit_lod0_roadmap_v1.md) parallel structure, tier `production` |
| PT-2-002 | Promote **4 pilot assemblies** to production GLBs + **keyframe** re-bake all matrix keys | `development_tier: production`; not lod0 ortho stub atlases |
| PT-2-003 | `tile_batch_validate` — fail ship batches with `source_tier: lod0` when `ship: true` | pytest |
| PT-2-004 | Atlas register — all production rows in `_tile_atlas_index.ron` | ≥4 atlases, ≥6 variant keys each (incl. night + fire row) |
| PT-2-005 | APS Atlas tab — filter **production** vs lod0 | Tri-mode witness |

**Witness:** `debug_runs/art_pipeline/procedural_tiles_production_bake_live.json` — `dry_run: false`, `tier: production`, `variant_count ≥ 6`.

---

### PT-3 — Variant set + sim tags `@coder-mcp` + `@planner-mcp`

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| PT-3-001 | `variant_set_v1` — `sim_tags` on each variant row | e.g. `sim_night`, `sim_power_on`, `sim_fire` |
| PT-3-002 | `variant_catalog.ron` — maps tag combinations → `variant_key` | Loaded by engine PT-4 |
| PT-3-003 | `tile_batch_run` expands Cartesian product from matrix (automated) | No hand-picked 2-variant pilots |
| PT-3-004 | MCP `variant_matrix_expand` tool | CLI parity |

---

### PT-4 — Engine resolver + iso primary `@coder`

| ID | Deliverable | Files | Acceptance |
|:---|:---|:---|:---|
| PT-4-001 | `TileVariantResolver` + `VariantCatalog` resource | `src/construction/procedural/tile_variant_resolver.rs`, load RON | Unit tests table |
| PT-4-002 | Wire `stamp_request_for_site` | `map_tile_atlas_stamp.rs` | Integration test with pilot atlas |
| PT-4-003 | `RepresentationResult` — suppress PG-2 meshes when production atlas hit | `representation_policy.rs` | Test: tactical + atlas → `procedural_module_meshes == false` |
| PT-4-004 | Power + day/night inputs | Hook `FireAtmosphereAggregate` / grid resource when available; stub constants behind `cfg(test)` | Documented TODOs with witness keys |
| PT-4-005 | Damage input | Site or building component scalar | Maps to damaged_* keys |

**Witness:** `debug_runs/procedural_tiles_runtime_live.json` — `resolver_version`, sample sites with expected `variant_key`, `stamp_applied: true`.

---

### PT-5 — Fire frame animation `@coder` + `@coder-mcp`

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| PT-5-001 | Bake `burning_00`…`07` for 4 production atlases | UVs in meta |
| PT-5-002 | Resolver frame tick from `SimStepStamp` | Deterministic frame index |
| PT-5-003 | Map raster refresh policy — stamp dirty on fire band change | No full-map rebuild every frame unless needed; subregion dirty rect |

---

### PT-6 — Program close + playtest `@orchestrator-mcp` + `@designer-mcp`

| ID | Gate | Pass when |
|:---|:---|:---|
| **TILE-PROD-001** | Production bake | All 4 archetype atlases `tier: production`, ≥6 variants, real PNG |
| **TILE-PROD-002** | Registry | Engine loads all rows; UV resolve for every matrix key |
| **TILE-PROD-003** | Resolver | Runtime witness: night+power+damage+fire cases |
| **TILE-PROD-004** | UX sign-off | Designer YAML per atlas (lights readable at 128px iso) |
| **TILE-PROD-005** | FULL_APP | 10 min sim: committed sites show iso swap; PG-2 meshes off when atlas present |
| **TILE-PROD-006** | Play scenario | G-PLAY slice includes procedural district with fire event |

**Program green file:** `debug_runs/art_pipeline/procedural_tiles_production_program_green_live.json` (mirrors `mcp_art_program_green_live.json` shape).

---

## Witness spec (PT-0-003)

**Path:** [`procedural_tiles_production_witness_v1.md`](procedural_tiles_production_witness_v1.md) (companion doc).

Minimum JSON fields:

```json
{
  "program_id": "PLAN-PROC-TILE-PROD-001",
  "green": false,
  "gates": {
    "TILE-PROD-001": { "pass": false, "witness": "..." },
    "TILE-PROD-002": { "pass": false },
    "TILE-PROD-003": { "pass": false },
    "TILE-PROD-004": { "pass": false },
    "TILE-PROD-005": { "pass": false },
    "TILE-PROD-006": { "pass": false }
  },
  "production_atlas_count": 0,
  "lod0_atlas_ship_allowed": false
}
```

---

## Fleet dispatch (orchestrator)

| Order | Owner | Depends | Queue row |
|:---|:---|:---|:---|
| **MCP-PT-1-001** | designer-mcp | — | variant matrices + production bar |
| **MCP-PT-2-001** | coder-mcp | PT-1-002 | kit_production_001 + production re-bake |
| **MCP-PT-3-001** | coder-mcp | PT-2-004 | variant_matrix_expand + sim_tags |
| **ENG-PT-4-001** | coder | PT-3-002 | TileVariantResolver |
| **ENG-PT-5-001** | coder | PT-4-001, PT-2 fire bakes | fire frames |
| **ORCH-PT-6-001** | orchestrator-mcp | all gates | program green witness |

**Parallel:** PT-2 module art can overlap PT-1 matrices for **lod0→production uplift** on existing 50 modules (designer review per module).

**Do not:** expand PG-4 grammar or organic growth until **TILE-PROD-003** pass (resolver stable).

---

## Dependencies (existing green)

| Prerequisite | Status |
|:---|:---|
| TILE-REAL-001 / TILE-ENGINE-001 | **PASS** (lod0 pilots) |
| PG-2-WIT | **PASS** (assembly spine) |
| APS suite | **PASS** (authoring) |
| AUTO-001…011 tile automation | **SHIPPED** per exec plan |

This program **promotes** tier and **closes the runtime gap** — does not redo automation spine.

---

## Risks

| Risk | Mitigation |
|:---|:---|
| Production art slower than lod0 | Phase PT-2 in waves; ship archetypes incrementally (rowhouse first) |
| Atlas size / memory | Cap variant count per building; pack 128px; lazy GPU cache (existing) |
| Resolver without power sim | Stub `power_on: true` until grid wired; witness documents stub |
| Perf: fire frame dirty | Subregion stamp only; cadence from `VisualCadence` |

---

## Non-goals (this program)

- Replacing terrain `bevy_ecs_tilemap` chunks with building tiles
- TMX import for procedural buildings
- Runtime Blender / LLM-generated PNGs
- `kit_production` for all 50 modules before first 4-atlas ship gate

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Initial production iso-tile program — resolver, fire frames, tier bar |
| v1.0.1 | 2026-06-03 | PT-0 signed — catalog aligned to `SiteConstructionPhase`; schema + pytest |
