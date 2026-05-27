# PLAN-LOG-E01-VISUAL-001 — LOG-E01 visual confirmation `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LOG-E01-VISUAL-001** |
| **Coder lane** | **LOG-E01-VISUAL-CONFIRM-001** (Coder B #4) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **QUALIFIED CLOSED** — harness `log_e01_visual_confirm_001.green` + `qualified_close`; operator `--test visual` = optional upgrade |
| **Impl spec** | [`log_e01_full_app_witness_spec_v1.md`](log_e01_full_app_witness_spec_v1.md) |
| **Visual gate** | [`visual_run_acceptance_matrix_v1.md`](visual_run_acceptance_matrix_v1.md) |
| **Witness** | `debug_runs/stage5_full_app_live.json` |

**No Rust in this deliverable.**

---

## Rule (one line)

**LOG-E01 visual confirm** = refresh `projection_graph.logistics_active_rows > 0` via **`--test visual`** or harness test with transport seed — **not** lib-only STALE disk.

---

## Lib vs visual

| Path | Proves | Closes LOG-E01 visual? |
|:---|:---|:---:|
| `cargo test … stage5` + seed writer | Spine + optional `log_rows` in harness | **Qualified** — refresh JSON same session |
| `cargo run … --test visual` | FULL_APP + logistics rows in live sim | **Yes** — operator/product |
| Cold `logistics_active_rows: 0` on disk | **STALE** | **No** — re-run visual |
| `minimap_compositor_live.json` `logistics_rows: 2` | M2 authoritative for minimap | **Does not** auto-close stage5 field |

---

## PASS gate (operator)

| # | Criterion | Evidence |
|:---:|:---|:---|
| V1 | Visual run exit 0 | Terminal |
| V2 | `readiness.passes: true` | `stage5_full_app_live.json` |
| V3 | `projection_graph.logistics_active_rows > 0` | Same file |
| V4 | `build_signature` contains `log_rows=N`, **N > 0** | When harness exercises lane |

---

## PASS gate (lib — regression)

| # | Criterion | Evidence |
|:---:|:---|:---|
| L1 | `cargo test -p proc_A_dine01 --lib stage5 logistics_visual` | green |
| L2 | Witness writer in harness refreshes JSON | timestamp current |

---

## Related witnesses (do not merge)

| File | Role |
|:---|:---|
| `logistics_throughput_live.json` | **S7P-LOG-001** play throughput |
| `minimap_compositor_live.json` | UI M2 logistics rows |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib stage5 logistics_visual
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **PLAN-LOG-E01-VISUAL-001** signed |
