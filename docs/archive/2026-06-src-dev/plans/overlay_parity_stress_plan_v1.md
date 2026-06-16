# PLAN-PHASE-D-PARITY-001 — overlay parity stress (VM-08) `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-PHASE-D-PARITY-001** |
| **Coder lane** | **TRIAGE-PHASE-D-PARITY-001** (Coder B #8) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **CLOSED** — baseline VM-08 + stress S1–S3 lib-green (`phase_d_parity_stress.rs`) |
| **Witness** | `debug_runs/infrastructure_view_isolation_live.json` |
| **Triage** | [`stage5_triage_backlog.md`](stage5_triage_backlog.md) TRIAGE-PHASE-D-PARITY |

**No Rust in this deliverable.**

---

## Baseline (closed)

| # | Criterion | Path |
|:---:|:---|:---|
| D0 | Per-view overlay masks aligned | `/vm_08/overlay_masks_aligned: true` |
| D0b | Rollup | `/infrastructure_view_isolation_green: true` |
| D0c | Fire per-view caps | `/vm_11/*` + F7-A exit |

Lib refresh: `infrastructure_view_isolation` tests + `coder_b_ui_w3_p6_proof.rs`.

---

## Stress matrix (P2 — triage)

| Case | Trigger | Pass when |
|:---|:---|:---|
| **S1** | Toggle WorldMain ↔ SimulationMap | Each view keeps distinct overlay mask |
| **S2** | Multiview split (2-up) | No cross-view fire/logistics bleed |
| **S3** | WorldGen → Simulation transition | Masks rebind without stale tactical tint |
| **S4** | Minimap + tactical same tick | Minimap heat-only; tactical has fire rows |
| **S5** | VM-09 invert bridge | `triage_vm09_v2_green: true` |

**Exit for TRIAGE-PHASE-D-PARITY-001 close:** S1–S3 lib or sim tests green + witness extension fields — **done** (`triage_phase_d_parity_001.stress` in `infrastructure_view_isolation_live.json`).

---

## Forbidden

| Anti-pattern | Why |
|:---|:---|
| Second overlay extract per view | Stage 5 convergence |
| Hand `overlay_masks_aligned: true` | Not product |
| Reopen VM-09 v2 without regression | Done 2026-05-26 |

---

## Verification (baseline)

```powershell
cargo test -p proc_A_dine01 --lib infrastructure_view_isolation
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **PLAN-PHASE-D-PARITY-001** signed — stress OPEN |
