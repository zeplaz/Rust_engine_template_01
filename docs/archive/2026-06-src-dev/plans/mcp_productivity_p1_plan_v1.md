# MCP-PRODUCTIVITY-P1-PLAN — thin P1 exec `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **MCP-PRODUCTIVITY-P1-PLAN** |
| **Parent** | $ref:docs/archive/2026-06-src-dev/plans/plan_mcp_productivity_chain_v1.md§P1 |
| **Order** | $ref:docs/archive/2026-06-src-dev/plans/orchestrator_order_mcp_productivity_p1_plan_v1.md |
| **DSM** | $ref:src/dev/plan_dsm_wrk_atl_closure_v1.md |
| **Spine** | $ref:docs/archive/2026-06-src-dev/plans/plan_building_tile_spine_001_v1.md |
| **Grammar** | $ref:docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md |
| **Lang** | $ref:src/dev/agent_lang_v1.md |
| **Planner-mcp** | **SIGNED** |
| **Date** | 2026-06-07 |

**Rule:** Plan only — @coder-mcp implements after this doc merges. Do **not** reopen P0 or shipped grammar tools.

---

## Summary

P1 closes the **SNAP → WRK → ATL** agent loop: grammar iterate without full JSON in chat (🟢 shipped), then optional **one-call tile spine** (🧊 deferred) and **atlas_meta_brief** (🧊 deferred). Grammar queue had **one** ready planner row — this doc is the deliverable.

---

## P1 status matrix

| ⟨ID⟩ | Label | φ | Owner | Witness |
|:---|:---|:---:|:---|:---|
| MCP-GRAMMAR-ITER-TOOL | `grammar_iterate` MCP + CLI | 🟢 | @coder-mcp | $ref:debug_runs/grammar_iter_001_massing_live.json |
| MCP-SNAPSHOT-DIFF-001-IMPL | `snapshot_diff_brief` | 🟢 | @coder-mcp | $ref:debug_runs/grammar_iter_001_aps1_live.json |
| MCP-MAT-BRIEF-001 | `material_profile_brief` | 🟢 | @coder-mcp | $ref:debug_runs/mcp_mat_brief_001_live.json |
| **MCP-SPINE-CHAIN-001** | `tile_spine_run` | 🧊→○ | @coder-mcp | $ref:debug_runs/tile_spine_run_001_live.json |
| **MCP-ATLAS-BRIEF-001** | `atlas_meta_brief` | 🧊→○ | @coder-mcp | $ref:debug_runs/mcp_atlas_brief_001_live.json |
| MCP-OPS-REPORT-001 | `ops_intelligence_scan` wrap | 🧊 | P2 defer | — |

---

## §1 — Grammar iterate loop (SHIPPED — ritual only)

**Authority:** $ref:tools/mcp/schemas/grammar_iterate_request_v1.schema.json · $ref:tools/mcp/schemas/grammar_iterate_result_v1.schema.json

### Agent ritual (BLANG)

```text
BLANG:PRE → BLANG:DIGEST(base_snapshot) → BLANG:P0
  → grammar_iterate(request_path)   # MCP + CLI parity — SHIPPED
  → BLANG:DIFF(before, after)       # snapshot_diff_brief — SHIPPED
  → validate_p0_gate_plain(child)   # if materials touched
  → BLANG:WIT → BLANG:Q✓
```

| Token | Tool | Replaces |
|:---|:---|:---|
| `BLANG:DIGEST` | `snapshot_digest(path)` | Read full assembly JSON |
| `BLANG:DIFF` | `snapshot_diff_brief(before, after)` | Two snapshots in chat |
| `BLANG:P0` | `validate_p0_gate_plain(path)` | validate-report + hint parse |
| — | `grammar_iterate(request_path)` | Manual bpy / full regen |

### Request sketch (modes)

| `mode` | Use | `overrides` keys |
|:---|:---|:---|
| `massing` | Footprint / hall strategy | `massing_strategy`, `footprint.*` |
| `material_strategy` | District materials | `district_material_profiles` |
| `placement` | Single node swap | `node_id`, `module_id`, `material_profile` |
| `roof` / `facade` | Partial regen | per $ref:docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md§Overrides-by-mode |

### `snapshot_diff_brief` response (≤12 lines)

```json
{
  "schema": "snapshot_diff_brief_v1",
  "ok": true,
  "cells_added": 12,
  "cells_removed": 4,
  "cells_changed": 6,
  "layers_touched": ["massing", "roof"],
  "footprint_before": "8x9x2",
  "footprint_after": "10x6x2",
  "hint": "Feed APS footprint diff legend — green/red/yellow"
}
```

**APS hook:** $sym:GrammarIteratePanel@tools/mcp/art_pipeline_suite/grammar_iterate_panel.py — diff drives heatmap; no second Read.

**Do not replan:** MCP-GRAMMAR-ITER-TOOL · MCP-SNAPSHOT-DIFF-001-IMPL queue rows stay **done**.

---

## §2 — ⟨MCP-SPINE-CHAIN-001⟩ `tile_spine_run` (PLANNED)

**Purpose:** One MCP/CLI call chains WRK→ATL steps with per-step witness — agent never memorizes six CLIs.

**Default:** `ship: false` · stop on first hard fail · plain step sentence.

### Request — `tile_spine_run_request_v1`

```json
{
  "schema": "tile_spine_run_request_v1",
  "snapshot_path": "assets/staging/assemblies/industrial_west_8x9_s43_f75a.json",
  "batch_id": "warehouse_industrial_west_production_v1",
  "steps": [
    "p0_gate",
    "snapshot_digest",
    "preview",
    "assembly_build",
    "tile_batch",
    "atlas_pack",
    "atlas_validate"
  ],
  "ship": false,
  "write_witness": true,
  "honest_bake": true
}
```

| Step | SHIPPED delegate | Hard fail if |
|:---|:---|:---|
| `p0_gate` | `validate_p0_gate_plain` | `status != pass` |
| `snapshot_digest` | `snapshot_digest` | missing snapshot |
| `preview` | `preview-assembly` (optional skip if `PREVIEW_SKIP=1`) | worker down |
| `assembly_build` | `assembly_build_run` | blend/job fail |
| `tile_batch` | `tile_batch_run` (`bake_source: keyframe_pack`) | batch validate fail |
| `atlas_pack` | `tile_atlas_pack_tool` | empty folder |
| `atlas_validate` | `validate-report tile_batch` + `tile_promotion_honest_check` | headless-as-ship |

**Honest gate (`honest_bake: true`):** Reject `RUST_ENGINE_TILE_DRY_RUN` ortho path and `keyframe_pack` with &lt;24 PNGs when `ship: true`. Default `ship: false` — integration test only per $ref:tools/orchestrator/queues/defer_registry.json.

### Response — `tile_spine_run_result_v1`

```json
{
  "schema": "tile_spine_run_result_v1",
  "ok": false,
  "stopped_at": "tile_batch",
  "steps": [
    { "step": "p0_gate", "ok": true, "duration_ms": 120, "witness_path": null },
    { "step": "assembly_build", "ok": true, "duration_ms": 8400, "witness_path": "debug_runs/build_worker_001_live.json" },
    { "step": "tile_batch", "ok": false, "duration_ms": 310, "artist_message": "Keyframe folder has 8 PNGs; need 24 for ship bake.", "witness_path": null }
  ],
  "witness_path": "debug_runs/tile_spine_run_001_live.json"
}
```

### Grammar → spine handoff

After `grammar_iterate` + `snapshot_diff_brief` green:

```text
child_snapshot_path → tile_spine_run_request_v1.snapshot_path
parent_lineage_id preserved in witness body
```

**Complexity budget:** value 7 · complexity 6 · ratio **1.17** — approve after P0 green (orchestrator ack).

**Files (implementer):** `tools/mcp/python/rust_engine_mcp/tile_spine_run.py` · schema `tools/mcp/schemas/tile_spine_run_request_v1.schema.json` · register in `server.py` + `cli.py` + MICRO_TOOLS_REGISTRY Tier 1d.

---

## §3 — ⟨MCP-ATLAS-BRIEF-001⟩ `atlas_meta_brief` (PLANNED)

**Purpose:** ≤40-line artist summary of atlas folder — UV grid, missing lookups, plain FAIL sentences. Closes ATL○ per $ref:src/dev/plan_dsm_wrk_atl_closure_v1.md.

### Request

```json
{
  "schema": "atlas_meta_brief_request_v1",
  "atlas_folder": "assets/staging/tiles/warehouse_industrial_west_production_v1",
  "batch_id": "warehouse_industrial_west_production_v1"
}
```

### Response — `atlas_meta_brief_v1`

```json
{
  "schema": "atlas_meta_brief_v1",
  "ok": false,
  "atlas_meta_schema": "v1",
  "facings": 4,
  "cells_expected": 32,
  "cells_present": 28,
  "missing_lookups": ["damage_heavy_f3", "fire_f2"],
  "uv_grid_summary": "8×4 grid · 512px cells",
  "artist_messages": [
    { "sentence": "Four facing slots are missing from the atlas.", "fix": "Re-run tile batch for states: damage_heavy, fire." }
  ],
  "plain_language_count": 1,
  "hint": "APS Atlas tab inline QC — not modal"
}
```

**Depends:** spine plan § `atlas_validate` hook only — may run standalone on existing production folder.

**Witness:** `debug_runs/mcp_atlas_brief_001_live.json`

**Planner review:** $ref:docs/archive/2026-06-src-dev/plans/plan_p1_atl_closure_review_v1.md — amendments G1–G4 (v1 pilot fail + v2 production pass; optional `legend_code`; honest_check stub warn until P2).

**Complexity budget:** value 6 · complexity 2 · ratio **3.0** — approve.

---

## §4 — Registry delta (Tier 1d — @coder-mcp implements)

Add to $ref:tools/mcp/MICRO_TOOLS_REGISTRY_v1.md after Tier 1c:

| CLI | MCP tool | BLANG | Status |
|:---|:---|:---|:---:|
| `grammar-iterate <req.json>` | `grammar_iterate` | — | **SHIPPED** |
| `snapshot-diff-brief <before> <after>` | `snapshot_diff_brief` | `BLANG:DIFF` | **SHIPPED** |
| `tile-spine-run <req.json>` | `tile_spine_run` | — | **PLANNED** |
| `atlas-meta-brief <folder>` | `atlas_meta_brief` | — | **PLANNED** |

---

## §5 — Undefer criteria (grammar queue)

| Row | Was | May become **ready** when |
|:---|:---|:---|
| MCP-SPINE-CHAIN-001 | deferred | This plan **SIGNED** + orchestrator ΔWF→@coder-mcp + BLANG ritual green ≥2 sessions |
| MCP-ATLAS-BRIEF-001 | deferred | MCP-SPINE-CHAIN-001 **ready** OR orchestrator waives spine dep for brief-only impl |

**Do not** auto-ready MCP-OPS-REPORT-001 (P2).

---

## §6 — Implement order (@coder-mcp)

```text
1. tile_spine_run — schema + pytest + witness tile_spine_run_001_live.json
2. atlas_meta_brief — may parallel if spine step list frozen
3. MICRO_TOOLS_REGISTRY Tier 1d + token_savings_guide grammar→spine chain note
4. BLANG:PY -k "tile_spine or atlas_meta"
```

**Paste:**

```text
BLANG:Q+("coder-mcp")
$ref:docs/archive/2026-06-src-dev/plans/mcp_productivity_p1_plan_v1.md

ΔWF:
  ⟨MCP-SPINE-CHAIN-001⟩ — §2 tile_spine_run · ship:false default
  ⟨MCP-ATLAS-BRIEF-001⟩ — §3 atlas_meta_brief · ATL closure keys

Grammar loop (maintain — do not rework):
  grammar_iterate + snapshot_diff_brief — 🟢 SHIPPED per §1

joint: "@designer-mcp — atlas_meta_brief artist_messages match aps_validator_plain tone?"
BLANG:PY → BLANG:WIT → BLANG:Q✓
```

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-07 | MCP-PRODUCTIVITY-P1-PLAN — grammar loop + spine + atlas brief |
