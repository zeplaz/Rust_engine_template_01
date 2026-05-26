# Logistics projection — implementation plan `v1` (PLAN-LOGISTICS-PROJECTION-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LOGISTICS-PROJECTION-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — spec exists; coder slices **OPEN** where noted |
| **Lane spec** | [`logistics_visual_lane_spec_v1.md`](logistics_visual_lane_spec_v1.md) (**LOG-E01**) |
| **Throughput (separate)** | [`logistics_throughput_live.json`](../../debug_runs/logistics_throughput_live.json) — economy only |

---

## North star

`log_rows > 0` in projection build signature when `LogisticsGraph` has edges and overlay policy enables logistics.

**Not sufficient:** `logistics_throughput_live.json` green alone.

---

## Witness targets

| File | Field |
|:---|:---|
| `debug_runs/stage5_full_app_live.json` | `projection_graph.logistics_active_rows`, `build_signature` contains `log_rows=N` |
| Console | `READINESS_PROJECTION_GRAPH_BUILD` |

---

## Slice map

| ID | Goal | Status |
|:---|:---|:---:|
| **VIS-01…04** | Fill, policy, projection node | **DONE** |
| **VIS-05** | HUD overlay tray toggle | **OPEN** |
| **VIS-06** | Per-view multiview | **DEFERRED** |
| **VIS-08** | Visual harness seeds graph | **DONE** (verify per run) |
| **VIS-09** | Lib test harness seed | **DONE** |
| **VIS-10** | Cap uses logistics rows | **DONE** |

---

## Copy-paste — LOG-E01 coder

```
Lane: LOG-E01 — logistics projection log_rows
Read: src/dev/logistics_projection_impl_plan_v1.md
      src/dev/logistics_visual_lane_spec_v1.md
Verify: cargo run -p proc_A_dine01 --release -- --test visual
        cargo test -p proc_A_dine01 --lib stage5 engine::test_harness
Witness: stage5_full_app_live.json logistics_active_rows > 0
```

---

## Forbidden

| Pattern | Reason |
|:---|:---|
| Second logistics extract | Stage 5 spine |
| Treat throughput JSON as visual proof | Separate domains |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | PLAN-LOGISTICS-PROJECTION-001 rollup |
