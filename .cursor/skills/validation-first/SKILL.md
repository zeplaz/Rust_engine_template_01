# Validation First Skill

Agents consume **ValidationReport JSON**, not raw terminal output.

## When to use

- After `cargo check`, `cargo test`, build, or clippy
- Before/after MCP tool execution
- GLB / asset QA
- Any time you would paste 100+ lines of compiler output into chat

## Quick workflow

1. Run validator (MCP or CLI):
   ```powershell
   python -m rust_engine_mcp.cli validate-report cargo --compress 3
   python -m rust_engine_mcp.cli validate-report bevy -p proc_A_dine01
   python -m rust_engine_mcp.cli validate-report asset_glb path/to/model.glb
   ```
2. Read `status`, `summary`, `errors[]`, `known_fixes[]` only
3. Act on `known_fixes` when `confidence >= 0.9`
4. Escalate to raw log only if report `confidence < 0.7`

## MCP tools

- `validate_cargo_report`
- `validate_bevy_report`
- `validate_asset_report`
- `validate_report` (generic)

## Token + queue (orchestration — use before reading big files)

- `agent_queue_next` / `agent_queue_update` — drain lane queue; never wait-only idle ([`plan_agent_queue_mcp_v1.md`](../../src/dev/plan_agent_queue_mcp_v1.md))
- `witness_brief` / `handoff_brief` / `file_digest` / `orchestrator_brief`
- `token_savings_guide` — policy reminder
- Prefer `compress=4` on validators when only pass/fail needed

## Schema

`tools/validators/schemas/validation_report_v1.schema.json`

## Knowledge base

`tools/validators/knowledge/error_signatures.json` — extend when you fix recurring errors.

## Related

- Orchestrator (full pipeline): `cargo orchestrate` → `tools/orchestrator/state/last_run.json`
- Plan: `src/dev/plan_validation_runtime_v1.md`
- Module production tier (not greybox smoke): `src/dev/plan_module_kit_production_tier_v1.md`
- Rule: `.cursor/rules/validation-first.mdc`

---

## Art pipeline — production vs smoke

**Pipeline smoke** (cube bpy ops, `kit_greybox_*` batches) proves MCP gates only.  
**Production modules** must satisfy [`design_procedural_module_kit_v1.md`](../../src/dev/design_procedural_module_kit_v1.md) validation contract (silhouette, pivot, PBR, canonical `module_id`).

| Tier | Purpose | May enter `_module_index.ron`? | Promote after |
|:---|:---|:---:|:---|
| `smoke` | MCP / Blender / witness harness | **No** (or `development_tier: smoke` only) | header parse only |
| `lod0` | PG-2 silhouette assembly in-engine | Yes, with tier flag | archetype silhouette + grid checks |
| `production` | StylePack / player-visible | Yes | full validation contract |

**Before promote (designer-mcp G3/G4):**

```powershell
python -m rust_engine_mcp.cli validate-report asset_glb path/to/model.glb --compress 3
python -m rust_engine_mcp.cli validate-report mcp_job path/to/job.json
```

**Reject promotion when report shows:**

- `development_tier` missing on new batches (defaults to smoke suspicion)
- `greybox:*` or `*_via_slab` tags used as final geometry excuse
- 24-vertex cube for non-box archetypes (pitched roof, arched window, sawtooth)
- No materials / PBR slots when `batch_id` matches `kit_production_*`
- `module_id` not in canonical kit inventory (§ Module inventory)

**Agents:** use `validate_asset_report` structured fields — never "green because glb exists."

**Escalation:** `@planner` for tier policy + Bevy load; `@designer-mcp` for AssetSpec; `@coder-mcp` for validator rules + bpy profiles.
