# CODEBASE INDEX v1 — Rust files only (src/, ~991 files, ~196k LOC)
# Generated 2026-07-03 by full-sweep. Companion files: plan_cleanup_v1.md (R#/S#/P#/T#/D#) ·
# plan_schedule_sync_v1.md (SCH-E#/A#/T#/P#/D# schedule ordering).
#
# ── CODE SYSTEM ──────────────────────────────────────────────────────────────
# Entry code  = DOMAIN-SUB[-nn]     e.g. RN-PRT = render particles, SIM-FIR = fire sim
# Concept     = K##                 cross-cutting idea; link with [K##]
# Issue       = R#/S#/P#/T#/D#      redundancy/smell/perf/tidy/debug — defined in plan_cleanup_v1.md
# Line format = CODE | path | ~LOC | purpose | links
# links: →CODE = depends-on/feeds, [K##] = embodies concept, (R#|S#|…) = has open issue
#
# ── DOMAIN PREFIXES ──────────────────────────────────────────────────────────
# CO core/engine/events/traits/utils   IO io (save/stream/snapshot/serial)
# CP compute       SIM systems/*       SUB substrate      TER terrain
# RN render        GU  gui             EN  entities       EC economy
# IF infrastructure CB construction    STR strategic      SCN scenario
# AI ai            DV  dev             BN  bin+bevysubengines
#
# ═════════════════════════════════════════════════════════════════════════════
# CONCEPTS (K-index)
# ═════════════════════════════════════════════════════════════════════════════
K01 | authority          | single-writer per resource; Sim→View→Render→UI never inverted | RN-VRT, GU-VWA
K02 | witness            | live-proof JSON → debug_runs/*.json; master gate witness_writes_enabled() (DV-WIT) | [K12]
K03 | wave-S             | save pipeline (DTO→compress→async IO→atomic swap) | IO-SAV
K04 | wave-C             | streaming spine (interest→hydrate→apply, main-thread apply invariant S6-22) | IO-STR
K05 | stage5             | render readiness proofs / full-app probes | RN-S5, DV-TODO
K06 | stage6             | atlas residency / virtualization windows | RN-S6
K07 | stage7             | operational telemetry + behavioral HUD | STR-S7, GU-HUD
K08 | lod-band           | WorldLodBand / FireLodBand zoom-band policy | GU-REP, RN-FCH
K09 | dual-write         | ECS↔slab mirrors + drift compare (compare_dual_write_drift_system) | SUB-SHM (R2,R3)
K10 | extraction         | Sim→Render snapshot contract, per-view frames | RN-EXT
K11 | clipmap            | substrate atmosphere hierarchical L0–L3 | SUB-ATM (R3)
K12 | env-latch          | runtime env-var toggles (RUST_ENGINE_DEEP_DEBUG, SIM_ANALYTICS, RUNTIME_WITNESS_WRITES, PERF_NO_VSYNC, STAGE5_PER_FRAME_HOOKS…) | DV-* (D2)
K13 | feature-gate       | cargo features: test_instrumentation, engine_deep_debug, dev_tools, hanabi_l3, legacy_engine, legacy_transport_ecs_stubs, bevy_tilemap_adapter, research_lmodels, tracy
K14 | todo-board         | dev live TODO boards (Stage5LiveTodoBoard etc.) | DV-TODO
K15 | slab               | non-ECS persistence-friendly substrate state | SUB-*
K16 | grammar            | building grammar T1 (CB-GRM) + landscape grammar LG1–LG5 (SIM-ECO)
K17 | view-runtime       | multiview isolation (VM-*), per-view surfaces/authority | RN-VRT
K18 | chunk              | streaming/sim/render unit; residency + interest scoring | IO-STR, RN-FCH
K19 | transport-graph    | IF-TRG = authoring authority, SIM-TRN = runtime sim; bridge sync | [K01]
K20 | minimap-paths      | GPU compositor (RN-MMC) vs tile fallback minimap (RN-TWF) | (R6)
K21 | ghost              | build preview ghost state/visual | CB-GHO
K22 | phase-notation     | M1-M4 / R9 / P2A / Wave P / F7 / LG# delivery-wave labels | (T2)
K23 | test-harness       | CLI --test scenario orchestration (weather|fire|visual…) | CO-THN (D1)
K24 | interest           | chunk priority scoring, orbs, ghost bands | IO-STR

# ═════════════════════════════════════════════════════════════════════════════
# CO — core / engine / events / traits / utils  (~5.3k LOC)
# ═════════════════════════════════════════════════════════════════════════════
CO-LIB | src/lib.rs + src/main.rs + src/idgen.rs | 160 | crate root, CLI entry, atomic EntityId | →CO-ENG, →DV
CO-ENG | src/engine/engine_with_worldgen.rs | 295 | CANONICAL EnginePlugin, 250+ add_plugins | →ALL (D1,D3)
CO-ENL | src/engine/engine.rs | 63 | legacy stub, feature legacy_engine [K13] | (R1)
CO-THN | src/engine/test_harness.rs | 1912 | --test scenario orchestrator [K23] | →SIM-*, →TER (D1,S9)
CO-PLY | src/engine/play_scenario.rs | 901 | default industrial play (Portland chain) | →STR, →SIM-TRN
CO-UXO | src/engine/ux_orchestration.rs + ux_states.rs + states.rs | 833 | UX state bridge legacy↔new AppState | →GU (T3)
CO-LNC | src/engine/launch_args.rs + debug_maneuver.rs | 483 | CLI test-mode routing, debug maneuvers | →CO-THN
CO-CHR | src/engine/worldgen_chrome_debug.rs | 305 | worldgen UI chrome trace [K12] | →GU
CO-EVT | src/events/ + src/traits/ + src/utils/ | 270 | ownership events, trait stubs (several empty) | (S8,T4)
CO-STB | src/engine/{sets,transitions,utils}.rs + lmodels/ | 50 | empty placeholders + research_lmodels stub [K13] | (T4)

# ═════════════════════════════════════════════════════════════════════════════
# IO — save / streaming / snapshot / serialization  (~5.4k LOC)
# ═════════════════════════════════════════════════════════════════════════════
IO-SAV | src/io/save/ (pipeline, dto, wire_format, manifest, load, apply, async_io, dirty_queue, autosave, snapshot_builder, registry_snapshot) | 1840 | Wave S save spine [K03] | →TER-MAT
IO-SVA | src/io/save/wave_s_artifacts.rs | 468 | shell capture/restore, blueprint presets [K02] | →DV-WIT (S3)
IO-SVO | src/io/save/{settlement,transport}_overlay.rs | 377 | settlement + transport net save/load | →STR, →SIM-TRN
IO-STR | src/io/streaming/ (mod 791, chunk_cache, hydrate, task_pool, budget, interest, residency, tile_storage_*, manifest_cache, preview_ghost, diagnostics, wave_c_*) | 2490 | Wave C streaming spine [K04][K18][K24] | →IO-SAV, →GU-REP
IO-SNP | src/io/snapshot/mod.rs | 112 | hybrid dev snapshot v0 | →DV
IO-SER | src/io/serialization/ (deserializers, legacy_drez, resource_deserializer) | 327 | config loaders; legacy_drez quarantined dead | (R4,S7)

# ═════════════════════════════════════════════════════════════════════════════
# CP — compute  (~750 LOC)
# ═════════════════════════════════════════════════════════════════════════════
CP-GRF | src/compute/compute_dispatch_graph.rs | 294 | compute node orchestrator (no ECS queries inside) | →GU-REP, →RN-EXT
CP-HTD | src/compute/heat_diffusion.rs + frame_snapshots.rs | 440 | GPU heat diffusion kernel + input snapshots | →SIM-FIR

# ═════════════════════════════════════════════════════════════════════════════
# SIM — systems/*  (~10.2k LOC)
# ═════════════════════════════════════════════════════════════════════════════
SIM-FIR | src/systems/fire/ (overlay, surface, fuel_profile, fire_fuel, smoke_field, combustion, surface_water, ember_spot_ignition, light_emission, play_visibility, witness_collectors) | 2120 | CANONICAL CPU fire sim [K09] | →SIM-ATM, →TER-FIR, →DV-WIT (S2,P1,P4,D4)
SIM-ATM | src/systems/atmosphere/ (field, incremental_schedule 640, page_residency, gpu_field_bridge, advect, update, coupling, emitter_sync, particles, pipeline, overlays, diagnostics, visual_extract, render_layers…) | 2530 | CANONICAL atmosphere field + P2-H GPU partial uploads | →RN, →SUB-ATM (R3,S2,P3)
SIM-WEA | src/systems/weather/ (chunk_weather, climate, regional_sample, effects, weather_visual 480, player_read_hud, witness) | 1420 | per-chunk weather, rain/fog/lightning | →SIM-ATM, →SIM-FIR
SIM-ECO | src/systems/ecology/ (chunk_ecology, vegetation_field, landscape_grammar 661, _burn 453, _lg2 785, _map 565, atlas registry, variant_catalog) | 3500 | ecology + landscape grammar LG1–LG5 [K16] | →SIM-FIR, →TER-GEN (S1b,P2,D5)
SIM-AGT | src/systems/agents/ (manager, permissions 411, multiplayer) | 960 | agent lifecycle, permissions, replication | →EN, →SIM-NAV
SIM-NAV | src/systems/navigation/ (nav, motion, road_vehicles_motion, logistics_floodfill 396, potential_field, schedule) | 1040 | pathfinding, motion, cost floodfill | →SIM-TRN
SIM-TRN | src/systems/transport/ (types, bake, snapshot 430, persistence) | 1180 | RUNTIME transport sim [K19] | ←IF-TRG (P5)
SIM-PRD | src/systems/production/ (runtime, power_systems, manifest, tools_ui, serialization) | 870 | production tick, power grid | →EN-POW, →IF
SIM-DMG | src/systems/damage/ + collision/ | 440 | damage from heat/impact/toxic, spatial queries | →SIM-FIR
SIM-TERB| src/systems/terrain/material_plugin.rs | 640 | material registry bridge terrain↔systems | →TER-MAT

# ═════════════════════════════════════════════════════════════════════════════
# SUB — substrate (slab state)  (~1.1k LOC)
# ═════════════════════════════════════════════════════════════════════════════
SUB-ATM | src/substrate/atmosphere/ (clipmap_advect, contamination_tick, bridge_legacy) | 290 | clipmap L0–L3 + LEGACY ECS→slab bridge [K11][K09] | ←SIM-ATM (R3!)
SUB-HYD | src/substrate/hydrology/ (background_tick, boundary, drain, event_bus, player_read) | 690 | slab hydrology saturation/pressure | →IF (R5)
SUB-SHM | src/substrate/shim.rs (via mod) | ~100 | ECS→slab dual-write mirrors + drift compare [K09] | ←SIM-FIR (R2)

# ═════════════════════════════════════════════════════════════════════════════
# TER — terrain  (~10k LOC)
# ═════════════════════════════════════════════════════════════════════════════
TER-GEN | src/terrain/generation/world_generator_enhanced.rs | 1517 | GOD FILE: noise+Voronoi+hydrology+biome+strategic | →TER-HYD (S1a,S6,P6)
TER-GPP | src/terrain/generation/passes/ (p1_fields…p6_materialize) | 700 | worldgen pass pipeline P1–P6 | →TER-MAT
TER-HYD | src/terrain/generation/hydrology/flow.rs | 508 | D8 flow, floodfill, lakes (gen-time only) | (R5)
TER-GSC | src/terrain/generation/ (chunk_worldgen_scheduler 351, dense_cache, diagnostics, noise, derived, tile_chunk_map, tuning_io, semantics…) | 1900 | chunk gen scheduling + caches + diagnostics | →CO-ENG
TER-MAT | src/terrain/material/ (registry 394, tags, rules, dependency, resolver, profile, runtime, preview_invalidation) | 1600 | material registry→family→visual resolution | →RN
TER-FIR | src/terrain/fire/ (fuel 249, fuel_layer, vegetation_fuel, structure_fire, scenario_hazard) | 630 | fuel material taxonomy (data, not sim) | ←SIM-FIR
TER-MOB | src/terrain/mobility/mod.rs | 330 | locomotion profiles, traversability | ←SIM-NAV
TER-EDT | src/terrain/editor/ (map_snapshot 283) | 400 | authoring snapshot save/load | →GU-EDT

# ═════════════════════════════════════════════════════════════════════════════
# RN — render  (~30k LOC)
# ═════════════════════════════════════════════════════════════════════════════
RN-EXT | src/render/extraction/ (fire_visual_extract 1205, render_projection_graph 795, scan, emission_profile, frame_snapshot, smoke/vegetation/procedural extracts) | 3030 | Sim→Render extraction [K10] | ←SIM-FIR, →RN-PRT (P4)
RN-PRT | src/render/gpu_particles.rs 1143 + gpu_particle_draw 555 + gpu_fire_particle_raster 497 + gpu_spark_compute 527 + fx_burst_request | 2830 | fire particle emit→dispatch→raster | →RN-GPU (R7)
RN-WPT | src/render/gpu_water_particles.rs 828 + draw 412 + raster 396 | 1640 | water particle emit→dispatch→raster (mirror of RN-PRT) | →RN-GPU (R7)
RN-WSV | src/render/water_surface_visual.rs 1009 + gpu_water_surface_draw 279 | 1290 | water surface catalog + overlay draw | →RN-GPU
RN-FCH | src/render/ (fire_chunk_runtime 324, fire_chunk_entity_index, fire_view_extract 645, fire_streaming, view_fire_projection, fire7_f7_a_exit) | 1920 | fire chunk LOD/residency/per-view frames [K08][K18] | ←RN-EXT
RN-LGT | src/render/lighting/ + light.rs | 485 | fire light clusters, light pooling | ←RN-EXT
RN-TWF | src/render/tile_world_fallback.rs | 1644 | CANONICAL CPU tile raster + minimap image + heat overlay (misnamed "fallback") [K20] | →GU (R6,S4,P7,T1)
RN-TMA | src/render/tilemap_adapter.rs | 485 | bevy_ecs_tilemap adapter, feature-gated OFF [K13] | (R6)
RN-TRA | src/render/ (terrain_render_authority 387, terrain_material_atlas 408, terrain_instanced_draw 523) | 1320 | terrain render authority + atlas + instancing [K01] | ←TER-MAT
RN-GPU | src/render/ (gpu_buffer_registry 365, gpu_bind_group_registry 303, gpu_packed_formats, gpu_indirect_draw 336, gpu_surface_teardown) | 1460 | GPU buffer/bindgroup/indirect infra | ←RN-PRT,RN-WPT
RN-VRT | src/render/view_runtime/ (ids, surface, layers, passes, input_routing, authority 219, bridge, commit, per_view_policy, witness_state, trace, view_fire_isolation, plugin, isolation_tests) | 1990 | multiview isolation runtime [K17][K01] | →GU-VWA
RN-VPP | src/render/viewport_pipeline.rs | 452 | viewport contract resolution | →GU-VWA
RN-OVL | src/render/ (overlay_field_buffers, domain_overlay_gpu, infrastructure_overlay 834, power_map_overlay_draw, tactical_vector_overlay, domain_projection_frame) | 2380 | shared overlays: fire heat, logistics, ecology, power/roads | ←STR, ←SIM-ECO
RN-VIS | src/render/ (visual_agreement 504, visual_domain_snapshots 508, visual_snapshot_commit, logistics/ecology_visual_snapshot, visual_perf_budget) | 1710 | visual snapshot publish + agreement hashes | ←RN-EXT
RN-MMC | src/render/minimap_compositor/ (composite 775, pass 543, render_target, gpu_compute, diagnostics, witness_collectors, plugin) | 2560 | GPU minimap compositor M1-M4 [K20] | →GU-MAP (R6)
RN-S5 | src/render/ (stage5_full_app_harness 2508!, stage5_readiness 981, stage5_closure_witnesses, phase_f_lod_proof) | 3930 | stage5 readiness proofs [K05] | →DV (S5,D3)
RN-S6 | src/render/ (stage6_virtualization 396, per_view_residency, vt_ci_matrix 598, vt_app_integration, vt_spatial_invariants) | 1480 | stage6 virtualization + VT CI matrix [K06] | →GU-HUD
RN-PERF| src/render/ (frame_perf 851, render_schedule_perf, stall_watch 465, perf_attribution_witness) | 1690 | frame/schedule perf instrumentation | →DV (D3)
RN-DIAG| src/render/ (full_render_diagnostic 933, visual_diagnostics 548, debug_render_trace, debug_viewport_overlay, visual_readiness_witness, spine_governance_matrix) | 2440 | render diagnostics, always-registered | (D3)
RN-WEA | src/render/ (gpu_weather_fire_field 671, atmosphere_partial_gpu 364, fire_smoke_shader_handles, hanabi_witness, hanabi_embellishment, vfx_capture_hook) | 1730 | weather/fire GPU field + hanabi opt-in [K13] | ←SIM-ATM
RN-SHD | src/render/shaders/ | 156 | WGSL shader registry | ←RN-*

# ═════════════════════════════════════════════════════════════════════════════
# GU — gui  (~35k LOC)
# ═════════════════════════════════════════════════════════════════════════════
GU-SHL | src/gui/hud/simulation_shell_phase2.rs | 3045 | GOD FILE: ops strip + context tray + build rail + minimap chrome (P2A) | →RN-MMC (S4a)
GU-HUD | src/gui/hud/ (~80 files: dock_shell 682, shell_framework 528, hud_root_tick, panels, trays, sheets, icon atlases, budgets, caches…) | ~14000 | HUD shell, docks, panels, tool sheets | →CB, →STR (T5)
GU-HDD | src/gui/hud/ (hud_dev_overlay 584, frame_budget_diagnostics 492, stage5/6/7 consumers, sim_view_sync_debug, viewport_*_debug, layout_debug) | 2500 | HUD dev/diag panels [K05][K06][K07] | →DV (D3)
GU-IGH | src/gui/in_game_hud.rs | 1830 | in-game HUD root orchestration | →GU-HUD
GU-CAM | src/gui/map_camera.rs 1600 + map_zoom_coherence + map_view_projection + sim_map_projection + map_presentation_* | 2650 | world camera control + projection + fit | →GU-VWA
GU-MAP | src/gui/map_view/ (plugin, view_state, resolved, presentation/, texture_cache/, projection/, backend/, consumers/, widgets/, debug/) | 3300 | map view instances (minimap/preview/tactical) | ←RN-MMC, ←RN-TWF (P8)
GU-REP | src/gui/ (world_representation 1343, representation_policy 735, view_representation 675, snapshots, governance, spine_audit, lod_zone_authoring) | 3600 | LOD policy engine [K08] | →RN-EXT, →CP (S4b,P9)
GU-VWA | src/gui/ (viewport_authority, viewport_layout_solver, view_authority 556, view_projection_authority, authoritative_viewport 471) | 1940 | viewport/view authority [K01] | →RN-VRT
GU-MMS | src/gui/ (minimap_shell 490, minimap_viewport_frame, minimap_egui_dev) | 780 | minimap shell UI container [K20] | ←RN-MMC
GU-EDM | src/gui/editor/map_editor/mod.rs | 1640 | GOD FILE: terrain brush M3 + road markers M4 + save M5 + bake R9 | →TER, →SIM-TRN (S4c)
GU-EDP | src/gui/editor/world_preview/ (30+ files: gpu_preview 509, render_raster 486, contracts, lifecycle, caches, ui_*) | 5600 | world raster preview (CPU+GPU paths, Wave P) | →RN (S10)
GU-EDW | src/gui/editor/ (world_gen_ui 968, world_gen_hints, scenario_script_panel, commit_bridge) | 1560 | world-gen UI + scenario editor | →TER-GEN
GU-APP | src/gui/ (app_shell 520, main_menu, splash, pause menus, ui_gates, ui_windows, egui_window) | 2100 | app shell, menus, gates | →CO-UXO
GU-INP | src/gui/ (input_bindings, input_frame, options_keybindings_ui 514, gameplay_capture) | 960 | input + keybindings | →GU-CAM
GU-STY | src/gui/style/ (theme, fonts, density, color_guard, scroll) | 900 | theme/design tokens | ←GU-*
GU-MSC | src/gui/ (map_tile_atlas_stamp 856, map_tile_raster, landscape_chunk_atlas_stamp, tile_readability, tile_debug_types, gpu_tile_debug, strategic_icon_instances, diagnostics_ui 809, pressure_tooling 582, agent_permissions_ui 538, ai_explainability_ui, faction_tools_ui, logistics panels, construction_growth_inspector, vfx_fire_test_highlight, assembly_snapshot_qc_ui, camera_focus_debug, gui_assets, gui_sets) | 6200 | misc panels, atlas stamps, tile debug | →RN, →EC

# ═════════════════════════════════════════════════════════════════════════════
# EN — entities  (~2.5k LOC)
# ═════════════════════════════════════════════════════════════════════════════
EN-CMP | src/entities/ (components, damages, entity, prelude, types_aliases, types_of) | 220 | base entity components | ←SIM
EN-PRD | src/entities/production/ (aluminum/, concrete/, core/ manufacturing+care+utils+resources) | 900 | production entity components + plugins | →SIM-PRD
EN-POW | src/entities/production/power/ (plant_definition 263, grid_topology, capabilities, components, states, systems, registry, profiles, failure_modes) | 850 | power plant registry + grid topology | →IF-UTL
EN-LEG | src/entities/production/prod_comps.rs + structure/legacy_transport_stubs.rs | 123 | LEGACY, not wired; stubs feature-gated [K13] | (R8)
EN-TYP | src/entities/types/ (e_flagz 203, p_enumz, s_flagz, v_flagz, requirements) | 340 | flag bitsets + enums | ←EN-*
EN-VEH | src/entities/vehicles/ (runtime 175, config, components, states, tools_ui) | 260 | vehicle runtime + configs | →SIM-NAV

# ═════════════════════════════════════════════════════════════════════════════
# EC — economy  (~7k LOC)
# ═════════════════════════════════════════════════════════════════════════════
EC-ACT | src/economy/activation/ (bridge 422, scale, grid_overload_ux, power_island_ux, concrete_chain_e2e 745) | 1520 | industrial activation + power UX | →SIM-PRD, →EN-POW
EC-ACW | src/economy/activation/witness_collectors.rs | 799 | activation witness board [K02][K14] | →DV (S3)
EC-LOG | src/economy/logistics/ (routes 403, solver, propagation, portals, async_district, types) | 1130 | route solver, multicommodity flow | →SIM-NAV (P10)
EC-LGW | src/economy/logistics/ (witness 281, witness_collectors 740, witness_fixture, tests 671 cfg-test) | 1780 | logistics witness boards + tests [K02] | →DV (S3)
EC-FLW | src/economy/ (resource_flow 612, supply_chain 329, spatial_district, concrete_batch, site_placement, logistics_bridge) | 1270 | resource flow net + supply chain | →IF, →STR

# ═════════════════════════════════════════════════════════════════════════════
# IF — infrastructure  (~2.3k LOC)
# ═════════════════════════════════════════════════════════════════════════════
IF-TRG | src/infrastructure/transport/ (graph, junction, spline, snapshot_bridge 254, sync, plugin) | 950 | AUTHORING transport graph authority [K19][K01] | →SIM-TRN
IF-UTL | src/infrastructure/utility/ (graph 272, connection, activation_link, authoring, mod) | 670 | utility networks (power/water) | →EN-POW
IF-PRF | src/infrastructure/profiles/mod.rs | 288 | corridor profiles (road/rail/power) | →CB
IF-SET | src/infrastructure/settlement/mod.rs | 85 | lightweight settlement node attach (≠ STR-SET, layered not redundant) | →STR-SET
IF-AUT | src/infrastructure/authoring/mod.rs | 96 | corridor authoring session | →GU

# ═════════════════════════════════════════════════════════════════════════════
# CB — construction  (~16k LOC)
# ═════════════════════════════════════════════════════════════════════════════
CB-GRM | src/construction/procedural/building_grammar.rs | 965 | GOD FILE: building grammar T1 [K16] | →CB-PRC (S1c,S11)
CB-PRC | src/construction/procedural/ (tile_variant_resolver 731, module_index 572, assembly_snapshot 561, tile_atlas_index 537, arch_v0 legacy 499, load, types, variant_recipe, footprint_grid, tile_visual_state, tests cfg-test) | 4600 | procedural build pipeline | →RN-EXT
CB-TOOL| src/construction/ (build_interaction 653, build_tool_authority, build_toolbox, build_mode, build_state, snap, sessions, menus×5, tool_hints) | 2400 | build tool input + toolbox + menus | →GU-HUD
CB-GHO | src/construction/ (build_ghost, ghost_visual, staged_ghost_panel 730, build_footprint_overlay, phase_visual, site_stub_overlay) | 1280 | ghost preview + staged panel [K21] | →RN (S4d)
CB-PIPE| src/construction/ (construction_pipeline 520, queue_intent 359, pending_construction + panel, site_stage_tick 300, site_stage*, history 408, demolish, upgrade, build_commit) | 2400 | build queue→site stage lifecycle | →STR-SIT
CB-CAT | src/construction/ (building_definitions 689, building_catalog, building_set, blueprint_preset 377, pilot_catalog 447) | 1830 | building catalogs + defs loaders | →IO-SER
CB-LANE| src/construction/ (roads/, rail/, power_lines/, zones/ — input/ghost/commit/placement/pathing per lane) | 2800 | per-lane construction tools | →IF-TRG
CB-MISC| src/construction/ (parametric_commit 434, procedural_build_spawn, visual_authority 570, weighted_footprint 360, round4_corridor 367, hydro_coupling, corridor_transport, terrain_conform, iso_draw_scale, map_egui_projection, site_zone_grid, grammar_labels…) | 3500 | parametric commit, visual authority, misc | →STR-SIT
CB-DBG | src/construction/ (placement_debug 837, scaling_audit 264, mock_shapes_menu, witness_collectors 783, construction_stage_witness 355, integration_tests 317 cfg-test) | 2560 | placement debug + witness boards [K02] | →DV (S4e)

# ═════════════════════════════════════════════════════════════════════════════
# STR — strategic  (~4k LOC)
# ═════════════════════════════════════════════════════════════════════════════
STR-SET | src/strategic/settlement/ (growth, district, market 223, assign, execute, town_rollup, pressure, policy, actors, town, block, ids, zoning) | 1330 | settlement growth sim (towns/districts/blocks) | →CB-PIPE
STR-SIT | src/strategic/site/ (components 212, systems 252, overlays, tile_occupation, validation, resources, parametric, provisioning, logistics, events) | 1430 | construction site lifecycle | ←CB-PIPE
STR-SIM | src/strategic/sim.rs | 788 | operational heatmaps (control/threat/recon/logistics) | →RN-OVL (S12)
STR-S7 | src/strategic/stage7_behavioral.rs | 605 | stage7 behavioral HUD + witness [K07] | →GU-HUD (S12)
STR-TBR | src/strategic/ (transport_bridge etc.) | ~300 | infra graph → strategic logistics rebuild | ←IF-TRG

# ═════════════════════════════════════════════════════════════════════════════
# SCN / AI — scenario + ai  (~1.1k LOC)
# ═════════════════════════════════════════════════════════════════════════════
SCN-ALL | src/scenario/ (scenario_types, objectives, validation, script_host 202, runner, steps, trigger_spec, plugin; tests/ cfg-test) | 890 | scenario file format + script host | →CO-PLY
AI-CON | src/ai/construction/mod.rs | 113 | construction AI validation probe | →CB-TOOL

# ═════════════════════════════════════════════════════════════════════════════
# DV — dev  (~10k LOC always-compiled + ~7k cfg(test))
# ═════════════════════════════════════════════════════════════════════════════
DV-WIT | src/dev/runtime_witness/ (gate.rs = master gate witness_writes_enabled) | 2836 | multi-domain witness orchestration [K02][K12] | ←ALL (D4)
DV-ENV | src/dev/debug_run_envelope.rs | 574 | debug_runs/*.json envelope + IO | ←DV-WIT
DV-DDBG| src/dev/engine_deep_debug/ (latch, witness, plugin) | 855 | intrusive GPU/schedule witnesses, feature+env gated [K12][K13] | →RN
DV-SPEC| src/dev/sim_spectrum_analytics.rs | 1093 | frame/ECS telemetry to disk, UNCONDITIONAL plugin add | (D2!)
DV-INST| src/dev/test_run_instrumentation.rs + perf_scope_frame_log.rs | 376 | --test auto-instrumentation latch [K12][K23] | →DV-SPEC
DV-TODO| src/dev/ (stage5_live_todos 857, stage5_finish, visual_aidv2 470, construction_* ×7, industrial_activation, logistics_throughput, replay_editor_parity) | 3620 | live TODO boards, always-compiled [K14] | (D5)
DV-HLTH| src/dev/orchestrator_health.rs + compile_hygiene_live.rs | 134 | thread health export, env-gated | 
DV-TEST| src/dev/ ~108 live_proof modules | ~7000 | cfg(test)-only witness refreshers — ZERO prod impact | [K02]

# ═════════════════════════════════════════════════════════════════════════════
# BN — bins + subengines + workspace  (~1.5k LOC)
# ═════════════════════════════════════════════════════════════════════════════
BN-PRV | src/bin/bevy_preview_worker.rs + src/preview/ | ~500 | headless assembly preview → PNG (APS-PREVIEW-004) | →CB-PRC
BN-WGN | src/bin/world_generator.rs | 29 | standalone world-gen binary | →TER-GEN
BN-SUB | src/bevysubengines/world_generator_plugin.rs | 734 | LEGACY parallel worldgen, NOT wired, "do not reintroduce" | (R9)
BN-HNB | experiments/hanabi_validation/ | ~400 | hanabi workspace experiment crate | [K13]

# ═════════════════════════════════════════════════════════════════════════════
# REPO-WIDE STATS (sweep snapshot 2026-07-03)
# ═════════════════════════════════════════════════════════════════════════════
# files=991 loc=195,966 | println!=14 dbg!=0 | allow(dead_code)=14 allow(unused)=3
# TODO/FIXME/HACK=226 | cfg(test)=614 cfg(debug_assertions)=7 | .unwrap()=303 .clone()=772
# stale target_* dirs at repo root: ~12 (gitignored, ~5-10 GB) → (T6)
