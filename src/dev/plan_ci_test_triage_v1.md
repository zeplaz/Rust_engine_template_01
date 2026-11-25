# PLAN-CI-TEST-TRIAGE-v1 — the 18 failing lib tests: broken feature or stale test

Status: **CLOSED** (2026-11-24): all 7 fixes applied, `cargo test --lib` 2 347 passed / 0 failed, `-D warnings` clean. CI green on `d11695fe` (two Python-step follow-ups: `cad07f75` installs jsonschema; `d11695fe` makes `repo_root()` ignore the committed Windows repo path when it does not exist). CI (`.github/workflows/ci.yml`) has been red since at least
2026-11-15. Step 1 (`cargo rustc --lib -- -D warnings`, STAB-CI-001) is fixed (476ac36d); step 4
(`cargo test -p proc_A_dine01 --lib`) fails on the 18 tests below. None were caused by the roads
work (checked against the commit before it). Each was traced to its root cause; two causes were
confirmed by a reverted experiment (noted).

## Verdict per failure

| # | Test(s) | Root cause | Verdict | Fix |
|---|---------|------------|---------|-----|
| T-1 | 12 tests: `economy::resource_flow` ×3 (aluminum / geopolymer chain edges, starved smelter), `economy::logistics` LOG-B ×3 + LOG-C ×2, `activation::concrete_chain_e2e` ×2, `economy::game_loop_e2e`, `ai::faction::scenario` | `activate_industrial_facilities_system` takes `If<Res<ManufacturingBlueprintRegistry>>` (f5d569c8). Bevy 0.20 `If` **skips the whole system** when that registry is absent, so no building activates at all — not just manufacturing plants. The game inits the registry (`ManufacturingPlugin`), so play works; every test app without it gets no activation → no flow edges, no supply chain, no production | **Latent code bug** (fragile coupling), tests are right | Make the blueprint registry `Option` and only skip the manufacturing node when it is missing; buildings still activate. **Confirmed:** with that change all 12 pass (experiment, reverted) |
| T-2 | `activation::bridge::operational_missile_body_plant_gets_mfg_and_buffers_no_flow` | `missile_body_plant.json` gained `required_research_ids: ["manuf.cruise_line"]` (ad33eb1b); the test world has no `ResearchCapabilityBook`, so the research gate correctly keeps it off | **Stale test fixture**, feature correct | Test inserts a research book with `manuf.cruise_line` unlocked (and add a case: locked → not activated) |
| T-3 | `engine::launch_args::cli_playquad_maps_quadrant_world` | When `--test dam` was added (ad33eb1b) its arm was inserted inside the `playquad` arm and took `maneuver = DemoOpen; visual_auto_exit = false` with it | **Real bug**: `--test playquad` lost its stay-open demo setup | Restore the two lines in the `playquad` arm |
| T-4 | `render::pipelines::core2d_overlay_order::rnd03_each_overlay_set_has_exactly_one_pass_membership` | DF-3 water field pass (`gpu_water_field_draw.rs`) joined `Core2dOverlaySet::WaterSurface`, which already holds the W1 surface pass (`gpu_water_surface_draw.rs`); two passes in one layer have no defined order | **Real (latent) ordering gap**, test is right | Give the field pass its own locked layer (`WaterField`, after `WaterSurface`) in `CORE2D_OVERLAY_DRAW_ORDER`, or order it `.after(water_surface_draw_pass)` and record the pair in the contract |
| T-5 | `render::pipelines::rnd10_init_pipeline_system_runs_on_render_startup_only` | `terrain_gpu_bake_rtt.rs` registers `(init_terrain_bake_rtt_pipeline, init_terrain_bake_mip_pipeline)` as a tuple on `RenderStartup`; the test text-matches only `add_systems(RenderStartup,{func})` | **Brittle test**, feature correct | Test accepts the tuple form (or register the two init systems in separate `add_systems` calls) |
| T-6 | `systems::atmosphere::frame::rebuild_is_revision_keyed` | 22d929a5 deliberately stopped sim time alone from forcing a rebuild and updated the middle asserts, but not the last count (now 1 → 2 resamples, test still expects 3) | **Stale test**, feature correct | Expect 2 |
| T-7 | `strategic::sim::strategic_bundle_runs_coupling_and_settlement_growth` | `StrategicFieldsAndAiPlugin` → `SimulationPlugin` now includes combat systems that read / write `WeatherPollSiteBook`, `BlastCraterBook`, `AudioOneShotQueue`, drone tuning — resources other plugins own; the test app lacks those plugins | **Stale test setup**; also a plugin-dependency leak | Combat plugin declares what it touches (`init_resource` is idempotent; same pattern as the `SiteDestroyed` / `CommitConstructionSiteEvent` fixes in R-7) |

**Summary:** 1 real bug in play (T-3 playquad launch), 2 latent code faults the tests caught
(T-1 activation depends on an unrelated registry, T-4 two passes sharing one draw layer), 4 stale or
brittle tests (T-2, T-5, T-6, T-7). No test is useless: each guards a behaviour that still exists.

## Order

1. T-3 (one-line restore) and T-1 (one param + one call; fixes 12).
2. T-6, T-5, T-2 (test-only updates).
3. T-7 (combat plugin declares its resources).
4. T-4 (draw-order contract: new layer or explicit after; update `CORE2D_OVERLAY_DRAW_ORDER` + count).
5. Run `cargo test -p proc_A_dine01 --lib` (expect 0 failures), push, watch CI to green.

## Applied (2026-11-24)

- T-3: `playquad` arm restored (`DemoOpen`, stay open).
- T-1: blueprint registry is `Option` in `activate_industrial_facilities_system`; only the manufacturing node needs it.
- T-2: missile plant test unlocks `manuf.cruise_line` and also checks locked → stays off.
- T-5: RND-10 test accepts a tuple `add_systems(RenderStartup, (…))`.
- T-6: expects 2 resamples.
- T-7: `CombatPlugin` declares `BlastCraterBook`, `AirOpsTuning`, `WeatherPollingStations`, `WeatherPollSiteBook`, `AudioOneShotQueue`, `ResearchCapabilityBook`, `LogisticsOrderRequest` (all no-ops when the owner already added them; loaded values still win: air-ops tuning is inserted at startup).
- T-4: new `Core2dOverlaySet::WaterField` layer after `WaterSurface` for the DF-3 field pass. Fixing it exposed a second gap the test had hidden: `TerrainInstanced` had no pass since GS-03 deleted the per-tile terrain draw (terrain is the world bake texture) — the empty layer is removed from the contract.

## Done when

CI run on master passes steps 1–4; `cargo test --lib` 0 failed.
