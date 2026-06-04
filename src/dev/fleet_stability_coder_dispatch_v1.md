# Fleet PHASE-STABLE — coder dispatch `v1.1`

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` |
| **Date** | 2026-06-02 |
| **Phase** | **PHASE-STABLE-2026-06** (P1 sweep) |
| **Plan** | [`plan_fleet_stability_integrity_001_v1.md`](plan_fleet_stability_integrity_001_v1.md) |
| **Exec (active)** | [`plan_fleet_stability_integrity_exec_002_v1.md`](plan_fleet_stability_integrity_exec_002_v1.md) |
| **Exec (historical)** | [`plan_fleet_stability_integrity_exec_001_v1.md`](plan_fleet_stability_integrity_exec_001_v1.md) |
| **Audit** | [`planner_status_audit_v17.md`](planner_status_audit_v17.md) |
| **Sweep** | [`production_jank_sweep_20260602_v1.md`](production_jank_sweep_20260602_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |

**Rule:** One workstream row per PR. **Never** mix engine boundary + render re-exports + logistics exports in one PR.

**Sign-off rule (exec-002):** Slice done when non-test code **cannot import/call** removed API without `cfg` — not when behavior is runtime-guarded only.

**Do not reopen:** Wave 7 PERF-VIS tails, DEV-CONTAIN-002–006, exec-001 done rows — unless witness regression.

---

## Where we are

| Layer | Reality |
|:---|:---|
| **Proof / CI** | Strong — `ProofGrade`, LOG-E01 key split, harness plugin gated (P0 #1) |
| **Production boundaries** | **OPEN** — harness + `refresh_*` + logistics shortcuts still in default public API |
| **Playability** | Partial — `PlayScenarioId::DefaultIndustrial` landed; G-PLAY-01 manual 10 min still open |

---

## Start today (P1 sweep only)

| Coder | Pick | Why |
|:---|:---|:---|
| **A** | **DEHACK-ENG-001** | Narrow `engine::test_harness` from default binary surface |
| **A** | **DEHACK-RENDER-001** | Parallel — move `refresh_*_live_witness` behind `dev_witness_tools` or `cfg(test)` |
| **B** | **DEHACK-LOG-001** | Parallel — cfg-gate `patch_s7p_*` / `apply_s7p_*` exports (behavior guard already landed) |

---

## Coder A — active queue

| P | ID | Focus | Exit |
|:---:|:---|:---|:---|
| 1 | **DEHACK-ENG-001** | `dev_harness` feature; stop `pub use test_harness::*` in default builds | `cargo check` without harness imports from `main` |
| 1 | **DEHACK-RENDER-001** | Dev-only witness refresh API | No `render::refresh_*` in default dependency graph |

---

## Coder B — active queue

| P | ID | Focus | Exit |
|:---:|:---|:---|:---|
| 1 | **DEHACK-LOG-001** | Logistics shortcut **compile** boundary | `dehack_log_001_*` + logistics 48/48; exports cfg-only |

---

## Optional P2 (planner)

| ID | Doc |
|:---|:---|
| **DEHACK-ENV-001** | [`runtime_env_policy_registry_v1.md`](runtime_env_policy_registry_v1.md) |

---

## Operator (unchanged — not in coder `active[]`)

| ID | Task |
|:---|:---|
| **OPS-PLAY-001** | 60s release + G-PLAY-01 manual checklist |
| **OPS-VT5-001** | VR-04 log if visual regression |

---

## Regression

```powershell
cargo check -p proc_A_dine01
cargo test -p proc_A_dine01 --lib stage5 wss_substrate construction stage7 logistics
.\tools\orchestrator\scripts\check_live_proof_containment.ps1 -HardFail
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-28 | Initial PHASE-STABLE dispatch (PLAY-TRUTH, DEHACK-VIEW/FIRE, …) |
| v1.1.0 | 2026-06-02 | P1 sweep only — exec-002 three PRs; P0 harness done |
