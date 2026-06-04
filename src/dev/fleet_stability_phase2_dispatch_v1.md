# Fleet PHASE-STABLE P2 — role dispatch `v1.0`

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` |
| **Date** | 2026-06-02 |
| **Sign-off** | **PLAN-STABLE-P2-SIGN** — 2026-06-02 |
| **Audit** | [`planner_status_audit_v18.md`](planner_status_audit_v18.md) |
| **Phase** | **PHASE-STABLE-2026-06** cycle 2 (post P1 boundary sweep) |
| **Parent** | [`plan_fleet_stability_integrity_001_v1.md`](plan_fleet_stability_integrity_001_v1.md) |
| **P1 exec** | [`plan_fleet_stability_integrity_exec_002_v1.md`](plan_fleet_stability_integrity_exec_002_v1.md) |
| **Sweep** | [`production_jank_sweep_20260602_v1.md`](production_jank_sweep_20260602_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |

**Rule:** Witness JSON wins. One PR per row. Do **not** reopen wave 7 PERF-VIS / DEV-CONTAIN-002–006 unless regression.

---

## Where we are (returns reconcile)

| Layer | Status |
|:---|:---|
| **P1 boundaries** | **Mostly landed** — harness plugin gated; ENG/RENDER/LOG compile gates in `proof_grade.rs`; `PlayScenarioPlugin` (PLAY-TRUTH-001); CONTAIN-D shim retire (wave_c/s/stage6/view_runtime) |
| **Proof honesty** | **G-PROOF-01** — `ProofGrade`, LOG-E01 fixture vs visual keys |
| **Playability** | **Partial** — default industrial scenario in code; **G-PLAY-01** (10 min manual, no harness) still **open** |
| **Operator** | **OPS-PLAY-001** / **OPS-VT5-001** still drive measured acceptance |
| **Residual jank** | Env knobs (P2), minimap shim, scaffold `allow(dead_code)`, WSS dual-write cutover, R4 product depth |

---

## P1 closed (do not re-pick unless regression)

| ID | Owner | Evidence |
|:---|:---|:---|
| DEHACK-ENG-001 | A | State/menu vs CLI plugin split; narrow `engine` re-exports; `dehack_eng_001_*` test |
| DEHACK-RENDER-001 | A | No `refresh_*` on `render/mod.rs`; proofs use `stage5_full_app_harness::` |
| DEHACK-LOG-001 | B | Shortcuts in `witness_fixture` `#[cfg(test)]` only |
| PLAY-TRUTH-001/002/003 | A/B | `PlayScenarioPlugin`, `ProofGrade`, LOG-E01 keys |
| CONTAIN-D-001 | B | 4 shims retired; HardFail CI |
| DEHACK-VIEW/FIRE, STAB-VT-001 | A | Authority commit + overlay env gate |

---

## @planner — instructions

| P | ID | Deliverable | Exit |
|:---:|:---|:---|:---|
| 1 | **PLAN-AUDIT-018** | `planner_status_audit_v18.md` | Columns: **Playability**, **Production surface**, **Proof grade** per witness |
| 2 | **PLAN-STABLE-P2-SIGN** | Sign this doc + update `coder_active_queue.json` `active[]` | Queue matches §Role boards below |
| 3 | **PLAN-G-PLAY-001** | `src/dev/play_scenario_acceptance_runbook_v1.md` | Operator + designer script for 10 min sim (no `--test`) |
| 4 | **PLAN-ENV-ENFORCE-001** | Optional exec slice after **DEHACK-ENV-002** | CI fails new `std::env::var` without registry row |

**Do not:** Reopen wave 7 perf/contain rows; add features without G-PLAY-01 path.

---

## @coder A — active queue (start **G-PLAY-001-BLOCKERS**)

| P | ID | Focus | Files | Exit |
|:---:|:---|:---|:---|:---|
| 1 | **G-PLAY-001-BLOCKERS** | Fix anything blocking 10 min default play | `play_scenario.rs`, construction, economy activation, HUD | Operator can complete runbook without harness / `--test` |
| 2 | **CONTAIN-MINIMAP-001** | Retire last containment shim | `minimap_compositor/live_proof.rs`, `runtime_witness/minimap.rs` | `exceptions_manifest` minimap path empty; HardFail green |
| 3 | **STAB-CI-001** | `-D warnings` on lib | `Cargo.toml`, hot modules from sweep §E | `cargo rustc -p proc_A_dine01 --lib -- -D warnings` in CI (scoped allowlist OK) |
| 4 | **DEHACK-ENV-002** | Code review env registry | Per [`runtime_env_policy_registry_v1.md`](runtime_env_policy_registry_v1.md) | Remove/deprecate one caller per PR where safe |

**Parallel OK with B:** G-PLAY blockers vs WSS/hydro — disjoint domains.

**Regression:**

```powershell
cargo test -p proc_A_dine01 --lib proof_grade play_scenario stage5
.\tools\orchestrator\scripts\check_live_proof_containment.ps1 -HardFail
```

---

## @coder B — active queue (start **PLAY-TRUTH-001-TAIL**)

| P | ID | Focus | Files | Exit |
|:---:|:---|:---|:---|:---|
| 1 | **PLAY-TRUTH-001-TAIL** | Default play without env seeds | `play_scenario.rs`, `concrete_chain_e2e.rs`, `economy/activation` | `play_scenario_live.json` green; no `RUST_ENGINE_STAGE7_PLAY_SEED` needed for G-PLAY-01 |
| 2 | **DEHACK-WSS-002** | Slab authoritative phase 2 | `substrate/shim.rs`, `ecs_retire.rs` | `dual_write_compare_only` default; drift witness under ε |
| 3 | **FEAT-WSS-HYDRO-READ-001** | Player read from slab (with designer) | `substrate/`, HUD overlay | Designer sign-off + `wss_substrate_live.json` keys |
| 4 | **CONSTRUCTION-R4-PRODUCT-001** | One scoped R4 vertical | [`plan_construction_r4_product_open_001_v1.md`](plan_construction_r4_product_open_001_v1.md) | One playable corridor-phase slice + construction witness |

**Do not:** Reintroduce `patch_s7p_*` exports; mix R4 with containment PR.

---

## @designer — active queue

| P | ID | Focus | Deliverable | Exit |
|:---:|:---|:---|:---|:---|
| ☑ 1 | **DESIGN-G-PLAY-001** | [`play_scenario_acceptance_runbook_v1.md`](play_scenario_acceptance_runbook_v1.md) **PASS (qualified)** | G-PLAY-01 script ready |
| ☑ 2 | **DESIGN-WSS-HYDRO-READ-001** | [`wss_hydro_read_feature_pass_001.md`](wss_hydro_read_feature_pass_001.md) **PASS** | Unblocks FEAT-WSS-HYDRO-READ-001 |
| ☑ 3 | **DESIGN-PERF-DEGRADE-VALIDATE-001** | [`visual_perf_spike_degrade_validate_001_v1.md`](visual_perf_spike_degrade_validate_001_v1.md) **PASS (qualified)** | OPS p95 fill pending |
| ☑ 4 | **DESIGN-R4-UX-001** | [`construction_r4_product_slice_ux_v1.md`](construction_r4_product_slice_ux_v1.md) **PASS** | Unblocks CONSTRUCTION-R4-PRODUCT-001 |

**On-call only:** New egui in sim (forbidden); witness JSON edits.

---

## Operator (not in coder `active[]`)

| ID | Task | Unblocks |
|:---|:---|:---|
| **OPS-PLAY-001** | 60s release visual + fill `perf_attribution_60s.md` | G-PLAY-001, DESIGN-PERF-DEGRADE |
| **OPS-VT5-001** | VR-04 log from `--test visual` | Close or WNF `visual_run_blockers.md` VR-04 |
| **VFX-CAPTURE-INSIM-001** | Reference PNGs | VFX charter |

---

## Gates (phase 2)

| Gate | Owner | Criteria |
|:---|:---|:---|
| **G-PLAY-01** | Operator + designer + coder A | 10 min `DefaultIndustrial` play; no harness bootstrap |
| **G-PROOF-01** | Maintained | No shortcut symbols on visual lane (grep tests) |
| **G-CONTAIN-01** | Coder A | Zero shims in manifest |
| **G-STAB-01** | Operator | Measured p95 in `perf_attribution_60s.md` |

---

## Parallel matrix (same week)

| Day | Coder A | Coder B | Designer |
|:---|:---|:---|:---|
| D1–D2 | G-PLAY-001-BLOCKERS | PLAY-TRUTH-001-TAIL | DESIGN-G-PLAY-001 |
| D3 | CONTAIN-MINIMAP-001 | DEHACK-WSS-002 (plan) | DESIGN-WSS-HYDRO-READ |
| D4–D5 | STAB-CI-001 | FEAT-WSS-HYDRO-READ + R4 (if signed) | DESIGN-R4-UX-001 |

---

## Sign-off (PLAN-STABLE-P2-SIGN)

| Role | Verdict | Date | Notes |
|:---|:---|:---|:---|
| `@planner` | **SIGNED** | 2026-06-02 | Queue aligned with §Role boards + dual track (CON-P2 + INFRA-E0) |
| `@coder` | **ACK** | — | Pull from `coder_a` / `coder_b` `active[]` in queue v5.3 |
| Operator | pending | — | G-PLAY-01 closes on runbook execution |

**Queue rule (v5.3):** P2 stability tails remain in `coder_a`/`coder_b` `active[]` alongside construction Phase 2 and INFRA-E0. Do not repopulate top-level `active[]` with wave 7 perf/contain rows.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-06-02 | PLAN-STABLE-P2-SIGN; audit v18; dual-track queue alignment |
| v1.0.0 | 2026-06-02 | P2 dispatch after fleet returns; P1 mostly closed |
