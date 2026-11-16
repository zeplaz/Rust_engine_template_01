# Game system → book router (start here)

Find your system, read its **gems** (niche, high-leverage sections chosen from the TOCs), then run
`lookup.py --code <path>` or `lookup.py <words>` for page ranges. Row ids point into [`matrix.md`](matrix.md).
Code roots are relative to `src/`.

## Dual matrices (cite both)

Every orient runs **NEW** and **LEGACY** in the same session — complementary, not either/or.
Record **both** cites in commits, adoption notes, and slice evidence.

| NEW (`src/dev/ref_books_index/`) | LEGACY (`$REF_BOOKS_DIR\lookup\`) | Prefer |
|:---|:---|:---|
| `gea4v1` | `gea_v1_e4` | foundations, HID, debug, asset pipeline |
| `gea4v2` | `gea_v2_e4` | graphics, lighting, streaming, gameplay runtime |
| `aig3` | `aig_e3` | PCG, grammars, AI tooling |
| `gea3` | `gea_e3` | fallback only when 4e has no section |

```bash
python src/dev/ref_books_index/lookup.py --dual <terms>
python src/dev/ref_books_index/lookup.py --dual --code <path-under-src>
```

**DBG / IO / PCG dual pairs** (NEW row id → LEGACY keyword):

| NEW | LEGACY keyword | Notes |
|:---|:---|:---|
| DBG-01..DBG-07 | `debug_tools` | gea_v1_e4 p566 |
| DBG-03 | `debug_drawing` | gea_v1_e4 p571 |
| DBG-06 | `debug_cameras_pause` | gea_v1_e4 p582 |
| DBG-04 | `render_debug_tools` | gea_v2_e4 p248 |
| IO-09 | `asset_pipeline` | gea_v1_e4 p68 |
| IO-11 | `engine_configuration` | gea_v1_e4 p454 |
| PCG-06 | `shape_grammars` | aig_e3 p748 |
| PCG-* | `pcg` | aig_e3 p690 |
| DES-04 | `ai_tools` | aig_e3 p896 |
| RND-09 / voxel VFX | `geometry_visual_effects` | gea_v2_e4 p141 |
| ECS-02 | `runtime_object_model` | gea_v2_e4 §17.2 p563 (property-centric) |
| RND-04 transparency / OIT | `geometry_visual_effects` | gea_v2_e4 p141 · gea4v2 11.3.12 / 11.3.12.1 (sort path in fire/weather VFX) |
| PHYS-07 explosions / blast funnel | `physics_integration` | gea_v2_e4 §14.5 p423 · gea4v2 14.5.3.2–4 (intent→execute; no rigid-body middleware) |
| PHYS-04 collision materials/filters | `collision_detection` · `physics_collision` | gea_v2_e4 §14.3 p359 / p352 · gea4v2 14.3.8 / 14.3.8.3 (tags → layers/masks; MaterialId indexes collision table) |

## Fire, smoke, ecology loop
Code: `systems/fire/`, `systems/ecology/`, `compute/heat_diffusion.rs`, `render/fire_vfx`, `render/lighting/`
- **TAC-04 cellular automata** (aig3 6.2.8): `cell_spread` is a CA; the book's neighbourhood and update-order notes apply directly.
- **TAC-03 convolution filters** (aig3 6.2.7): use separable kernels for heat diffusion and smoke smoothing instead of 8-neighbour loops.
- **PHYS-06 islands and sleeping** (gea4v2 14.4.7.7): the "quiet chunk short path" is sleeping; borrow the wake-on-neighbour rule.
- **LGT-09 radiometric falloff** (gea4v2 12.2.4.5): a physical basis for radiant-heat exposure in `entity_exposure`.
- **RND-15 decals** / **RND-14 card effects** / **RND-04 transparency and OIT**: burn scars, smoke sheets, layering order.
- **LGT-02 Forward+ / clustered lights** (gea4v2 12.5.12.4): many fire lights → per-tile light lists (`fire_light_cluster`).
- **ANIM-01 normalized phase** (gea4v2 13.4.1.5): global-clock phase keeps looping flames from popping on LOD switch.
- Rows: TAC-04 TAC-03 PHYS-06 PHYS-05 LGT-03 LGT-04 LGT-05 LGT-09 RND-13 RND-14 RND-15 CONC-07 CONC-08 PCG-05 DES-06

## Weather, wind, atmosphere
Code: `systems/weather/`, `systems/atmosphere/`, `render/weather_vfx`
- **LOOP-08 object state caching** (gea4v2 17.6.3.4): the `WeatherField` prev/next pair; the book covers interpolation and when to swap.
- **LOOP-02 abstract timelines** (gea4v1 8.4): weather steps on game time; pause and speed must never leak into real-time consumers.
- **SENSE-02 polling stations** (aig3 11.3): `AirOpsWeatherStatus` is one; route every "what's the weather here" read through cached stations.
- **SENSE-04 finite-element sense manager** (aig3 11.4.5): sensor range degrading by weather, propagated over cells.
- **LGT-04 volumetrics, clouds** / **LGT-10 baking and probes**: cloud sheets and `overlay_bake`.
- **AUD-05 atmospheric absorption** (gea4v2 15.1.3.2, 15.4.3.4): rain and fog should low-pass the mix.
- Rows: LOOP-02 LOOP-08 SENSE-02 SENSE-04 LGT-04 LGT-05 LGT-10 EXEC-02 MEM-10 AUD-05 MOVE-09

## Logistics, haul, transport
Code: `economy/logistics/` (`orders/` = `LogisticsOrderSim`), `systems/navigation/`, `systems/transport/`, `infrastructure/transport/`
- **DEC-10 smelly GOB** (aig3 5.7.7): plants and sites *advertise* demand and haulers pick, the Sims smart-object pattern; the book shows the scoring.
- **NAV-09 open-goal pathfinding** (aig3 4.7.1): "nearest depot that has the cargo" in one search, not N searches.
- **NAV-10 dynamic pathfinding** (aig3 4.7.2): fire and weather close roads; replan incrementally.
- **NAV-13 continuous-time pathfinding** (aig3 4.8): edge costs that change over time (weather haul windows, advancing fronts).
- **NAV-14 tactical pathfinding** (aig3 6.3): danger-weighted cost and concern blending; `logistics_floodfill` already tracks max risk.
- **NAV-04 node-array A\*** (aig3 4.3.7 · LEGACY `node_array_astar` aig_e3 p249 · V4-NAV04): dense `LogisticsNodeRecord` / `LogisticsNodeArray` + generation reuse in `logistics_floodfill`; Dijkstra = A\* with h=0 (concern blend); `nav04_*` tripwires.
- **NAV-07 / NAV-08 hierarchical pathfinding and its artefacts** (aig3 4.6.3–4.6.4): portals and districts, plus the odd routes they cause.
- **EXEC-04 anytime algorithms** (aig3 10.2): the throughput solver should always return a usable answer.
- **ECS-15 tool vs runtime split** (gea4v2 16.2.2 · V4-ECS15): IF-TRG authoring vs SIM-TRN runtime (`infrastructure/transport` → `systems/transport`); same split for Studio ParamFrame / massing_preview (`sim_occupation: false`) vs runtime massing — occupation only via construction funnel.
- Rows: DEC-10 NAV-01..NAV-14 MOVE-02 MOVE-03 MOVE-08 EXEC-04 LOOP-11 MEM-01 MEM-06 MEM-08 ANIM-03 ECS-15

## Fronts, formations, combat, ballistics, air missions
Code: `combat/`, `render/world_map_front_mesh.rs`, `gui/front_controls.rs`
- **TAC-01 influence maps** + **TAC-02 map flooding** + **TAC-03 convolution**: `relax.rs` and `front.rs` are exactly these; aig3 6.2 is the core chapter.
- **TAC-06 update-rate tiers for tactical layers** (aig3 6.2.5): classify each layer static, slow or dynamic.
- **MOVE-05/06 formations and slot assignment** (aig3 3.7.4, 3.7.8): two-level steering plus cost-based slot assignment.
- **MOVE-07 bounding overwatch** (aig3 3.7.10, 6.4.4): tactical advance behaviour.
- **MOVE-09 firing solutions with drag, iterative targeting** (aig3 3.5.4–3.5.5): missiles and shells under wind drift.
- **PHYS-02 swept collision** (gea4v2 14.3.5.7): fast projectiles tunnel through cells without it.
- **PHYS-09 bookkeeping over rigid bodies** (gea4v2 14.1 / 14.1.2.1 / 14.1.3 · LEGACY `physics_collision` · `physics_middleware` gea_v2_e4 p352/p358 · V4-PHYS09): `ballistics.rs` kinematic flights + BlastIntent enqueue — no rigid-body middleware world (ADOPTED).
- **TAC-10 multi-tier AI** (aig3 6.4.1): faction → formation → unit → mission packages.
- **ECS-07 deferred inter-object mutation** (gea4v2 17.7.4.5): `MissionIntentQueue` → `MissionBook` sole writer.
- Rows: TAC-01..TAC-11 MOVE-05..MOVE-09 PHYS-02 PHYS-07 PHYS-08 PHYS-09 DEC-13 ECS-07

## Faction AI and strategic behaviour
Code: `ai/faction/`, `strategic/` (behavior_*, hybrid_*, world_read_snapshot, strategic_command_queue)
- **DEC-15 perception window, changes of behavior** (aig3 2.1.3–2.1.4): players only notice *changes*; the AI explain panel (`Insert`) should surface those.
- **DEC-08 timing in GOB** (aig3 5.7.4): score goals per unit time.
- **DEC-12 blackboard** (aig3 5.9): `WorldReadSnapshot` is a blackboard; experts write through queues.
- **DEC-06 Markov state machines** + **DEC-05 fuzzy** (aig3 5.6, 5.5): formal models for the hybrid brain's probabilistic resolution.
- **EXEC-05 group / behavioral LOD** (aig3 10.3.4–10.3.5): `hybrid_fields` is group LOD for distant agents.
- **LEARN-02 N-grams** (aig3 7.3.4): cheap prediction of player patterns.
- **TAC-14 MCTS** / **TAC-15 evaluation functions** (aig3 9.5, 9.8): if plain weights stop scaling.
- **MATH-02 numeric mixing and seeds** (aig3 8.1.1): `ai/faction/rng.rs` sub-seed derivation.
- Rows: DEC-01..DEC-15 EXEC-01 EXEC-05 SENSE-03 SENSE-05 LEARN-01 LEARN-02 TAC-07 TAC-12 MATH-02 LGT-08

## Construction, settlement growth, procedural buildings
Code: `construction/`, `strategic/settlement`, `strategic/site`, `construction/procedural/`
- **ANIM-02 transition matrix** (gea4v2 13.10.4.3): table-driven allowed transitions for `site_stage_transitions`.
- **PCG-06 shape grammars and grammar planning** (aig3 8.5.2): building grammar and voxel massing.
- **ECS-15 tool vs runtime** (gea4v2 16.2.2 · V4-ECS15): Studio/tool massing hydrate refuses `sim_occupation: true`; sole runtime writer is `sync_voxel_massing_visual_requests`.
- **PCG-07 spanning tree, recursive subdivision** (aig3 8.4.2–8.4.3): settlement road skeletons and blocks.
- **PCG-08 generate and test** (aig3 8.4.4): `assembly_quality_gate`.
- **INPUT-02 chords and gestures** (gea4v1 9.5.4.3): drag-to-build timing windows.
- **ECS-14 editor placement and alignment aids** (gea4v2 16.4.1.7): snapping.
- **PHYS-07/08 explosions, destructibles** (gea4v2 14.5.3.2–5 · LEGACY `physics_integration`): `blast.rs` intent→execute funnel + falloff; `demolish.rs` `DestructibleDamageStage` ladder + sole terminal execute (ADOPTED).
- Rows: ANIM-02 PCG-06 PCG-07 PCG-08 INPUT-02 INPUT-04 ECS-03 ECS-14 ECS-15 PHYS-07 PHYS-08 DEC-02

## World generation, terrain, hydrology
Code: `terrain/generation/`, `terrain/voronoi*.rs`, `terrain/material/`
- **PCG-03 thermal and hydraulic erosion** (aig3 8.3.5–8.3.6): believable valleys feeding the D8 flow pass.
- **PCG-02 faults** / **PCG-04 altitude filtering** (aig3 8.3.4, 8.3.7): ridge lines and height-distribution shaping.
- **TAC-05 terrain analysis** (aig3 6.2.3): precompute chokepoints and defensibility in `p5_agent_overlay`.
- **MATH-03 Halton, Poisson disk, phyllotaxis** (aig3 8.1.2–8.1.4): clump-free vegetation and prop scatter.
- Rows: PCG-01..PCG-05 TAC-05 MATH-02 MATH-03 RND-17 RND-06

## Schedule, ECS architecture, determinism (engine-wide)
Code: `systems/sim_control.rs`, `systems/sim_attention.rs`, `systems/chunk_sim_lod.rs`, `systems/chunk_environment_set.rs`, `substrate/`
- **LOOP-05/06 phased and bucketed updates** (gea4v2 17.6.3.1–2): named SystemSets as dependency buckets.
- **LOOP-07 one-frame-off lag** (gea4v2 17.6.3.3): document which reads may be one tick stale.
- **LOOP-09 time-stamping** (gea4v2 17.6.3.5): epochs and dirty masks for staleness.
- **CONC-03 lock-not-needed assertions** (gea4v1 4.9.7.5): a debug-only check that enforces "one writer per domain".
- **ECS-08 jobs / async updates** (gea4v2 17.7.4.1–2 · 17.7.4.6) + **CONC-01** job counters (gea4v1 8.6.4): chunk-parallel job products; main-thread sole apply (`chunk_fire_overlay` edge deposits · `io/streaming/task_pool` hydrate completions). LEGACY: job systems / async game-object update.
- **ECS-12 data-pathway triggers** (gea4v2 17.8.11.1–2 · V4-ECS12): `trigger_spec`/`trigger_registry` resolve registered edges to intents only; unregistered/malformed edges do not broadcast. LEGACY: `events_message_passing`.
- **MEM-08 fixed vs floating point** (gea4v1 3.3.1.3–4) + **LOOP-04 float clocks**: determinism hygiene.
- **EXEC-01 frequency and phase scheduling** (aig3 10.1.1): stagger heavy systems across ticks.
- **MEM-10 virtual-memory pages** (gea4v1 3.5.2 / 3.5.2.3–4; LEGACY `memory_architectures` gea_v1_e4 p183): `ChunkSlab` + `field_page_residency` mirror page table + residency + fault-in (handles, not raw ptrs).
- **ECS-02 property-centric slabs/layers** (gea4v2 17.2.2 / 17.2.2.3–4 · V4-ECS02; LEGACY `runtime_object_model` gea_v2_e4 p563): `ChunkSlab<T>` = homogeneous property/page columns; `FieldScalarLayer` + AoS `ChunkFieldCell` = named properties of space (SoA-vs-AoS trade-offs documented; MEM-05/10 unregressed).
- Rows: LOOP-01..LOOP-12 ECS-01..ECS-13 CONC-01..CONC-10 MEM-01..MEM-10 EXEC-01..EXEC-03

## Save, load, streaming
Code: `io/save/`, `io/streaming/`, `substrate/hydrate.rs`
- **IO-02 air locks** (gea4v2 17.4.2): a hysteresis band between load and unload radii.
- **IO-07 cross-references and pointer fix-up** (gea4v1 7.2.2.9): stable ids plus one remap pass on apply.
- **IO-08 post-load initialization** (gea4v1 7.2.2.10): rebuild derived caches; never save them.
- **IO-03 spawn memory** (gea4v2 17.4.4 / 17.4.4.1 offline · 17.4.4.2 dynamic; LEGACY `memory_management` · `world_streaming`): per-chunk offline ceiling + per-frame dynamic cap clamp hydrate/reconstruct; RemoteAllocator reserve fills the gated batch.
- Rows: IO-01..IO-11 ECS-05 CONC-05 CONC-09 ANIM-03 MEM-09
- **ANIM-03 quantization / key omission / curve compression** (gea4v2 13.8.2–13.8.4 · LEGACY `animation_compression` gea_v2_e4 §13.8 p306 · V4-ANIM03): save `quantize_unit_interval_f32` + Identity-only payload codec; logistics snapshot omits fill/throughput keys and quantizes before compare. No AnimationCurve host — true keyframe compression deferred.
- **CONC-09 async requests** (gea4v2 17.7.2 / 17.7.2.1): request early (`SaveIoDispatcher::submit`, `submit_stream_hydrate_work`), consume a frame later on main (`apply_save_io_completion`, `apply_stream_io_completion`); `hydrate.rs` = worker-side body only. LEGACY: async / job / streaming.

## Rendering, views, overlays, map
Code: `render/`, `gui/representation_policy.rs`, `gui/world_representation.rs`, `gui/map_view`, `gui/tactical/`
- **RND-06 LOD cracks and LOD graphs** (gea4v2 11.6.8.4–5): chunk seams between LOD bands.
- **RND-07 virtual textures** (gea4v2 11.3.14.8): the stage-6 virtualization model.
- **RND-08 mipmapping, texel density** (gea4v2 11.3.14.5–6): atlas bleed and readability across zoom bands.
- **RND-20 color spaces** (gea4v2 11.2.3.12–13): perceptual heatmap ramps; blend in linear.
- **RND-19 double/triple buffering, v-sync** (gea4v2 11.3.4.1; gea4v1 8.5.2.5; LEGACY gea_v2_e4 p50-51 · gea_v1_e4 p518-519) — **ADOPTED** V4-RND19: `PresentSurfaceConfig` in `render/view_runtime/present.rs`; debug AutoVsync+triple latency 3; `PERF_NO_VSYNC`→AutoNoVsync+latency 1; `rnd19_*` tripwires.
- **DBG-04 GPU capture** (gea4v2 12.9.4; LEGACY `render_debug_tools` gea_v2_e4 p248) — **ADOPTED** V4-DBG04: `VFX_CAPTURE` / `RTT_SPRITE_TRACE` + env contract `render/probes/vfx_capture_env.rs` (PlayQuad → `VFX_CAPTURE_VIEW=voxel`, frame ≥2000); `dbg04_*` tripwires.
- **MATH-06 Solving 3D problems in 2D** (gea4v1 5.1 p350–351; LEGACY `math_3d` gea_v1_e4 p350): reduce world→map XY, lift for presentation; `gui/sim_map_projection.rs` owns the contract + `math06_*` tripwires.
- Rows: RND-01..RND-22 LGT-01..LGT-07 UI-01 DBG-02 DBG-04 MATH-04 MATH-06

## Audio
Code: `audio/`
- **AUD-02 ducking, mix snapshots, instance limiting** (gea4v2 15.5.8.2–4): warning bus ducks ambience; cap combat one-shots.
- **AUD-08 dialog priority and exclusivity** (gea4v2 15.6.2.4–7): alert barks.
- **AUD-03 sound cues** (gea4v2 15.5.7.2): randomized variants behind one id.
- Rows: AUD-01..AUD-08

## Input and UI
Code: `gui/input_*.rs`, `gui/gameplay_capture.rs`, `render/view_runtime/input_routing.rs`, `gui/hud/`, `gui/style/`
- **ECS-10 chain of responsibility** (gea4v2 17.8.7): UI-first capture and the ESC cascade.
- **INPUT-04 context-sensitive controls, disabling inputs** (gea4v1 9.5.8–9): build mode, pointer gate.
- **INPUT-05 dead zone and filtering** (gea4v1 9.5.2–3): edge-pan and zoom smoothing.
- Rows: INPUT-01..INPUT-05 ECS-10 UI-01 UI-02 IO-11 DBG-06

## Tuning and review
Code: `tools/sim_review/`, `combat/balance.rs`
- **LEARN-01 hill climbing and annealing** (aig3 7.2.4): auto-tune constants against `matrix.json` scores.
- **LGT-08 Monte Carlo variance** (gea4v2 12.6.2.7): how many runs a score needs before a change is real.
- Rows: LEARN-01 LGT-08 DBG-02 DBG-07 DES-04
