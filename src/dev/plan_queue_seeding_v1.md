# Plan — queue seeding after drain `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **PLAN-QUEUE-SEEDING-001** |
| **Date** | 2026-06-08 |
| **Audience** | @planner · @orchestrator |
| **Active queue** | $ref:tools/orchestrator/queues/post_drain_phase3_queue.json |
| **Prior** | Phase 1 + Phase 2 **drained** |

---

## Problem

When witnesses close faster than queue rows, agents go **idle** while HANDOFF still lists stale picks. This is a **process** failure, not lack of architecture work.

**Symptoms:**
- `post_drain_phase2_queue.json` shows `ready` for slices already green (EGUI-QC, TRIAGE-REPLAY)
- HANDOFF §open slices table contradicts `grammar_continuation_queue.json`
- @coder idle while triage backlog T3–T5 has unpromoted rows

---

## Seeding loop (every drain close)

```text
1. WITNESS SCAN   — unified_witness_index.json + program witnesses
2. QUEUE CLOSE    — mark done/cancelled where green (no re-implementation)
3. TRIAGE PICK    — promote 1–3 rows from stage5_triage_backlog.md per agent
4. DEFER REVIEW   — after DSM milestones, review defer_registry.json
5. THIN EXEC      — PLAN-PHASE{N}-EXEC-001 witness keys only
6. HANDOFF SYNC   — agent drain table + remove stale open-slice tables
7. DISPATCH       — paste blocks per agent (planner_dispatch_prompts pattern)
```

**Rule:** Never assign implementation when witness is already green — only **verify + close row**.

---

## Promotion criteria (triage → active queue)

Promote a triage row when **all** of:

| # | Check |
|:---:|:---|
| 1 | Not a Stage 5/6 closed gate reopen |
| 2 | Witness file named or creatable in `debug_runs/` |
| 3 | Single owner agent (or paired designer→coder) |
| 4 | Thin exec or existing plan doc with exit keys |
| 5 | Does not preempt G-PLAY-01 operator session |

**Do not promote:** warehouse Track B · duplicate grammar/MCP rows · egui-as-sim-HUD (lane 4 ≠ lane 5).

---

## Surface boundaries (do not merge queues)

| Surface | Lane | Queue home | Idle means |
|:---|:---|:---|:---|
| Tk APS | Track A | grammar_continuation (drained) | maintain pytest |
| egui Assembly QC | Lane 4 | **shipped** — dev tooling only | cancel re-impl rows |
| Bevy sim HUD | Lane 5 | Phase 3 `SIM-HUD-PRODUCT-CLOSE` | polish/close program |
| Bevy preview worker | APS | separate worker witness | maintain |
| MCP productivity | Chain B | drained | maintain |

Reference: $ref:docs/archive/2026-06-src-dev/plans/bevy_hud_lanes_agent_orders_v1.md · $ref:src/dev/plan_territory_matrix_002_v1.md

---

## Phase 3 seed (2026-06-08)

**Closed without code (sync only):**

| ⟨ID⟩ | Why |
|:---|:---|
| EGUI-QC-IMPL-001 | v2 shipped — egui QC not sim HUD |
| TRIAGE-REPLAY-001 | `parity_green: true` — verify only |
| DSM-SIGNOFF-001 | tensor ATL★ RT★ |

**Real open work** (witness + queue `status=ready`):

| Priority | ⟨ID⟩ | Agent | Source |
|:---:|:---|:---|:---|
| P0 | G-PLAY-01 | Operator | product gate |
| P1 | TRIAGE-FIRE-LOD-TIERS-001 | @coder | triage T3 |
| P2 | TRIAGE-PHASE-F-CULL-001 | @coder | triage T4 |
| P2 | TRIAGE-FIRE-OVERLAY-DBG-001 | @designer | triage spec |
| P2 | APS-ARTIST-TOOL-E2E-REVIEW-001 | @designer-mcp | defer registry |
| P3 | PERF-SHELL-001 · OPS-F03 | Operator | OPS-F01 |
| defer | MCP-OPS-REPORT-001 | @coder-mcp | P2 |
| defer | PLAN-AUDIT-020 | @planner | after G-PLAY |

**Closed on disk (do not re-assign):** TRIAGE-FIRE-EXTRACT-FINAL-001 · SIM-HUD-PRODUCT-CLOSE-001 · TRIAGE-REPLAY-VERIFY-001

---

## Agent picks (Phase 3 BLANG:Q+)

| Agent | Primary | Secondary | Mode |
|:---|:---|:---|:---|
| @planner | idle — Phase 3 Cycle 1 dispatched | PLAN-AUDIT-020 after G-PLAY | maintain |
| @coder | TRIAGE-FIRE-LOD-TIERS-001 | TRIAGE-PHASE-F-CULL-001 | implement |
| @coder A/B/C | regression | — | idle |
| @coder-mcp | maintain pytest | MCP-OPS if promoted | maintain |
| @designer | TRIAGE-FIRE-OVERLAY-DBG-001 | — | spec |
| @designer-mcp | APS-ARTIST-TOOL-E2E-REVIEW | — | review |
| @sim-steward | regression watch fire/replay | — | maintain |
| Operator | G-PLAY-01 | PERF-SHELL · OPS-F03 | **blocking** |

---

## Truth hygiene (PLAN-TRUTH-HYGIENE-001)

**Poison:** OPS rollups and markdown exec § that lag witnesses — fake AUTH `WRK○`, stale ΔWF, truncated `_agent_meta`, illustrative stubs read as disk truth.

**Antidote:** run `ops_intelligence_scan.ps1` after queue sync · AUTH from tensor · contract $ref:docs/archive/2026-06-src-dev/plans/witness_exec_shape_v1.md · **never assign from markdown when witness JSON is green**.

---

## When queues drain again

1. Run **PLAN-QUEUE-SYNC-00N** (increment N)
2. Scan triage T3–T6 + `post_stage6_active_todos.md` + `stage_tracks_execution_index_v1.md`
3. Create `post_drain_phase{N+1}_queue.json` — do not append unbounded rows to old file
4. Archive prior queue `_meta.program_green: true`
5. Update tensor `active_program` + `active_queue`

**Cadence:** one **primary** slice per implementer per cycle · max **three** parallel lanes across team.

---

## Planner paste (seed Phase 3)

```text
@planner — PLAN-QUEUE-SEEDING-001

Phase 2 DRAINED. Read src/dev/plan_queue_seeding_v1.md.

This session:
1. Confirm PLAN-QUEUE-SYNC-004 (Phase 2 close list in doc)
2. Write PLAN-PHASE3-EXEC-001 — witness keys for FIRE-EXTRACT-FINAL + SIM-HUD-CLOSE
3. Issue dispatch pastes:
   @coder → TRIAGE-FIRE-EXTRACT-FINAL-001
   @designer → SIM-HUD-PRODUCT-CLOSE-001
   Operator → G-PLAY-01
4. Do NOT assign EGUI-QC-IMPL (cancelled — lane 4 shipped)

Exit: Phase 3 Cycle 1 dispatched + HANDOFF v2.4
```

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | After Phase 2 drain · seeding loop + Phase 3 queue |
