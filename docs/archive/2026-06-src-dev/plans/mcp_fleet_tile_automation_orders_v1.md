# MCP fleet — tile automation orders `v1` (coder-mcp)

| Field | Value |
|:---|:---|
| **Queue ID** | **MCP-FLEET-TILE-AUTO-001** |
| **Owner** | `@orchestrator-mcp` |
| **Implementer** | **`@coder-mcp` only** (Blender bpy + `tools/mcp/python/`) |
| **Exec plan** | [`plan_tile_pipeline_automation_exec_v1.md`](plan_tile_pipeline_automation_exec_v1.md) |
| **Status** | **CLOSED** (spine shipped) — real bake = **TILE-REAL-001** in [`mcp_fleet_aps_pilot_orders_v1.md`](mcp_fleet_aps_pilot_orders_v1.md) |
| **Policy** | **No manual primary path** — MCP + CLI + viewer automation only |

---

## Paste prompt — @coder-mcp

> Read **`docs/archive/2026-06-src-dev/plans/mcp_fleet_tile_automation_orders_v1.md`** and execute **AUTO-001 through AUTO-011** from **`plan_tile_pipeline_automation_exec_v1.md`**.  
> **⚠️ 2026-06-03 convergence:** [`design_tile_bake_spine_convergence_v1.md`](design_tile_bake_spine_convergence_v1.md) — **ship/production** uses `keyframe_render` → `tile-atlas-pack` (`bake_source: keyframe_pack`). **`tile_ortho_bake` (AUTO-004) is CI/smoke only** until it matches civ-truck quality — do not use bunker lod0 pilot atlases as production template.  
>
> **Zero manual for pack/register:** MCP wraps `utils/tilemapgen` + index. **Variant stills** remain Blender + `keyframe_render.py` + `Light_keysshotsetup.blend` until headless parity is signed.  
> Replace `tile_batch_run` `not_implemented` stub. validation-first + pytest. ≤3 files per PR; follow existing `run_job.py` / `blender_runner` patterns.

---

## Task checklist

| ID | Priority | Status | Acceptance |
|:---|:---:|:---|:---|
| **MCP-AUTO-001** | P0 | ready | `assembly_snapshot_v1.schema.json` + example |
| **MCP-AUTO-002** | P0 | blocked on 001 | `assembly_build_job_v1.schema.json` |
| **MCP-AUTO-003** | P0 | blocked on 002 | `assembly_import.py` headless test |
| **MCP-AUTO-004** | P0 | blocked on 003 | `tile_ortho_bake.py` + light blend append |
| **MCP-AUTO-005** | P1 | blocked on 004 | `run_tile_job.py` |
| **MCP-AUTO-006** | P1 | blocked on 005 | `blender_runner.run_tile_job` + status JSON |
| **MCP-AUTO-007** | P1 | blocked on 006 | `tile_batch_run()` full pipeline |
| **MCP-AUTO-008** | P1 | blocked on 007 | MCP tools + CLI parity table complete |
| **MCP-AUTO-009** | P1 | ready | `assembly_snapshot_generate` from StylePack + footprint |
| **MCP-AUTO-010** | P2 | blocked on 008 | module viewer → automation buttons; GUI debug flag |
| **MCP-AUTO-011** | P0 | blocked on 007 | pytest + `tile_*_live.json` witness G3 |

---

## Verification (orchestrator gate)

```powershell
cd C:\dev\github\Rust_engine_template_01\tools\mcp\python
python -m pytest tests/test_tile_pipeline.py -q
python -m rust_engine_mcp.cli assembly-snapshot-generate --style-pack style_victorian --footprint 4x3 --seed 42
python -m rust_engine_mcp.cli tile-batch-run tools\mcp\schemas\examples\tile_batch_factory_floor_v1.json
python -m rust_engine_mcp.cli write-witness tile_factory_floor_greybox_001
```

**Program green when:** AUTO-011 witness `gates.G3` pass + no `not_implemented` in `tile_batch_run`.

---

## Parallel lane (do not block AUTO)

| Lane | Agent | Note |
|:---|:---|:---|
| Engine PG-2 | @coder | Consumes `assembly_snapshot.json` — same schema as AUTO-001 |
| lod0 modules | done | 50/50 index |
| Production tier | frozen | lod0 bakes OK for pipeline proof |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Fleet orders — automation-only tile lane |
