# Coder dual-queue — multi-stage checklist `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Matrix** | [`coder_fleet_multistage_matrix_v1.md`](coder_fleet_multistage_matrix_v1.md) |
| **Machine** | [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |

Copy-paste checklist — mark ☑ in PR or session note.

---

## @coder A

### WSS substrate

| ☐ | ID | Done when |
|:---:|:---|:---|
| ☐ | **A-W1** WSS-CHUNK-SLAB-001 | `wss_substrate_live.json` **`green: true`** (types landed; CS-003 sim hydrate) |
| ☐ | **A-W2** WSS-ATMOS-CLIPMAP-001 | `wss_atmos_clipmap_001` block present |
| ☐ | **A-W3** WSS-SLAB-PR-2 | `dual_write_drift_max` under ε (future) |
| ☐ | **A-W4** WSS-SMOKE-BRIDGE-001 | [`plan_wss_smoke_bridge_exec_001_v1.md`](plan_wss_smoke_bridge_exec_001_v1.md) — `smoke_stub_removed` |

### VFX fallbacks

| ☐ | ID | Done when |
|:---:|:---|:---|
| ☐ | **A-V1** F7-DEBUG-WIRE-001 | F3 labels in streaming witness |
| ☐ | **A-V2** FIRE-F2-EXTRACT-001 | [`plan_fire_f2_extract_exec_001_v1.md`](plan_fire_f2_extract_exec_001_v1.md) — `fire_instance_buffer_rows > 0` |
| ☐ | **A-V3** WSS-SMOKE-BRIDGE-001 | [`plan_wss_smoke_bridge_exec_001_v1.md`](plan_wss_smoke_bridge_exec_001_v1.md) — `smoke_stub_removed` |
| ☐ | **A-V4** S7B-M4-LIVE-001 | live enqueue witness |

---

## @coder B

### Construction (parametric)

| ☑ | ID | Done when |
|:---:|:---|:---|
| ☑ | **B-C0** PARAM-001 weighted footprint | `weighted_raster_tests_green` |
| ☑ | **B-C1** PARAM-002 P2-A | `enter_commits_single_ghost` + `shift_queue_building_removed` |
| ☑ | **B-C2** PARAM-003 P1-B | `overlap_blocks_commit` |
| ☑ | **B-C3** PARAM-005 P2-B | partial-alpha overlay |
| ☑ | **B-C4** PARAM-004 P3-A | staging panel |
| ☑ | **B-C5** PARAM-006 P4-A | economy scale |
| ☑ | **B-C6** rollup | `construction_parametric_placement_001.green` |

### WSS hydro

| ☐ | ID | Done when |
|:---:|:---|:---|
| ☐ | **B-H1** WSS-HYDRO-RUNTIME-001 | `hydrate_wired` prereq + hydro block |
| ☐ | **B-H2** hydro ↔ construction coupling | event bus only |

### Product fallbacks

| ☐ | ID | Done when |
|:---:|:---|:---|
| ☐ | **B-P1** M3-UNITS-DEPTH-001 | minimap unit reader live |
| ☐ | **B-P2** REPLAY-RING-LIVE-001 | replay ring len ≥ 2 in sim |
| ☐ | **B-P3** UI-P3-M2-TRAY-OPT | tray bridge |

### Deferred

| ☐ | **R4-MV-GHOST-001** | only after **B-C3** |
