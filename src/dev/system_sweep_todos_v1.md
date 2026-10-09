# System sweep — fire ecology + construction

Code slices only. No playtest gate, no witness JSON, no new env flag.

Queue: `tools/orchestrator/queues/system_sweep_drain_v1.json` (`system_sweep`).

| Id | What changed | Test |
|----|----------------|------|
| FIRE-SPARK-GATE-001 | Ambient spark is zero below the wildland mass gate | `sweep_spark_below_wildland_mass_is_zero` |
| FIRE-CELL-FUEL-002 | Spark stays zero when the cell is below the spread-fuel floor | `sweep_spark_below_cell_fuel_is_zero` |
| FIRE-SPREAD-003 | A burning cell deposits heat on one neighbor and draws that neighbor's fuel | `sweep_spread_deposits_heat_and_draws_neighbor_fuel` |
| FIRE-BURN-HOLD-004 | A depleted cell keeps residual heat for six ticks, then goes cold | `sweep_burned_cell_holds_residual_for_fixed_tick_window` |
| CON-PREVIEW-005 | A preview move does not write occupation or the player pending queue | `sweep_preview_move_does_not_change_occupation_or_pending` |
| CON-OCCUPY-006 | An unweighted footprint on an occupied tile blocks commit and leaves pending rows | `sweep_occupied_rect_blocks_commit_and_keeps_pending` |
| FIRE-RAIN-FUEL-007 | Rain does not refill fuel consumed by heat | `sweep_rain_does_not_restore_consumed_fuel` |
| FIRE-HOLD-NOSPREAD-008 | A cell inside the burned window neither accepts nor emits neighbor spread | `sweep_spread_does_not_land_on_a_held_burn_cell` |
| CON-PHASE-FOUNDATION-009 | Foundation fill is not the place-feedback green or Planned settle | `sweep_foundation_rgba_is_not_a_place_feedback_color` |
| CON-PHASE-CLEARING-010 | Clearing and Foundation use different fills on the phase map | `sweep_clearing_rgba_differs_from_foundation` |
| CON-PHASE-OFFLINE-011 | Offline fill is distinct from Damaged and from the place pulse | `sweep_offline_rgba_differs_from_damaged` |
| FIRE-SNOW-SPREAD-012 | Snow at or above 0.35 blocks a neighbor deposit | `drain_snow_blocks_neighbor_spread` |
| FIRE-WET-SPREAD-013 | A wet cell refuses neighbor heat and fuel draw | `drain_wet_neighbor_refuses_spread` |
| FIRE-SNOW-HOLD-014 | Deep snow clears the burned window and residual heat | `drain_snow_clears_the_burn_hold` |
| CON-PHASE-SURVEY-015 | Surveying fill differs from Planned | `drain_surveying_rgba_differs_from_planned` |
| CON-PHASE-PROVISION-016 | Provisioning fill differs from Under Construction | `drain_provisioning_rgba_differs_from_building` |
| CON-PHASE-RATE-017 | Foundation advances at half the survey step. Damaged stays put | `drain_foundation_step_is_slower_than_survey` |
| CON-EMPTY-FOOT-018 | A zero-size footprint does not commit or touch pending rows | `drain_empty_footprint_does_not_commit` |
| FIRE-LIGHTNING-019 | Lightning spark is zero without risk, wildland mass, or cell fuel | `live_lightning_spark_respects_the_fuel_gate` |
| FIRE-FOG-GAIN-020 | Fog at 1.0 halves neighbor heat gain and still draws fuel | `live_fog_halves_neighbor_heat_gain` |
| FIRE-SNOW-ARM-021 | Deep snow does not arm a burned hold | `live_snow_does_not_arm_a_burn_hold` |
| CON-CLEAR-RATE-022 | Clearing advances at half the Planned step | `live_clearing_advances_slower_than_planned` |
| CON-PLACEABLE-023 | A zero-size or rejected footprint cannot enter the commit funnel | `live_unplaceable_footprint_cannot_queue_commit` |
| FIRE-RAIN-024 | Rain at or above 0.85 stops cardinal spread | `push_heavy_rain_blocks_neighbor_spread` |
| FIRE-EMBER-025 | An ember does not heat a cell below the fuel floor | `push_ember_skips_an_empty_cell` |
| FIRE-ONCE-026 | A neighbor accepts one cardinal deposit per tick | `push_second_neighbor_does_not_stack_heat` |
| CON-TURN-027 | An odd quarter-turn swaps the committed footprint | `push_odd_turn_swaps_the_footprint` |
| CON-WORD-028 | Damaged stays off Built, Abandoned stays off Offline | `push_phase_words_keep_damaged_off_built` |
| FIRE-WIND-029 | Wind burns fuel faster than still air | `next_wind_burns_fuel_faster_than_still_air` |
| FIRE-DT-030 | A zero timestep does not consume fuel | `next_zero_timestep_does_not_consume_fuel` |
| CON-CLEAR-031 | Clearing with no substep starts on trees | `next_clearing_without_substep_starts_on_trees` |
| CON-PROV-032 | Provisioning does not advance | `next_provisioning_stage_does_not_move` |
| CON-MIRROR-033 | Mirror flips the local column | `next_mirror_flips_the_local_column` |
| FIRE-SAT-034 | A full cell does not give up more fuel | `cont_saturated_neighbor_keeps_its_fuel` |
| FIRE-GAIN-035 | A zero heat gain does not draw fuel | `cont_zero_heat_gain_does_not_draw_fuel` |
| CON-TILES-036 | A zero-size footprint lists no tiles | `cont_empty_footprint_lists_no_tiles` |
| CON-RECT-037 | Occupancy follows the quarter-turn | `cont_occupancy_rect_follows_the_quarter_turn` |
| CON-PROG-038 | Phase progress stops at one | `cont_phase_progress_stops_at_one` |

Filter: `cargo test -p proc_A_dine01 --lib <test name>`.

Wired into `chunk_fire_overlay_tick` (sole overlay writer), the ghost validation refresh, `try_commit_active_building_ghost` (still one `CommitConstructionSiteEvent`), and `phase_color`.
