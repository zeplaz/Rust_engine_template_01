# PLAN-OVR-QUEUE-REGISTRY-001 — APS UI/UX overhaul queue registry `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-OVR-QUEUE-REGISTRY-001
Date: 2026-06-17
Status: **SIGNED** (@planner-mcp)
Parent: PLAN-APS-UIUX-OVERHAUL-001
```

**Goal:** Register `aps_uiux_overhaul_queue.json` in `queue_registry_v1.json` for `validate-report queue_integrity`.

## Deliverable

| Field | Value |
|:---|:---|
| `queue_id` | `aps_uiux_overhaul` |
| `path` | `tools/orchestrator/queues/aps_uiux_overhaul_queue.json` |
| `rows_path` | `drain` |
| `id_field` | `id` |
| `status_field` | `status` |

## Acceptance

| # | Criterion | Pass |
|:---:|:---|:---:|
| R1 | Row present in `tools/mcp/schemas/queue_registry_v1.json` | ✓ |
| R2 | `validate-report queue_integrity` indexes queue without parse error | ✓ |

```text
⟦/PLAN-OVR-QUEUE-REGISTRY-001⟧
```
