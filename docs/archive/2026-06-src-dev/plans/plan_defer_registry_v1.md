# PLAN-DEFER-REGISTRY-001 — signed defer list `v1`

| Field | Value |
|:---|:---|
| **Registry ID** | **PLAN-DEFER-REGISTRY-001** |
| **Machine index** | [`tools/orchestrator/queues/defer_registry.json`](../../tools/orchestrator/queues/defer_registry.json) |
| **Status** | **SIGNED** |
| **Date** | 2026-06-03 |
| **Rule** | Orchestrator / agents **must not** promote DEFER rows to `active[]` without planner sign-off |

---

## Active deferrals

| ID | Track | Reason | Un-defer when |
|:---|:---|:---|:---|
| **MCP-PILOT-GRAMMAR-001** | B | Manual keyframe ship; headless bake ≠ ship proof | Operator keyframe + G4 checklist + `tile_promotion_honest_check` green |
| **TRACK-B-G4-SHIP** | B | Warehouse integration test paused | Above + honest bake MCP |
| **APS-ARTIST-TOOL-E2E-001** | A | CLI E2E green; **product** sign-off not operator gate | Designer-mcp sign-off row + DSM ATL★ |
| **APS-PHASE-9-PRODUCT-GATE** | A | Same as E2E — witness exists, gate deferred | WRK★ + ATL★ per [`plan_dsm_wrk_atl_closure_v1.md`](plan_dsm_wrk_atl_closure_v1.md) |
| **TILE-SPINE-RUN-WAREHOUSE** | B | `tile_spine_run` MCP — integration test only | Track B unpause OR explicit orchestrator request |
| **EGUI-DEV-UX-001** | A′ | Bevy QC HUD polish — dev surface | After APS-UX-ASYNC-001 |
| **PLAN-AUDIT-020** | planner | Next fleet audit | G-PLAY-01 operator EXECUTED + E5-002 green |

---

## Not defer (common wrong picks)

| ID | Status |
|:---|:---|
| CON-P2 / CON-P3 / organic save/approve/policy | **CLOSED** — audit v19 |
| INFRA E0–E3 + B-half | **CLOSED** |
| APS-UX-ASYNC-001 | **ACTIVE** — not defer |
| MCP P0 briefs (preflight, digest, p0 plain) | **ACTIVE** |
| GRAMMAR-ITER-001-APS1/API | **ACTIVE** |
| INFRA-E4/E5/E6 A-tail | **ACTIVE** |
| WEATHER-WITNESS-001 | **ACTIVE** (coder C) |

---

## Queue hygiene

When adding a row to `defer_registry.json`:

```json
{
  "id": "EXAMPLE-ID",
  "track": "A|B|planner",
  "reason": "one line",
  "unblock_when": "witness or gate id",
  "signed": "2026-06-03"
}
```

**Orchestrator:** paste picks from audit v19 § Active work — cross-check defer registry before assign.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Initial signed defer list |
