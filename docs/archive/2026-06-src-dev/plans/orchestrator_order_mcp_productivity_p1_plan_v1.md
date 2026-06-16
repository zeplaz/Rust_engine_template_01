# Orchestrator explicit order — ⟨MCP-PRODUCTIVITY-P1-PLAN⟩

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **MCP-PRODUCTIVITY-P1-PLAN** |
| **Issuer** | @orchestrator only |
| **Assignee** | @planner-mcp |
| **φ** | 🟢 — $ref:docs/archive/2026-06-src-dev/plans/mcp_productivity_p1_plan_v1.md landed |
| **Parent** | $ref:docs/archive/2026-06-src-dev/plans/plan_mcp_productivity_chain_v1.md§P1 |
| **Rule** | No self-start — @planner-mcp idle until this paste is issued |

---

## P1 status (do not replan shipped)

| ⟨ID⟩ | φ | Owner |
|:---|:---:|:---|
| MCP-GRAMMAR-ITER-TOOL | 🟢 | @coder-mcp — done |
| MCP-SNAPSHOT-DIFF-001-IMPL | 🟢 | @coder-mcp — done |
| MCP-MAT-BRIEF-001 | 🟢 | @coder-mcp — done |
| **MCP-SPINE-CHAIN-001** | 🧊 | @coder-mcp — **needs thin plan** |
| **MCP-ATLAS-BRIEF-001** | 🧊 | @coder-mcp — **needs thin plan** |
| MCP-OPS-REPORT-001 | 🧊 | P2 — mention defer only |

**Unblocks after plan:** @coder-mcp may implement spine + atlas brief; @orchestrator may unddefer queue rows.

---

## Paste — @orchestrator → issue to @planner-mcp

```text
EXPLICIT ORDER ⟨MCP-PRODUCTIVITY-P1-PLAN⟩ — orchestrator authorized only.

Assign @planner-mcp NOW (readonly — no Python):

DELIVERABLE: docs/archive/2026-06-src-dev/plans/mcp_productivity_p1_plan_v1.md

SCOPE (thin plan only — 2–4 pages max):
  1. ⟨MCP-SPINE-CHAIN-001⟩ tile_spine_run
     - JSON request/response sketch (steps[], per-step witness)
     - Stop-on-fail + plain-language step errors
     - Honest gate: ship:false default · no headless-as-ship
     - Witness: debug_runs/tile_spine_run_001_live.json
     - Complexity budget vs chaining 6 CLIs manually

  2. ⟨MCP-ATLAS-BRIEF-001⟩ atlas_meta_brief
     - Input: atlas path / batch id
     - Output: UV grid summary, missing lookups, artist sentences (≤12 lines)
     - Depends: spine plan § integration hook only — not full spine ship
     - Witness: debug_runs/mcp_atlas_brief_001_live.json

  3. Registry delta table for MICRO_TOOLS_REGISTRY_v1.md (plan section — @coder-mcp implements)

  4. Undefer criteria for grammar queue rows MCP-SPINE-CHAIN-001 + MCP-ATLAS-BRIEF-001

READ (BLANG:DOC intent=ref):
  $ref:docs/archive/2026-06-src-dev/plans/plan_mcp_productivity_chain_v1.md§P1
  $ref:tools/mcp/python/rust_engine_mcp/mcp_productivity_p0.py
  $ref:src/dev/plan_dsm_wrk_atl_closure_v1.md

DO NOT:
  Reopen P0 tools · rewrite grammar queue · implement code · plan MCP-OPS-REPORT (P2 defer)

EXIT:
  mcp_productivity_p1_plan_v1.md on disk
  BLANG:Q✓ agent_queue_update("MCP-PRODUCTIVITY-P1-PLAN", "done", note=deliverable path)
  ⟨BP:SHARE⟩ joint: "@coder-mcp — spine step list matches assembly_build_run CLI flags?"

THEN ΔWF→@coder-mcp: implement only after plan merged — not parallel with plan write.
```

---

## Paste — @planner-mcp (receiver)

```text
You are @planner-mcp. EXPLICIT ORDER ⟨MCP-PRODUCTIVITY-P1-PLAN⟩ is ACTIVE.

Execute orchestrator paste above — deliver docs/archive/2026-06-src-dev/plans/mcp_productivity_p1_plan_v1.md only.

BLANG:PRE → BLANG:Q+("planner-mcp") → if slice ≠ MCP-PRODUCTIVITY-P1-PLAN, STOP and ask orchestrator.

Chain C remains maintain_only — this slice is the ONLY authorized planner-mcp work.
```

---

## After plan lands (@orchestrator)

```text
1. Review mcp_productivity_p1_plan_v1.md complexity budget ≥ 1.2
2. grammar queue: MCP-SPINE-CHAIN-001 status deferred → ready (if criteria met)
3. grammar queue: MCP-ATLAS-BRIEF-001 depends_on spine plan ack
4. ΔWF→@coder-mcp explicit paste from plan § implement order
5. Update master_chain_tensor_v1.json chains.A/B next rows
```

---

## Queue row

| Field | Value |
|:---|:---|
| id | MCP-PRODUCTIVITY-P1-PLAN |
| agent | planner-mcp |
| status | ready (orchestrator-gated) |
| deliverable | docs/archive/2026-06-src-dev/plans/mcp_productivity_p1_plan_v1.md |
| depends_on | AGENT-LANG-005-HANDOFF (done) |

---

## Changelog

| Ver | Date |
|:---|:---|
| v1.0.0 | 2026-06-07 | Explicit orchestrator order only |
