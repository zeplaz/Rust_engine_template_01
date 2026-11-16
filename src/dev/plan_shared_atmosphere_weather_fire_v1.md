# PLAN-SHARED-ATMOSPHERE-WEATHER-FIRE-v1 — one atmospheric read path for weather, smoke, fire, air threats

| Field | Value |
|:---|:---|
| **Version** | `1.0.8` |
| **Date** | 2026-11-15 |
| **Owner** | `@planner` (read-only; implementation → `@coder`, drift/cleanup → `@sim-steward`) |
| **Status** | **SIGNED** |
| **Builds on** | [`plan_weather_wind_v1.md`](plan_weather_wind_v1.md) (WeatherField) · [`wssr_plan_004_atmosphere_unification_v1.md`](wssr_plan_004_atmosphere_unification_v1.md) (clipmap L0, DEBT-011) · [`plan_drone_subsystem_fit_v1.md`](plan_drone_subsystem_fit_v1.md) (air_ops) |
| **Does not replace** | WeatherField, `AtmosphereClipmapStack`, `gpu_weather_fire_field`, `WeatherPrecipFrame` — it adds one frame and one set of source-term inputs. |
| **Review path** | `sim_run_dump.json` counters → `tools/sim_review/score_dump.py` (AGENTS.md). No `*_green` predicates, no witness resources in production. |

---

## Summary

Weather and fire/smoke already share more than the docs suggest: `ChunkWeather` (a projection of `WeatherField`) feeds both the fire overlay (wind, rain suppression) and the L0 fold, and `air_ops` reads weather plus L0/ChunkSmoke smoke. What is **not** shared is the *read path*. Three weather routes reach the VFX (`ClimateVisualAggregate` chunk scan, `CameraLocalWeatherSample`, per-streak `WeatherField::sample`), four smoke representations exist (`ChunkSmokeField`, L0, `ChunkSmokeGpu`, `SmokeProjectionNode`), and the loop **fire → cloud** does not exist at all. Smoke also never reaches weather visibility (the solver comment says "smoke joins in W2"; it never landed), and aircraft grounding ignores smoke even though link/sensor models include it.

**Stance:** keep the two existing sim authorities, each with a fixed domain, and add **one derived, revision-keyed per-frame snapshot, `AtmosphereFrame`**, as the only thing consumers read. Cross-domain coupling goes through **source terms into the solver that owns the target resource**, so every resource keeps exactly one writer.

```text
⊚WeatherField      ═▶ ⊨authority  synoptic: pressure · wind · temp · humidity · cloud · rain · lightning · visibility
⊚AtmosphereClipmapStack ═▶ ⊨authority  fire-borne tracers: smoke · ash · ember · heat · toxicity (+ L0 fog until retired)
⊚AtmosphereFrame   ⊰ ⊚WeatherField ⊕ ⊚AtmosphereClipmapStack      derived, rebuilt on revision change
⊚WeatherSourceTerms ⊰ ⊚AtmosphereClipmapStack   input to weather_field_step only (fire → cloud)
⊚ChunkWeather / ChunkSmokeField / ClimateVisualAggregate / CameraLocalWeatherSample ⊰ ⊚AtmosphereFrame   (transitional → frame-fed)
⊚AirOpsWeatherStatus, WeatherPrecipFrame, gpu field uniforms, PixelWorld layers ◂⊳[snapshot] ⊚AtmosphereFrame
```

---

## 1. Audit — current authorities (verified on disk 2026-11-15)

| Item | Where | Writer | State / finding |
|:---|:---|:---|:---|
| **WeatherField** | `src/systems/weather/field.rs` | `weather_field_step` only | 1 km grid, `prev`/`next` snapshots, bilinear+time `sample`. Fields: pressure, wind, temp, humidity, cloud, rain_mm_h, fog, lightning, visibility_m. Pressure systems are Gaussian highs/lows. **No smoke term in visibility. No icing. No vertical dimension** (only ground temp with lapse rate by terrain elevation). Hurricane-class is not modelled (wind clamped by `tuning.max_wind_mps`). Saved as RON (`io/save/weather_overlay.rs`, schema 1). |
| **ChunkWeather** | `weather/chunk_weather.rs` | `project_chunk_weather` | Projection at chunk centre. Fire reads `wind_mps`, `rain_intensity` (`chunk_fire_overlay.rs`: rain suppress ×0.78, rain damp). **This is the existing Cloud→Fire half.** |
| **ChunkSmokeField** | `systems/fire/chunk_smoke_field.rs` | **Two writers on one component:** `chunk_smoke_field_tick` (fire heat, rain cleanse from ChunkWeather) and `chunk_smoke_field_pull_from_advected_atmosphere` (blend toward L0). | Rain scavenging exists here (per chunk), not in L0. `visibility_penalty` mixes smoke + fog. |
| **AtmosphereClipmapStack** | `substrate/atmosphere/mod.rs`, `clipmap_advect.rs` | `atmosphere_field_fill_from_chunks`, `atmosphere_field_blend_fire_overlay_sources` (FieldFill); `advect_atmosphere_field` (WindAdvect); **and `sync_atmos_clipmap_witness_system`** (scheduled at `substrate/mod.rs:152`) which also folds smoke and runs `advect_l0_preserving_mass` | L0 128², L1 64², L2 32², L3 16². Lanes: smoke, fog, toxicity, ash, ember, heat_distortion. **No humidity/cloud/wind/precip lanes; no revision counter.** Likely **dual L0 advect writer** — W1 must confirm. L0 `fog` is a copy of `ChunkWeather.fog` (+rain×0.2): stale mirror of WeatherField. |
| **air_ops** | `weather/air_ops.rs`, `assets/configs/weather/air_ops_v1.ron` | `sync_air_ops_weather_status` → `AirOpsWeatherStatus` | Grounding = wind, crosswind, `visibility_m`, lightning. Ground speed = head/tail wind. `link_quality` / `sensor_range_mul` include `smoke01`. **Gaps: grounding ignores smoke; no icing; no turbulence/updraft; centre-tile proxy for smoke.** |
| **Fire heat → atmosphere** | `systems/atmosphere/update.rs` | FieldFill | `ChunkSurfaceFire.heat`, `ChunkFireOverlay` heat/smoke/toxic → L0 `heat_distortion`/`ember`/`smoke` by max-blend. Heat is a distortion lane only; **nothing feeds moisture, lift or aerosol back to weather.** |
| **Plume physics** | `systems/fire/ember_physics.rs` | pure fns | `plume_updraft_mps(intensity)` 2–12 m/s, buoyancy τ. Per-ember only; **no column/firestorm aggregate.** |
| **Firestorm hooks** | — | — | **None.** Grep across `src/` finds no firestorm/pyrocumulus/icing code. |
| **weather_vfx** | `render/weather_vfx/*` | `fill_weather_precip_frame_from_climate` → `WeatherPrecipFrame` | Rain/snow from `CameraLocalWeatherSample` + per-streak `WeatherField::sample`; falls back to `ClimateVisualAggregate`. Cloud sheets read `CameraLocalWeatherSample`. Presentation-only. |
| **ClimateVisualAggregate** | `systems/atmosphere/visual_extract.rs` | `publish_climate_visual_aggregate` | Re-scans all `ChunkWeather` + `ChunkEcology` each frame → means. Feeds GPU field uniforms and precip fallback. A third weather read path. |
| **GPU field** | `render/pipelines/gpu_weather_fire_field.rs`, `systems/atmosphere/gpu_field_bridge.rs` | uniform sync | 128² `Rgba32Float` ping-pong; means from `ClimateVisualAggregate` + `RenderProjectionGraph` fire/smoke nodes. Visual only, no readback. Smoke additionally uploaded as `SimChunkSmokeVisualExtract`/`ChunkSmokeGpu`, precip as its own instance buffer. |
| **Save** | `io/save/atmosphere_clipmap_overlay.rs`, `weather_overlay.rs` | — | Both saved independently. Source terms must be re-derivable from saved L0 (no new save state for them). |
| **Dump** | `dev/sim_run_dump_weather.rs` | `sample_weather_series` | Tracks wind/rain/cloud/lightning stats, `smoke01_centre`, `link_quality_*`, haul/renewable factors. **No cloud↔smoke or air-threat-composite counters.** |

---

## 2. Target architecture — one substrate, explicit trust handoff

### 2.1 Why not one merged grid
`WeatherField` is 1 km, global, time-interpolated, seed-deterministic, with a versioned RON save. L0 is chunk-indexed, mass-conserving, and advected per sim tick. Merging them would break the save schema, the `prev/next` interpolation, and the "WeatherField sole weather writer" rule. So: **domains stay split by physics scale; the read path and the coupling path are unified.**

### 2.2 `AtmosphereFrame` (new, derived)
- **Single writer:** `build_atmosphere_frame` in new set `AtmospherePipelineSet::Frame` — after `WindAdvect`, before `VisualExtract`.
- **Key:** `(WeatherField.revision, AtmosphereClipmapStack.revision, SimTick)`. Rebuild only when a key part changes (weather solves are infrequent; L0 changes per tick but only dirty tiles matter). `AtmosphereClipmapStack` gains a `revision` counter bumped by its writers.
- **Content per L0 tile (chunk-indexed, same indexing as `atmos_chunk_to_tile`):** weather samples (wind, gust, temp, humidity, cloud, rain_mm_h, lightning, weather visibility_m) **once**, L0 tracers (smoke, ash, ember, heat, toxicity), and derived scalars: `visibility_m_combined`, `icing_risk` (profile-altitude band), `convective` (plume updraft + low-depth proxy), `storm_class` (enum, §3).
- **Consumers (read only):** `air_ops`, `ChunkSmokeField` pull, `ClimateVisualAggregate` (replace chunk scan), `CameraLocalWeatherSample`, `WeatherPrecipFrame`, gpu field uniforms / `RenderProjectionGraph` smoke node, PixelWorld layers, sim dump.
- **Not stored in saves.** Rebuilt from the two saved authorities after load.

### 2.3 Coupling through source terms (keeps one writer each)
```text
Fire / smoke ─(WeatherSourceTerms: heat_flux, aerosol, moisture)─▶ weather_field_step ─▶ WeatherField ─▶ AtmosphereFrame
WeatherField (rain, wind) ─(frame)─▶ advect_atmosphere_field (wind + washout) ─▶ AtmosphereClipmapStack
WeatherField (rain, wind) ─(ChunkWeather, already)─▶ fire overlay (suppress / spread)
```
- `WeatherSourceTerms` (new resource, coarse 1 km grid matching `WeatherField.dims`) is written by **one** system that aggregates L0 `heat_distortion`/`smoke`/`ember` into weather cells each weather step; `weather_field_step` reads it inside `solve` (extra humidity/cloud lift, smoke shade on temp). Derived from saved L0 → no extra save state.
- Rain scavenging is applied **once**, in the L0 advect writer, using `AtmosphereFrame.rain_mm_h`. `chunk_smoke_field_tick`'s own `rain_cleanse` becomes redundant and is removed in the retire wave (otherwise double washout).

### 2.4 Presentation rule
PixelWorld rain/clouds, `weather_vfx` precip, cloud sheets, and fire/smoke VFX **read** the frame (directly or through the existing frames they already own). They do not sample sim, do not own cloud/smoke state, and cannot write back. Visual-only noise (puff placement, flicker) stays view-local. A presentation layer may show *less* than the frame (zoom band), never different values.

### 2.5 Explicit REJECT
| Rejected | Why |
|:---|:---|
| Parallel smoke cache **and** parallel cloud cache with separate GPU uploads | Duplicate writers, duplicate bandwidth; the GPU field + one uniform path already exist. Cloud/precip means ride the existing `WeatherFireFieldUniforms`, not a new texture. |
| A second advected cloud field on the CPU (e.g. cloud lane in L0 advected independently of `WeatherField`) | Two cloud truths. Cloud stays in `WeatherField`; L0 only receives *tracers*. |
| Minecraft-style voxel atmosphere / voxel clouds or smoke | Sim is chunk/tile + clipmap; PixelWorld is a presentation view only. |
| `massing_columns` / `FootprintMatrix` as atmosphere or plume geometry | Construction massing is authored infrastructure, not a medium. |
| Hanabi/particles as weather or smoke authority | Already forbidden by WSS-PLAN-004. |
| Test-shaped production code (`*_green`, `*_LIVE_JSON`, witness resources) | AGENTS.md; use dump counters. |

---

## 3. Pattern taxonomy and air-threat coupling

`storm_class` is **derived** in the frame (classifier over frame scalars, plus pressure-system data for hurricanes). It is never a second authority.

| Class | Detection (frame / field) | Fire/smoke coupling | Air threats via existing `air_ops` | New |
|:---|:---|:---|:---|:---|
| **Developing storm** | cloud rising over threshold, `low_depth` high, humidity > ~0.7, rain small | Humid air + cloud shade suppress ignition (ChunkWeather already); smoke aerosol lifts cloud | `visibility_m` ↓ → grounding Visibility; headwind ↑ → ground speed ↓ | combined visibility; icing band at cruise altitude |
| **Thunderstorm** | `lightning` > threshold, cloud ≈ 1, rain ≥ moderate | Lightning is a natural ignition input (event bus later); rain washes smoke (L0) and suppresses fire | Grounding Lightning; `link_quality` (radio/lightning) and PassiveRf already degrade | turbulence/gust margin; icing peak |
| **Hurricane-class** | sustained deep low (needs vortex term on `PressureSystem`), wind ≥ class threshold over a large area, rain band | Extreme wind spreads fire/embers (existing wind term); rain damps unless heat flux dominates | Wind/Crosswind grounding for all profiles; missile drift; ground speed clamp | `PressureSystem` vortex extension, save schema bump (separate wave) |
| **Firestorm plume** | contiguous L0 cells with `heat_distortion` + `smoke` + `ember` over threshold and area (> N cells) | Heat → buoyant column (`plume_updraft_mps`) → moisture/aerosol source term → **pyrocumulus** cloud → local rain/lightning → partial washout/suppression | Updraft/turbulence hazard for rotors/drones near column; smoke vis loss (grounding + EO/IR sensor); icing in pyroCu anvil if altitude band cold | plume aggregate (derived), source terms |

**Icing (new, honest scope):** the weather model has no altitude. `icing_risk` = f(temp at profile altitude, cloud/humidity liquid-water proxy). Temperature at altitude = ground `temp_c` − `lapse_rate_c_per_km` × altitude (the solver already uses this lapse). Each `AirOpsProfile` gains `max_icing` and `ice_alt_m` (RON schema bump, defaults preserve behavior). Grounding gains `GroundingReason::Icing`. No airframe entities are assumed.

**Visibility:** `visibility_m_combined = weather visibility_m / (1 + k_smoke·smoke + k_ash·ash)` (k from `weather_tuning` RON). `evaluate_grounding` reads the combined value. This is a **behavior change** (smoke now grounds aircraft) and must be called out in the commit.

---

## 4. Implementation phases (DAG: W1 → W2 → W3 → W4 → W5; W6 after WIP lands; W7 last)

### W1 — Audit lock + counters skeleton (`@sim-steward`)
- **Goal:** confirm/resolve L0 dual advect writer (`advect_atmosphere_field` vs `sync_atmos_clipmap_witness_system`); classify `ChunkSmokeField` double write and L0 `fog` mirror (A/B/C/D via cleanup-completion-intelligence); add `revision` to `AtmosphereClipmapStack`; add empty `atmosphere` block to the dump with counters at 0.
- **Files:** `src/substrate/atmosphere/mod.rs`, `src/substrate/mod.rs`, `src/systems/atmosphere/diagnostics.rs`, `src/dev/sim_run_dump_weather.rs` (or new `sim_run_dump_atmosphere.rs`), `tools/sim_review/score_dump.py`.
- **Authority owner:** ⊚AtmosphereClipmapStack ═▶ single advect writer.
- **Rollback trigger:** removing the witness-system fold changes `clipmap_l0_smoke_max` in existing tests → keep fold, drop only the advect call.
- **Accept:** `cargo check`; `cargo test -p proc_A_dine01 --lib atmosphere::`; dump has `atmosphere.l0_advect_writers == 1`.

**W1 status — DONE 2026-11-15 (`@sim-steward`).** Rollback trigger did not fire (`clipmap_l0_smoke_max` only depends on the seed fold, which is kept).

*Answer to §8 Q1:* `sync_atmos_clipmap_witness_system` advect was a **leftover duplicate (class A_obsolete)**, not a headless/proof path: it called `advect_l0_preserving_mass` (blend of linear-index neighbours `i-1,i`; no wind, no `dt`, not gated by `should_tick`, so it ran while paused, and unordered vs `WindAdvect`). Advect call removed; the smoke-seed fold stays. `advect_l0_preserving_mass` is now test-only → delete in W7.

**L0 writer matrix** (`AtmosphereClipmapStack`, level 0):

| System | Set | Op | Class / decision |
|:---|:---|:---|:---|
| `atmosphere_field_fill_from_chunks` | FieldFill | max-blend 6 lanes from ChunkWeather/Ecology/SurfaceFire | authority — preserve |
| `atmosphere_field_blend_fire_overlay_sources` | FieldFill | max-blend from ChunkFireOverlay **and ChunkSmokeField** | authority — see feedback loop below (W4) |
| `advect_atmosphere_field` | WindAdvect | semi-Lagrangian advect + decay, all lanes | **sole advect writer** (`L0_ADVECT_WRITER_WIND_ADVECT`) |
| `contamination_tick_system` (substrate) | none (after `AdvanceSimTick`) | max-blend slab airborne → cell (0) smoke | B_transitional seed → W7: move into FieldFill |
| `sync_atmos_clipmap_witness_system` (substrate) | none (after `AdvanceSimTick`) | max-blend smoke seed → cell (0); ~~advect~~ removed | B_transitional seed fold → W7 merge with contamination seed |

Advect writers on L0 before: **2** · after: **1**. The two seed writers are commutative max-blends but sit outside `AtmospherePipelineSet` (ordering ambiguity vs advect; benign today, fix W7).

**`ChunkSmokeField` double write — class B_transitional, decision `refactor` (W4/W7), not a same-frame race.** `chunk_smoke_field_tick` (`ChunkEnvironmentSet::Fire`; emit from fire heat, decay, `rain_cleanse`) and `chunk_smoke_field_pull_from_advected_atmosphere` (`WindAdvect`; 0.16 blend toward L0) are ordered by set (fire → atmosphere, see `pipeline::atmosphere_pipeline_runs_after_chunk_fire_in_order`). Hazards that remain: (1) **feedback loop** — FieldFill blend reads `ChunkSmokeField` back into L0 max-blend, which the pull then reads again, so chunk smoke can sustain L0 against its decay; (2) **washout ownership** — only the chunk tick scavenges rain today; adding L0 washout (W4) double-counts unless `rain_cleanse` is dropped. W4 picks the single washout owner; W7 makes `ChunkSmokeField` a read-only projection of the frame/L0.

**L0 `fog` lane — class B_transitional, decision `refactor` (W7):** written by FieldFill as `ChunkWeather.fog_density + rain×0.2` (a mirror of `WeatherField`), then max-blended and advected with decay 0.99 → stale mirror. Alias to `AtmosphereFrame` / drop the lane in W7.

**Dump counters shipped (scaffolding, no verdicts):** `atmosphere` block from `src/dev/sim_run_dump_atmosphere.rs`. Wired: `l0_advect_writers` (distinct advect writers observed via `AtmosphereClipmapStack::advect_writer_mask`), `l0_revision` (`AtmosphereClipmapStack::revision`, bumped by every L0 writer), `l0_smoke_max`, `l0_smoke_mass`, `advect_runs`, `field_fill_runs`. The remaining §6 keys are emitted as `0` and named in `atmosphere.stubs` (0 = not wired yet; each wave removes its keys from `ATMOSPHERE_DUMP_STUBS`). Matrix row `atmosphere.l0_advect_single_writer` (`l0_advect_writers ≤ 1`; reads `1` once the weather field has solved).

**Residual → W4:** ~~pick washout owner; break the L0⇄ChunkSmokeField feedback~~ **done (W4):** L0 washout sole owner; ChunkSmoke→L0 blend removed. **Residual → W7:** ~~delete `advect_l0_preserving_mass` + `L0_ADVECT_WRITER_RETIRED_WITNESS`; fold seed writers into FieldFill; fog lane alias; `ChunkSmokeField` second writer; `clipmap_advect_wired`/`weather_runbook_phase2_green`~~ **done (W7).**

### W2 — `AtmosphereFrame` + consumers migrate (`@coder`)
- **Goal:** build the frame; move `ClimateVisualAggregate`, `CameraLocalWeatherSample`, and `air_ops` smoke01 (per-tile, not centre proxy) onto it; add `visibility_m_combined`.
- **Files:** new `src/systems/atmosphere/frame.rs`; `pipeline.rs` (add `Frame` set); `visual_extract.rs`; `weather/camera_local.rs`; `weather/air_ops.rs`; `weather/mod.rs`; `systems/atmosphere/mod.rs`.
- **Do not touch:** `render/weather_vfx/extract.rs`, `cloud_sheets.rs`, `render/pixel_world/*`, `weather/overlay_bake.rs` (WIP, see conflicts).
- **Accept:** `cargo test -p proc_A_dine01 --lib atmosphere::frame`; frame determinism test (same seed → same frame hash); dump: `frame_builds > 0`, `legacy_weather_direct_reads == 0` for migrated consumers, `vis_smoke_delta_m > 0` in fire fixture.

**W2 status — DONE 2026-11-15 (`@coder`).** Frame key = `(WeatherField.revision, AtmosphereClipmapStack.revision, SimTimeMicros)` (no ECS change ticks). Consumers: climate aggregate + camera-local + air_ops status smoke via frame; `visibility_m_combined` on every tile; dump wires `frame_builds` / `legacy_weather_direct_reads` / `vis_*`. Sense-field still uses transitional ChunkSmoke/L0 helpers (pre-Frame schedule) → W3/W7.

### W3 — Air threats: icing, combined visibility, storm classifier (`@coder`)
- **Files:** `weather/air_ops.rs`, `assets/configs/weather/air_ops_v1.ron` (schema bump), new `systems/atmosphere/storm_class.rs`, weather tuning RON for `k_smoke`/`k_ash`/icing thresholds.
- **Accept:** lib tests `air_ops::` (icing grounds when cold+humid at `ice_alt_m`; calm clear stays clear; smoke lowers vis and can ground); dump: `icing_risk_max`, `icing_grounded_samples`, `storm_class_*` counts.

**W3 status — DONE 2026-11-15 (`@coder`).** `GroundingReason::Icing` + `evaluate_grounding_with` (combined vis); air_ops schema 3 (`max_icing`/`ice_alt_m`); weather_v1 schema 2 (`k_smoke`/`k_ash`/icing band/storm thresholds); `storm_class.rs` Developing/Thunder live (Hurricane/Firestorm Clear until W5/W4); dump un-stubs icing_* + storm_developing/thunder_cells.

### W4 — Fire↔cloud coupling (`@coder`; `@sim-steward` reviews one-writer)
- **Goal:** `WeatherSourceTerms` writer + solver read; rain washout once in L0 advect; plume aggregate; remove `rain_cleanse` double count in `chunk_smoke_field_tick` (or keep as the only one and drop L0 — pick one in review).
- **Files:** `weather/field.rs` (solve reads terms), `weather/systems.rs`, new `systems/atmosphere/coupling_terms.rs`, `substrate/atmosphere/clipmap_advect.rs`, `systems/fire/chunk_smoke_field.rs`; `ember_physics::plume_updraft_mps` reused, not duplicated.
- **Accept:** tests: same seed + fire fixture → cloud over fire cells > no-fire baseline; rain over plume reduces L0 smoke mass; weather solve deterministic with terms; dump: `pyro_cloud_delta > 0`, `pyro_moisture_injected > 0`, `firestorm_plumes ≥ 1` (fixture), `rain_washout_mass > 0`, `smoke_cloud_overlap_cells > 0`.

**W4 status — DONE 2026-11-15 (`@coder`).** Washout owner = **L0 advect** (`apply_l0_rain_washout`); `chunk_smoke_field_tick` `rain_cleanse` removed. `WeatherSourceTerms` sole writer `build_weather_source_terms` (Frame); `weather_field_step` reads terms via `advance_with_terms`. ChunkSmoke→L0 feedback blend dropped in FieldFill. Dump un-stubs pyro_*/firestorm_plumes/rain_washout_mass/smoke_cloud_overlap_cells/plume_updraft_peak_mps.

### W5 — Hurricane-class (`@coder`)
- **Goal:** vortex term on `PressureSystem`, tuning preset, `WEATHER_FIELD_SAVE_SCHEMA` 2 with load migration from 1.
- **Files:** `weather/field.rs`, `weather/tuning.rs`, tuning RON, `io/save/weather_overlay.rs`.
- **Accept:** save v1 loads; preset spawns a vortex; `max_wind_mps` clamp respected; dump `hurricane_class_cells > 0` under preset, 0 otherwise.

**W5 status — DONE 2026-11-15 (`@coder`).** `PressureSystem.vortex_mps` + Rankine tangential wind in solve (clamped by `max_wind_mps`); `WeatherPresetId::HurricaneClass` + RON `vortex_mps`; save schema 2 with v1→v2 migrate (`vortex_mps` default 0); storm classifier HurricaneClass via vortex influence; dump un-stubs `hurricane_class_cells`.

### W6 — GPU single path + presentation (`@coder`; **blocked until PlayQuad/PixelWorld WIP lands**)
- **Goal:** `gpu_field_bridge` and `RenderProjectionGraph` smoke node read the frame; cloud/precip means through existing `WeatherFireFieldUniforms`; `WeatherPrecipFrame`/PixelWorld clouds read frame-derived samples; no new texture, no second upload.
- **Files:** `systems/atmosphere/gpu_field_bridge.rs`, `render/pipelines/gpu_weather_fire_field.rs`, `render/weather_vfx/*`, `render/pixel_world/{clouds,rain}.rs`.
- **Accept:** dump: `gpu_field_upload_paths == 1`, `presentation_frame_revision == sim_frame_revision`; no sim resource written from render (grep gate in review).

**W6 status — DONE 2026-11-15 (`@coder`; dual X4).** Bridge fills `WeatherFireFieldUniforms` from `AtmosphereFrame` weather means + projection fire/smoke; smoke node prefers frame tracers; precip streaks sample frame (not `WeatherField`); PixelWorld clouds/rain remain RO presentation on frame-derived samples; dump wires `gpu_field_upload_paths` / `presentation_frame_revision` (stubs empty). Residual → W7: retire ChunkSmoke extract path / ClimateVisualAggregate ecology-only scan / L0 fog mirror.

### W7 — Retire duplicates (`@sim-steward`)
- **Goal:** delete `ClimateVisualAggregate` chunk scan, L0 `fog` mirror lane (or alias to frame), `ChunkSmokeField` second writer; classify before delete.
- **Accept:** `cargo test --lib`; dump `second_writer_violations == 0` from reviewer script (not production).

**W7 status — DONE 2026-11-15 (`@sim-steward`).** Classifications + actions:
| Item | Class | Decision |
|:---|:---|:---|
| `advect_l0_preserving_mass` + `L0_ADVECT_WRITER_RETIRED_WITNESS` | A_obsolete | **deleted** |
| `contamination_tick` / witness L0 seed folds | B_transitional | **completion:** sole seed in FieldFill `atmosphere_field_seed_slab_smoke`; witness RO; contamination unscheduled |
| L0 fog mirror (`ChunkWeather`→L0 fill+advect) | B_transitional | **completion:** stop fill/advect; weather fog via `AtmosphereFrame`; tactical LOS smoke-only on L0 |
| `ChunkSmokeField` tick + L0 pull | B_transitional | **completion:** sole writer `project_chunk_smoke_from_atmosphere_frame` (Frame set) |
| `SimChunkSmokeVisualExtract` via ChunkSmoke | B_transitional | **completion:** extract from `AtmosphereFrame` tiles (slab cutover kept) |
| ClimateVisualAggregate ecology scan | B_transitional | **completion:** ecology means owned by `publish_ecology_visual_snapshot`; climate copies snapshot |
| `weather_runbook_phase2_green` | A_obsolete (test-shaped) | **renamed** → `weather_runbook_phase2_ready` (factual AND, no `*_green`) |
Dump wires `second_writer_violations` (`l0_advect_writers.saturating_sub(1)`); matrix row `atmosphere.second_writer_violations_clear`.

---

## 5. ECS schedule plan
```text
ChunkEnvironmentSet::Weather   weather_field_step (reads WeatherSourceTerms) → project_chunk_weather → polling → air_ops
ChunkEnvironmentSet::Fire      fire overlay / surface fire / chunk smoke tick
AtmospherePipelineSet::FieldFill      fire → L0 tracers
AtmospherePipelineSet::WindAdvect     L0 advect (wind + rain washout from previous frame) [single writer]
AtmospherePipelineSet::Frame  (new)   build_atmosphere_frame ; build WeatherSourceTerms for next weather step
AtmospherePipelineSet::VisualExtract  ClimateVisualAggregate / smoke extract from frame
→ gpu field bridge → precip/cloud frames → PixelWorld layers
```
Frame ordering: weather terms computed this frame apply to the **next** weather solve (one-step lag; no mid-frame authority swap).

## 6. Diagnostics required (dump counters, no verdicts)
`atmosphere.*`: `frame_revision`, `frame_builds`, `legacy_weather_direct_reads`, `l0_advect_writers`, `l0_smoke_max`, `l0_smoke_mass`, `smoke_cloud_overlap_cells`, `vis_weather_min_m`, `vis_combined_min_m`, `vis_smoke_delta_m`, `icing_risk_max`, `icing_grounded_samples`, `storm_developing_cells`, `storm_thunder_cells`, `hurricane_class_cells`, `firestorm_plumes`, `plume_updraft_peak_mps`, `pyro_moisture_injected`, `pyro_cloud_delta`, `rain_washout_mass`, `gpu_field_upload_paths`, `presentation_frame_revision`. Scoring in `tools/sim_review/score_dump.py`.

## 7. Edge cases
Chunks outside L0 128² (no tile) → frame returns weather-only sample; L0 focus shift → frame rebuild on stack revision; weather solve interval vs per-tick L0 (use `prev/next` weather at sim time, tracers per tick); save/load (frame and source terms rebuilt, not saved); replay determinism (terms derived from L0 at fixed ticks); minimap/strategic reads L2/L3, not L0; zero-fire worlds (terms are zero, behavior identical to today except combined visibility); multi-window (frame is global, views sample).

## 8. Open questions
1. Is the `sync_atmos_clipmap_witness_system` advect intentional (headless/proof path) or a leftover duplicate? (W1)
2. Aircraft altitude: profile-fixed band (planned) vs future airframe entities.
3. Lightning-as-ignition: event bus now or defer.
4. Hurricane save migration: **resolved W5 — v1→v2 loader** (`migrate_weather_field_save`; `vortex_mps` defaults to 0).
5. `rain_cleanse` ownership: **resolved W4 — L0 advect** (`apply_l0_rain_washout`); chunk tick no longer cleanses.

## Changelog
| Version | Date | Notes |
|:---|:---|:---|
| 1.0.0 | 2026-11-15 | Signed. Audit, `AtmosphereFrame`, source-term coupling, taxonomy, W1–W7. |

## Agent continuity

W1–W7 agents obey [`plan_ref_books_upgrade_v3.md`](plan_ref_books_upgrade_v3.md) §0.1 when that program is active:
- **Tripwire:** response without Accept paste / `BLOCKED:`+next slice / hard-gate cite → incomplete; parent RESUMEs — no user ping.
- No lazy/confused stop; on file conflict cut next free atmosphere wave (W2→W7 order).
- Do not freeze on “voxels deferred” — PixelWorld presentation is in scope (W6).
- Keep going until W1–W5 Accept green before W6 GPU.

| 1.0.1 | 2026-11-15 | W1 done: witness-system advect removed (single L0 advect writer), `AtmosphereClipmapStack.revision`, dump `atmosphere` block, writer matrix + classifications. |
| 1.0.2 | 2026-11-15 | Continuity tripwire aligned to v3 §0.1 response-stop. |
| 1.0.3 | 2026-11-15 | W2 done: AtmosphereFrame + consumer migration, revision-keyed rebuild, dump counters. |
| 1.0.4 | 2026-11-15 | W3 done: icing grounding, storm Developing/Thunder, dump icing_*/storm_* (see W3 status). |
| 1.0.5 | 2026-11-15 | W4 done: WeatherSourceTerms + solve coupling, L0 rain washout sole owner, plume aggregate, dump pyro_*/washout/overlap. |
| 1.0.6 | 2026-11-15 | W5 done: PressureSystem vortex, HurricaneClass preset, save schema 2+v1 migrate, dump hurricane_class_cells. |
| 1.0.7 | 2026-11-15 | W6 done (+ v3 X4): frame-fed GPU field + smoke node + precip/PixelWorld; dump gpu_field_upload_paths / presentation_frame_revision. |
| 1.0.8 | 2026-11-15 | W7 done: retire duplicate writers (fog lane, ChunkSmoke dual write, seed folds, ecology scan, obsolete advect/green witnesses); dump second_writer_violations. |
