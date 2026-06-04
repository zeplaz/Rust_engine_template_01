# MCP orchestrator snap — CURRENT (authoritative)

**Date:** 2026-06-03

| Doc | Role |
|:---|:---|
| **This file** | Dispatch truth |
| [`mcp_fleet_production_sprint_rowhouse_v1.md`](../../src/dev/mcp_fleet_production_sprint_rowhouse_v1.md) | **Paste prompts + week plan** |
| [`mcp_fleet_production_pilot_rowhouse_v1.md`](../../src/dev/mcp_fleet_production_pilot_rowhouse_v1.md) | Unfreeze scope |
| [`mcp_active_queue.json`](mcp_active_queue.json) | Machine queue |
| [`HANDOFF.md`](HANDOFF.md#production-sprint-rowhouse) | Session handoff |

---

## Status

| Program | State |
|:---|:---|
| MCP-ART-PROGRAM-GREEN-001 | **CLOSED** |
| MCP-PROD-SPRINT-ROWHOUSE-001 | **ACTIVE** |

---

## Dispatch order (strict)

```text
Week 1 — @coder-mcp: MCP-PROD-B2 → MCP-PROD-C-PILOT
         @designer-mcp: MCP-PROD-PBR-PILOT (parallel)

Week 2 — @designer-mcp: MCP-PROD-MOD-G0-G5 → MCP-PROD-ATLAS-G0-G4
         @coder-mcp: MCP-PROD-TILE-VAL + MCP-PROD-INDEX (after G4)

Week 3 — @coder: ENG-PT-4-001 → ENG-PT-5-001
         @designer: on-call rowhouse player read
```

**Copy prompts from:** [`mcp_fleet_production_sprint_rowhouse_v1.md`](../../src/dev/mcp_fleet_production_sprint_rowhouse_v1.md) § Paste prompts

---

## Post TILE-FIX spine (`@coder-mcp` only)

Plan: [`plan_building_tile_spine_001_v1.md`](../../src/dev/plan_building_tile_spine_001_v1.md). **Do not** assign implementation to `@coder`.

| ID | Owner | Notes |
|:---|:---|:---|
| ARCH-003 | **@coder-mcp** | Assembly snapshot `material_profile` + tags per placement |
| APS-UI-003b | **@coder-mcp** | Assembly Editor (Suite tab); not slot-grid-only |
| BUILD-001 | **@coder-mcp** | Build graph nodes + per-node witness |
| RENDER-001 | **@coder-mcp** | Headless blender-worker; production bake when green — not DEHACK-RENDER-001 |

**Blocked on:** ARCH-001/002 specs (@planner-mcp); warehouse ship = manual keyframe + designer G4 ([`mcp_orchestrator_tile_fix_warehouse_slice_v2.md`](../../src/dev/mcp_orchestrator_tile_fix_warehouse_slice_v2.md)).

**@coder** after PILOT-001: RUNTIME-001 / ENG-PT-4 map stamp (index row only after green register).

---

## Unfrozen / frozen

**Unfrozen:** `kit_production_001` · `tile_rowhouse_victorian_production_v1`  
**Frozen:** `kit_production_002+` · other archetype production batches · `kit_greybox_004+`
