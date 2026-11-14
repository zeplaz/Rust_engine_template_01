# M0 lib-test DEFER log (Bevy 0.20)

**Date:** 2026-11-13  
**Result after WeatherField `If` fix:** `1558 passed; 25 failed; 2 ignored`.

## Allowed by M0 SIGN-OFF

Plan exit: lib green **or** named failures listed here.

## Root cause (most)

Bevy **0.20** panics when `Res<T>` is missing (was softer before). Minimal test Apps omit resources that production always inserts. Pattern: wrap with `If<Res<T>>` (skip system) or insert the resource in the test harness.

Fixed already: `advect_atmosphere_field` → `If<Res<WeatherField>>`.

Known missing in remaining failures: e.g. `ManufacturingBlueprintRegistry` in `activate_industrial_facilities_system` (economy tests).

## Remaining failures

| Test |
|:---|
| `ai::faction::scenario::ai_faction_builds_supplies_produces_and_deploys` |
| `ai::faction::scenario::decisions_are_throttled_to_the_interval` |
| `ai::faction::scenario::same_seed_same_orders` |
| `ai::faction::scenario::seed_only_affects_tie_breaks_not_the_plan` |
| `combat::formation::tests::deployment_order_moves_a_formation_to_its_target` |
| `combat::formation::tests::fortify_order_digs_in_over_time_and_queues_construction_rows` |
| `combat::formation::tests::supplied_three_to_one_attack_wins_ground` |
| `construction::integration_tests::hydro_coupling_execute_plan_emits_preview_validate_does_not` |
| `construction::integration_tests::infra_e2_003_execute_road_increments_transport_binds_markers_and_paths` |
| `economy::activation::concrete_chain_e2e::tests::concrete_portland_chain_e2e_operational_production_tick` |
| `economy::activation::concrete_chain_e2e::tests::concrete_portland_chain_in_play_commit_and_production` |
| `economy::game_loop_e2e::building_logistics_construction_production_logistics_loop_closes` |
| `economy::logistics::tests::log_b_02_in_transit_ledger_uses_route_handle_and_progress` |
| `economy::logistics::tests::log_b_freight_moves_through_ledger_not_same_tick_teleport` |
| `economy::logistics::tests::log_b_partial_fulfillment_records_shortage` |
| `economy::logistics::tests::log_c_01_soa_throughput_solver_aligned_vecs` |
| `economy::logistics::tests::log_c_congestion_rises_when_edge_saturated` |
| `economy::logistics::tests::log_c_corridor_pressure_diffuses_to_successor` |
| `economy::logistics::tests::log_c_geographic_cascade_starves_downstream_smelter` |
| `economy::resource_flow::tests::aluminum_chain_links_edges` |
| `economy::resource_flow::tests::geopolymer_chain_links_edges` |
| `economy::resource_flow::tests::starved_smelter_drops_efficiency_when_upstream_empty` |
| `strategic::sim::tests::strategic_bundle_runs_coupling_and_settlement_growth` |
| `systems::chunk_environment_set::tests::fire_fuel_field_spawns_and_derives_before_fire_pass` |
| `systems::fire::chunk_fire_overlay::link_tests::link_west_and_north_receive_the_deposit_a_diagonal_does_not` |

## Follow-up

Continue `If<Res<_>>` / harness inserts for the resources above until this list is empty (before or as part of PROGRAM CLOSE polish — does **not** block M0 SIGN-OFF).
