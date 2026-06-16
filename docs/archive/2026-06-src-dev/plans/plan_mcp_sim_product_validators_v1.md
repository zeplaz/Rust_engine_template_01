# MCP-P2-SIM-VALIDATORS-PLAN — sim product + ops validator toolchain `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **MCP-P2-SIM-VALIDATORS-PLAN-001** |
| **Program** | ⟨MCP-PRODUCTIVITY-P2-001⟩ · **secondary** to G-PLAY 🧩 (not art spine) |
| **Owner** | `@planner-mcp` → `@coder-mcp` |
| **Parent intel** | ⟨REVIEW-ORDER-MAP-VFX-UI-DEBUG⟩ · VR-10 · AGENT-LANG queue sync 2026-06-02 |
| **Status** | **SIGNED** (@planner-mcp 2026-06-13) |
| **Unblocks** | MCP-P2-OPS-BRIEF-002 · MCP-P2-VALID-CONSTRUCTION-001 · MCP-P2-QUEUE-PHASE4-001 |

---

## Lattice

```text
⟨MCP-P2-SIM-VALIDATORS-PLAN-001⟩  🟢⏳⊗☊
Lattice  Ct:🟩🟩  Cx:🟨🟨🟨  Au:🟩🟩  Q:🎯🟩🟩
Flow     ⊚session-intel ═▶ ⊚planner-mcp(plan) ═▶ ⊚coder-mcp(tools) ═▶ ⊚@coder(Rust witness export)
Review◈  🧠? art MCP ≠ sim validators — separate Tier 1e · ⊗! no bpy for viewport · ⚡! G-PLAY first
Result   🟢 witness_brief profiles + phase4 queue drain + BLANG:REVIEW
```

---

## Why (lessons from operator session)

| Lesson | Gap in shipped MCP | Planned tool |
|:---|:---|:---|
| Agents re-read HANDOFF + exec + VR-10 prose | `handoff_brief` lacks slice-specific exit/files | **`slice_exec_brief(id)`** |
| `witness_brief` generic — misses placement fields | No profile for construction/map-pick | **`witness_brief(path, profile=...)`** |
| Phase 4 queue not in `QUEUE_REGISTRY` | `agent_queue_next` ignores product polish rows | **`QUEUE_REGISTRY["phase4"]`** + default for `@coder` |
| Validation-first stops at cargo/bevy/glb | No compressed read for placement witness | **`validate_report construction`** |
| REVIEW-ORDER has 4 P0s across queues | No single compressed board | **`review_order_brief()`** |
| `ops_get_retry_guidance` stub | Not wired to phase4 blocker rows | **`ops_get_retry_guidance(task_id)`** v2 |
| Placement debug is Rust egui only | No JSON witness for MCP | **@coder:** `construction_placement_live.json` writer |

**Rule:** `@coder-mcp` owns **Python MCP/CLI** compressors. **`@coder`** owns **Rust witness export** (one JSON path). No chat-only diagnostics.

---

## Tier 1e — Sim product validators (**AUTHORIZED** — @coder-mcp implements Phases 1–3)

Register in `$ref:tools/mcp/MICRO_TOOLS_REGISTRY_v1.md` after ship.

| BLANG | MCP tool | CLI | Input | Output (compress 3–4) |
|:---|:---|:---|:---|:---|
| **`BLANG:REVIEW`** | `review_order_brief()` | `review-order-brief` | phase4 queue + `$ref:src/dev/visual_run_blockers.md` ids | P0-A…D status · blocker · ΔWF |
| **`BLANG:SLICE`** | `slice_exec_brief(id)` | `slice-exec-brief` | queue row + `$ref:exec.md` | exit · files · do_not_pick · witness path |
| **`BLANG:WIT`** (ext) | `witness_brief(path, profile=...)` | same | witness JSON | `construction` · `map_pick` · `fire_product` profiles |
| **`BLANG:PLACE`** | `validate_construction_report(path, compress=3)` | `validate-report construction` | `debug_runs/construction_*.json` | ValidationReport + known_fixes |
| **`BLANG:OPS`** (ext) | `ops_get_retry_guidance(task_id)` | `ops-get-retry-guidance` | phase4 row id | status · blocker · exec_doc · next_agent |
| — | `ops_get_active_blockers()` | `ops-get-active-blockers` | G-PLAY gate tensor | list of `{id, agent, exec_doc}` |

### Profiles for `witness_brief` (schema contract)

**`profile=map_pick`** — required keys when present:

```json
{
  "footprint_projection_ok": "bool",
  "cursor_delta_px": "number",
  "gpu_path_active": "bool",
  "authority_drift": "bool",
  "pick_tile": "[x,z]",
  "ghost_origin": "[x,z]"
}
```

**`profile=construction`** — rollup from `$ref:debug_runs/construction_stage_live.json`:

```json
{
  "construction_parametric_placement_001": { "green": "bool" },
  "footprint_projection_ok": "bool"
}
```

---

## Phase plan (@coder-mcp)

### Phase 1 — Queue + brief wiring (**MCP-P2-QUEUE-PHASE4-001**)

| # | Deliverable | Files |
|:---:|:---|:---|
| 1 | `QUEUE_REGISTRY["phase4"]` → `post_drain_phase4_queue.json` | `agent_queue.py` |
| 2 | `agent_queue_next("coder")` prefers phase4 when G-PLAY 🧩 | `agent_queue.py` |
| 3 | `slice_exec_brief(slice_id)` | `agent_queue.py` or `ops_intelligence.py` |
| 4 | `review_order_brief()` | `ops_intelligence.py` |
| 5 | CLI parity + MCP server registration | `cli.py`, `server.py` |
| 6 | pytest smoke | `tools/mcp/python/tests/test_review_order_brief.py` |

**Witness:** `debug_runs/agent_ops/mcp_phase4_queue_live.json`

### Phase 2 — Witness profiles (**MCP-P2-VALID-CONSTRUCTION-001**)

| # | Deliverable | Files |
|:---:|:---|:---|
| 1 | `witness_brief` profile dispatch | `agent_queue.py` |
| 2 | `validate_report construction` | `validators/construction_witness.py` |
| 3 | Hook `_cmd_validate_report` | `cli.py` |
| 4 | `validate_construction_report` MCP tool | `server.py` |
| 5 | Register `BLANG:PLACE` in token map | `agent_queue.py` token_savings_guide |

**Depends on @coder (parallel, not blocking Phase 1):**

- Export `$ref:debug_runs/construction_placement_live.json` from `$ref:src/construction/placement_debug.rs` on `--test vfx` exit or F3 toggle.

**Witness:** `debug_runs/agent_ops/mcp_valid_construction_live.json`

### Phase 3 — OPS retry guidance v2 (**MCP-P2-OPS-BRIEF-002**)

| # | Deliverable |
|:---:|:---|
| 1 | `ops_get_retry_guidance("TRIAGE-MAP-PICK-CLOSURE-001")` returns exec_doc + hotfix A steps |
| 2 | `ops_get_active_blockers()` reads `$ref:master_chain_tensor_v1.json` gates |
| 3 | Compose into `ops_get_project_brief` delta_wf section |

**Witness:** extend `debug_runs/agent_ops/ops_mcp_function_layer_live.json`

---

## SHIPPED vs PLANNED honesty

| Item | Label |
|:---|:---|
| `validate_cargo_report`, `validate_bevy_report`, `validate_asset_report` | **SHIPPED** |
| `witness_brief`, `handoff_brief`, `agent_queue_next` | **SHIPPED** (generic) |
| `ops_get_project_brief` | **SHIPPED** (v1 compose) |
| Tier 1e sim product validators | **AUTHORIZED** (signed 2026-06-13 — implement per Phases 1–3) |
| Postgres-backed ops fn_* | **DEFER** — JSON compose only until OPS gate S3 |
| Blender/building footprint mesh tools | **DEFER** — not viewport; `@designer-mcp` if catalog art |

---

## Agent routing

| Agent | Role |
|:---|:---|
| **@planner-mcp** | Sign this doc · no code |
| **@coder-mcp** | Phases 1–3 MCP/CLI |
| **@coder** | Rust `construction_placement_live.json` export · footprint hotfix A/B |
| **@orchestrator-mcp** | Sequence after `MCP-P2-KIT002-PLAN` or **parallel** secondary lane |
| **@designer-mcp** | **idle** — no new art tools for placement projection |

---

## Acceptance (planner sign-off)

| # | Criterion |
|:---:|:---|
| P1 | `python -m rust_engine_mcp.cli review-order-brief` lists P0-BUILD + 3 sibling rows |
| P2 | `agent_queue_next coder --queue phase4` returns `TRIAGE-MAP-PICK-CLOSURE-001` |
| P3 | `witness_brief debug_runs/construction_stage_live.json profile=construction` ≤ 40 lines JSON |
| P4 | `validate-report construction debug_runs/construction_placement_live.json` returns ValidationReport |
| P5 | `token_savings_guide` documents BLANG:REVIEW · BLANG:SLICE · BLANG:PLACE |

---

## References

- $ref:src/dev/plan_build_footprint_vm09_exec_v1.md
- $ref:src/dev/ops_mcp_function_layer_v1.md
- $ref:docs/archive/2026-06-src-dev/plans/plan_agent_queue_mcp_v1.md
- $ref:tools/orchestrator/queues/post_drain_phase4_queue.json
- $ref:.cursor/skills/validation-first/SKILL.md

---

**ΔWF→@coder-mcp:** Phase 1 after planner sign-off · **ΔWF→@coder:** placement witness export in parallel

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| **@planner-mcp** | 2026-06-13 | **SIGNED** — Tier 1e scope, phase boundaries, and acceptance P1–P5 are explicit. Unblocks `MCP-P2-QUEUE-PHASE4-001`, `MCP-P2-VALID-CONSTRUCTION-001`, `MCP-P2-OPS-BRIEF-002`. |

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Draft — READY for planner review |
| v1.1.0 | 2026-06-13 | **SIGNED** — @coder-mcp may implement Phases 1–3 |
