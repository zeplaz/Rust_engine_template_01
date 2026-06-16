# PLAN-TILE-BATCH-V1 — Tile lane architecture (spec-only) `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-TILE-BATCH-001** |
| **Parent** | [`plan_mcp_pipeline_recovery_and_agent_fleet_v1.md`](plan_mcp_pipeline_recovery_and_agent_fleet_v1.md) (reconciled **R0+FLT DONE**) |
| **Sprint** | MCP next — **Phase 0 @planner-mcp** (read-only, no execution) |
| **Owner** | `@planner-mcp` |
| **Date** | 2026-06-02 |
| **Status** | **SIGNED** — **MCP-T0-001** (Wave 2); execution **DEFER** — hand **MCP-T0-002** to @coder-mcp |
| **Fleet gate** | [`mcp_fleet_wave2_orders_v1.md`](mcp_fleet_wave2_orders_v1.md) Stream TILE |

---

## Summary

Geometry/module kit lane is **SHIPPED** (20 modules, `_module_index.ron`, validators, witnesses). **Tile lane** stays **spec-only** until `tile_batch` / `tile.generate` tooling exists. This plan defines **`tile_batch_v1`** JSON schema, atlas contract, gate mapping **G0–G5**, and separates **MCP tile artifacts** from **Bevy module GLB binding** (already partially wired via `procedural_module_id`).

**Architecture decision:** Tile batch uses the **same orchestrator pattern** as geometry (batch_id → variants → validate → promote → index), but outputs **2D atlas PNGs + `atlas_meta.json`**, not GLB. **`tile.generate` remains PLANNED** — no agent may call it until MCP-HARD-003 + toolchain ship.

**Coupling (authoritative):** Building iso tiles bake from **assembled 3D buildings**, not raw module GLBs. See [`design_assembly_to_tile_coupling_v1.md`](design_assembly_to_tile_coupling_v1.md). `tile_batch_v1` gains optional `assembly_ref` (required for building tiles); bake blocked until PG-2 assembly snapshot exists.

---

## Order critique

| Question | Verdict |
|:---|:---|
| Reuse `rust-engine-art` MCP server? | **Yes** — add `tile_batch_validate` / stub `tile_batch_run` later; no second server |
| Execute draft `tile_batch_factory_floor_v1.json` now? | **No** — violates recovery plan + designer-mcp lane C rules |
| Single-tile MCP calls? | **Rejected** — `batch_processing` rule requires batch + atlas plan |
| Blender for tiles vs geometry? | **Same Blender headless host** — different bpy op graph (`tile_ortho_bake`), not `module_wall` |
| Tile atlas in Bevy now? | **DEFER** — `@planner` + `@coder` define `TileVariant` + atlas loader after schema frozen |

---

## SHIPPED / PLANNED / DEFER

| Component | Label | Notes |
|:---|:---:|:---|
| Geometry spine (spec → job → glb → promote) | **SHIPPED** | Tier 1 + Tier 2 bpy |
| `_module_index.ron` + `library_*` | **SHIPPED** | G5 module catalog |
| `validate_glb_asset` / `validate_asset_report` | **SHIPPED** | GLB only |
| `validate-report mcp_*` | **SHIPPED** | Schema validators |
| Draft `tile_batch_factory_floor_v1.json` | **PLANNED** | Drafts folder only |
| `tile_batch_v1.schema.json` | **PLANNED** | This plan → coder-mcp Phase T0 |
| `tile.generate` MCP tool | **PLANNED** | Do not schedule |
| `tile_batch` CLI/MCP runner | **PLANNED** | MCP-HARD-003 stub `not_implemented` first |
| `tools/tile/tile_generator.py` | **PLANNED** | Draft path in skill reference |
| `tools/utils/atlas_packer.py` | **PLANNED** | Post-bake pack |
| `tile_validator` / `tile_validator` report | **PLANNED** | PNG dimensions, naming, atlas UV |
| Bevy `Tile` + atlas swap | **DEFER** | Engine lane — `@planner` |
| `RepresentationResult` tile layer | **DEFER** | Stage 5 contract — not MCP scope |

---

## Target architecture (tile lane)

```text
@designer-mcp (G0–G1)
  tile_batch_v1 JSON (all variant axes explicit, seed, batch_id)
  → spec_validate_tile_batch (PLANNED validator)
  → G4 sign-off YAML (atlas readability checklist)

@coder-mcp (G2–G3, when SHIPPED)
  tile_batch_run → Blender ortho bake loop
  → assets/staging/tiles/<batch_id>/*.png
  → atlas_packer → assets/textures/tiles/<batch_id>_atlas.png + atlas_meta.json
  → tile_validate_batch → ValidationReport

@planner (DEFER)
  TileVariant ECS + sim state → variant_key
  Atlas loader + RepresentationResult / map view binding
```

**Forbidden:** `tile.generate` in agent prompts until registry row says **SHIPPED**.

---

## Schema plan — `tile_batch_v1`

**Path (promote from draft):**
- Draft: `tools/mcp/schemas/drafts/tile_batch_*.json`
- Canonical: `tools/mcp/schemas/tile_batch_v1.schema.json`
- Examples: `tools/mcp/schemas/examples/tile_batch_factory_floor_v1.json`

### Required top-level fields

| Field | Type | Rule |
|:---|:---|:---|
| `schema_version` | `1` | const |
| `batch_id` | string | `^[a-z][a-z0-9_]*$` — same discipline as geometry batches |
| `tile_id` | string | Catalog id e.g. `factory_floor` |
| `base` | enum | `wood` \| `stone` \| `concrete` \| `dirt` \| `asphalt` \| `metal_plate` |
| `rules_applied` | string[] | Must include four production rules ids |
| `render` | object | See below |
| `variants` | array | **Min 2** — no orphan single tile |
| `atlas` | object | Target atlas name, tile size px, padding |
| `expected_outputs` | string[] | Manifest for witness |

### `render` object

```json
{
  "method": "blender_orthographic_iso",
  "isometric": true,
  "seed": 42,
  "tile_size_px": 128,
  "camera_elevation_deg": 35.264
}
```

- `seed` **required** (deterministic_output)
- `method` enum — only `blender_orthographic_iso` in v1 (no diffusion)

### `variants[]` item (all axes explicit per variant)

| Field | Type | Required |
|:---|:---|:---:|
| `variant_key` | string | auto or explicit — see naming |
| `state` | enum | clean \| dirty \| damaged \| ruined |
| `damage` | number 0–1 | yes |
| `power` | enum | off \| partial \| on |
| `fill` | enum | empty \| quarter \| half \| full |
| `lighting` | enum | day \| night_off \| night_on |

**Variant key naming (deterministic):**

```text
{tile_id}_{base}_s{state}_d{damage*100}_p{power}_f{fill}_l{lighting}
```

Example: `factory_floor_concrete_sdamaged_d45_pon_fhalf_lnight_on`

### `atlas` object

```json
{
  "atlas_id": "factory_floor_greybox_001",
  "columns": 4,
  "rows": 2,
  "tile_px": 128,
  "padding_px": 2,
  "output_png": "assets/textures/tiles/factory_floor_greybox_001_atlas.png",
  "meta_json": "assets/textures/tiles/factory_floor_greybox_001_atlas_meta.json"
}
```

### Witness (post-execution, PLANNED)

`debug_runs/art_pipeline/tile_<batch_id>_live.json` — mirror geometry witness shape:

```json
{
  "batch_id": "tile_factory_floor_greybox_001",
  "gates": { "G0": "pass", "G1": "pass", "G3": "pass", "G4": "pass" },
  "variants": [{ "variant_key": "...", "valid": true, "png": "..." }],
  "atlas": { "path": "...", "variant_count": 6 }
}
```

---

## Gate alignment (orchestrator-mcp) — Lane C spec-only sprint

| Gate | Owner | Tile lane (this sprint) | Blocks |
|:---|:---|:---|:---|
| **G0** | designer-mcp | `rules_audit` for tile batch | all tile work |
| **G1** | designer-mcp | `tile_batch_v1` JSON valid against schema (manual/jsonschema until tool) | bake |
| **G2** | planner-mcp + coder-mcp | Schema file + `validate-report` stub / `not_implemented` runner | execution |
| **G3** | coder-mcp | N/A until bake — **skip** | promote |
| **G4** | designer-mcp | Sign-off: variant axes readable, atlas grid sane | promote |
| **G5** | `@coder` | Tile atlas registry in Bevy — **not** `_module_index.ron` | engine |

**This sprint stops at G1 (+ G2 schema PR).** Do not schedule G3–G5 for tiles.

---

## Implementation phases (for orchestrator-mcp)

### Phase T0 — Schema only (~3 days, coder-mcp)

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| **TILE-SCHEMA-001** | `tile_batch_v1.schema.json` | jsonschema validates draft + example |
| **TILE-SCHEMA-002** | Move example to `schemas/examples/` | Remove `status: PLANNED` from example; add `batch_id` |
| **TILE-SCHEMA-003** | `validate-report tile_batch <path>` | PLANNED validator id; returns ValidationReport |
| **TILE-SCHEMA-004** | MCP `tile_batch_validate` | CLI/MCP parity |

### Phase T1 — Tool stub (~2 days, coder-mcp)

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| **TILE-TOOL-001** | CLI `tile-batch-run` returns `not_implemented` + link to plan | pytest |
| **TILE-TOOL-002** | Registry + README SHIPPED/PLANNED row | |

### Phase T2 — Bake + atlas (DEFER until T0–T1 green)

| ID | Deliverable | Label |
|:---|:---|:---|
| **TILE-BAKE-001** | `tools/tile/tile_generator.py` | PLANNED |
| **TILE-BAKE-002** | `tile_batch_run` MCP | PLANNED |
| **TILE-BAKE-003** | `atlas_packer.py` | PLANNED |
| **TILE-BAKE-004** | First batch witness | PLANNED |

---

## MCP tools plan (future names — all PLANNED)

| Tool | Phase | Input | Output |
|:---|:---:|:---|:---|
| `tile_batch_validate` | T0 | path or JSON | ValidationReport — **SHIPPED** (MCP + CLI) |
| `tile_atlas_pack_tool` | T0 | PNG folder | atlas via `utils/tilemapgen` — **SHIPPED** |
| `lod0_batch_run_tool` | ART | batch_id + phase | procedural modules — **SHIPPED** |
| `tile_batch_run` | T2 | tile_batch_v1 path | ortho bake — **PLANNED** (`not_implemented`) |
| `tile_batch_status` | T2 | batch_id | status JSON |
| ~~`tile.generate`~~ | — | **Do not ship single-tile tool v1** — use batch only |

**Tri-mode:** see [`MICRO_TOOLS_REGISTRY_v1.md`](../../tools/mcp/MICRO_TOOLS_REGISTRY_v1.md) Tier 2c.

Register in `MICRO_TOOLS_REGISTRY_v1.md` Tier 2c when T0 lands.

---

## Bevy registry questions — for `@planner` + `@coder`

**Context:** Module GLB path is **already partially answered** in `src/construction/` (not tile).

### A. Promoted `model.glb` → BuildingDefinition (SHIPPED pattern)

| Question | Current repo truth | Recommendation |
|:---|:---|:---|
| Which JSON field binds a building to MCP mesh? | `procedural_module_id` in `assets/configs/buildings/*.json` | Keep — value = **`module_id`** (catalog), not `job_id` |
| Who resolves disk path? | `ProceduralModuleRegistry` from `_module_index.ron` | Load at startup; `attach_procedural_glb_paths()` |
| Runtime fields on `BuildingDefinition` | `procedural_glb_path` (repo-relative), `procedural_glb_asset` (Bevy path) | Set only via attach — agents do not hand-edit |
| Lookup API | `BuildingDefinitionRegistry::procedural_glb_asset(&modules, id)` | Prefer building id → def.module_id → modules.glb_asset |
| `job_id` vs `module_id` | Index has both; promote folder uses `job_id` | **Bind `module_id` only** in building JSON; `job_id` is pipeline artifact |

**Open for @coder (G5 engine slice):**
1. When should `attach_procedural_glb_paths` run relative to `RepresentationResult` build — same frame as building registry hydrate?
2. Does `RepresentationResult` need a **`procedural_mesh_handles`** map or only tactical vector path today?
3. Fallback when `procedural_module_id` set but GLB missing — greybox primitive vs hide?

### B. Tile atlas → Bevy (DEFER — planner-owned)

| Question | For |
|:---|:---|
| New resource `TileAtlasRegistry` vs extend terrain material registry? | @planner |
| `TileVariant` key hash — string key vs packed u32? | @planner |
| Sim state → variant swap — which system owns authority? | @coder + bevy-simulation-grade |
| Map view / `RepresentationResult` — tile layer vs overlay field buffers? | @planner — Stage 5 contract |
| Atlas path convention — `assets/textures/tiles/` + manifest RON? | @planner-mcp proposes; @coder implements |

**Explicit non-goal:** `_module_index.ron` must **not** mix tile PNG entries with GLB module rows without a `kind: module \| tile` discriminator (recommend separate `_tile_atlas_index.ron` in T2).

---

## Parallel lane schedule (post greybox)

```text
Lane C (NOW):     designer-mcp → finalize tile_batch_v1 example (6 variants)
                  planner-mcp  → DONE (this doc)
                  coder-mcp    → TILE-SCHEMA-001..004 only

Lane B (engine):  @planner + @coder → BuildingDefinition G5 + RepresentationResult
                  (parallel safe with Lane C)

Lane A (idle):    kit_greybox_003 only if new bpy ops — else maintenance
```

---

## Open questions

| # | Blocker | Owner |
|:---:|:---|:---|
| 1 | Confirm `tile_size_px` 128 vs 256 for iso readability | @designer-mcp |
| 2 | `fill` axis required on floor tiles or optional default `empty`? | @designer-mcp |
| 3 | Separate `_tile_atlas_index.ron` vs unified index | @planner-mcp → @planner |
| 4 | Blender bpy op: new `tile_ortho_bake` vs extend `export_glb` path | @coder-mcp T2 |
| 5 | CI: tile bake without GPU — software Blender only | @coder-mcp |

---

## Handoff — `@orchestrator-mcp` next sprint

```md
### Phase 0: Tile schema (planner-mcp DONE)
- Deliverable: plan_tile_batch_v1_planner_mcp_v1.md

### Phase 1: Schema impl (coder-mcp)
- TILE-SCHEMA-001..004
- Gate: G2 partial
- No tile_batch_run execution

### Phase 2: Designer spec (designer-mcp, parallel)
- Finalize tools/mcp/schemas/examples/tile_batch_factory_floor_v1.json
- G0 + G1 rules_audit + spec validate
- Gate: G1

### Phase 3: Engine binding (orchestrator → @planner + @coder)
- Answer Bevy questions §A.1–A.3
- Witness: construction procedural test green + one building JSON with procedural_module_id
- Gate: G5 engine (modules only; tiles DEFER)
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Phase 0 planner-mcp tile_batch_v1 + G0–G5 + Bevy registry questions |
