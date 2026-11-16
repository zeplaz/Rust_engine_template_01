# Play-test quadrant world `v1`

| Field | Value |
|:---|:---|
| **ID** | **PLAYTEST-QUADRANT-WORLD-001** |
| **Status** | **SHIPPED** — `engine/playquad.rs` + `TestScene::PlayQuad` landed; plan retained as design record. |
| **Signed by** | `@planner` · 2026-11-15 |
| **Executors** | `@coder` (all Rust, `tools/sim_review/`) · `@designer` (read review only) · `@coder-mcp` (**no phase**, see §9) |
| **Parents** | [`feature_map_from_sim_dump_v1.md`](feature_map_from_sim_dump_v1.md) · [`plan_studio_massing_grammar_bridge_v1.md`](plan_studio_massing_grammar_bridge_v1.md) · [`plan_voxel_massing_presentation_v1.md`](plan_voxel_massing_presentation_v1.md) · [`plan_operator_build_readability_exec_001_v1.md`](plan_operator_build_readability_exec_001_v1.md) · [`plan_dual_view_pixel_world_v1.md`](plan_dual_view_pixel_world_v1.md) |
| **Scale contract** | [`world_scale_contract.rs`](../terrain/world_scale_contract.rs) (`CHUNK_TILES_SIM = 32`) · [`TileExtentPreset`](../terrain/world_map_scale.rs) |
| **Supersedes** | The earlier draft of this file. Its corrections are in §1. |

**Intent.** One larger harness world with four spatial quadrants, so fire, the build funnel, hydrology and weather can interact and show up in the dump. `--test buildread` and VoxelMassing stay separate from this world.

**Baseline.** The working tree already holds uncommitted implementation of parts of this plan (`TestScene::PlayQuad`, lake attachment, marsh belts, `render/pixel_world/rain.rs`). I read it and did not validate it. §1 records where it disagrees with this plan. Treat each WIP item as `landed`, `partial` or `reconcile`. The first gate is `cargo check`, run through `validate-report cargo`.

---

## 1. Corrections to the earlier draft, and WIP findings

| # | Finding (checked in source) | Consequence |
|:--|:---|:---|
| C1 | `WORLD_HARNESS_TILES_AXIS` **does not exist**. The extent is set at [`world_gen_params_for_maneuver`](../engine/test_harness.rs): `FullCapture`/`DemoOpen`/`None` use `MediumSmall`, and `City`/`Vfx`/`SubFx` re-apply `MediumSmall`. | Add a **PlayQuad-only** branch (P1). Do not edit the shared arms. |
| C2 | `WorldGenParams::default()` is already **512×512**. The harness narrows it to 320. | `Standard` is the editor and product default, so the 512 path is not new. |
| C3 | `WeatherVisualSettings.particles` is **retired** (ES-5-3, always false at runtime). The harness still writes `wx.particles`. The harness chrome that is real is `wx.enabled`/`wx.overlay`, `WeatherFireFieldDebugOverlay.show` (`gpu.show`), `tile_debug.use_batched_mesh_overlay` and `fire_heat`. | The hard split is expressed in terms of those real switches, not `particles`. |
| C4 | Real rain is the **Core2d** pass `weather_precip_pass` ([`weather_vfx/draw.rs`](../render/weather_vfx/draw.rs)), gated by `Has<TileDebugRenderHost>`. Under PixelWorld3D, `MainWorldCamera.is_active = !is_pixel_world` ([`map_camera.rs`](../gui/tactical/map_camera.rs)) and [`PixelWorldCamera`](../render/pixel_world/camera.rs) is a `Camera3d` that draws only `PIXEL_WORLD_RENDER_LAYER`. | Swapping the camera query does not fix this. Rain needs a PixelWorld-native draw that reads the existing `WeatherPrecipFrame` (P5). The GPU fire and water-particle Core2d passes have the same gap. They are out of scope; in PixelWorld3D fire reads through the stand layer (`layers.rs`). |
| C5 | The earlier draft made **SE the weather quadrant**. The user's intent is that **SW and SE are both hydrology**, and rain is a global product concern. The weather field is one preset per world, sampled per streak. | Rain is a separate acceptance track (P5), not a quadrant. SW and SE are re-roled in §3. |
| C6 | **Orientation.** [`tile_to_world3`](../render/pixel_world/mod.rs) documents tactical **+Y = north**, so **NW = `x < W/2 ∧ y ≥ H/2`** and **SW = `x < W/2 ∧ y < H/2`**. WIP `apply_marsh_belts` (`water_class.rs`) defines `in_sw = x < w/2 && y >= h/2`. That is the **NW** quadrant, which is the fire quadrant. | **Reconcile.** P0 fixes one partition function. Marsh and fire must not share a quadrant by accident. |
| C7 | **Global leak (WIP).** `LandFeatureRhythm::default()` now carries `lake_river_attach_*` and `marsh_belt_km = 0.2`. `HydrologyParams::from_biome_tuning` defaults to radius 2 and weight 2.0. `world_gen_pipeline_tick` calls `apply_marsh_belts` for every world. | This changes lake choice and `Wetland` cells for the editor, product and all existing baselines. The defaults must be **neutral** (0) unless a sign-off accepts a product-wide change. PlayQuad applies `LandFeatureRhythm::playquad_sw_hydrology()` (P4). |
| C8 | `TestScene::PlayQuad` doc comment says "SE storm-front rain · SW reserved". It also references `crate::engine::playquad`, which **does not exist yet**. `PLAYQUAD_WORLD_SEED = 4096` has **not** been scored against any quadrant criterion. | Reconcile the doc (C5). P0 creates the module. P4 picks the seed. |
| C9 | WIP `rain.rs` rebuilds a mesh each frame from `WeatherPrecipFrame` (streak cap `WEATHER_PRECIP_STREAK_CAP = 220`). | Acceptable at that cap. It carries a per-frame-allocation note, so P5 must reuse buffers or show it is bounded. |

---

## 2. Hard split (non-negotiable)

### 2.1 `--test buildread` and VoxelMassing must reject

| Forbidden | Where it is enforced | Why |
|:---|:---|:---|
| Fire seeds and `fire_heat` overlay | `seeds_fire_overlay()` is false; seed count and heat/fuel arms are `0` / `(0,0)` ([`launch_args.rs`](../engine/launch_args.rs), [`test_harness.rs`](../engine/test_harness.rs)) | Fire belongs to fire/VFX scenes and PlayQuad NW. |
| Weather, overlay and `gpu.show` chrome | `weather_chrome` excludes `BuildReadability`; its defaults arm forces them off | Not a stand-in for silhouette read. |
| LGT-04 volumetrics or smoke raymarch as massing | [`voxel_massing.rs`](../construction/voxel_massing.rs) imports no `systems::{fire,weather,atmosphere}` | Homonym false friend; the bridge REJECT ledger applies. |
| Rain or precip in `massing_columns` or `massing_preview.json` | `voxel_massing` is the sole writer of `ConstructionVisualRequests::massing_columns`; PixelWorld rain reads only `WeatherPrecipFrame` | Presentation columns are not weather. |
| Sim occupation, PHYS, AI-on-columns, nav-on-stacks | **DR-VOXEL-OCCUPATION**; sim truth stays `FootprintMatrix` / weighted raster | Presentation-only. |

### 2.2 PixelWorld3D product view must provide

Rain is a signature product concept. In `WorldPresentationKind::PixelWorld3D` it must draw as weather precip driven by the sim `WeatherField` (via `WeatherPrecipFrame`). It must not be CPU mesh particles and must not be massing. Acceptance is separate from buildread (P5).

---

## 3. Target architecture

### 3.1 World extent

| | Today | PlayQuad |
|:---|:---|:---|
| Tiles | `MediumSmall` 320² (10×10 chunks) | **`Standard` 512²** (16×16 = 256 chunks, 2.56× area, 1.6× linear) |
| Derived hydrology counts (`derive_land_features`, 100 m/tile) | 5 rivers, 3 lakes | 9 rivers, 8 lakes at default rhythm |

- 640² is not a preset. A true 2× linear world would be 640² (20×20 chunks, 4× area). **v1 takes Standard 512.**
- `LargeStrategic` 1024 is a follow-on, not in v1.
- Untouched: `FullCapture`/`--test visual` (its matrix baselines assume 320), `FrameScreen` (TacticalSmall), `UnittestWorld`, `City`/`Vfx`/`SubFx`/`BuildReadability`.

### 3.2 Quadrants

One pure partition (P0) shared by seeders, tests and the dump:

```text
Quadrants(w,h) = 4 disjoint tile AABBs, +Y north, split at chunk-aligned (w/2, h/2)
   NW = x<w/2 ∧ y≥h/2      NE = x≥w/2 ∧ y≥h/2
   SW = x<w/2 ∧ y<h/2      SE = x≥w/2 ∧ y<h/2
```

At 512 the split is at tile 256, which is chunk 8, so it is chunk-aligned. A test asserts alignment for any extent that is a multiple of 64.

| Quadrant | Role | Systems under test | Seed rule (data only) |
|:---|:---|:---|:---|
| **NW** | Fire | `ChunkSurfaceFire`, `ChunkFireOverlay`, stands (`ChunkCellStand`), smoke, logistics heat, rain on fire | Fire seed patches only on land chunks with ≥ 0.5 burnable fraction and live fuel **inside NW**. |
| **NE** | Build order | Catalog pilot, `CommitConstructionSiteEvent` → site phases → logistics draw | Sites are committed through the construction funnel on buildable land inside NE. No fire at t0. |
| **SW** | **Lake and river basin** | Big, river-attached lake(s) beside full channel rivers; shoreline faction anchor | Seed chosen so SW holds ≥ 1 designed `Lake` that touches `river_mask`. |
| **SE** | **Marsh belt and delta contrast** | `Wetland` belts against channel rivers | Seed chosen so SE holds a measurable `Wetland` belt next to a river mouth or lake. |

Authority:

```text
⊚engine::playquad ═▶ ⊨quadrant-partition      (pure fn; ¬ECS; ¬writes)
⊚test_harness ⊰ ⊚playquad                       seeds (fire heat, construction commit events) = harness-only writers
⊚dev::sim_run_dump ◂⊳[snapshot] ⊚playquad       counts per quadrant (reader)
⊚WorldWaterGrid (pass 4) ═▶ ⊨water-class        sole water authority — marsh/lake logic runs before stamp, never after
⊚weather_field_step ═▶ ⊨WeatherField            sole weather writer; harness sends WeatherPresetRequest only
⊚fill_weather_precip_frame_from_climate ═▶ ⊨WeatherPrecipFrame
⊚PixelWorldPrecipLayer ◂⊳[snapshot] ⊚WeatherPrecipFrame      render-only
⊚voxel_massing ═▶ ⊨massing_columns              unchanged; weather/fire never write it
```

### 3.3 Hydrology rules

| Rule | Detail |
|:---|:---|
| Lake attachment | `compute_hydrology_world` ranks lakes by size, with a bonus when a lake touches `river_mask` within `radius`. Only the top `lake_count` become `Lake`; the others stay `Wetland` through `classify_world_water`. WIP `rank_lakes_by_river_attachment` is deterministic (stable sort over scan order). |
| Marsh | `Wetland` comes from two sources. One is unlisted depressions. The other is the WIP soft belt (`apply_marsh_belts`): wet, low `Dry` cells near rivers or lakes. It never touches open water, so channel rivers keep their mask. |
| Quadrant emphasis | Marsh emphasis targets **SE**, with SW as the lake basin. This is a data knob, not a noise change. The WIP knob targets the wrong half of the map (C6) and needs a quadrant argument or a rename. |
| Neutral defaults | Default `LandFeatureRhythm` and `HydrologyParams` keep legacy behaviour (attach 0, marsh 0). Only PlayQuad applies `playquad_sw_hydrology()` (C7). |
| Seed over noise | If a seed sweep over N ≤ 64 candidates cannot satisfy §6 PQ-H2/PQ-H3, escalate to a moisture-bias option (an Open Question). Do not edit the shared noise path for v1. |
| Faction link | Future data only (settlement and AI attention toward shore or marsh edge). **Not in v1** and never voxel columns. |

### 3.4 Weather and rain

- PlayQuad requests one preset (`StormFront`) through `WeatherPresetRequest`, plus the `Windy` atmosphere request. `WeatherField` stays the only weather writer.
- Rain is **spatial** (sampled per streak by XY) but the preset is world-wide. Rain reaching the NW fire is a real interaction. The plan **does not force an outcome**; the dump reports rain at each quadrant centre and fire counts, and a human reads the story.
- PixelWorld rain (P5) consumes `WeatherPrecipFrame`. Its single writer already switches to the voxel ground footprint when `PixelWorldCameraFrame.active`. Recon must confirm that `ExtractedCameraMetrics` and `ParticleDomainRegistry::weather_precip_active()` still update while the 2D camera is inactive.

### 3.5 Schedule placement

```text
Update:  harness seeds (once, defaults_applied latch) ═▶ sim chunk slabs (256 at 512)
Sim:     weather_field_step ═▶ fire/ecology ═▶ construction commit ═▶ …            (unchanged order)
Extract: fill_weather_precip_frame_from_climate ═▶ WeatherPrecipFrame
View:    PixelWorldViewSet::Layers ⊃ sync_pixel_world_precip (after PixelWorldCameraFrame publish)
Dump:    sim_run_dump (debug only) reads snapshots — no sim writes
```

---

## 4. Scene and CLI matrix

| Entry | Extent | Fire seeds | Weather and wind | Build seeds | Rain-in-PixelWorld accepted? | Massing |
|:---|:---|:---|:---|:---|:---|:---|
| `--test buildread` | 320 (unchanged) | No | No | Ghost only | Out of scope | Yes |
| `--test fire` / `vfx` | 320 | World-centre (unchanged) | Vfx no; fire only | No | N/A | No |
| `--test subfx` / `weather` | 320 | No | Yes | No | **P5 applies in any PixelWorld run** | No |
| `--test visual` / `sandbox` | 320 (FullCapture baseline kept) | World-centre (unchanged) | Yes | No | Same as above | No |
| **`--test playquad`** | **512** | **NW only** | Yes (global) | **NE only** | **Yes (P5)** | NE ghost only |

`--test playquad` is stay-open `DemoOpen` and camera whole-map fit. It must not arm `VfxFireTestRegion` camera focus: `harness_camera_focus_armed` currently returns true for any `seeds_fire_overlay()` scene, so PlayQuad needs an exclusion (P2).

---

## 5. Phases (ordered DAG)

`P0 → P1 → { P2, P3, P4 } → P6 → P7`. `P5` starts after `P1` and runs in parallel. Every phase ends with `cargo check` through `validate-report cargo` plus the listed tests.

| Phase | Goal | Owner | Files | Depends | Status vs WIP |
|:--|:---|:---|:---|:---|:---|
| **P0** | One quadrant partition. Fix orientation and doc drift. | `@coder` | new [`src/engine/playquad.rs`](../engine/) (register in `engine/mod.rs`); [`world_map_scale.rs`](../terrain/world_map_scale.rs) and [`water_class.rs`](../terrain/generation/water_class.rs) (swap the ad-hoc `in_sw` for the partition or a quadrant arg); `launch_args.rs` doc comment | — | **DONE 2026-11-15** — `quadrant_tile_rect` sole partition; +Y north; tests green |
| **P1** | `PlayQuad` scene plumbing and the Standard extent. | `@coder` | [`launch_args.rs`](../engine/launch_args.rs) (CLI `playquad`/`quad`/`quadrant`, flags, `isolation_world_seed`); [`test_harness.rs`](../engine/test_harness.rs) (`world_gen_params_for_maneuver` PlayQuad branch → `Standard`, `ChunkCellMatrixAuthoritative`, `settlement_count = 0`; every exhaustive `match` arm: seed count, heat/fuel, `sim.overlays.fire_heat`, defaults arm); [`map_camera.rs`](../gui/tactical/map_camera.rs) and [`tile_world_fallback.rs`](../render/tile_world_fallback.rs) (whole-map fit) | P0 | **DONE 2026-11-15** — 512² PlayQuad-only; other scenes stay 320/192 |
| **P2** | NW fire. | `@coder` | `test_harness.rs`: parametrise `seed_test_fire_near_world_center` with a placement (`WorldCenter` \| `Rect`). Route `apply_test_scene_fire_seeds` and `preserve_test_scene_fire_after_sim_tick` through it. Pass NW rect for PlayQuad. Exclude PlayQuad in [`vfx_fire_test_focus.rs`](../gui/vfx_fire_test_focus.rs) `harness_camera_focus_armed`. | P1 | **DONE 2026-11-15** — NW land-only fire region |
| **P3** | NE build order through the funnel. | `@coder` | `test_harness.rs` (one-shot, latch-guarded NE seeder emitting `CommitConstructionSiteEvent` from the generic [`PilotCatalog`](../construction/pilot_catalog.rs), following the `seed_visual_concrete_chain_e2e` precedent); [`build_readability_harness.rs`](../construction/build_readability_harness.rs) (`arm_catalog_pilot_ghost` with an origin param, `partial`) | P1 | **DONE 2026-11-15** — ≤3 catalog commits + PixelWorld3D switch |
| **P4** | Hydrology: gate the globals, re-role SW and SE, pick the seed. | `@coder` | [`world_map_scale.rs`](../terrain/world_map_scale.rs) (neutral `Default`; `playquad_*` constructor); [`hydrology/mod.rs`](../terrain/generation/hydrology/mod.rs), [`flow.rs`](../terrain/generation/hydrology/flow.rs); [`water_class.rs`](../terrain/generation/water_class.rs); [`world_generator_enhanced.rs`](../terrain/generation/world_generator_enhanced.rs) (`apply_marsh_belts` call guarded by params); `test_harness.rs` (PlayQuad sets `feature_rhythm`, `recompute_symbolic_land_features`); a `#[ignore]` seed-sweep test; `PLAYQUAD_WORLD_SEED` | P0, P1 | **DONE 2026-11-15** — neutral defaults; `LandFeatureRhythm::playquad_hydrology`; SW river-attached lakes; SE marsh only; seed **8**; 41 filtered tests pass |
| **P5** | Rain in PixelWorld3D (separate acceptance). | `@coder` | [`render/pixel_world/rain.rs`](../render/pixel_world/rain.rs), [`weather_vfx/extract.rs`](../render/weather_vfx/extract.rs) (`from_ground_footprint`); recon on `ExtractedCameraMetrics` and `ParticleDomainRegistry` with the 2D camera off | P1 | **DONE 2026-11-15** — rain+clouds PixelWorld layers; PlayQuad dry→StormFront @1800 |
| **P6** | Dump and matrix: per-quadrant counts, scored rows, dashboard section. | `@coder` | [`sim_run_dump.rs`](../dev/sim_run_dump.rs) (read-only `quadrants` block and `presentation`); [`tools/sim_review/expected/matrix_v1.json`](../../tools/sim_review/expected/matrix_v1.json) (rows with `when_scene: ["PlayQuad"]`, which is the `{:?}` of the scene); [`score_dump.py`](../../tools/sim_review/score_dump.py) `_sections`; [`playcheck.py`](../../tools/sim_review/playcheck.py) (scene arg) | P2, P3, P4, P5 | **DONE 2026-11-15** — `sim_run_dump_quadrants.rs`; matrix `playquad.*` rows; dashboard PQ-R3 table; `counts.wetland_cells`; `validate-report cargo` 0 errors |
| **P7** | Regression sweep. | `@coder` | Full `--lib` filters in §6; confirm other scenes still resolve 320; manual capture in both views | P6 | **DONE 2026-11-15** — §6 lib sweep **62 passed / 1 ignored** (`playquad_seed_sweep`); PQ-S3 `other_scenes_keep_their_extents` (320² / 192); `validate-report cargo` 0 errors; witness `debug_runs/validators/p7_regression_result.txt`; PQ-X3: no new `*_green` / `*_LIVE_JSON` / witness; unblock `strategic/site/mod.rs` voxel re-exports (2 lines) |

### Phase details

**P0.** Acceptance: a pure function returns four disjoint AABBs that cover the world. Chunk alignment holds for extents that are multiples of 64. A test pins NW as the `y ≥ H/2` half. Risk: if the top-down raster or minimap flips Y, the labels disagree with the screen; add one test against the existing tile↔world helper. Rollback: revert the module and keep seeds disabled.

**P1.** The branch is keyed on `launch.test_scene.is_playquad()`. Rollback trigger: any regression in `launch_args::` or `test_harness::` tests for non-PlayQuad scenes.

**P2.** Land-only seeding stays (no ocean fallback). Rollback trigger: seeds touch a chunk outside NW (a test fails).

**P3.** Placement still ends at `commit_construction_site_system`. This is `construction_invariants.md` rule 2: preview never mutates, only execute commits. The harness emits events and never spawns sites directly. Open item for recon: confirm `CommitConstructionSiteEvent` validates terrain (invariant 7). If it does not, the seeder must pre-check land.

**P4.**
- Seed sweep criteria (all deterministic, no RNG beyond the seed):
  - SW holds a `Lake` with ≥ L cells touching `river_mask`.
  - SE holds ≥ M `Wetland` cells within R tiles of `River` or `Lake`.
  - NW has ≥ 0.6 land with burnable fuel.
  - NE has ≥ 0.7 buildable land.
  - No quadrant is more than 40 % ocean.
- Thresholds L, M and R are set by the first sweep and recorded in the test.
- `Wetland` extent from depressions alone may be tiny. That is why the belt exists; measure before widening.

**P5.** Steps: (a) recon. Run PixelWorld3D with `VFX_CAPTURE_VIEW=voxel`, a storm preset and the dump, then read the `weather.*` block and the precip instance count. (b) Confirm the layer draws. (c) Bound allocations. Look is a `@designer` read of streak size, tint and density only, with no spec changes to massing. Rollback trigger: any write to `massing_columns`, `FootprintMatrix` or sim state from `render/pixel_world/rain.rs`.

**P6.** Dump rows are **counts only**. The matrix is the verdict, per `tools/sim_review/README.md`: no `green` in the dump, no new witness resources.

---

## 6. Acceptance rows

Tests live in Rust (`cargo test -p proc_A_dine01 --lib <filter>`). Verdicts live in the matrix (`score_dump.py`). Rows tagged **report** are shown on the dashboard and are never pass/fail, because they are emergent interactions.

| ID | Row | How checked | Phase |
|:--|:---|:---|:--|
| PQ-S1 | Quadrants are disjoint, cover the world, chunk-aligned. NW is the `y ≥ H/2` half. | `playquad::` unit tests | P0 |
| PQ-S2 | `--test playquad` → `TestScene::PlayQuad`, `DemoOpen`, stay-open. Aliases resolve. Extent is exactly 512² with `ChunkCellMatrixAuthoritative`. | `launch_args::`, `test_harness::` | P1 |
| PQ-S3 | **Other scenes still resolve their prior extents:** `City`, `Vfx`, `SubFx`, `BuildReadability` and `FullCapture` → 320; `FrameScreen` → 192. | `test_harness::` params test | P1 |
| PQ-F1 | Fire seeder touches only chunks inside NW. Land-only. `VfxFireTestRegion` is not armed for PlayQuad. | `test_harness::` | P2 |
| PQ-F2 | Dump `quadrants.nw.fire_heat_cells > 0` at t0. **report:** spread into NE, SW and SE over ticks. | matrix + dashboard | P2, P6 |
| PQ-B1 | NE: at least one construction site committed through the commit event. No `ConstructionSite` is spawned outside the strategic commit handler. | test + dump `quadrants.ne.sites` | P3 |
| PQ-B2 | NE: `fire_heat_cells == 0` at t0. **report:** sites that exist but are not progressing. | matrix | P3, P6 |
| PQ-H1 | **Default worlds unchanged:** default `LandFeatureRhythm`/`HydrologyParams` produce byte-identical `classes` and lake lists to HEAD for a fixed seed at 320. | `water_class::` / `hydrology::` golden test | P4 |
| PQ-H2 | SW has ≥ 1 designed `Lake` that touches `river_mask` (via `lake_touches_river`). | `#[ignore]` sweep test and dump `quadrants.sw.lake_river_attached` | P4 |
| PQ-H3 | SE has a `Wetland` belt of ≥ M cells within R tiles of a river or lake. River `river_mask` cells are unchanged by belts. | sweep test and dump `quadrants.se.wetland_cells` | P4 |
| PQ-H4 | Marsh belts never convert `River`, `Lake` or `Ocean`. | existing `marsh_belt_is_soft_wetland_never_open_water` plus the quadrant variant | P4 |
| PQ-R1 | In PixelWorld3D with a storm preset, the precip frame has instances > 0, `PixelWorldCameraFrame.active` is true, and the layer entity exists. | dump `weather.precip_instances` and `launch.presentation == PixelWorld3D`; capture screenshot | P5 |
| PQ-R2 | `massing_columns` is identical with and without rain (rain writes nothing to massing). `voxel_massing.rs` has no weather, fire or atmosphere imports. | unit test; source tripwire test | P5 |
| PQ-R3 | **report:** rain at each quadrant centre against NW fire counts. | dashboard | P6 |
| PQ-X1 | `buildread` keeps `seeds_fire_overlay`, `seeds_weather` and `menu_vfx_bootstrap` false and its fire/weather chrome off. | `launch_args::` + a pure `harness chrome` table test | P1, P7 |
| PQ-X2 | Release perf at 512 has no new stalls: `perf.stalls == 0`. Debug is informational only. | matrix row with `when_build: ["release"]` | P6 |
| PQ-X3 | No `*_green` predicates, `*_LIVE_JSON` constants or witness resources are added (`AGENTS.md`). | review + `rg` | all |

---

## 7. Diagnostics required

- `sim_run_dump.json`, read-only: `quadrants.{nw,ne,sw,se}` with land, `river`, `lake`, `ocean` and `wetland` cells, `lake_river_attached`, `veg_burning_cells`, `fire_heat_cells`, construction sites by phase, and the rain sample at the centre. Also `launch.presentation`, and `weather.precip_instances` if it is not already in the `weather` block.
- `water_open_cells` and `river_cells` already exist. Add `wetland_cells` globally.
- Dashboard: a quadrant story table in `_sections`.
- Logs: reuse the `test_harness::fire` and `test_harness::industrial` targets.

## 8. Edge cases and risks

| Risk | Handling |
|:---|:---|
| Fire seed lands on water | Land-only rule plus P4 seed criteria. |
| NW fire is extinguished by global rain | Intended interaction, reported not judged. If it hides the fire, add a selectable preset later. |
| 256 chunk slabs and a 512² voxel field | Debug builds may stall. Release is the acceptance bar (PQ-X2). |
| PlayQuad whole-map fit hides detail | Whole-map fit is a spawn default only. |
| Re-arm loop (`preserve_test_scene_fire_after_sim_tick`) reseeds outside NW | It must share the P2 placement. |
| `CommitConstructionSiteEvent` on non-buildable land | P3 recon. Pre-check land if the funnel does not reject. |
| Seed sweep cost | Run `#[ignore]`d; commit only the chosen seed and thresholds. |
| Y orientation mismatch between raster, minimap and PixelWorld | P0 test; open question below. |
| Precip frame metrics tied to the 2D camera | P5 recon step (a). |

## 9. Owners and out of scope

- **`@coder`** owns all phases. `tools/sim_review/*.py` is not `tools/mcp/`.
- **`@coder-mcp`** has **no phase**. This plan adds no MCP tool or schema. Cargo validation consumes `validate_cargo_report`. The studio massing JSON (`tools/mcp`) is unchanged and sits on the buildread side.
- **`@designer`** reviews the look of the lake shore, marsh and rain only, and writes no code.

**Out of scope for v1**
- Sim voxel occupation and dig (DR-VOXEL-OCCUPATION). Inferring gameplay from massing height.
- RND-09 instanced massing, PHYS and AI-on-columns.
- `LargeStrategic` 1024 and a true 640² extent.
- Porting the GPU fire or water Core2d particle passes to PixelWorld3D.
- Changing the shared noise path or product-world defaults.
- Faction or settlement preference for water features.
- Migrating Bevy `Interaction` deprecations.
- Editing `City`/`Vfx`/`SubFx`/`Visual` extents.

## 10. Open questions

1. **Orientation.** P0 pins +Y = north from `tile_to_world3`. If the raster or minimap disagrees on screen, the partition flips. The operator confirms with one look at the capture.
2. **SW and SE assignment.** I read the user's text as SW = lake and river and SE = marsh. If the intent is the reverse, only the §3.3 emphasis target and PQ-H2/H3 swap.
3. **Rain versus fire.** One world-wide preset, or a scheduled dry-then-wet sequence so NW burns before rain arrives?
4. **Neutral defaults (C7).** Accept a product-wide lake and marsh change, or keep neutral defaults? The plan assumes neutral.
5. **Moisture-bias fallback.** If no seed in N ≤ 64 satisfies PQ-H2/H3, may a quadrant moisture bias enter the shared noise path?

---

## Sign-off

`@planner` — 2026-11-15 — **SIGNED.** Checked: the single-writer map in §3.2; the sim → view → render → UI direction (rain, dump and massing only read); determinism (fixed seed, stable sort, no unseeded RNG); the hard split in §2; no new witness or queue artifacts. Items marked `reconcile` in §1 and §5 must be closed before P6 can score.
