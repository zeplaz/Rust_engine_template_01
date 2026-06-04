# PLAN-TILE-AUTO-EXEC-001 — Fully automated tile + assembly pipeline `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-TILE-AUTO-EXEC-001** |
| **Owner** | `@planner-mcp` → **`@coder-mcp`** (Python/Blender) + **`@coder`** (PG-2 manifest only) |
| **Date** | 2026-06-02 |
| **Status** | **READY — zero manual as primary path** |
| **Parent** | [`design_assembly_to_tile_coupling_v1.md`](design_assembly_to_tile_coupling_v1.md) · [`plan_tile_batch_v1_planner_mcp_v1.md`](plan_tile_batch_v1_planner_mcp_v1.md) |
| **Fleet orders** | [`mcp_fleet_tile_automation_orders_v1.md`](mcp_fleet_tile_automation_orders_v1.md) |
| **Registry** | [`tools/mcp/MICRO_TOOLS_REGISTRY_v1.md`](../../tools/mcp/MICRO_TOOLS_REGISTRY_v1.md) Tier 2c |

---

## Policy (user directive)

| Rule | Meaning |
|:---|:---|
| **No manual primary path** | Agents and CI never depend on Blender GUI, keyframe UI, or “open light blend” buttons |
| **Tri-mode required** | Every step: **MCP tool** = **CLI** = same `rust_engine_mcp.*` function |
| **Manual UI** | Module viewer Pipeline tab calls **same automation** — optional `RUST_ENGINE_ART_DEBUG_GUI=1` for Blender GUI only |
| **validation-first** | All steps return JSON (`ValidationReport`, job status, witness) — not raw logs to agents |

---

## End-state pipeline (100% automated)

```text
tile_batch_v1.json + assembly_snapshot.json
    → assembly_build_job (Blender headless: import GLBs + light rig)
    → tile_variant_jobs[] (state axes: damage/power/fill/lighting)
    → tile_ortho_bake per variant → PNG stills
    → tile_atlas_pack (tilemapgen or atlas_packer.py)
    → atlas_meta.json + witness JSON
    → (engine) _tile_atlas_index.ron  [@coder separate lane]
```

**Forbidden for agents:** “open Blender”, “use keyframe panel”, “pack folder by hand”.

---

## Phase map (coder-mcp)

| ID | Deliverable | Files (new/changed) | Acceptance |
|:---|:---|:---|:---|
| **AUTO-001** | `assembly_snapshot_v1.schema.json` + example | `tools/mcp/schemas/` | jsonschema pass |
| **AUTO-002** | `assembly_build_job_v1.schema.json` | `tools/mcp/schemas/` | references snapshot + module paths from index |
| **AUTO-003** | bpy `assembly_import.py` | `tools/mcp/blender/scripts/ops/` | headless: N GLBs → collection `ASSEMBLY` |
| **AUTO-004** | bpy `tile_ortho_bake.py` | `tools/mcp/blender/scripts/ops/` | append camera/lights from `Light_keysshotsetup.blend` via `RUST_ENGINE_TILE_LIGHT_BLEND` |
| **AUTO-005** | `run_tile_job.py` | `tools/mcp/blender/scripts/` | `--job assembly_build.json` \| `--job tile_variant.json` |
| **AUTO-006** | `blender_runner.run_tile_job()` | `rust_engine_mcp/blender_runner.py` | status files like geometry jobs |
| **AUTO-007** | `tile_batch_run()` real impl | `rust_engine_mcp/tile_pipeline.py` | replaces `not_implemented` |
| **AUTO-008** | MCP + CLI | `server.py`, `cli.py` | see tool table below |
| **AUTO-009** | `generate_assembly_snapshot()` | `rust_engine_mcp/assembly.py` | StylePack RON + footprint → snapshot JSON (no Blender) |
| **AUTO-010** | Viewer automation only | `module_viewer/app.py` | buttons invoke CLI-equivalent; GUI Blender behind debug flag |
| **AUTO-011** | pytest + witness | `tests/test_tile_pipeline.py` | `debug_runs/art_pipeline/tile_*_live.json` G3 pass |

**@coder (engine, not coder-mcp):** PG-2 emits same `assembly_snapshot.json` shape — **AUTO-009** contract is shared; do not duplicate Blender import in `src/`.

---

## Blender job types (extend `run_job.py` → `run_pipeline_job.py` or parallel runner)

### `assembly_build`

```json
{
  "schema_version": 1,
  "job_id": "asm_rowhouse_victorian_s42",
  "operation": "assembly_build",
  "assembly_snapshot": "assets/staging/assemblies/rowhouse_victorian_4x3_s42.json",
  "light_blend": "utils/Light_keysshotsetup.blend",
  "output": { "blend": "assets/staging/assemblies/rowhouse_victorian_4x3_s42.blend" }
}
```

### `tile_variant_bake`

```json
{
  "schema_version": 1,
  "job_id": "tile_factory_floor_sdamaged_d45",
  "operation": "tile_variant_bake",
  "assembly_blend": "assets/staging/assemblies/....blend",
  "variant": {
    "state": "damaged",
    "damage": 0.45,
    "power": "on",
    "fill": "half",
    "lighting": "night_on"
  },
  "render": {
    "method": "blender_orthographic_iso",
    "seed": 42,
    "tile_size_px": 128
  },
  "output": { "png": "assets/staging/tiles/factory_floor/....png" }
}
```

**Concept port from legacy:**

| Legacy | Automated replacement |
|:---|:---|
| `keyframe_render.py` UI | `tile_variant_bake` jobs generated from `tile_batch_v1.variants[]` |
| `tilemapgen` | Keep — wrap in `tile_atlas_pack` (**done**) → add `atlas_meta.json` emitter in **AUTO-011** |
| `Light_keysshotsetup.blend` | Linked append in `tile_ortho_bake.py` — **no GUI** |

---

## MCP / CLI tool table (target — all SHIPPED after AUTO-011)

| MCP tool | CLI | Input | Output |
|:---|:---|:---|:---|
| `assembly_snapshot_generate` | `assembly-snapshot-generate` | style_pack + footprint + seed | snapshot JSON path |
| `assembly_build_job` | `assembly-build-run` | snapshot path | blend path + status |
| `tile_batch_run` | `tile-batch-run` | tile_batch_v1.json | batch status + png paths |
| `tile_batch_validate` | `validate-report tile_batch` | path | ValidationReport (**SHIPPED**) |
| `tile_atlas_pack_tool` | `tile-atlas-pack` | folder | atlas png (**SHIPPED**) |
| `tile_batch_status` | `tile-batch-status` | batch_id | status JSON |
| `lod0_batch_run_tool` | `lod0-batch-run` | batch_id + phase | log JSON (**SHIPPED**) |

**Remove / gate manual-only MCP:** none added for “open Blender”.

---

## `tile_batch_run` algorithm (AUTO-007)

1. `validate_report tile_batch` — fail fast.
2. If `assembly_ref` present: ensure snapshot exists or call `assembly_snapshot_generate`.
3. `assembly_build_job` if `.blend` missing or stale (hash snapshot).
4. For each `variants[]` entry: emit `tile_variant_bake` job → `run_tile_job` sequential (or batch queue).
5. `tile_atlas_pack` on output folder + write `atlas_meta.json` (UV grid from variant keys).
6. `write_witness tile_<batch_id>`.

**Terrain-only batches** (no `assembly_ref`): skip 2–3; bake flat plane material variants only (floor tiles).

---

## Module viewer (AUTO-010)

| Old (debug only) | New default |
|:---|:---|
| Open light setup | Hidden unless `RUST_ENGINE_ART_DEBUG_GUI=1` |
| Keyframe addon | Removed from default UI |
| Pack tile atlas | **Run tile batch** (full pipeline) |
| Run lod0 batch | Keep (module lane) |

Add: **Run tile_batch_v1** file picker → calls `tile_batch_run` end-to-end.

---

## Tests

```powershell
cd tools\mcp\python
python -m pytest tests/test_tile_pipeline.py -q
python -m rust_engine_mcp.cli tile-batch-run tools\mcp\schemas\examples\tile_batch_factory_floor_v1.json
python -m rust_engine_mcp.cli validate-report tile_batch tools\mcp\schemas\examples\tile_batch_factory_floor_v1.json
```

**Fixture:** minimal assembly of 3 lod0 modules (wall+door+roof) + 2 variants → 2 PNGs + atlas + witness green.

---

## Dependencies (do not start out of order)

```text
AUTO-001 → AUTO-002 → AUTO-003 → AUTO-004 → AUTO-005/006
    → AUTO-009 (parallel with 003 if snapshot from JSON only)
    → AUTO-007/008 → AUTO-011
AUTO-010 last (UI wraps 007/008)
```

**Blocked on:** Wave 2 lod0 modules in index (**unblocked** — 50/50). StylePack RON (**7 packs** — unblocked).

**Not in coder-mcp scope:** Bevy `TileAtlasRegistry` — file issue for `@coder` when AUTO-011 witness green.

---

## Anti-patterns (reject in PR)

- New workflow that says “artist opens Blender” without automated headless path
- `tile.generate` single-tile tool
- Baking from single `module_wall` GLB when `assembly_ref` is set
- Agent prompts that shell out to Blender without `run_tile_job` status JSON

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Zero-manual automation exec plan for @coder-mcp |
