# Fleet maturity — sign-off routing & next horizon `v1`

| Field | Value |
|:---|:---|
| **Date** | 2026-05-27 |
| **Audit** | [`planner_status_audit_v14.md`](planner_status_audit_v14.md) |
| **Nav** | [`plan_elemental_wave2_index_001_v1.md`](plan_elemental_wave2_index_001_v1.md) |
| **Coder dispatch** | [`fleet_wave6_coder_dispatch_v1.md`](fleet_wave6_coder_dispatch_v1.md) |
| **Rule** | `debug_runs/*.json` wins · one primary per role per session |

---

## Executive summary

The fleet has **completed the WSS hybrid spine through PR-4/PR-5 fixture** (weather/fire authority off slab; smoke ECS still authoritative), **Wave S BQ-128 apply + merge/replace**, **industrial default-play**, and **infra slice 3**. Planner/designer wave 4 is **done**. **Doc/queue drift** was the main gap (PR-5 and BQ-002 closed on disk but still listed `active`).

To move **much further**, shift from “close the last witness flag” to **three parallel horizons**:

1. **WSS production cutover** — smoke ECS retire + live sim proof (not fixture-only).
2. **L3 embellishment** — H-A2 Hanabi behind `hanabi_l3` (planner exec + coder + designer re-sign).
3. **Stage 7 + operator spine** — fix `s7b_m3` / steward rollup; full witness cadence + optional `--test visual`.

---

## Who signs what (authority matrix)

| Milestone type | Signs | Evidence required | Must NOT |
|:---|:---|:---|:---|
| **Architecture / exec plan** | **@planner** SIGNED | `plan_*_exec_*_v1.md` + audit row | Reopen archived `wave6_archive` CLOSED rows |
| **UX / player read** | **@designer** PASS (qualified) | signoff registry + design doc | Edit Rust |
| **Implementation** | **@coder** done | witness JSON keys green + targeted `cargo test` | Second writer on same resource |
| **Fleet truth reconcile** | **@planner** audit vN | witness spot-check table | Collapse planner sign-off with coder witness |
| **Sim / visual proof** | **@operator** | `--test visual`, PNG captures, live sim refresh | Mark FULL_APP green without JSON |
| **Cross-lane drift** | **@sim-steward** | witness vs queue diff; steward rollup | Implement product features |

**Closure rule:** Coder row → **CLOSED** only when witness keys on disk match exec plan acceptance. Planner → **SIGNED** when doc exists. Designer → **PASS** when review doc exists. **QUALIFIED CLOSED** when fixture green but production tail remains (PR-5 smoke, H-A2 default binary).

---

## Witness truth table (2026-05-27 disk)

| Domain | Witness file | Key gates | Verdict |
|:---|:---|:---|:---:|
| **WSS slab** | `wss_substrate_live.json` | `green`, hydrate, dual-write | **CLOSED** |
| **WSS PR-4** | same | `substrate_persist_roundtrip_ok`, `dynamic_overlay_migrated` | **CLOSED** |
| **WSS PR-5** | same | `ecs_retire_fixture_green`, `hybrid_ecs_weather/fire: false` | **QUALIFIED CLOSED** |
| **WSS PR-5 tail** | same | `hybrid_ecs_smoke_authoritative: true` | **OPEN** (smoke ECS retire) |
| **Atmos + hydro** | same | `wss_atmos_*`, `wss_hydro_*`, hydro coupling | **CLOSED** |
| **Construction param + R4** | `construction_stage_live.json` | parametric, r4 corridor/mv | **CLOSED** |
| **BQ-128** | same + `wave_s_hydrate_live.json` | apply ghost + merge_replace_002 | **CLOSED** |
| **Industrial** | `industrial_activation_live.json` | `ind_e02_default_play_002` | **CLOSED** |
| **Infra slice 3** | `stage6_virtualization_live.json` | `infra_slice3_001`, `wc_d04` | **CLOSED** |
| **Stage 5** | `stage5_full_app_live.json` | `readiness.passes`, F2 extract | **CLOSED** |
| **Stage 7 behavioral** | `stage7_behavioral_live.json` | `s7b_m4_play_green` | **CLOSED** |
| **Stage 7 behavioral** | same | `s7b_m3_green`, `s7b_steward_green` | **REGRESSION** |
| **Hanabi H-A2** | — | no `hanabi_l3` in main crate | **OPEN** |
| **Hanabi spike** | `experiments/.../report_v1.md` | PASS qualified | **CLOSED** |

---

## Coder return — reconciled

### @coder A — closed

| ID | Witness |
|:---|:---|
| WSS-SLAB-PR-2/3/4/5 (fixture) | `wss_substrate_live.json` |
| H-A-SPIKE-001 | `report_v1.md` |
| WSS atmos/hydro | substrate JSON |
| S7B-M4-PLAY | `s7b_m4_play_green` |
| Infra stress bundle | isolation + stage6 (2026-05-26) |

### @coder B — closed

| ID | Witness |
|:---|:---|
| BQ-128-APPLY-001/002 | construction + wave_s JSON |
| INFRA-SLICE3-001 | stage6 + perf md |
| IND-E02-DEFAULT-PLAY-002 | industrial JSON |
| Parametric 002..006, R4, M3, replay, tray, hydro boundary | per lane witnesses |

---

## Planner — next (unblocks horizon)

| P | ID | Why now |
|:---:|:---|:---|
| **1** | **PLAN-LEDGER-REFRESH-010** | ☑ audit v14 |
| **2** | **PLAN-WSS-PR5-SMOKE-PROD-001** | ☑ [`plan_wss_pr5_smoke_prod_001_v1.md`](plan_wss_pr5_smoke_prod_001_v1.md) |
| **3** | **PLAN-HANABI-H-A2-EXEC-001** | ☑ [`plan_hanabi_h_a2_exec_001_v1.md`](plan_hanabi_h_a2_exec_001_v1.md) |
| **4** | **PLAN-STAGE7-M3-STEWARD-001** | Rollup spec when `s7b_m3_green` false on disk |
| **5** | **PLAN-WSS-POST-SPINE-001** | Logistics pressure on slab, weather runbook Phase 2, F7-DEEP optional |
| **6** | **PLAN-CONSTRUCTION-R4-PRODUCT-OPEN-001** | deferred product board |

[`plan_elemental_wave2_index_001_v1.md`](plan_elemental_wave2_index_001_v1.md) v1.1 — PR-5 fixture + BQ-128 **CLOSED**; smoke prod **OPEN**.

---

## Designer — next

| P | ID | Unblocks |
|:---:|:---|:---|
| **1** | **DESIGN-PR4-RETIRE-UX-001** | PR-5 smoke production cutover copy |
| **2** | **DESIGN-HANABI-H-A2-PROD-001** | After coder wires `hanabi_l3` — production preset disposition |
| **3** | **DESIGN-S7B-M3-READ-001** | If M3 witness regresses — overlay readability |

Wave 4 designer batch (**Hanabi review, VFX wave6, dual-write, active-runtime, BQ128 UX**) — **do not reopen**.

---

## Coder — wave 6 (see dispatch doc)

| Coder | Primary | Qualified tail |
|:---|:---|:---|
| **A** | **WSS-PR5-SMOKE-PROD-001** | smoke authority off in live JSON |
| **A** | **H-A2-001** | after planner exec |
| **B** | **S7B-M3-STEWARD-REMEDY-001** | `s7b_m3_green` + `s7b_steward_green` |
| **B** | **LOG-E01-FULLAPP-UPGRADE-001** | optional + operator |

---

## Operator — parallel (raises fleet maturity fastest)

| ID | Action |
|:---|:---|
| **OPS-WITNESS-CADENCE** | Refresh full bundle per [`plan_ops_witness_cadence_001_v1.md`](plan_ops_witness_cadence_001_v1.md) |
| **OPS-VISUAL-001** | `cargo run -p proc_A_dine01 --release -- --test visual` → refresh `stage5_full_app_live.json` |
| **OPS-F03** | Live sim `stage6_virtualization_live.json` |
| **VX-P0-04** | PNG round from wave6 capture matrix |

---

## Regression bundle (fleet gate)

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate substrate construction industrial_activation stage7
cargo test -p proc_A_dine01 --lib wss_substrate_pr5 infra_slice3
cargo check -p hanabi_validation
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-27 | Post–coder-return maturity routing |
