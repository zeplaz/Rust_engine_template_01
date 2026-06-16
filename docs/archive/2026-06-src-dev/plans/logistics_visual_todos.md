# Logistics visual lane (TRIAGE-LOGISTICS-VIS)

**Full spec:** [`logistics_visual_lane_spec_v1.md`](logistics_visual_lane_spec_v1.md)  
**Throughput board:** [`logistics_throughput_todos.rs`](logistics_throughput_todos.rs) — economy causality (separate from render `log_rows`).

**Done when:** `log_rows > 0` in `READINESS_PROJECTION_GRAPH_BUILD` and `stage5_full_app_live.json` → `logistics_active_rows > 0`.

---

## Code status

| ID | Item | Status |
|----|------|--------|
| VIS-01 | Graph + solver in `fill_logistics_snapshot` | Done |
| VIS-02 | Corridor book fallback | Done |
| VIS-03 | `overlay_matrix.logistics` gate | Done |
| VIS-04 | `apply_representation_result` sets bit from graph | Done |
| VIS-08 | `--test visual` seeds graph + solver | Done — `seed_test_logistics_visual_proof` |
| VIS-09 | Lib test harness seed | Done |
| VIS-10 | Projection cap uses `reserved_capacity` not fire cap | Done |
| VIS-05 | HUD overlay tray logistics stress default in visual test | Partial |
| VIS-06 | Per-view logistics extract | Deferred (VM-08) |
| VIS-07 | Triage backlog Done | **Operator** — run `--test visual` |

---

## Verify

```powershell
cargo test -p proc_A_dine01 engine::test_harness::tests --lib
cargo test -p proc_A_dine01 render::visual_domain_snapshots --lib
cargo run -p proc_A_dine01 --release -- --test visual
```

Expect log line: `log_rows=2` (or higher) before graceful exit.
