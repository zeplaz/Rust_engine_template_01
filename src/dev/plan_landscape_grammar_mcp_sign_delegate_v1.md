# PLAN-LANDSCAPE-GRAMMAR-MCP-SIGN — planner-mcp delegate `v1`

```text
⟦SYMLANG⟧⟐v1  ◈DELEGATE
⟨ID⟩ MCP-LANDSCAPE-GRAMMAR-SIGN-001
Date: 2026-06-14
Status: **SIGNED** (@planner-mcp 2026-06-14)
Delegated by: @planner (PLAN-QUEUE-SYNC-001 chain)
Parent exec: $ref:src/dev/plan_landscape_grammar_exec_001_v1.md LG-0
Queue: $ref:tools/orchestrator/queues/mcp_active_queue.json#MCP-LANDSCAPE-GRAMMAR-SIGN-001
```

**Scope:** @planner **SIGNED** charter + exec + lexicon. @planner-mcp owns **schema toolchain sign-off** and **preset validator CI** — not Rust ecology code.

---

## Deliverables (@planner-mcp)

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| **MCP-LG-SIGN-001** | Confirm `landscape_grammar_v0.schema.json` validates all presets under `assets/configs/landscape/presets/` | 0 schema errors |
| **MCP-LG-SIGN-002** | Add `validate-report landscape_grammar <path>` CLI parity (or document existing mcp_spec path) | compression 3 report |
| **MCP-LG-SIGN-003** | Preset catalog index JSON (`assets/configs/landscape/_preset_index.json`) with 30 topology_preset_id rows | matches lexicon §5 |
| **MCP-LG-SIGN-004** | SIGN row in mcp_active_queue · witness `debug_runs/mcp_landscape_grammar_sign_live.json` | `signed: true` |

**Do not:** bpy atlas · LG-5 tile bake · Bevy registry (coder consumer after SIGN).

---

## Dependency order

```text
MCP-LANDSCAPE-GRAMMAR-SIGN-001 (planner-mcp SIGN)
      ▼
MCP-LG-VALID-PRESET-001 (coder-mcp — optional Phase 2)
      ▼
VEG-F02-MCP-ATLAS-001 (blocked until kit002 + designer charter)
```

**Parallel OK with:** MCP-P2-KIT002-PLAN · ARCH-002 (separate lanes).

---

## Handoff to @coder-mcp (after SIGN)

Consumer-only tasks — no `src/systems/ecology/` edits:

- Batch validate new preset JSON drops
- Wire validate-report into orchestrator post_build hook (optional)
- LG-5 atlas spec when designer-mcp unblocks

---

## Sign-off

| Deliverable | Status |
|:---|:---|
| **MCP-LG-SIGN-001** | 10/10 ship presets pass `landscape_grammar_v0` schema |
| **MCP-LG-SIGN-002** | `validate-report landscape_grammar` — **SHIPPED** (LG-0-001) |
| **MCP-LG-SIGN-003** | `assets/configs/landscape/_preset_index.json` — 30 topology + 10 ship rows |
| **MCP-LG-SIGN-004** | `debug_runs/mcp_landscape_grammar_sign_live.json` — `signed: true` |

| Role | Date | Verdict |
|:---|:---|:---|
| **@planner-mcp** | 2026-06-14 | **SIGNED** — LG-0 MCP toolchain authoritative; consumer presets schema-valid |

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-14 | Delegate READY |
| v1.1.0 | 2026-06-14 | **SIGNED** — witness green |

---

## Handoff to @coder (after SIGN)

Unchanged — $ref:src/dev/plan_veg_runtime_proof_001_v1.md. Schema is authority; evaluator reads presets from disk.

```text
⟦/MCP-LANDSCAPE-GRAMMAR-SIGN-001⟧  ★ SIGNED 2026-06-14 · ΔWF→@coder-mcp MCP-LG-VALID-PRESET-001 (optional)
```
