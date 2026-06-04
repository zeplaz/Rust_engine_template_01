# MCP fleet — APS pilot orders `v1` (designer-mcp + coder)

| Field | Value |
|:---|:---|
| **Program ID** | **MCP-FLEET-APS-PILOT-001** |
| **Parent** | [`mcp_art_program_green_gates_v1.md`](mcp_art_program_green_gates_v1.md) |
| **Snap** | [`mcp_orchestrator_snap_CURRENT.md`](../../tools/orchestrator/queues/mcp_orchestrator_snap_CURRENT.md) |
| **Status** | **ACTIVE** |

**Planner / planner-mcp / coder-mcp (AUTO spine):** **DRAINED** — on-call only unless TILE-REAL-001 fails.

---

## Paste — @designer-mcp (ART-APS-USE)

> Read `src/dev/mcp_fleet_aps_pilot_orders_v1.md` + `design_art_pipeline_suite_v1.md`. Execute **MCP-APS-PILOT-001**:
>
> 1. Align `variant_set_rowhouse_victorian_v1.json` with `assembly_snapshot_rowhouse_victorian_v1.json` (`assembly_id` must match).
> 2. Author **`tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_pilot_v1.json`** with `assembly_ref` + `variant_set_ref` (see gates doc).
> 3. G0 rules YAML: `debug_runs/art_pipeline/aps_pilot_rowhouse_g0_rules.yaml`.
> 4. Validate-only: `validate_report tile_batch`, `variant_set_validate`, `assembly_snapshot` schema paths.
> 5. Document APS flow in witness **`debug_runs/art_pipeline/aps_pilot_rowhouse_live.json`** (tabs used, CLI commands — **no promote**, no real bake — that is coder-mcp TILE-REAL-001).
>
> Use Suite or CLI only. No Blender GUI unless documenting `RUST_ENGINE_ART_DEBUG_GUI=1`.

---

## Paste — @coder-mcp (TILE-REAL-001)

> Read `mcp_art_program_green_gates_v1.md`. Execute **MCP-TILE-REAL-001** after designer-mcp delivers `tile_batch_rowhouse_victorian_pilot_v1.json`:
>
> 1. Confirm **`RUST_ENGINE_TILE_DRY_RUN` is NOT set** (real Blender bake).
> 2. `python -m rust_engine_mcp.cli tile-batch-run tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_pilot_v1.json`
> 3. validation-first: witness via `write-witness` — G3 pass with **real** PNG dimensions (not 1×1 stub).
> 4. Outputs under `assets/staging/tiles/<batch_id>/` + atlas PNG + `atlas_meta.json`.
> 5. Update `debug_runs/art_pipeline/mcp_art_program_green_live.json` gate `TILE-REAL-001: pass`.
>
> If Blender missing on machine: report blocker — do not fake with dry run for this gate.

---

## Paste — @coder (TILE-ENGINE-001)

> **Blocked on TILE-REAL-001.** After real bake witness green, load **`assets/configs/buildings/_tile_atlas_index.ron`** (or generate from atlas_meta per `plan_tile_pipeline_automation_exec_v1.md`). Wire map/tactical tile swap — separate from PG-2 (already green). Witness: extend `mcp_art_program_green_live.json`.

---

## Paste — @planner / @planner-mcp

> **DRAINED** for new plans. Production scope: **`mcp_fleet_production_pilot_rowhouse_v1.md`** — rowhouse only; do not replan multi-archetype PT-2.

---

## Task MCP-APS-PILOT-001 — @designer-mcp

| Step | Deliverable |
|:---|:---|
| 1 | `assembly_id` consistency across variant set + assembly snapshot |
| 2 | `tile_batch_rowhouse_victorian_pilot_v1.json` |
| 3 | `aps_pilot_rowhouse_g0_rules.yaml` |
| 4 | `aps_pilot_rowhouse_live.json` |
| 5 | Optional: catalog notes for modules used in rowhouse (sidecar tags) |

**Acceptance:** all validate_report green; witness lists exact CLI/MCP tool names used.

---

## Task MCP-TILE-REAL-001 — @coder-mcp

| Step | Deliverable |
|:---|:---|
| 1 | Real bake (dry_run false in job status JSON) |
| 2 | ≥2 variant PNGs non-stub size |
| 3 | `atlas_meta.json` present |
| 4 | `tile_rowhouse_victorian_pilot_live.json` or batch witness G3 pass |

---

## Task MCP-TILE-ENGINE-001 — @coder

| Step | Deliverable |
|:---|:---|
| 1 | `TileAtlasRegistry` or equivalent loads index |
| 2 | Test: atlas handle resolves for pilot batch id |
| 3 | Gate TILE-ENGINE-001 in program green witness |

---

## Verification

```powershell
# Designer-mcp (validate only)
python -m rust_engine_mcp.cli variant-set-validate tools/mcp/schemas/examples/variant_set_rowhouse_victorian_v1.json
python -m rust_engine_mcp.cli validate-report tile_batch tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_pilot_v1.json

# Coder-mcp (real bake — Blender required)
$env:RUST_ENGINE_TILE_DRY_RUN = ""
python -m rust_engine_mcp.cli tile-batch-run tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_pilot_v1.json

# Regression
python -m pytest tools/mcp/python/tests/ -q
cargo test -p proc_A_dine01 --lib procedural
```
