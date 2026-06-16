# MCP fleet — Production pilot: Victorian rowhouse only `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **MCP-PROD-PILOT-ROWHOUSE-001** |
| **Owner** | `@orchestrator-mcp` |
| **Date** | 2026-06-03 |
| **Status** | **ACTIVE** |
| **Slice rule** | **One archetype · one style pack · one kit batch · one tile batch** |
| **Sprint (paste prompts)** | [`mcp_fleet_production_sprint_rowhouse_v1.md`](mcp_fleet_production_sprint_rowhouse_v1.md) |

**Stops designer boiling the ocean:** no warehouse / shopfront / bunker production matrices, no 50-module acceptance, no multi-style `kit_production_001` wall sweep in this pilot.

---

## Unfrozen (this slice only)

| ID | What | Path |
|:---|:---|:---|
| **`kit_production_001`** | 5 modules, **`style_victorian` only**, rowhouse assembly slots | [`batch_kit_production_001.manifest.json`](../../tools/mcp/schemas/examples/batch_kit_production_001.manifest.json) |
| **`tile_rowhouse_victorian_production_v1`** | Production iso bake (Object-Plus / keyframe rig) | [`tile_batch_rowhouse_victorian_production_v1.json`](../../tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json) |

**Prerequisite:** Program green gates closed ([`mcp_art_program_green_live.json`](../../debug_runs/art_pipeline/mcp_art_program_green_live.json)) — lod0 pilot + PG-2 + tile registry.

---

## Still frozen

| Item | Reason |
|:---|:---|
| `kit_production_002` … `kit_production_*` | After rowhouse pilot G4 |
| `kit_greybox_*` (004+) | Smoke tier — never player |
| `tile.generate` (single-tile) | Use `tile_batch_run` |
| `tile_batch_*_production_v1` except **rowhouse victorian** | warehouse / shopfront / bunker deferred |
| `variant_matrix_warehouse_*` / `shopfront_*` / `bunker_*` | Designer-mcp **do not open** for this pilot |
| Full **DESIGN-PROC-ART-ACCEPTANCE-001** (50 modules) | Unscoped — rowhouse production signoff only |

---

## Module set (`kit_production_001` — rowhouse)

| module_id | Slot on rowhouse 4×3×2 | material_id |
|:---|:---|:---|
| `wall_brick_1u` | W bays | `brick_red_01` |
| `corner_L` | C corners | `brick_red_01` |
| `door_residential` | D ground | `brick_red_01` |
| `roof_pitched_gable` | R plane | `brick_red_01` |
| `prop_chimney` | prop_clutter | `brick_red_01` |

All `development_tier: production`, `pbr_status: shipped`, `style_pack_id: style_victorian`.

---

## Dispatch order

| P | Task | Agent | Pass when |
|:---|:---|:---|:---|
| **P0** | **MCP-PROD-G0-001** — G0/G4 on production batch + variant set | @designer-mcp | `rowhouse_production_g0_rules.yaml`; **no** other archetype YAML |
| **P0** | **MCP-PROD-KIT-001** — run `kit_production_001` manifest (5 modules) | @coder-mcp | Promoted GLBs + index rows `development_tier: production` |
| **P1** | **MCP-PROD-TILE-001** — `tile-batch-run` production batch | @coder-mcp | `dry_run: false`, G3, `source_tier: production`, atlas under `assets/textures/buildings_iso/production/` |
| **P2** | **MCP-PROD-INDEX-001** — register production atlas in `_tile_atlas_index.ron` | @coder-mcp | Row `tile_rowhouse_victorian_production_v1` |
| **P3** | **ENG-PROD-RUNTIME-001** — variant resolver + map stamp (optional tail) | @coder | Per [`mcp_fleet_procedural_tiles_production_orders_v1.md`](mcp_fleet_procedural_tiles_production_orders_v1.md) ENG-PT-4 — **after** P1 green |

---

## Paste — @designer-mcp

> **MCP-PROD-PILOT-ROWHOUSE-001** from `docs/archive/2026-06-src-dev/plans/mcp_fleet_production_pilot_rowhouse_v1.md`.
>
> **Scope: Victorian rowhouse only.** Do **not** edit warehouse / shopfront / bunker matrices or production tile batches.
>
> 1. G0 audit: `tile_batch_rowhouse_victorian_production_v1.json` + `variant_set_rowhouse_victorian_production_v1.json` (create/align if missing) vs `assembly_snapshot_rowhouse_victorian_v1.json`.
> 2. Witness: `debug_runs/art_pipeline/rowhouse_production_g0_rules.yaml` — `proceed_production_bake: yes|no`.
> 3. Sign-off template: `rowhouse_production_signoff.yaml` (G4 checklist from [`design_procedural_tile_production_bar_v1.md`](design_procedural_tile_production_bar_v1.md)) — **rowhouse only**.
> 4. **Reject** expanding to other archetypes until orchestrator publishes next slice.

---

## Paste — @coder-mcp

> **MCP-PROD-KIT-001** then **MCP-PROD-TILE-001** from `docs/archive/2026-06-src-dev/plans/mcp_fleet_production_pilot_rowhouse_v1.md`.
>
> `kit_production_001`: manifest `batch_kit_production_001.manifest.json` — Victorian rowhouse modules only. validation-first on each job. Promote + `library_register` with `development_tier: production`.
>
> Then: unset `RUST_ENGINE_TILE_DRY_RUN`, run `tile-batch-run` on `tile_batch_rowhouse_victorian_production_v1.json`. Witness G3 + `procedural_tiles_production_bake_live.json` (rowhouse section only). Register `_tile_atlas_index.ron`.

---

## Paste — @designer (HUD / player read)

> **On-call — rowhouse production review only** after MCP-PROD-TILE-001 witness. Tactical check: production atlas reads as *brick rowhouse* vs lod0 pilot. **Do not** run full DESIGN-PROC-ART-ACCEPTANCE-001 (50 modules). Sign `rowhouse_production_signoff.yaml` if player read passes.

---

## Paste — @planner / @planner-mcp

> **DRAINED** for new plans. Production pilot scope is **frozen in this doc** — Victorian rowhouse only. Do not replan multi-archetype PT-2 until ORCH closes MCP-PROD-PILOT-ROWHOUSE-001.

---

## References

| Doc | Role |
|:---|:---|
| [`plan_module_kit_production_tier_v1.md`](plan_module_kit_production_tier_v1.md) | Tier policy (signed) |
| [`design_procedural_tile_production_bar_v1.md`](design_procedural_tile_production_bar_v1.md) | Production bar (signed) |
| [`variant_matrix_rowhouse_v1.yaml`](../../debug_runs/art_pipeline/variant_matrix_rowhouse_v1.yaml) | **Only** matrix in scope |
| [`mcp_fleet_procedural_tiles_production_orders_v1.md`](mcp_fleet_procedural_tiles_production_orders_v1.md) | Full PT program — **narrowed by this slice** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Unfreeze kit_production_001 + rowhouse production tile only |
