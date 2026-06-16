# PLAN-MCP-PRODUCTIVITY-CHAIN-001 — MCP for throughput, resilience, quality `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-MCP-PRODUCTIVITY-CHAIN-001** |
| **Sources** | Token usage review · [`plan_agent_queue_mcp_v1.md`](plan_agent_queue_mcp_v1.md) · [`plan_agent_operations_intelligence_v1.md`](plan_agent_operations_intelligence_v1.md) · art pipeline chain |
| **Status** | **ACTIVE** — planner recommendation |
| **Date** | 2026-06-03 |

---

## Chain (what MCP must serve)

```text
MAT★ → APS★ → SNAP★ → WRK○ → ATL○ → RT○
         ↑______________________________|
              preview + validate loop
```

| Node | Today (MCP/CLI) | Gap |
|:---|:---|:---|
| **MAT** | `material-studio-witness`, catalog in APS | No MCP **category tree resolve**; no **profile completeness** brief |
| **APS** | Tk + `preview-assembly` | No MCP **snapshot_digest**; no **grammar_iterate** tool |
| **SNAP** | `validate-report assembly_*` | P0 returns **engineer hints**, not [`aps_validator_plain_language_v1.md`](aps_validator_plain_language_v1.md) |
| **WRK** | `assembly_build_run`, `geometry_run_job` | No **preflight** bundle; job errors not plain-language |
| **ATL** | `tile_atlas_pack`, `tile_batch_run`, `tile-atlas-register` | No **spine_chain** one-call with step witnesses; **honest bake** gate weak |
| **RT** | `tile-atlas-register` | No **runtime_lookup_brief** (atlas meta + missing cells) |
| **Agents** | Tier 1a briefs + queue | **Ritual not enforced**; no **run_event** telemetry |

---

## Design principles (token + quality)

1. **Micro tools, not macro prompts** — LLM picks tool + JSON args; never re-implements bpy/cargo/tilemapgen in chat.
2. **Briefs before reads** — every witness/snapshot/log has a `*_brief` MCP that returns ≤40 lines.
3. **Structured validators only** — `ValidationReport` + plain-language overlay; never raw stderr in chat.
4. **Queue-drain, not poll** — `agent_queue_next` → work → `agent_queue_update` → next; no “waiting on planner” turns.
5. **Honest gates** — MCP names must match reality (`keyframe_pack` ≠ headless minimum bake).
6. **Complexity budget** — new MCP must beat `witness_brief` + existing CLI on token ROI or stay CLI-only.

---

## Priority matrix

| P | ID | Dimension | MCP / CLI | Token ROI | Quality / resilience |
|:---|:---|:---|:---|:---:|:---|
| **P0** | **MCP-PREFLIGHT-001** | Resilience | `pipeline_preflight` | High | Blender path, schema versions, queue stale, disk paths |
| **P0** | **MCP-SNAPSHOT-DIGEST-001** | Productivity | `snapshot_digest` | **Very high** | placements, materials missing, grammar one-liner — no full JSON read |
| **P0** | **MCP-P0-PLAIN-001** | Quality | `validate_p0_gate_plain` | High | Artist sentences per APS-VALIDATOR-PLAIN-001 |
| **P1** | **MCP-GRAMMAR-ITER-001** | Productivity | `grammar_iterate` | High | Partial regen; uses new schemas |
| **P1** | **MCP-SNAPSHOT-DIFF-001** | Productivity | `snapshot_diff_brief` | High | cells +/- after iterate; feeds APS heatmap |
| **P1** | **MCP-SPINE-CHAIN-001** | Throughput | `tile_spine_run` (opt steps) | Medium | snapshot→build→pack→validate→register with per-step witness |
| **P2** | **MCP-ATLAS-BRIEF-001** | Quality | `atlas_meta_brief` | High | UV grid, missing lookups, plain errors |
| **P2** | **MCP-MAT-BRIEF-001** | Quality | `material_profile_brief` | Medium | maps status + category path from tree |
| **P2** | **MCP-RUN-EVENT-001** | Resilience | `agent_run_append` | Medium | OPS intelligence Phase 1; no Postgres |
| **P3** | **MCP-OPS-REPORT-001** | Resilience | `ops_intelligence_scan` | Medium | DSM rollup for @operations-intelligence |
| **P3** | **MCP-HONEST-BAKE-001** | Quality | `tile_promotion_honest_check` | Medium | Reject headless-as-ship before G4 |

**Defer:** PostgreSQL telemetry, LLM-in-validator, chat-only material generation, duplicate APS features in MCP without headless path.

---

## P0 specs (implement next — @coder-mcp)

### MCP-PREFLIGHT-001 — `pipeline_preflight`

```json
{
  "blender_ok": true,
  "repo_root": "...",
  "bevy_preview_worker": true,
  "schemas": { "assembly_snapshot_v1": true, "grammar_iterate_request_v1": true },
  "queues": { "grammar_stale_rows": 0 },
  "paths": { "grammars_dir": true, "material_profiles": true }
}
```

**Replaces:** 5–8 ad-hoc `ping` / `locate_blender` / file exists checks per session.

---

### MCP-SNAPSHOT-DIGEST-001 — `snapshot_digest(path)`

```json
{
  "assembly_id": "industrial_west_8x9_s43_f75a",
  "footprint": "8x9x2",
  "placements": 40,
  "material_profiles": { "assigned": 40, "missing": 0, "unique": 6 },
  "grammar": { "archetype": "IndustrialWarehouse", "massing": "yard_complex", "seed": 43 },
  "lineage": { "parent": null, "mode": null },
  "glb_missing": 0,
  "hint": "Ready for P0 gate"
}
```

**Replaces:** Reading 2–4K line assembly JSON into context.

---

### MCP-P0-PLAIN-001 — `validate_p0_gate_plain(path)`

Wraps `validate_assembly_p0_gate` + [`aps_validator_plain_language_v1.md`](aps_validator_plain_language_v1.md):

```json
{
  "status": "failed",
  "artist_messages": [
    { "sentence": "Footprint is too small to read as a building.", "fix": "Increase W×D (minimum 3×3)." }
  ],
  "signature_count": 2,
  "technical": { "compress": 4, "available": true }
}
```

**Quality:** Same gate, fewer mis-fixes, less agent re-read loops.

---

## P1 — throughput chain

### MCP-SPINE-CHAIN-001 — `tile_spine_run` (optional steps)

```json
{
  "snapshot_path": "...",
  "steps": ["p0_gate", "preview", "assembly_build", "tile_batch", "atlas_pack", "atlas_validate"],
  "ship": false,
  "write_witness": true
}
```

Returns **per-step** `{ step, ok, duration_ms, witness_path }` — agent never chains 6 CLIs from memory.

**Resilience:** Stop on first hard fail; plain message per step.

---

### MCP-GRAMMAR-ITER-001 — `grammar_iterate(request_path)`

CLI parity with [`grammar_iterate_request_v1.schema.json`](../../tools/mcp/schemas/grammar_iterate_request_v1.schema.json) → result schema.

**Productivity:** Iteration without full regen + without pasting snapshot in chat.

---

## Agent ritual (enforce in prompts + `token_savings_guide`)

```text
Session start (mandatory):
  1. token_savings_guide()
  2. pipeline_preflight()
  3. agent_queue_next("<agent>")

Per artifact touch:
  snapshot_digest(path)     NOT Read(full snapshot)
  validate_p0_gate_plain()  NOT validate-report + parse hints
  witness_brief(path)       NOT Read(witness json)

Session end:
  agent_queue_update(id, status, note=witness_path)
  agent_run_append({...})   # P2
```

**Throughput rule:** One queue slice per session turn when possible — drain, don't parallel-read plans.

---

## What already works (do not rebuild)

| Tool | Keep using |
|:---|:---|
| `agent_queue_next` / `update` / `board` | Orchestration |
| `witness_brief` / `handoff_brief` / `file_digest` | Token compression |
| `validate_cargo_report(compress=4)` | Build |
| `validate_asset_report` | GLB |
| `assembly_build_run` | WRK |
| `tile_atlas_pack` / `tile_batch_run` | ATL |
| `preview-assembly` | APS/Bevy |

---

## Measurement (OPS Phase 1 — no Postgres)

Append to `debug_runs/agent_ops/run_events.jsonl` on each `agent_queue_update`:

```json
{ "slice_id": "APS-MAT-003", "agent": "coder-mcp", "witness": "...", "tools_called": ["snapshot_digest", "validate_p0_gate_plain"] }
```

@operations-intelligence aggregates monthly — which tools correlate with `green: true` witnesses.

---

## Orchestrator paste

```text
MCP productivity program: PLAN-MCP-PRODUCTIVITY-CHAIN-001

Assign @coder-mcp P0 (1–2 days):
  MCP-PREFLIGHT-001
  MCP-SNAPSHOT-DIGEST-001
  MCP-P0-PLAIN-001

Then P1:
  MCP-GRAMMAR-ITER-001 (schemas ready)
  MCP-SNAPSHOT-DIFF-001
  MCP-SPINE-CHAIN-001 (warehouse integration test only)

Update token_savings_guide + MICRO_TOOLS_REGISTRY when shipped.
Enforce agent ritual in HANDOFF + @coder-mcp agent file.
```

---

## Complexity budget

| Proposal | Value | Complexity | Ratio |
|:---|:---:|:---:|:---:|
| P0 briefs + preflight | 9 | 3 | **3.0** — **approve** |
| Spine chain macro tool | 7 | 6 | 1.2 — approve after P0 |
| Postgres / AOI dashboard | 6 | 9 | 0.67 — **defer** |
