# G-PLAY-001-BLOCKERS — default play path audit `v1`

| Field | Value |
|:---|:---|
| **Gate** | G-PLAY-01 |
| **Owner** | @coder A |
| **Date** | 2026-05-28 |
| **Dispatch** | [`fleet_stability_phase2_dispatch_v1.md`](fleet_stability_phase2_dispatch_v1.md) |

## Exit

Operator completes 10 min **DefaultIndustrial** session (New Game → Simulation) **without** `--test`, `TestWorldHarness`, or witness `refresh_*` on the ship path.

## Blockers closed (code)

| # | Blocker | Fix |
|:---:|:---|:---|
| B1 | Logistics chain at `(0,0)` far from Portland `(40,40)` | `DEFAULT_INDUSTRIAL_LOGISTICS_CHAIN_TILES` colocated with Portland origin |
| B2 | `seed_ind_e02_default_play_once` no-op when `MessageWriter` was `Option::None` | Required `MessageWriter<CommitConstructionSiteEvent>` |
| B3 | Steward `production_green` without construction blocked default play seed | DefaultIndustrial falls through; stage7 spawn skipped when scenario is default |
| B4 | `DEFAULT_INDUSTRIAL_MIN_WORLD_TILES` unused | `ensure_default_play_world_extent_on_enter_simulation` clamps `WorldGenParams` |
| B5 | Play witness only via lib `refresh_*` | `write_play_scenario_live_proof_from_sim` (throttled, runtime) |
| B6 | Default play still tied to `RUST_ENGINE_*` seeds | **PLAY-TRUTH-001-TAIL**: `play_truth_001_tail` + `seed_ind_e02_default_play_once` (scenario-only); stage7 env path opt-in |

## Already green (no change)

| Area | Evidence |
|:---|:---|
| HUD PLAY-01 | `apply_simulation_hud_defaults` + `enforce_simulation_product_egui_gates` |
| IND-E02 play seed | `seed_ind_e02_default_play_once` in activation bridge |
| Logistics seed | `seed_default_industrial_logistics_once` after transport topology |
| Lib proof | `cargo test -p proc_A_dine01 --lib play_scenario` |

## Operator follow-up (not coder queue)

| ID | Task |
|:---|:---|
| **OPS-PLAY-001** | 60s release + manual runbook |
| **DESIGN-G-PLAY-001** | Signed acceptance checklist |

## Regression

```powershell
cargo test -p proc_A_dine01 --lib play_scenario economy::activation::witness_collectors
```
