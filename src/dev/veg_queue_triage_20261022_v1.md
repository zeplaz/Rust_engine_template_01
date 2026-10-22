# Vegetation queue triage `v1` (2026-10-22)

**Lane:** L4 trees/vegetation (`todo_and_review_20261022_v1.md`) · **Queues:** [`coder_vegetation_drain_queue.json`](../tools/orchestrator/queues/coder_vegetation_drain_queue.json) · [`landform_veg_drain_v1.json`](../tools/orchestrator/queues/landform_veg_drain_v1.json)
**Method:** read each row, then the current code and the local witnesses under `debug_runs/` (written 2026-10-22; the directory is gitignored since 0e199885, so witnesses are local proof only). No code was run or edited.

## What changed since the queue was written

| Change | Effect on vegetation rows |
|:---|:---|
| egui removed (51f8efb5) | `gui/diagnostics_ui.rs` lost the ecology program panel, the disturbance timeline and the veg extract panel. C06 and G01 regressed. `veg_diag_extract_panel_live.json` still claims the panel exists (false-green). |
| `debug_runs/**` gitignored (0e199885) | Exit predicates on witness JSON only prove things on the machine that ran the harness. Rows below cite the code and a test as well. |
| Dual view (a1f3f37d) | PixelWorld3D paints LG-5 atlas stamps per chunk (`render/tile_world_fallback.rs` `queue_pixel_world_landscape_atlas_stamps`). TacticalFlat2D has biome plus relief only. `VegetationExtractFrame` is read by the minimap compositor and nothing on the main sim map. |
| Bevy 0.19 migration closed | No MIG rows involved. |

The 55 `blocked` rows were stamped by the 2026-06-20 intel cull for "done without witness path or exit_predicate". That is a paperwork status, not a code blocker. Most of the code and witnesses exist.

## Verdict counts (55 blocked rows)

| Verdict | Count | Queue status now |
|:---|---:|:---|
| DONE-ALREADY | 33 | `done` |
| OBSOLETE | 3 | `superseded` |
| STILL-BLOCKED | 3 | `blocked` |
| UNBLOCKED | 16 | `ready` |

Also: `VEG-G02-STEWARD-REGRESS-001` (was `done`) reopened to `blocked`, because `replay_editor_parity_live.json` now reads `parity_green=false`, `replay_ring_len=0`, `proof_grade=lib_fixture_retired`. Owner sim-steward, not vegetation code. `VEG-G03-LG6-FLOWERS-001` stays deferred. Four new rows added: `VEG-H01..H04`. After the update the queue reads 58 done, 20 ready, 4 blocked, 3 superseded, 1 deferred.

`landform_veg_drain_v1.json` (6 non-done rows): all stay as they were. `DRAIN-STALE-CULLED-ROWS` deferred (scope narrowed), `OPS-VIS-EYE`, `G-PLAY-OPERATOR-01`, `KIT-G4-STILLS` deferred (operator or art), `PCI-DO-NOT-DRAIN` blocked (policy), `SIM-STEWARD-FIRE-REPLAY-001` blocked (parity witness false). Each carries a `triage_20261022` note.

## STILL-BLOCKED (3)

| Row | Blocker | Unblocks when |
|:---|:---|:---|
| VEG-C13-VISUAL-SMOKE-001 | Needs a windowed GPU capture; the smoke witness is lib-only (`visual_capture_required: true`). | Someone runs `cargo test -p proc_A_dine01 --test visual` on a display machine after VEG-H02, and records the PNG. |
| VEG-C14-OPERATOR-CHECKLIST-001 | Operator human sign-off. | Operator closes `G-PLAY-OPERATOR-01`. |
| VEG-F01-DESIGN-ATLAS-001 | Art-ship G4 manual keyframe review. Both LG-5 atlases are tier `pilot`, `ship_allowed: false`. | designer-mcp signs `proceed_ship: yes`. PixelWorld3D vegetation art stays pilot until then. Not a code gate. |

## OBSOLETE (3)

| Row | Why |
|:---|:---|
| VEG-C02-PREVIEW-SHADER-001 | The terrain material has no topology input by design. The play map is the RTT raster and PixelWorld3D stamps topology per chunk; the editor GPU preview already samples topology kinds (`gpu_preview.rs:361`). |
| VEG-F08-QUEUE-SYNC-001 | Process row. HANDOFF already points at the landform queue; this doc is the sync. |
| VEG-G04-DRAIN-SYNC-001 | v3 is not drained by design. Replaced by this doc and the VEG-H rows. |

## DONE-ALREADY (33), evidence in each row's `triage_20261022`

- A01, A02, A03, A05, A07, A10, A12, A14, A16: `landscape_grammar_lg2.rs` systems and tests plus `landscape_grammar_sim_harness.rs`. Grade is `headless_sim`, not a live app run.
- BIND/BMIL/BSET/BFOR/BAGR LOAD and EVAL (10 rows): presets on disk carry the exit topology, signed 10/10 in `mcp_landscape_grammar_sign_live.json`, catalog tests green. Only agri has a dedicated eval test.
- B-ROLLOUT-Q1..Q4: replaced by one continuous `rollout_landscape_program_on_chunks`; witness 17 chunks.
- B-PARTITION, B-NESTED-DEPTH, C01, C03, C05, C08, C09, E01, F03, F06.

Caveats that matter, folded into open rows: canopy has three writers (C07); `SubcellPopulationGrid` values are an index bias and unread (E02); agri preset has 9 flat nodes, over the 3 to 8 goal (F07); harvest and construction clear only log history (D03).

## Ordered implementation list (UNBLOCKED, 16 rows + 4 new)

Size: S under half a day, M about a day, L multi-day. **Both views** = needs TacticalFlat2D and PixelWorld3D checked. **Fire** = touches fire or burn visuals; coordinate with the fire/vfx owner (`src/render/fire_vfx` is being edited).

| # | Id | Files | Done when | Size | Both views | Fire |
|--:|:---|:---|:---|:--:|:--:|:--:|
| 1 | **VEG-C07-POPULATION-AUTH-001** | `systems/ecology/landscape_grammar_lg2.rs`, `vegetation_field.rs`, `ecology/mod.rs` | One writer of `VegetationField.canopy_density`. `derive_vegetation_population_from_graph` and `sync_fuel_bridge_from_succession` feed targets into `vegetation_field_tick` or run explicitly `.after` it. Test: snow, climate and slope canopy effects survive on a chunk that has a program. | M | Indirect (density feeds both) | Yes: fuel bridge also writes canopy |
| 2 | **VEG-H01-DISTRICT-CLASS-MAP-001** (new) | `systems/ecology/landscape_grammar_map.rs` | `district_kind_from_preset_class` matches the real classes (`natural`, `agricultural`, ...). Test over 10 presets: `old_growth_core_v0` gives `OldGrowthCore`, at least 4 distinct districts. Today `natural` falls through to riparian. | S | No | No |
| 3 | **VEG-H02-TACTICAL-VEG-DRAW-001** (new) | `render/tile_world_fallback.rs`, `gui/map_tile_raster.rs`; read-only `vegetation_visual_extract.rs` | Tactical raster draws cover density and burn/scar symbols from `VegetationExtractFrame` (symbol style per `design_dual_view_art_home_v1.md`). Test: dense-canopy and `BurnScar` chunks differ. No write to `WorldPresentationMode` or `RepresentationResult`. | M | **Yes, core.** PixelWorld path must stay unchanged. | **Yes** (burn glyph and frame) |
| 4 | **VEG-D03-CONSTRUCTION-CLEAR-001** (also fixes the A14 caveat) | `systems/ecology/landscape_grammar_lg2.rs` | A committed footprint resets `SuccessionState` (age 0, `Grass`) and dips canopy on affected chunks; harvest does the same. Tests extend `construction_disturbance_maps_footprint_chunk` and `harvest_queue_drains_to_history`. | S | Yes (stage drives extract and stamp key) | No |
| 5 | **VEG-H03-WITNESS-HONESTY-001** (new) | `dev/landscape_grammar_sim_harness.rs`, `dev/veg_diag_extract_panel_live_proof.rs`, `landscape_grammar_map.rs` | Phase D/E computed, not hard-coded `true`. `anchor_source` reflects real adapters, else `proxy`. Diag-panel witness reads real UI or is retired. Harness test stops asserting on gitignored files it does not write. **Wait for other agents to finish in `src/dev/*.rs`.** | S | No | No |
| 6 | **VEG-B-LAMBDA-HYDRO-001 + VEG-D04-HYDRO-RIPARIAN-001** | `landscape_grammar_map.rs` (`lambda_inputs_from_live_fields`), read-only `terrain/generation/derived.rs` | `hydrology_bias` comes from chunk-mean `drainage` and `flood_basin_id` (LAND-LAYERS-002), not `soil_moisture`. Test: valley chunk picks wetland or riparian preset, ridge does not. | M | No (preset pick; both views read the result) | No |
| 7 | **VEG-B-LAMBDA-TRANSPORT-001 + VEG-D02-TRANSPORT-CORRIDOR-001** | `landscape_grammar_map.rs`; read-only `infrastructure/transport/graph.rs` | `transport_access` from `TransportGraph` edge proximity per chunk (replaces the `cells_per_chunk*0.01` stand-in). Test: chunk on an edge picks the corridor/industrial preset. Do not edit `src/infrastructure`. | M | No | No |
| 8 | **VEG-B-LAMBDA-CONSTRUCTION-001 + VEG-D01-SETTLEMENT-LANDUSE-001** | `landscape_grammar_map.rs`; read-only `strategic/settlement/district.rs` (`DistrictBook`, `DevelopmentPressureBook`) | `construction_pressure` from `ConstructionClear` history and development pressure; `LandUseInfluence.district` from the `DistrictBook` record covering the chunk. Do not edit `src/strategic`. | M | No | No |
| 9 | **VEG-E02-INSTANCE-SPAWN-001** (absorbs E01 terrain-driven subcell) | `systems/ecology/landscape_grammar_lg2.rs`, new instance component, read-only terrain `cell_canopy_scales`/`drainage` | Deterministic instances spawned within `instance_budget`, placed by per-cell canopy scale and drainage. `SubcellPopulationGrid` filled from terrain, and read. Seeded, bounded count, test for determinism. | L | Data only; draw is row 10 | No |
| 10 | **VEG-E03-INSTANCE-EXTRACT-001** | `render/extraction/vegetation_visual_extract.rs`, consumers in `tile_world_fallback.rs` | `VegetationExtractFrame` carries per-instance rows; Tactical draws symbols, PixelWorld3D stamps atlas rows per instance. | M | **Yes** | Yes (burn state per instance) |
| 11 | **VEG-C06-PREVIEW-OVERLAY-001** | `gui/diagnostics_ui.rs` (Bevy text) | F3 panel shows program/topology/density lines and veg extract sample rows (`format_veg_extract_diag_sample`). Test on `diagnostics_body_text`. | S | No | No |
| 12 | **VEG-G01-OPERATOR-HISTORY-001** | `gui/diagnostics_ui.rs` | F3 panel lists the last disturbance events from `DisturbanceHistory`. | S | No | No |
| 13 | **VEG-E04-FIRE-CORRIDOR-001** | `systems/ecology/landscape_grammar_lg2.rs`; read-only `systems/fire/chunk_fuel_profile.rs` | Fuel bridge reads `VegetationPopulation` (and subcell cells once row 9 lands), not stage alone. Witness asserts a delta against stage-only fuel. | M | No | **Yes** |
| 14 | **VEG-E05-SUCCESSION-GRAPH-001** | `landscape_grammar_lg2.rs` | Succession stage tracked per topology node (or per partition), and a disturbance persists instead of being re-derived from age each tick. | M | Yes (stage drives visuals) | Yes (BurnScar) |
| 15 | **VEG-D06-SIM-EFFECT-001** | `systems/ecology/landscape_grammar_lg2.rs`, read-only `sim/effects/drain.rs` | Real fire and construction disturbances emit a telemetry-only `SimEffect`. No double write: `drain.rs` maps non-harvest to a `ConstructionClear` row. | S | No | Yes (shared SimEffect spine) |
| 16 | **VEG-F07-NESTED-DEPTH-003-001** | `assets/configs/landscape/presets/industrial_barrier_v0.json`, `agri_riparian_v0.json` | `industrial_barrier_v0` reaches depth 3; `agri_riparian_v0` at most 8 flat nodes; per-preset eval loop test over all 10; `validate-report landscape_grammar` green. | S | No | No |
| 17 | **VEG-H04-ATLAS-KIND-COVERAGE-001** (new) | `landscape_atlas_registry.rs`, `gui/landscape_chunk_atlas_stamp.rs` | `topology_kind_to_variant_key` resolves all 10 on-disk kinds (Network, Pocket, Barrier, Spine, Basin have none now) through an explicit fallback. Art rows for new kinds stay in F01. | M | PixelWorld only | Atlas burn rows |

Rows 1 to 4 come first: they are small, they fix correctness, and row 3 is the first time trees (cover) show in the default Tactical view. Rows 6 to 8 are independent of each other and each reads a different system read-only. Rows 9 and 10 are the real "individual trees" work and need row 1 and row 4 first. Rows 11 and 12 are cheap UI wins any time.

## Cross-lane notes

- **Parity needed (both TacticalFlat2D and PixelWorld3D):** rows 3, 4, 10, 14. Row 3 is the only one that adds a Tactical draw; Tactical stays symbols and icons, not APS art (`plan_dual_view_pixel_world_v1.md`).
- **Fire and effects overlap:** rows 1 (canopy and fuel writer), 3 and 10 (burn glyph and frame), 13, 14, 15, 17. `ActiveBurn.frame_index` drives both burn draw and `veg_burn_*` keys; do not add a second burn timer.
- **Other agents editing now:** `src/dev/*.rs` (row 5), `src/gui` (rows 3, 11, 12), `src/render/fire_vfx` (rows 3, 10). Rows 1, 2, 4, 6 to 9 stay inside `src/systems/ecology` and read other lanes only.
- **Bevy 0.19 NoCpuCulling:** any new sprite or mesh2d entity in rows 3 and 10 must not set entity-level `NoCpuCulling` (silent invisibility, see memory note); verify with `RTT_SPRITE_TRACE=1`.
- **Witness caveat:** `phase_d_green` and `phase_e_green` are hard-coded `true` in the harness close body, and the lg3 `anchor_source` is a literal. Treat `vegetation_program_close_live.json all_green` as stale until row 5.
