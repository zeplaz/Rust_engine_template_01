# Landform and vegetation drain

**Date:** 2026-09-25 · **Branch:** `master` · **Queue:** [`landform_veg_drain_v1.json`](../tools/orchestrator/queues/landform_veg_drain_v1.json)

Pick with `get-que coder --queue landform_veg`. Coder rows on this queue are done through the climate, border-drainage, and preview-cover batch. What remains is acceptance and the blocked list.

The June 20 intel cull marked old vegetation, phase-6, and APS layout rows blocked because they were stamped done with no witness. That stamp is not a lock. This list is the work order. Do not flip those hundreds of rows to ready. Do not reopen MIG-*. Do not re-pick nuclear cooling.

## Already in the tree

| ID | What landed | Proof |
|:---|:---|:---|
| LAND-CURVE-001 | Profile curvature, stitched with slope. Hollow positive, ridge negative. | `bowl_is_positive_curvature_ridge_is_negative`, `stitch_two_chunks_vertical_edge` |
| VEG-CHUNK-BIAS-001 | Steep chunks hold less canopy. | `steep_slope_thins_canopy_versus_flat` |
| MAP-PREVIEW-RELIEF-001 | Preview biome, height, and ecology tiles shade by slope and curvature. | `relief_curvature_brightens_ridges_and_darkens_hollows` |
| VIS-BLOB-FIRE-RAIL-CURSOR-001 | Corner blob, spark motion, rail words, cursor writer. | Named lib tests in the queue row |
| PLANT-FOCUS-LIVE-001 | Utilities card reads live containment. | `live_runtime_fills_scram_heat_and_fuel` |
| MAP-SIM-RELIEF-001 | Play-map raster shades by slope and curvature. | `play_map_ridge_is_lighter_than_hollow` |
| VEG-CELL-CANOPY-001 | Ridge and hollow in one chunk get different cover. | `same_chunk_hollow_holds_more_cover_than_ridge` |
| VEG-DRAW-FOLLOW-001 | Vegetation draw density follows the center cell. | `veg_draw_follows_center_curvature` |
| LAND-LAYERS-002 | Drainage, support, concealment, and flood basins follow elevation. Erosion stays zero. | `valley_drains_and_hides_more_than_a_peak_and_erosion_stays_zero` |
| MAP-DRAIN-SHADE-001 | Play raster blues a pit more than a peak. | `play_map_pit_is_wetter_than_peak` |
| MAP-FLOOD-SHADE-002 | Flood-basin cells are darker than a peak. | `play_map_flood_basin_is_darker_than_peak` |
| MAP-SUPPORT-MARK-003 | A steep cell is dimmer than a flat cell. | `play_map_steep_support_is_dimmer_than_flat` |
| MAP-CONCEAL-SHADE-004 | A concealed pit is less green than a peak. | `play_map_concealed_pit_is_less_green_than_peak` |
| LAND-EROSION-003 | Erosion is slope times drainage. Curvature is not an input. An eroded play cell differs from a flat cell. | `valley_drains_and_hides_more_than_a_peak_and_erosion_stays_zero`, `play_map_eroded_peak_is_less_green_than_flat` |
| MAP-EDITOR-HYDRO-005 | Editor preview blues a pit more than a peak. | `editor_preview_pit_is_bluer_than_peak` |
| MAP-EDITOR-FLOOD-006 | Editor preview darkens a basin against a peak. | `editor_preview_basin_is_darker_than_peak` |
| MAP-EDITOR-SUPPORT-007 | Editor preview dims steep ground. | `editor_preview_steep_is_dimmer_than_flat` |
| VEG-CLIMATE-CANOPY-010 | Moisture raises canopy and heat lowers it at the same slope. | `wet_canopy_exceeds_dry_canopy_at_the_same_slope` |
| DRAIN-BORDER-004 | A higher neighbor chunk adds drainage on the shared edge. Erosion is refreshed from slope times that drainage. | `neighbor_chunk_adds_drainage_on_the_shared_edge` |
| MAP-MOIST-SLOPE-008 | Play-map moisture darkens a steep cell against a flat cell. | `play_map_moisture_steep_is_darker_than_flat` |
| VEG-FUEL-012 | Wet ground holds less surface fuel. | `wet_ground_holds_less_surface_fuel` |
| VEG-PREVIEW-COVER-011 | Ecology preview green follows cell cover. | `preview_green_follows_cell_cover` |
| MAP-EDITOR-MOIST-009 | Editor chunk moisture darkens when elevation is steep. | `editor_moisture_graph_steep_is_darker_than_flat` |
| MAP-EDITOR-TEMP-010 | Editor chunk temperature darkens when elevation is steep. | `editor_temperature_graph_steep_is_darker_than_flat` |
| VEG-PREVIEW-CLIMATE-013 | Ecology preview green follows wet-cool versus dry-hot cover. | `preview_green_follows_climate_cover` |
| LOG-HAUL-DISRUPT-014 | Half disruption slows the haul. Full disruption stays at the origin. | `drain_half_disruption_slows_the_haul_marker` |
| SHOT-RANGE-015 | A shot past 24 tiles never leaves the origin and never impacts. | `drain_shot_past_range_does_not_impact` |
| SHOT-LAND-016 | Tracer danger clears on landing. Impact owns the target. | `drain_danger_clears_when_the_shot_lands` |
| VEG-SNOW-CANOPY-014 | Snow thins canopy. Bare ground does not. | `drain_snow_thins_canopy_against_bare_ground` |
| TOWN-PREFIX-017 | Later generated towns do not move earlier centers. | `drain_later_town_does_not_move_an_earlier_center` |
| LOG-EDGE-018 | Haul markers stop on the last map cell. | `live_haul_marker_stops_at_the_map_edge` |
| SHOT-BOOK-019 | Book danger is the hottest tracer. Book impact is the strongest stamp. | `live_book_danger_is_the_hottest_tracer` |
| SHOT-TRACE-020 | An in-range tracer stays inside the origin-to-target span. | `live_trace_stays_on_the_segment` |
| VEG-UNDER-SNOW-021 | Snow thins understory less than canopy. | `live_understory_thins_in_snow` |
| TOWN-HOUSE-022 | Generated housing and jobs stay at or under population. | `live_generated_housing_does_not_exceed_population` |
| HUD-HAUL-023 | A fully disrupted haul reads idle. | `live_disrupted_haul_reads_idle` |
| HUD-SHOT-024 | Tray copy reads book max danger and impact. | `live_shot_hud_reads_book_impact` |
| LOG-STAMP-025 | Play haul stamps stay inside the map. | `live_play_haul_stamp_stays_inside_the_map` |
| LOG-SOLVER-ZERO-026 | A solver load of zero does not restore static capacity. | `push_zero_solver_load_does_not_restore_static_capacity` |
| LOG-SOLVER-CLOSE-027 | Full disruption closes overlay flow even with solver load. | `push_disrupted_solver_load_closes_the_edge` |
| SHOT-POINT-028 | A zero-length shot impacts on the fire tick. | `push_zero_length_shot_impacts_on_the_fire_tick` |
| SHOT-EARLY-029 | Danger stays clear before the shot is fired. | `push_danger_stays_clear_before_the_shot_is_fired` |
| SHOT-FACTION-030 | Another faction's tracer does not raise book danger. | `push_other_faction_does_not_raise_book_danger` |
| VEG-REGROW-031 | Snow slows the regrowth drive. | `push_snow_slows_regrowth_drive` |
| VEG-FRAG-032 | Burn raises stand fragmentation. | `push_burn_raises_fragmentation` |
| TOWN-COUNT-033 | A generated book stays between one and eight towns. | `push_generated_town_count_stays_in_bounds` |
| TOWN-SPACE-034 | Stacked centers separate by four tiles inside the seed band. | `push_stacked_town_centers_stay_four_apart` |
| ED-ERODE-035 | Editor green drops with slope times drainage. | `push_editor_erosion_dims_a_drained_slope` |
| HUD-STOCK-036 | A short stock reads short, not ready. | `push_short_stock_does_not_read_ready` |
| HUD-FACTION-037 | The tray stays clear on another faction's tracer. | `push_other_faction_reads_clear_on_the_tray` |
| LOG-HALF-038 | One anchored end keeps half the flow. | `next_one_anchor_keeps_its_half_of_the_flow` |
| LOG-COST-039 | Cost above 1 slows the haul phase. | `next_costly_haul_advances_slower` |
| LOG-NEG-040 | Negative capacity does not open flow. | `next_open_flow_rejects_negative_capacity` |
| SHOT-RING-041 | Range is the longer axis. A diagonal inside 24 still flies. | `next_diagonal_shot_stays_inside_the_range_ring` |
| SHOT-DUP-042 | The same shot does not stack. | `next_duplicate_shot_does_not_stack` |
| VEG-FOG-043 | Fog lowers concealment. | `next_fog_lowers_concealment` |
| VEG-HEAT-044 | Heat lowers surface fuel. | `next_heat_lowers_ground_fuel` |
| TOWN-GAP-045 | Four tiles is enough separation. | `next_four_tiles_is_enough_separation` |
| ED-CLEAR-046 | A transparent pixel ignores fire tint. | `next_clear_pixel_ignores_fire_tint` |
| ED-SMOKE-047 | Smoke darkens green and does not raise red. | `next_smoke_does_not_raise_the_red_channel` |
| HUD-ONE-048 | One town reads singular. | `next_one_town_does_not_pluralize` |
| LOG-SLOT-049 | A missing solver slot stays closed. | `cont_missing_solver_slot_stays_closed` |
| LOG-SPAN-050 | A zero-size map stamps no haul. | `cont_zero_span_stamps_no_haul` |
| LOG-NAN-051 | Non-finite flow stays closed. | `cont_non_finite_flow_stays_closed` |
| SHOT-OWN-052 | Another faction's impact stays off this side. | `cont_other_faction_impact_stays_off_this_side` |
| VEG-RAIN-053 | Rain washes held smoke. | `cont_rain_lowers_held_smoke` |
| VEG-SNOW-054 | Snow lowers the drawn stand. | `cont_snow_lowers_the_drawn_stand` |
| VEG-FUEL-055 | Rain lowers stand fuel load. | `cont_rain_lowers_the_stand_fuel_load` |
| TOWN-BAND-056 | Town centers stay inside the seed band. | `cont_town_center_stays_inside_the_seed_band` |
| HUD-IN-057 | Inbound stock is not ready or short. | `cont_inbound_stock_is_not_ready_or_short` |
| ED-HOLE-058 | A clear pixel ignores hydro tint. | `cont_clear_pixel_ignores_hydro_tint` |
| ED-FLAT-059 | A clear pixel keeps its support color. | `cont_clear_pixel_keeps_its_support_color` |

## Fan-out (not blocked)

Same session. These lanes do not wait on playtest, keyframe stills, or the June 20 cull. Queue: [`sim_feature_fanout_drain_v1.json`](../tools/orchestrator/queues/sim_feature_fanout_drain_v1.json). Coder rows are done.

| Lane | Done | Proof |
|:---|:---|:---|
| AI towns | Civic zoning queues a hall. The tray counts towns and growth sites. | `civic_zoning_queues_a_hall`, `town_board_line_counts_live_towns` |
| Logistics | The tray says east or north only while the haul is open. | `haul_heading_names_axis_only_when_flow_is_open` |
| Weapons | The tray says in flight, then impact, from the live shot. | `shot_hud_word_follows_flight_then_impact` |
| Vegetation harness | Harness density follows center-cell curvature when a cell matrix is present. | `harness_density_follows_center_curvature` |
| Generated towns | Each town has a district, centers stay four tiles apart, and the play map stamps those centers. A saved Portland book is left alone. | `generated_towns_each_get_a_district_and_stay_apart`, `play_map_town_center_differs_from_empty`, `saved_town_book_is_kept_when_seeding` |

## Drain now

Nothing left on the coder drain. Acceptance and blocked rows stay where they are.

## Do not wait

| Item | Why it is not a lock |
|:---|:---|
| Playtest `G-PLAY-OPERATOR-01` | Acceptance log. Coder sub-gates are closed. |
| Eye check `OPS-VIS-EYE` | Sparks, labels, ghost, blob. Comes after the code, does not gate the next slice. |
| Kit and warehouse keyframe stills | Art promotion. `MCP-P2-KIT002-G4` stays rejected until stills exist. |
| `FACTION-REACT-001` blocked on G-PLAY | The dependency is the playtest row. Code does not wait. |
| 56 vegetation-drain rows and the phase-6 coder board | Same June 20 cull note. Pick this queue instead of those ids. |

## Stay blocked

| Item | Why |
|:---|:---|
| PCI-04, 08, 12, 15, 25, 26, 29, 30, 35, 36, 37, 39 | No cron daemon, bake stays default off, no false sole-writer claim, shims stay. |
| Replay parity | `replay_editor_parity_live.json` is not green. |
