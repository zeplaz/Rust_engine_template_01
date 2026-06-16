# MCP art program green gates `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **MCP-ART-PROGRAM-GREEN-001** |
| **Owner** | `@orchestrator-mcp` |
| **Date** | 2026-06-02 |
| **Status** | **ACTIVE** |
| **Witness rollup** | [`mcp_art_program_green_live.json`](../../debug_runs/art_pipeline/mcp_art_program_green_live.json) |
| **Snap** | [`mcp_orchestrator_snap_CURRENT.md`](../../tools/orchestrator/queues/mcp_orchestrator_snap_CURRENT.md) |

**Program green:** all four gates below pass. Until then, **APS is the authoring system of record**; the game does not consume full pipeline output.

---

## Strong (use with confidence)

| Capability | Evidence |
|:---|:---|
| Full **lod0** module kit (50/50) | `_module_index.ron`, `mcp_fleet_wave2_art_closure_live.json` |
| **7 style packs** + manifest | `style_packs/style_*.ron`, `style_packs_manifest_live.json` |
| **USD-like variants** — one assembly, many layer compositions | `variant_set_v1.schema.json`, `variant_set_rowhouse_victorian_v1.json` |
| **Art Pipeline Suite (APS)** = MCP parity | `tools/mcp/art_pipeline_suite/`, `design_art_pipeline_suite_v1.md` |
| **Tier filter** — smoke ≠ StylePack art | `ProceduralModuleRegistry`, 6/6 tests |
| **PG-2 assembly** in engine | `procedural_assembly_live.json` `green: true`, `smoke_fallback_used: false` |
| **Tile automation spine** (dry-run default) | `tile_batch_run`, `assembly_build`, 37 pytest |

---

## Open gates (order for “art in the game”)

| Gate ID | Owner | Pass when | Status |
|:---|:---|:---|:---:|
| **ART-APS-USE** | `@designer-mcp` + art team | One full pilot: Catalog → Assembly → Variants → Atlas via **Suite or CLI** (not Blender GUI unless `RUST_ENGINE_ART_DEBUG_GUI=1`); witness documents variant set + assembly | **READY** |
| **TILE-REAL-001** | `@coder-mcp` | One `tile_batch_run` with **`RUST_ENGINE_TILE_DRY_RUN` unset** on building batch with `assembly_ref` → real PNGs + `atlas_meta.json` + witness G3 | **PASS** |
| **PG-2-WIT** | `@coder` | `procedural_assembly_live.json` green, `smoke_fallback_used: false` | **PASS** |
| **TILE-ENGINE-001** | `@coder` | Bevy loads `_tile_atlas_index.ron` after TILE-REAL-001 witness | **PASS** |

**Production pilot (2026-06-03):** [`mcp_fleet_production_pilot_rowhouse_v1.md`](mcp_fleet_production_pilot_rowhouse_v1.md) — **unfrozen:** `kit_production_001` + `tile_rowhouse_victorian_production_v1` only. All other `kit_production_*` / multi-archetype batches remain frozen.

---

## Practical policy — art team

| Do now | Defer |
|:---|:---|
| **Production pilot:** Victorian rowhouse — `kit_production_001` + production tile batch | Warehouse / shopfront / bunker production |
| Author assemblies + variant sets in **APS** | Expect final iso quality from lod0 alone |
| Tag variants (`sim_night`, `user_approved`, agent patches) | `kit_production_002+` |
| **Request agent → patch JSON → Apply patch → Bake selected** | Blender GUI (unless debug flag) |
| Catalog pass on modules for next buildings | Folding tiles into map view before PG-2 (PG-2 **done**) |
| **Pilot:** `variant_set_rowhouse_victorian_v1.json` + `assembly_snapshot_rowhouse_victorian_v1.json` + new `tile_batch_rowhouse_victorian_pilot_v1.json` | Manual-only tile workflow as primary |

---

## Pilot building (canonical first real bake)

| Artifact | Path |
|:---|:---|
| Assembly snapshot | `tools/mcp/schemas/examples/assembly_snapshot_rowhouse_victorian_v1.json` |
| Variant set | `tools/mcp/schemas/examples/variant_set_rowhouse_victorian_v1.json` |
| Tile batch (author) | `tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_pilot_v1.json` **(new)** |
| Batch runner | `python -m rust_engine_mcp.cli tile-batch-run <path>` |
| Suite | `python tools/mcp/art_pipeline_suite/run.py` |

**Tile batch must include:**

```json
"assembly_ref": {
  "style_pack_id": "style_victorian",
  "assembly_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_rowhouse_victorian_v1.json",
  "footprint": { "width": 4, "depth": 3, "floors": 2 }
},
"variant_set_ref": "tools/mcp/schemas/examples/variant_set_rowhouse_victorian_v1.json"
```

---

## Queue reconcile note (2026-06-02)

`mcp_active_queue.json` previously showed **MCP-AUTO-*** as blocked while code was ahead. **Truth:** AUTO-001→011 **shipped** (schemas, bpy, `tile_batch_run`, APS UI, 37 pytest). Remaining work is **real bake (TILE-REAL-001)**, not stub implementation.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Unified gates post Wave 2 + AUTO + PG-2 witness |
