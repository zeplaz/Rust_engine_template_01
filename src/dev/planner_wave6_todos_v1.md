# Planner wave 6 todos `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WAVE-6-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Trigger** | Reopen coder fleet after PLAN-LEDGER-REFRESH-006 |
| **Rule** | Docs + queue hygiene only (no Rust) |

---

## P0 — Ledger + queue hygiene (required)

| ☐/☑ | # | Queue ID | Deliverable | Status / Unblocks |
|:---:|---:|:---|:---|:---|
| ☑ | 1 | **PLAN-LEDGER-REFRESH-006** | [`planner_status_audit_v8.md`](planner_status_audit_v8.md) | **SIGNED** — fleet closed / tails only |
| ☑ | 2 | **STALE FLAG** | Flag stale markdown boards | `active_coder_queue_v1.md`, `stage_open_todos_v1.md`, `continuation_queue.json` (do not use as active queue) |
| ☑ | 3 | **QUEUE: planner** | `tools/orchestrator/queues/planner_active_queue.json` → Wave 6 section | queue points at exec plans |
| ☑ | 4 | **QUEUE: coder** | `tools/orchestrator/queues/coder_active_queue.json` `v4.0` with coder A/B active lanes | coders get full impl plans |

---

## P1 — Construction Round 4 “lift” product gate (required for coder B primary)

| ☐/☑ | # | Exec ID | Deliverable | Owner | Witness |
|:---:|---:|:---|:---|:---|:---|
| ☑ | 5 | **PLAN-CONSTRUCTION-R4-EXEC-001** | Expand `R4-PLAN-001` into phased impl | @planner | `construction_stage_live.json` (`construction_r4_corridor_001`) |
| ☑ | 6 | **PLAN-CONSTRUCTION-R4-MV-EXEC-001** | Implement `R4-MV-GHOST-001` paired with `R4-PLAN-002` | @planner | `construction_stage_live.json` (`construction_r4_mv_ghost_001`) |

---

## P1 — Minimap M3 product depth (required for coder B secondary)

| ☐/☑ | # | Exec ID | Deliverable | Owner | Witness |
|:---:|---:|:---|:---|:---|:---|
| ☑ | 7 | **PLAN-M3-DEPTH-EXEC-001** | Expand `m3_minimap_product_depth_plan_v1.md` into full impl | @planner | `minimap_compositor_live.json` |

---

## P1 — Fire streaming depth (required for coder A primary)

| ☐/☑ | # | Exec ID | Deliverable | Owner | Witness |
|:---:|---:|:---|:---|:---|:---|
| ☑ | 8 | **PLAN-F7-STREAM-EXEC-001** | Expand `fire7_streaming_depth_plan_v1.md` into full impl | @planner | `fire_streaming_live.json` |

---

## P2 — Replay + behavioral tails (deferred in this wave doc)

| ☐/☑ | # | Exec ID | Deliverable | Owner |
|:---:|---:|:---|:---|:---|
| ☑ | 9 | **PLAN-REPLAY-RING-EXEC-001** | Expand `replay_live_ring_impl_plan_v1.md` | @planner |
| ☐ | 10 | **PLAN-S7B-M4-LIVE-EXEC-001** | Expand `s7b_m4_live_sim_playtest_plan_v1.md` | @coder A |

---

## P2 — UI / infra (pick 2 minimum; deferred)

| ☐/☑ | # | Exec ID | Deliverable | Owner |
|:---:|---:|:---|:---|:---|
| ☐ | 11 | **PLAN-UI-P3-M2-TRAY-EXEC-001** | overlay tray → `MinimapOverlayMask` bridge | @coder B |
| ☐ | 12 | **PLAN-FIRE-F2-EXTRACT-001** | per-tile hot-cell extract contract | @coder A or B |

---

## P2 — Operator witness refresh (deferred)

| ☐/☑ | # | Exec ID | Deliverable | Owner |
|:---:|---:|:---|:---|:---|
| ☐ | 13 | **PLAN-OPS-WITNESS-REFRESH-001** | operator runbook tying OPS-F01/F03 to witness cadence | operator / @coder (if wiring required) |

---

## G — Coder assignment matrix (required section)

| Priority | ID | Owner | Depends on designer | Witness |
|:---:|:---|:---|:---|:---|
| 1 | R4-CORRIDOR-001 | coder B | DESIGN-R4-CORRIDOR-001 | construction_stage_live.json |
| 2 | R4-MV-GHOST-001 | coder B | DESIGN-R4-MV-001 (exists) + corridor UX | construction_stage_live.json |
| 3 | M3-UNITS-DEPTH-001 | coder B | DESIGN-M3-DEPTH-001 | minimap_compositor_live.json |
| 4 | REPLAY-RING-LIVE-001 | coder B | DESIGN-REPLAY-LIVE-001 | replay_editor_parity_live.json |
| 5 | F7-STREAM-DEEP-001 | coder A | DESIGN-F7-STREAM-001 | fire_streaming_live.json |
| 6 | F7-DEBUG-WIRE-001 | coder A | DESIGN-F7-B-DEBUG-001 (exists) | infra JSON |
| 7 | S7B-M4-LIVE-001 | coder A | none | stage7_behavioral_live.json |
| 8 | UI-P3-M2-TRAY-OPT | coder B | DESIGN-M3-TRAY-001 | minimap_compositor_live.json |

---

## P1 — Parametric placement (planner signed)

| ☐/☑ | # | Exec ID | Deliverable | Owner | Witness |
|:---:|---:|:---|:---|:---|:---|
| ☑ | 14 | **PLAN-CONSTRUCTION-PARAM-001** | [`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) **SIGNED** | @planner | — |
| ☑ | 15 | **CONSTRUCTION-PARAM-DESIGN-001** | UX tray + staged panel | @designer | signoff record |
| ☑ | 16 | **CONSTRUCTION-PARAM-CODER-001…006** | Phased impl per [`plan_construction_param_exec_phases_v1.md`](plan_construction_param_exec_phases_v1.md) | @coder B | `construction_parametric_placement_001.green: true` |

Board: [`planner_wave6_parametric_todos_v1.md`](planner_wave6_parametric_todos_v1.md)

---

## Do not reopen list (wave 6 scope constraints)

Do NOT reopen:
- **F7-A / F7-B / F7-C** exit gates (regression only)
- **dual-queue** closure rows
- **steward preflights** (W3/S7B/FIRE7/WATER/S7P/VM-09/witness-sync/spark/UI-OH)
- **UI-P2B gate**
- **INFRA-PROJ2**
- **S7P production slice**

