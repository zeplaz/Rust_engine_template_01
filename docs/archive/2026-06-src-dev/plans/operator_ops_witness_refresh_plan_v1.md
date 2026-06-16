# PLAN-OPS-F01-F03-001 — operator OPS witness refresh `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-OPS-F01-F03-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Infra** | [`infra_slice3_wc_d04_ops_f01_plan_v1.md`](infra_slice3_wc_d04_ops_f01_plan_v1.md) |

**No Rust.**

---

## OPS-F01 — perf attribution (60s)

| Step | Command / artifact |
|:---|:---|
| 1 | Run sim 60s with `PERF=1` if supported |
| 2 | Write [`debug_runs/perf_attribution_60s.md`](../../debug_runs/perf_attribution_60s.md) |
| 3 | No Stage 5 gate failure on perf alone |

---

## OPS-F03 — stage6 sim-time refresh

| Step | Command / artifact |
|:---|:---|
| 1 | Enter Simulation, advance ticks |
| 2 | Refresh `debug_runs/stage6_virtualization_live.json` |
| 3 | `stage6_ops_witness_001.green` when sim writer lands (**STAGE6-OPS-WITNESS-001**) |

```powershell
cargo test -p proc_A_dine01 --lib stage6_live_proof
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Operator runbook |
