# MCP lane order — `@orchestrator-mcp` begin work (SYMLANG)

**Canonical:** `$ref:prompts/SYMBOLIC_LANGUAGE.meta.md` · **Machine queue:** `$ref:tools/orchestrator/queues/mcp_active_queue.json` · **Snap:** `$ref:tools/orchestrator/queues/mcp_orchestrator_snap_CURRENT.md`

```text
⟨MCP-LANE-ORDER⟩ 🟢⏳  ◕
 Lattice  Au:🏛🟩🟩🟩  Ct:🟨🟨  Q:🎯🟩🟩
 Flow     ⊚orchestrator-mcp ═▶ ⊚HANDOFF ⊐ ⊚mcp_active_queue ⊐ ⊚G0…G5 graph
 Program  ⟨MCP-PRODUCTIVITY-P2-001⟩  ·  rowhouse ★  ·  ⟨MCP-P2-SIM-VALIDATORS⟩ secondary
 NEXT     ΔWF→@coder-mcp ⟨MCP-P2-QUEUE-PHASE4-001⟩ + ⟨MCP-P2-VALID-CONSTRUCTION-001⟩ (parallel — plan SIGNED)
          ΔWF→@planner-mcp ⟨MCP-P2-KIT002-PLAN⟩ · ⟨ARCH-002⟩ (orthogonal planner drain)
```

---

## P0 operator blocker (Phase 4 — not MCP)

```text
⟨G-PLAY-01⟩ 💬 OPEN 🧩 ⟨TRIAGE-MAP-PICK-CLOSURE-001⟩ alias ⟨P0-BUILD-FOOTPRINT-001⟩
$ref:src/dev/plan_build_footprint_vm09_exec_v1.md
ΔWF→@coder ONLY · @orchestrator-mcp @coder-mcp @designer-mcp ⏸ secondary
```

---

## Session boot (every `@orchestrator-mcp` turn)

```text
BLANG:STATS → BLANG:BOOT(agent=orchestrator-mcp) → BLANG:HO → orchestrator_mcp_lane_brief()
```

| Step | CLI / MCP | Purpose |
|:---|:---|:---|
| `BLANG:BOOT` | `agent_session_bootstrap(agent='orchestrator-mcp')` | FIELD◈ · SYMLANG◈ digest |
| `BLANG:HO` | `handoff_brief()` | AUTH spine + agent drain |
| **Lane pick** | `orchestrator-mcp-lane-brief` | P2 task graph + recommend_next |

**Role reads:** `$ref:tools/mcp/README.md` · `$ref:tools/mcp/MICRO_TOOLS_REGISTRY_v1.md` · `$ref:docs/archive/2026-06-src-dev/plans/plan_mcp_productivity_chain_v1.md`

---

## Lane split (do not collapse)

| Agent | Owns |
|:---|:---|
| `@orchestrator-mcp` | Phase graph · G0–G5 gates · queue picks · HANDOFF close |
| `@planner-mcp` | New tool/schema architecture · unfreeze criteria |
| `@designer-mcp` | G0 rules audit · G1 spec · G4 sign-off |
| `@coder-mcp` | G2 tooling · G3 validate · G5 registry |
| `@orchestrator` | ECS/render spine only — not bpy batches |

---

## P1 close (do not replan)

| ⟨ID⟩ | φ | Witness |
|:---|:---:|:---|
| MCP-PRODUCTIVITY-P1-PLAN | ★ | `$ref:docs/archive/2026-06-src-dev/plans/mcp_productivity_p1_plan_v1.md` |
| MCP-SPINE-CHAIN-001 | ★ | `$ref:debug_runs/tile_spine_run_001_live.json` |
| MCP-ATLAS-BRIEF-001 | ★ | `$ref:debug_runs/mcp_atlas_brief_001_live.json` |
| MCP-PROD-SPRINT-ROWHOUSE-001 | ★ | `$ref:debug_runs/art_pipeline/rowhouse_production_atlas_g0_g4_live.json` |

---

## P2 active graph (form A — gated pipeline)

```text
◎close-P1★ ▷⊳ ◆route P2
  ├─═[plan-needed]▶ ⦿planner-mcp ▷⊳ ⟨MCP-P2-SIM-VALIDATORS-PLAN-001⟩ ⬡ops·validators
  ├─═[plan-needed]▶ ⦿planner-mcp ▷⊳ ⟨MCP-P2-KIT002-PLAN⟩ ⬡G0·scope
  ├─═[schema]▶      ⦿planner-mcp ▷⊳ ⟨ARCH-002⟩ ⬡variant-graph
  ├─═[resilience]▶   ⦿coder-mcp   ▷⊳ ⟨MCP-P2-RUN-EVENT-001⟩ ⬡G2 ★
  └─═[quality]▶     ⦿coder-mcp   ▷⊳ ⟨MCP-P2-HONEST-BAKE-001⟩ ⬡G3 ★
⧗ paused: ⟨MCP-PILOT-GRAMMAR-001⟩ Track B — operator manual keyframe only
⛔ frozen: kit_production_002+ until ⟨MCP-P2-KIT002-PLAN⟩ ★ ∧ designer G0
⛔ coder-mcp blocked: ⟨MCP-P2-OPS-BRIEF-002⟩ until ⟨MCP-P2-QUEUE-PHASE4-001⟩ done
```

### Task order (strict)

| # | ⟨ID⟩ | Agent | Gate | Goal |
|:--|:---|:---|:---|:---|
| 1 | **MCP-P2-SIM-VALIDATORS-PLAN-001** | @planner-mcp | ops plan | **SIGNED** 2026-06-13 — unblocked Phase 1+2 |
| 2 | **MCP-P2-KIT002-PLAN** | @planner-mcp | G0 scope | Unfreeze criteria + manifest sketch for `kit_production_002` |
| 3 | **ARCH-002** | @planner-mcp | schema | `variant_graph_v1.schema.json` — VariantNode overrides |
| 4 | **MCP-P2-RUN-EVENT-001** | @coder-mcp | G2 | `agent_run_append` MCP wrapper (OPS Phase 1 — no Postgres) ★ |
| 5 | **MCP-P2-HONEST-BAKE-001** | @coder-mcp | G3 | `tile_promotion_honest_check` — reject headless-as-ship before G4 ★ |
| — | MCP-OPS-REPORT-001 | @coder-mcp | 🧊 P3 defer | Postgres gate — mention only |

---

## Delegate paste — cycle 1 (issue now)

```text
⟨EXPLICIT-ORDER⟩ ⟨MCP-P2-KIT002-PLAN⟩ ⚡P0
Issuer: @orchestrator-mcp only
Assign: @planner-mcp (readonly)

DELIVERABLE: docs/archive/2026-06-src-dev/plans/mcp_kit_production_002_unfreeze_v1.md (thin — ≤3 pages)

SCOPE:
  · Unfreeze criteria for kit_production_002 (production tier contract)
  · batch manifest sketch — tools/mcp/schemas/examples/batch_kit_production_002.manifest.json
  · G0–G5 gate table for designer-mcp / coder-mcp slices
  · Explicit ¬reopen: rowhouse sprint witnesses · P1 spine tools

READ (BLANG:DOC intent=ref):
  $ref:docs/archive/2026-06-src-dev/plans/plan_module_kit_production_tier_v1.md
  $ref:tools/orchestrator/queues/mcp_active_queue.json
  $ref:debug_runs/art_pipeline/kit_production_001_live.json

EXIT: plan on disk → BLANG:Q✓ → ΔWF→@designer-mcp ⟨MCP-P2-KIT002-G0⟩ (rules audit before any bpy)
```

---

## After plan ★ — cycle 2

```text
ΔWF→@coder-mcp ⟨MCP-P2-RUN-EVENT-001⟩
  BLANG:PRE → implement agent_run_append wrapper → BLANG:PY → BLANG:WIT
ΔWF→@coder-mcp ⟨MCP-P2-HONEST-BAKE-001⟩ (parallel only if ¬same staging folder)
```

---

## Stream / handoff

| Delimiter | When |
|:---|:---|
| `⟨CONT⟩` | Same P2 program — `$ref:mcp_lane_order_v1.md` + last ⟨ID⟩ |
| `⟨BRK⟩` | Lane exit → update `$ref:HANDOFF.md` + `ops_intelligence_scan` |
| `⟨DRIFT⟩` | Re-anchor: `orchestrator_mcp_lane_brief()` + tensor AUTH |

```text
⟦/MCP-LANE-ORDER⟧ NEXT ⚑ boot → lane-brief → issue explicit order → G0 before tools
```
