# PLAN-IND-E02-PLAY-EXEC-001 — Default play witness hardening `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-IND-E02-PLAY-EXEC-001** |
| **Slice ID** | **IND-E02-DEFAULT-PLAY-002** |
| **Parent spec** | [`ind_e02_default_play_spec_v1.md`](ind_e02_default_play_spec_v1.md) (**PLAN-IND-E02-PLAY-001** — SIGNED) |
| **Board reconcile** | [`ind_board_reconcile_plan_v1.md`](ind_board_reconcile_plan_v1.md) § IND-E02-DEFAULT-WITNESS |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |

**Planner sign-off:** PASS (2026-05-27). **-001** closed the lib proof-app writer; **-002** hardens the **default Simulation** play path and witness discrimination (not seed env, not commit-only test).

**Do not** use `spawn_concrete_portland_chain_operational` for -002 exit.

---

## Summary

| Lane | Scope |
|:---|:---|
| **IND-E02-DEFAULT-PLAY-001** | Lib industrial proof app + `simulation_ind_e02_default_play_writer_sets_ind_e02_green` — **CLOSED** |
| **IND-E02-DEFAULT-PLAY-002** | Production `IndustrialActivationPlugin` schedule + live JSON writer metadata |

**Disk baseline (2026-05-27):** `industrial_activation_live.json` already shows `ind_e02_green: true` with construction path fields — treat -002 as **witness hardening + regression guard**, not greenfield feature work.

---

## Authority map

| Resource | Single writer | Allowed | Must NOT |
|:---|:---|:---|:---|
| `ConcreteChainE2eWitness` | `refresh_concrete_chain_e2e_witness_system` | counters from sim entities | direct test mutation outside systems |
| `industrial_activation_live.json` | `write_industrial_activation_live_proof_system` | `concrete_chain_e2e` block + writer flags | manual JSON |
| `seed_ind_e02_default_play_once` | `concrete_chain_e2e.rs` | one-shot play seed in **Simulation** | second construction execute funnel |
| `stage7_play_live.json` | `write_stage7_play_live_proof_system` | mirror `ind_e02_green` | merge files with industrial proof |

**Schedule (already wired — verify, do not duplicate):**

```text
OnEnter(Simulation) → reset IndE02DefaultPlaySeedState
Update (Simulation) → seed_ind_e02_default_play_once
                    → commit_construction_site_system
                    → fast_forward_portland_chain_sites_to_operational
                    → activate_industrial_facilities_system
                    → write_industrial_activation_live_proof
```

**Code:** [`bridge.rs`](../economy/activation/bridge.rs) · [`concrete_chain_e2e.rs`](../economy/activation/concrete_chain_e2e.rs)

---

## PR plan (≤3 files each)

### IND-E02-EX-1 — Witness writer discrimination

| File | Change |
|:---|:---|
| `src/economy/activation/live_proof.rs` | emit `concrete_chain_e2e.default_play_writer: true` when play seed path completed |
| `src/economy/activation/concrete_chain_e2e.rs` | set flag on `IndE02DefaultPlaySeedState` completion (idempotent) |
| `src/dev/stage7_play_live.rs` or play proof writer | mirror `default_play_writer` optional cross-check |

**JSON contract:**

| Path | Required (-002) |
|:---|:---:|
| `concrete_chain_e2e.ind_e02_green` | `true` |
| `concrete_chain_e2e.placed_via_construction` | `true` |
| `concrete_chain_e2e.sites_committed` | `>= 3` |
| `concrete_chain_e2e.default_play_writer` | `true` |
| `concrete_chain_e2e.seed_only_path` | `false` or absent |

### IND-E02-EX-2 — Regression test (Simulation plugin stack)

| File | Change |
|:---|:---|
| `src/economy/activation/live_proof.rs` | extend or add `simulation_ind_e02_default_play_full_stack_sets_ind_e02_green` |
| `src/economy/activation/mod.rs` | export test module if needed |
| — | test uses `IndustrialActivationPlugin` + `BaseState::Simulation`, **not** `RUST_ENGINE_IND_E02_SEED` |

**Assert:** proof JSON written by `write_industrial_activation_live_proof_system`, not manual fixture injection.

### IND-E02-EX-3 — Board + fleet sync (docs-only in coder PR optional)

| File | Change |
|:---|:---|
| `src/dev/witness_status_live_v1.md` | -002 row CURRENT when writer flag present |
| `tools/orchestrator/queues/coder_active_queue.json` | mark **IND-E02-DEFAULT-PLAY-002** done when acceptance met |

---

## Predicate — `in_play_green()` (unchanged)

```rust
production_green()
  && placed_via_construction
  && sites_committed >= 3
```

**-002 green rollup:**

```text
ind_e02_green == true
AND placed_via_construction == true
AND sites_committed >= 3
AND default_play_writer == true
AND seed_only_path != true
```

---

## PASS gates

| # | Criterion | Evidence |
|:---:|:---|:---|
| E02-2-P1 | Play seed in main plugin | `IndustrialActivationPlugin` Update chain |
| E02-2-P2 | Witness writer flag | `default_play_writer: true` in live JSON |
| E02-2-P3 | No seed env | test omits `RUST_ENGINE_IND_E02_SEED` |
| E02-2-P4 | Lib regression | new/extended test in `live_proof.rs` |
| E02-2-P5 | Disk refresh | `cargo test … simulation_writes_industrial_activation_live_json` |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib simulation_ind_e02_default_play industrial_activation
cargo test -p proc_A_dine01 --lib simulation_writes_industrial_activation_live_json
```

---

## Anti-patterns

- Marking IND-E02 failed when only **seed JSON** lacks `ind_e02_green` (see board reconcile)
- Direct operational spawn for play exit
- Touching `src/substrate/active_runtime*` (coder A mutex)
- Reopening **PLAN-IND-E02-PLAY-001** spec (reference only)

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | `IND-E02-DEFAULT-PLAY-002` |
| **Witness** | `debug_runs/industrial_activation_live.json` |
| **Secondary** | `debug_runs/stage7_play_live.json` mirror |
| **Acceptance** | -002 green rollup above |
