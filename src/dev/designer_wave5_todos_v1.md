# Designer wave 5 todos `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Trigger** | Wave 4 closure — coders need new visual contracts |
| **Coder backlog** | [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) (context only) |
| **Workboard** | [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/designer_active_queue.json`](../tools/orchestrator/queues/designer_active_queue.json) |

**Rule:** Design / review / sign-off records only. No Rust.

---
## Already SIGNED / ready-for-signoff (coders can proceed)

| ID | Verdict | Record |
|:---|:---|:---|
| **DESIGN-R4-CORRIDOR-001** | PASS | [`construction_r4_corridor_map_ux_v1.md`](construction_r4_corridor_map_ux_v1.md) |
| **DESIGN-R4-TRAY-001** | DEFER | [`construction_r4_tray_legend_v1.md`](construction_r4_tray_legend_v1.md) |
| **DESIGN-R4-MV-PASS-001** | DEFER | [`construction_r4_mv_pass_record_v1.md`](construction_r4_mv_pass_record_v1.md) |
| **DESIGN-M3-DEPTH-001** | DEFER | [`minimap_m3_unit_aggregation_visual_v1.md`](minimap_m3_unit_aggregation_visual_v1.md) |
| **DESIGN-REPLAY-LIVE-001** | DEFER | [`minimap_replay_live_ring_visual_v1.md`](minimap_replay_live_ring_visual_v1.md) |
| **DESIGN-M3-TRAY-001** | PASS (qualified) | [`minimap_m2_tray_overlay_bridge_v1.md`](minimap_m2_tray_overlay_bridge_v1.md) |
| **DESIGN-F7-STREAM-001** | PASS | [`fire_streaming_neighbor_wake_visual_v1.md`](fire_streaming_neighbor_wake_visual_v1.md) |
| **DESIGN-F7-DEBUG-PASS-001** | DEFER | [`fire7_f7_b_debug_pass_record_v1.md`](fire7_f7_b_debug_pass_record_v1.md) |
| **DESIGN-VT-SPREAD-001** | PASS (qualified) | [`fire_vt_spread_visual_acceptance_v1.md`](fire_vt_spread_visual_acceptance_v1.md) |

---
## Master board (wave 5)

| ☐ | # | ID | Unblocks | Verdict |
|:---:|:---:|:---|:---|:---|
| ☑ | 1 | **DESIGN-R4-CORRIDOR-001** | `R4-CORRIDOR-001` | PASS |
| ☑ | 2 | **DESIGN-R4-TRAY-001** | `R4-TRAY-001` | DEFER |
| ☑ | 3 | **DESIGN-R4-MV-PASS-001** | `R4-MV-GHOST-001` | DEFER |
| ☑ | 4 | **DESIGN-M3-DEPTH-001** | `UI-P3-M3-DEPTH-001` | DEFER |
| ☑ | 5 | **DESIGN-REPLAY-LIVE-001** | `REPLAY-LIVE-RING-001` | DEFER |
| ☑ | 6 | **DESIGN-M3-TRAY-001** | `UI-P3-M2-TRAY-OPT` | PASS (qualified) |
| ☑ | 7 | **DESIGN-F7-STREAM-001** | `F7-STREAM-DEEP-001` | PASS |
| ☑ | 8 | **DESIGN-F7-DEBUG-PASS-001** | `FIRE7-F7-B-DEBUG-UI-001` | DEFER |
| ☑ | 9 | **DESIGN-VT-SPREAD-001** | `STAGE5-VT-DEEP-001` | PASS (qualified) |

---
## Witness sync (2026-05-26 follow-up)

**Policy:** DEFER = design signed; **witness pending implementation** — does **not** block `@coder` lanes. Re-run designer on DEFER rows only when witness keys land; no new specs.

| ID | Flip | Witness key on disk |
|:---|:---|:---|
| **DESIGN-R4-CORRIDOR-001** | DEFER → **PASS** | `construction_r4_corridor_001.green` |
| **DESIGN-F7-STREAM-001** | PASS (qualified) → **PASS** | `fire_streaming_live.json` → `neighbor_wake_observed: true` |
| **DESIGN-R4-TRAY-001**, **DESIGN-R4-MV-PASS-001** | stay DEFER | await `construction_r4_mv_ghost_001` |
| **DESIGN-M3-DEPTH-001**, **DESIGN-REPLAY-LIVE-001** | stay DEFER | await M3 depth / live-ring sim witness (not lib seed) |
| **DESIGN-F7-DEBUG-PASS-001** | stay DEFER | F3 debug overlay wiring witness |

**Coder handoff (parallel, design sufficient):** `@coder B` **R4-CORRIDOR-001** done; next **R4-MV-GHOST-001**. `@coder A` **F7-STREAM-DEEP-001** done.

---
## Changelog
| Version | Date | Notes |
|:---|:---|:---|
| v1.0.1 | 2026-05-26 | Witness sync: R4-CORRIDOR + F7-STREAM PASS; remaining DEFER keyed to pending witness blocks |
| v1.0.0 | 2026-05-26 | Wave 5 designer records delivered (14 deliverables total across wave 5/6) |

