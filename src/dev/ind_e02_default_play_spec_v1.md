# PLAN-IND-E02-PLAY-001 — industrial E02 default play spec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-IND-E02-PLAY-001** |
| **Coder lane** | **IND-E02-DEFAULT-PLAY-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — default play writer **CLOSED** on lib + disk |
| **Board reconcile** | [`ind_board_reconcile_plan_v1.md`](ind_board_reconcile_plan_v1.md) § IND-E02-DEFAULT-WITNESS |
| **Witness** | `debug_runs/industrial_activation_live.json` |

**No Rust in this deliverable.**

---

## Executive summary

| Path | `ind_e02_green` | `placed_via_construction` | When |
|:---|:---:|:---:|:---|
| **Seed / operational spawn** | `false` typical | `false` | `spawn_concrete_portland_chain_operational` |
| **Lib refresh** (**IND-E02-DEFAULT**) | `true` | `true` | `refresh_ind_e02_default_live_witness()` test harness |
| **Default play** (**this spec**) | `true` | `true` | `seed_ind_e02_default_play_once` in **Simulation** |

**Product exit:** Default **Simulation** session writes **`ind_e02_green: true`** via construction commit funnel — **without** `RUST_ENGINE_IND_E02_SEED` or manual test-only env.

---

## Predicate — `in_play_green()`

```rust
production_green()
  && placed_via_construction
  && sites_committed >= 3
```

**Code:** `ConcreteChainE2eWitness::in_play_green()` in [`concrete_chain_e2e.rs`](../../src/economy/activation/concrete_chain_e2e.rs).

---

## Three writers (do not collapse)

| Lane ID | System | Trigger |
|:---|:---|:---|
| **IND-E02** (commit test) | `simulation_writes_industrial_activation_live_json_ind_e02_in_play` | Explicit test commits |
| **IND-E02-DEFAULT** | `refresh_ind_e02_default_live_witness` | Lib-only industrial proof app |
| **IND-E02-DEFAULT-PLAY-001** | `seed_ind_e02_default_play_once` | **OnEnter(Simulation)** + Update chain |

**Schedule (play):**

```text
OnEnter(Simulation) → reset IndE02DefaultPlaySeedState
Update (Simulation) → seed_ind_e02_default_play_once
                    → commit_construction_site_system
                    → fast_forward_portland_chain_sites_to_operational
                    → activate_industrial_facilities_system
                    → write_industrial_activation_live_proof
```

**Forbidden:** Direct `spawn_concrete_portland_chain_operational` for **IND-E02 play** exit; second construction execute funnel; marking board failed when only **seed JSON** lacks `ind_e02_green`.

---

## Witness contract — `industrial_activation_live.json`

| Path | Required (play) | 2026-05-26 disk |
|:---|:---:|:---:|
| `concrete_chain_e2e.production_green` | `true` | ☑ |
| `concrete_chain_e2e.ind_e02_green` | `true` | ☑ |
| `concrete_chain_e2e.placed_via_construction` | `true` | ☑ |
| `concrete_chain_e2e.sites_committed` | `>= 3` | ☑ **3** |
| `concrete_chain_e2e.chain_operational` | `true` | ☑ |

**Cross-check:** `stage7_play_live.json` may mirror `ind_e02_green` — maintain, do not merge files.

---

## PASS gates

| # | Criterion | Evidence |
|:---:|:---|:---|
| E02-P1 | Play seed runs in sim | `seed_ind_e02_default_play_once` registered in activation bridge |
| E02-P2 | Construction commits | `placed_via_construction: true` in JSON |
| E02-P3 | Site count | `sites_committed >= 3` |
| E02-P4 | Lib test | `simulation_ind_e02_default_play_writer_sets_ind_e02_green` |
| E02-P5 | No seed-only env | test does **not** require `RUST_ENGINE_IND_E02_SEED` |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib simulation_ind_e02_default_play_writer_sets_ind_e02_green industrial_activation
```

**Refresh disk:**

```powershell
cargo test -p proc_A_dine01 --lib simulation_writes_industrial_activation_live_json
```

---

## Board policy

| Doc | IND-E02 row |
|:---|:---|
| [`post_stage6_active_todos.md`](post_stage6_active_todos.md) | **[x]** commit + default play |
| [`stage_open_todos_v1.md`](stage_open_todos_v1.md) | **IND-E02-DEFAULT-PLAY-001** → **done** |
| [`witness_status_live_v1.md`](witness_status_live_v1.md) | `ind_e02_green: true` **CURRENT** when disk matches table |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **PLAN-IND-E02-PLAY-001** signed |
