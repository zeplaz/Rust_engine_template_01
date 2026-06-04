# PLAN-MCP-RECOVERY-AND-FLEET-001 — Pipeline repair + MCP agent fleet `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-MCP-RECOVERY-001** |
| **Parent** | [`plan_designer_mcp_art_toolchain_exec_001_v1.md`](plan_designer_mcp_art_toolchain_exec_001_v1.md) |
| **Skills** | mcp-asset-pipeline · mcp-production-rules · blender-geometry · tile-generation |
| **Agents** | orchestrator-mcp · planner-mcp · designer-mcp · coder-mcp |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@planner-mcp` (architecture) · execution `@orchestrator-mcp` |
| **Status** | **R0 + FLT greybox DONE (2026-06-02)** — next: tile spec lane + Bevy G5 (`@coder`) |

---

## Summary

The MCP art lane has a **working Tier‑1 micro spine** (spec → geometry → validate → promote) but is **operationally sad** because documentation, gates, registry, witnesses, and agent contracts are **out of sync** with shipped tool names and **stop at promotion** without Bevy registry closure. Tile/material/reference lanes remain **PLANNED** and must not be scheduled as shipped.

This plan **(A)** repairs the spine to production-trustworthy state in **~2 weeks**, then **(B)** runs a **four-agent fleet** (`orchestrator-mcp`, `planner-mcp`, `designer-mcp`, `coder-mcp`) with explicit **parallel-safe** work packages and **G0–G5** gates so multiple batches (geometry modules, schema/docs, future tile spec) can proceed without authority collisions.

---

## Order critique

| Question | Verdict |
|:---|:---|
| New mega-MCP server? | **No** — extend `rust-engine-art` until Phase 4 split in exec plan |
| Is pipeline “broken”? | **Partially** — tools run; **trust layer** (witnesses, index, doc parity, seed enforcement) is missing |
| Start tile.generate now? | **No** — tile MCP is PLANNED; only **spec + schema prep** in parallel |
| Skip designer gate for speed? | **Rejected** — violates mcp-production-rules + orchestrator-mcp G4 |
| `@designer` vs `@designer-mcp`? | **Split enforced** — HUD stays `@designer`; all `tools/mcp/` work uses **designer-mcp** |

---

## Current state (reconciled 2026-06-02)

| Component | Label | Notes |
|:---|:---:|:---|
| `rust_engine_mcp` server + CLI | **SHIPPED** | Flat `server.py`; 19 MCP tools in Cursor |
| Tier-1 + validators + library + witness | **SHIPPED** | See [`MICRO_TOOLS_REGISTRY_v1.md`](../../tools/mcp/MICRO_TOOLS_REGISTRY_v1.md) |
| Blender ops wall/roof/door/window/prop | **SHIPPED** | `kit_greybox_001` + `kit_greybox_002` batches promoted |
| `promote` + validate_glb in promote path | **SHIPPED** | G4 sign-off YAML: `debug_runs/art_pipeline/kit_greybox_001_signoff.yaml` |
| `assets/models/modules/*` | **SHIPPED** | 20+ greybox modules |
| `_module_index.ron` + `library_*` MCP | **SHIPPED** | `assets/configs/buildings/_module_index.ron` |
| `write_witness` | **SHIPPED** | e.g. `kit_greybox_002_live.json` |
| RON sidecar from AssetSpec on promote | **PLANNED** | Copies `.module.json` if present; no generator |
| `art_validator` Rust crate | **PLANNED** | Grid/pivot rules not enforced in Rust |
| gltf-transform / Material Maker | **PLANNED** | Tier 3 registry |
| Tile MCP + atlas packer | **PLANNED** | Draft only — `tools/mcp/schemas/drafts/tile_batch_*.json` |
| Agent doc tool names | **REPAIRED** | `designer-mcp.md`, `planner-mcp.md`, onboarding, README |
| Bevy `RepresentationResult` hook | **NEXT** | `@coder` after G5 — witness `next` field in batch live JSON |
| Local `python` without venv | **RISK** | Bare 3.14 → `jsonschema` missing; use MCP env / `install_designer_mcp.ps1` |

---

## Target architecture (post-recovery)

```text
@orchestrator-mcp
  G0 designer-mcp rules audit
  G1 designer-mcp AssetSpec / job JSON
  G2 planner-mcp (only if schema/tool gap) + coder-mcp impl
  → geometry_run_job (MCP or CLI — same function)
  → validate_glb_asset
  G3 validate green
  G4 designer-mcp staging sign-off YAML
  → promote_staging_module
  G5 coder-mcp: library_register + _module_index.ron + witness JSON
  → @coder (engine): BuildingDefinition / RepresentationResult path
```

**Authority**

| Artifact | Writer |
|:---|:---|
| AssetSpec / geometry job JSON | designer-mcp |
| Staging glb | Blender via coder-mcp toolchain |
| `ResolvedViewports` / sim | **never** MCP lane |
| `_module_index.ron` | coder-mcp `library_register` (new) |
| Promotion | MCP/CLI `promote_staging_module` only after G3+G4 |

---

## Wave R0 — Pipeline repair (sequential spine, ~10 days)

**Goal:** Make the existing spine **trustworthy** before parallel fleet scale-up.

### R0-0 — Cursor user config (coder-mcp, same day)

| Task | Acceptance |
|:---|:---|
| Run `tools/mcp/install_designer_mcp.ps1` | Writes `~/.cursor/mcp.json` + `~/.cursor/rust_engine_art_mcp.env` |
| `tools/mcp/scripts/verify_mcp_setup.ps1` green | ping, blender, pytest |
| Python **3.13** pinned in MCP env (`RUST_ENGINE_PYTHON`) | Not bare `python` 3.14 |
| Restart Cursor → **rust-engine-art** green in Settings → MCP | |

**Asset work queue:** [`tools/mcp/schemas/examples/batch_kit_greybox_001.manifest.json`](../../tools/mcp/schemas/examples/batch_kit_greybox_001.manifest.json) — first batch after config green.

### R0-1 — Doc + contract parity (coder-mcp, 1 day)

| Task | Acceptance |
|:---|:---|
| Align all agent/skill references to **shipped** tool names | Grep clean: no `geometry_submit_job`, `validate_asset`, `spec_create` as primary |
| Update [`designer.md`](../../.cursor/agents/designer.md) MCP section → pointer to designer-mcp + shipped names | |
| Add **SHIPPED vs PLANNED** table to [`tools/mcp/README.md`](../../tools/mcp/README.md) status section | Matches this plan |

### R0-2 — Dev env + smoke harness (coder-mcp, 1 day)

| Task | Acceptance |
|:---|:---|
| Pin Python in onboarding (3.13 venv path from README, not bare 3.14) | `pip install -r requirements.txt && pip install -e .` |
| `python -m pytest tests/` green on designer machine | |
| Add `tools/mcp/scripts/smoke_geometry.ps1` | Runs example wall job → validate → **no promote** → writes witness |

### R0-3 — Witness envelope (coder-mcp, 2 days)

| Task | Acceptance |
|:---|:---|
| After validate/promote, write `debug_runs/art_pipeline_live.json` | Uses [`debug_run_envelope.rs`](../../src/dev/debug_run_envelope.rs) pattern (JSON fields: job_id, valid, paths, gates) |
| Batch witness `art_pipeline_batch_<id>.json` for multi-job runs | |

### R0-4 — Schema hardening (coder-mcp + designer-mcp, 2 days)

| Task | Acceptance |
|:---|:---|
| Add optional `seed` to `geometry_job_v1` when params imply variation | Reject in runner if variation flags without seed |
| Remove `module_window` / `module_prop` from enum **or** stub bpy ops | No schema ops without implementation |
| Pre-run: `validate_geometry_job` in CLI/MCP before Blender spawn | Same function both paths |

### R0-5 — Library closure (coder-mcp, 3 days)

| Task | Acceptance |
|:---|:---|
| Create `assets/configs/buildings/_module_index.ron` schema + empty seed | |
| MCP/CLI `library_register(job_id)` | Updates index after promote |
| `library_search(tags?)` read-only | |
| Promote optionally emits RON sidecar from AssetSpec (`assets/configs/buildings/modules/<id>.ron`) | Fields from [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) |

### R0-6 — Designer gate artifact (designer-mcp, 1 day)

| Task | Acceptance |
|:---|:---|
| Standard YAML `staging_signoff_v1` template | `proceed: yes|no`, checked paths, scale/pivot checklist |
| orchestrator-mcp **blocks promote** in runbooks until G4 file referenced in HANDOFF | |

**R0 rollback trigger:** Blender missing on machine → still pass schema/pytest; geometry smoke skipped with witness flag `blender_skipped: true`.

---

## Wave F1 — Agent fleet (parallel work packages)

**Entry:** R0-2 complete (pytest + smoke script exist).

### Fleet roles (unchanged agents, explicit boundaries)

| Agent | Parallel? | Owns |
|:---|:---:|:---|
| **orchestrator-mcp** | Coordinates | Phase graph, gate summaries, HANDOFF |
| **planner-mcp** | Yes (read-only spikes) | Schema plans, lane splits, SHIPPED labels |
| **designer-mcp** | Yes (per batch id) | AssetSpec batches, sign-off, tile **spec only** |
| **coder-mcp** | Yes (non-overlapping files) | Python/bpy/tests; **one coder-mcp per bpy op file** |

**Escalation:** Bevy registry / ECS → `@planner` + `@coder`. Never implement in planner-mcp.

### Parallel lanes (after G0 pass per batch)

```text
Lane A (Geometry production)     Lane B (Tooling hardening)      Lane C (Tile — spec only)
designer-mcp: module batch specs  coder-mcp: R0-3..R0-5           designer-mcp: tile_batch_v1 draft
coder-mcp: bpy op files           planner-mcp: art_validator plan  planner-mcp: tile schema v1
orchestrator-mcp: gates           orchestrator-mcp: R0 sequencing   DEFER execution until tile tool shipped
```

| Lane | Parallel safe with | Must stay sequential |
|:---|:---|:---|
| A vs B | Different job_ids; A uses finished tools; B touches `promote.py` | Same `job_id` folder |
| A vs C | C is JSON-only in `tools/mcp/schemas/drafts/` | C calling `tile.generate` |
| B internal | `validate_glb.py` vs `library_register.py` | `promote.py` + index writer same PR if atomic register |

### First production batch (designer-mcp + orchestrator-mcp)

**ART-KIT-R0:** 10 greybox modules (2 walls, 2 roofs, 2 doors, 2 props, 2 variants) from procedural kit — **one geometry job per module**, shared `batch_id: kit_greybox_001`.

| Gate | Owner | Parallel tasks |
|:---|:---|:---|
| G0 | designer-mcp | Single rules_audit for batch |
| G1 | designer-mcp | 10 AssetSpecs (can parallelize authoring in one session) |
| G2 | coder-mcp | Only if new op needed per spec |
| Execute | MCP/CLI | **Parallel** Blender jobs **only** if distinct `job_id` dirs |
| G3 | coder-mcp | validate all glbs → batch witness |
| G4 | designer-mcp | One sign-off per batch |
| G5 | coder-mcp | `library_register` × N + index update |

---

## Implementation phases (coder-mcp workboard)

### Phase P1 — Recovery (maps to R0)

| ID | Goal | Owner | Witness |
|:---|:---|:---|:---|
| **MCP-REC-001** | Doc/tool name parity | coder-mcp | grep |
| **MCP-REC-002** | pytest + smoke script | coder-mcp | pytest green |
| **MCP-REC-003** | art_pipeline_live.json | coder-mcp | file exists |
| **MCP-REC-004** | seed + schema/op alignment | coder-mcp + designer-mcp | invalid job rejected |
| **MCP-REC-005** | library_register + index | coder-mcp | `_module_index.ron` has entries |

### Phase P2 — Fleet batch (maps to F1)

| ID | Goal | Owner |
|:---|:---|:---|
| **MCP-FLT-001** | orchestrator-mcp runbook in [`tools/orchestrator/agents/`](../../tools/orchestrator/agents/) `mcp_art_pipeline_agent.md` | orchestrator-mcp |
| **MCP-FLT-002** | 10-module greybox batch + batch witness | designer-mcp + coder-mcp |
| **MCP-FLT-003** | HANDOFF template row for MCP lanes | orchestrator-mcp |

### Phase P3 — Hardening (exec plan continuation)

| ID | Goal | Label |
|:---|:---|:---|
| **MCP-HARD-001** | `tools/art_validator` Rust crate | PLANNED |
| **MCP-HARD-002** | gltf-transform adapter | PLANNED |
| **MCP-HARD-003** | tile_batch schema + tool stub (returns not_implemented) | PLANNED |

---

## Schema plan

| Schema | Version | Action |
|:---|:---:|:---|
| `geometry_job_v1` | 1 → 1.1 | Add `seed`, `batch_id`; trim enum to shipped ops |
| `asset_spec_v1` | 1 | No breaking change |
| `staging_signoff_v1` | **new** | designer-mcp gate artifact |
| `module_index_v1` | **new** | RON + JSON schema for index entries |
| `tile_batch_v1` | draft | PLANNED — `tools/mcp/schemas/drafts/` only |

---

## Gate alignment (orchestrator-mcp)

| Gate | Blocks | Recovery phase |
|:---|:---|:---|
| G0 Rules | all tool runs | R0 signoff template |
| G1 Spec | geometry_run_job | R0-4 |
| G2 Tooling | execution | R0-2 smoke |
| G3 Validate | promote | existing + witness |
| G4 Designer review | promote | R0-6 |
| G5 Registry | Bevy slice | R0-5 |

---

## Edge cases

| Case | Handling |
|:---|:---|
| Blender absent on CI | smoke sets `blender_skipped`; pytest still runs |
| Mid-batch validation fail | orchestrator-mcp halts lane; no partial promote |
| `force=True` promote | Only coder-mcp with HANDOFF note; witness records `forced: true` |
| Doc/agent drift again | MCP-REC-001 is recurring gate on any tool rename PR |

---

## Open questions

| # | Blocker | For |
|:---:|:---|:---|
| 1 | Who owns first `_module_index.ron` seed content — designer-mcp or coder-mcp? | **Recommend:** coder-mcp generates empty; designer-mcp approves first 10 entries |
| 2 | Minimum Bevy load path for promoted `model.glb` — which `BuildingDefinition` field? | `@planner` + `@coder` before G5 engine slice |
| 3 | Split MCP servers (exec Phase 4) — defer until REC + FLT green? | **Yes** — DEFER |
| 4 | Material Maker install on designer machine? | User env — optional Tier 3 |

---

## Delegation — first `@orchestrator-mcp` sprint

```md
## MCP Execution Plan — Sprint REC+FLT-001

### Phase 0: Critique + rules
- Task 0.1 → designer-mcp
  Goal: rules_audit YAML for "pipeline recovery + greybox batch"
  Gate: G0
  Acceptance: proceed != no

### Phase 1: Recovery tooling (parallel after G0)
- Task 1.1 → coder-mcp — MCP-REC-001 doc parity
- Task 1.2 → coder-mcp — MCP-REC-002 pytest + smoke_geometry.ps1
  Deps: none between 1.1 and 1.2
  Gate: G2 partial

### Phase 2: Witness + library (sequential)
- Task 2.1 → coder-mcp — MCP-REC-003 + MCP-REC-005
  Gate: G3 infrastructure
- Task 2.2 → designer-mcp — MCP-REC-004 seed examples + R0-6 template
  Gate: G1

### Phase 3: Greybox batch (fleet)
- Task 3.1 → designer-mcp — 10 AssetSpecs batch_id kit_greybox_001
- Task 3.2 → coder-mcp / MCP — parallel geometry jobs
- Task 3.3 → designer-mcp — G4 sign-off
- Task 3.4 → coder-mcp — promote + library_register + batch witness
  Gate: G3, G4, G5
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Recovery diagnosis + four-agent fleet parallel model |
