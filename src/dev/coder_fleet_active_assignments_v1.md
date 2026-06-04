# Coder fleet — active assignments `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.2.3` |
| **Date** | 2026-05-27 |
| **START HERE** | [`coder_unblock_dispatch_v1.md`](coder_unblock_dispatch_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) v4.2+ |

---

## Pick ONE primary per coder (today)

| Coder | P1 (now) | P2 (next) | Do not touch |
|:---|:---|:---|:---|
| **A** | **WSS-ATMOS-CLIPMAP-001** | **WSS-HYDRO-RUNTIME-001** / **S7B-M4-LIVE-001** | `src/construction/*` |
| **B** | **closed** — regression guard (`construction` + `coder_b_*` bundles) | `src/substrate/active_runtime.rs` (Coder A mutex) |

**Witness truth:** `wss_substrate_live.json` `green: true` and `construction_parametric_placement_001.green: true` (parametric 002..006 closed).

---

## @coder A — WSS-CHUNK-SLAB-001 (closed)

| Field | Value |
|:---|:---|
| **Plan** | [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) |
| **Witness** | `debug_runs/wss_substrate_live.json` → `green: true` |
| **Next** | Move to A-W2 (`plan_wss_atmos_clipmap_exec_001_v1.md`) |

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate
```

---

## @coder A — FIRE-F2-EXTRACT-001 (P2 or parallel)

| Field | Value |
|:---|:---|
| **Plan** | [`plan_fire_f2_extract_exec_001_v1.md`](plan_fire_f2_extract_exec_001_v1.md) |
| **Witness** | `stage5_full_app_live.json` — `fire_instance_buffer_rows > 0` |

**Blocked for primary until slab green:** WSS-ATMOS-CLIPMAP-001 · WSS-HYDRO-RUNTIME-001

---

## @coder B — B-H2 closed (lane drained)

| Field | Value |
|:---|:---|
| **Closed** | **CONSTRUCTION-PARAM-CODER-002..006** · **R4** · **M3/replay/tray** · **WSS-HYDRO-BOUNDARY-001** |
| **Witness** | `wss_hydro_runtime_001.construction_hydro_coupling_wired: true` · `construction_events_drained: 1` |
| **Next** | Await planner next exec slice |

```powershell
cargo test -p proc_A_dine01 --lib construction
cargo test -p proc_A_dine01 --lib coder_b_wave3 coder_b_queue_bundle
```

---

## Planner docs (not blank — use these)

| Doc | Coder slice |
|:---|:---|
| [`plan_fire_f2_extract_exec_001_v1.md`](plan_fire_f2_extract_exec_001_v1.md) | A-V2 |
| [`plan_wss_smoke_bridge_exec_001_v1.md`](plan_wss_smoke_bridge_exec_001_v1.md) | A-V3 |
| [`plan_m3_depth_exec_001_v1.md`](plan_m3_depth_exec_001_v1.md) | B primary |
| [`plan_replay_ring_exec_001_v1.md`](plan_replay_ring_exec_001_v1.md) | B secondary |

**Draft only:** [`plan_wss_slab_pr2_dual_write_v1.md`](plan_wss_slab_pr2_dual_write_v1.md) · [`weather_simulation_runbook_v2_plan_v1.md`](weather_simulation_runbook_v2_plan_v1.md)

---

## Deferred

None — R4-MV-GHOST-001 closed (`construction_r4_mv_ghost_001.green: true`).
