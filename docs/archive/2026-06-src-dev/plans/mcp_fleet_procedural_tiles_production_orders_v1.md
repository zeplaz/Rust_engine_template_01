# MCP fleet — PLAN-PROC-TILE-PROD-001 orders `v1`

> **Orchestrator narrow slice (authoritative for dispatch):** [`mcp_fleet_production_pilot_rowhouse_v1.md`](mcp_fleet_production_pilot_rowhouse_v1.md) — **Victorian rowhouse only.** Do not run warehouse / shopfront / bunker PT-2 steps until that slice closes.

| Field | Value |
|:---|:---|
| **Program** | [`plan_procedural_building_tiles_production_v1.md`](plan_procedural_building_tiles_production_v1.md) |
| **Owner** | `@orchestrator-mcp` |
| **Date** | 2026-06-03 |
| **Status** | **ACTIVE (narrowed)** |

---

## Dispatch (strict order)

| Step | ID | Owner | Blocked by |
|:---|:---|:---|:---|
| 1 | **MCP-PT-1-001** | @designer-mcp | — |
| 2 | **MCP-PT-2-001** | @coder-mcp | MCP-PT-1-001 matrices for target archetype |
| 3 | **MCP-PT-3-001** | @coder-mcp | MCP-PT-2-001 production atlases |
| 4 | **ENG-PT-4-001** | @coder | MCP-PT-3-001 `_variant_catalog.ron` stable |
| 5 | **ENG-PT-5-001** | @coder | ENG-PT-4-001 + burning bakes |
| 6 | **ORCH-PT-6-001** | @orchestrator-mcp | TILE-PROD-001…006 |

---

## MCP-PT-1-001 (@designer-mcp) — **DONE**

1. Author [`design_procedural_tile_production_bar_v1.md`](design_procedural_tile_production_bar_v1.md). ✓
2. Publish `debug_runs/art_pipeline/variant_matrix_{rowhouse,warehouse,shopfront,bunker}_v1.yaml`. ✓
3. Sign-off template: `*_production_signoff.yaml`. ✓

**Witness:** [`pt1_designer_mcp_closure_live.json`](../../debug_runs/art_pipeline/pt1_designer_mcp_closure_live.json)

**Exit:** PT-1 checklist in main plan. ✓

---

## MCP-PT-2-001 (@coder-mcp) — **narrowed → MCP-PROD-KIT/TILE-001**

**Full 4-archetype bake deferred.** Execute only [`mcp_fleet_production_pilot_rowhouse_v1.md`](mcp_fleet_production_pilot_rowhouse_v1.md):

1. `kit_production_001` — 5 Victorian rowhouse modules (`batch_kit_production_001.manifest.json`).
2. **One** production bake: `tile_rowhouse_victorian_production_v1.json` (not warehouse/shopfront/bunker).
3. `tile_batch_validate` — reject `ship: true` + `lod0`.
4. Witness: `procedural_tiles_production_bake_live.json` **rowhouse section only**.

**Env:** `RUST_ENGINE_TILE_DRY_RUN` unset; `RUST_ENGINE_TILE_LIGHT_BLEND` → `utils/Light_keysshotsetup.blend`.

---

## MCP-PT-3-001 (@coder-mcp)

1. Add `sim_tags` to variant_set examples.
2. `variant_matrix_expand` — MCP/CLI parity.
3. Register all production atlases in `_tile_atlas_index.ron`.

---

## ENG-PT-4-001 (@coder)

1. Load `assets/configs/buildings/_variant_catalog.ron`.
2. Implement `TileVariantResolver` per plan § Sim → variant resolver.
3. Wire `map_tile_atlas_stamp`; suppress PG-2 meshes when production atlas present.
4. Witness: `procedural_tiles_runtime_live.json`.

---

## ENG-PT-5-001 (@coder)

1. Fire frame tick in resolver.
2. Subregion dirty for map stamp on fire band change.

---

## ORCH-PT-6-001

Roll up `procedural_tiles_production_program_green_live.json` when all gates pass ([`procedural_tiles_production_witness_v1.md`](procedural_tiles_production_witness_v1.md)).
