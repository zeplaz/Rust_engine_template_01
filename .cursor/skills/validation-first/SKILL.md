`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

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

## AGENT-LANG ritual (attach [agent-lang](../agent-lang/SKILL.md))

```text
BLANG:PRE → BLANG:Q+ → BLANG:CARGO|BEVY → BLANG:Q✓
```

| BLANG | This skill |
|:---|:---|
| `BLANG:CARGO` | `validate_cargo_report(compress=4, use_cached=true)` |
| `BLANG:BEVY` | `validate_bevy_report(compress=4)` |
| `BLANG:WIT` | After tests — witness path only 🟢/🔴 |

**Status:** 🟢 pass · 🟡 qualified (notes in `summary`) · 🔴 `status=failed` — escalate raw log only if `confidence < 0.7`.

**Refs:** `$ref:tools/validators/schemas/validation_report_v1.schema.json` · `$sym:validate_cargo_report@tools/mcp/python/rust_engine_mcp/`

## Token + queue (orchestration — use before reading big files)

- `agent_queue_next` / `agent_queue_update` — drain lane queue; never wait-only idle ([`plan_agent_queue_mcp_v1.md`](../../docs/archive/2026-06-src-dev/plans/plan_agent_queue_mcp_v1.md))
- `witness_brief` / `handoff_brief` / `file_digest` / `orchestrator_brief`
- `token_savings_guide` — policy reminder
- Prefer `compress=4` on validators when only pass/fail needed

## Schema

`tools/validators/schemas/validation_report_v1.schema.json`

## Knowledge base

`tools/validators/knowledge/error_signatures.json` — extend when you fix recurring errors.

## Related

- **agent-lang** — BLANG tokens, `$ref`, stream delimiters
- Orchestrator (full pipeline): `cargo orchestrate` → `tools/orchestrator/state/last_run.json`
- Plan: `docs/archive/2026-06-src-dev/plans/plan_validation_runtime_v1.md`
- Module production tier (not greybox smoke): `docs/archive/2026-06-src-dev/plans/plan_module_kit_production_tier_v1.md`
- Rule: `.cursor/rules/validation-first.mdc`

---

## Art pipeline — production vs smoke

**Pipeline smoke** (cube bpy ops, `kit_greybox_*` batches) proves MCP gates only.  
**Production modules** must satisfy [`design_procedural_module_kit_v1.md`](../../docs/archive/2026-06-src-dev/plans/design_procedural_module_kit_v1.md) validation contract (silhouette, pivot, PBR, canonical `module_id`).

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
