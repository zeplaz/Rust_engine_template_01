# PLAN-CLOSE-VOXEL-WATER-v1 — recover voxel ground, then water

| Field | Value |
|------|------|
| **Status** | Lane V DONE (2026-11-19); lane W in progress |
| **Context** | Cursor agents halted mid-wave; all their work is in commit `09cda60b` ("mid work run problems", 276 files). Crate compiles (`cargo check` clean). |
| **Order** | **V (voxel ground) first, then W (water).** W depends on V: voxel water reads the same `PixelWorldTerrainField`. Do not run them in parallel — both touch `src/render/pixel_world/`. |
| **Out of scope** | Destruction/undo (lane C), dam ops UI, combat matrix reds. Leave for a later plan. |
| **Parent plans** | `plan_final_gpu_presentation_v1.md`, `plan_dam_flood_benchmark_v1.md` (DF-3), `plan_destroy_t2_mask_v1.md`, notes `playtest_dam_flood_notes_v1.md` |

## Symptoms (operator)

- Voxel 3D view: ground vanishes, flickers, half-map holes.
- Water broken (both views).

## Rules for executors

- One lane at a time, one agent (`@coder`) per lane. Commit per slice: `[fix] CLOSE-V1 …`.
- Follow `AGENTS.md`: ≤ 8 params per system, one domain per system, no debug panels, no test-shaped production code.
- **Repro before fix, eye-check after fix.** Each slice ends with the verify step listed — not just `cargo test`.
- If a hypothesis below proves wrong, stop and record the real cause in this file before changing code.
- Don't restore deleted code wholesale; a fallback must be one switch, not a parallel path (no duplicate extraction).

## Baseline (do once, before V1)

```powershell
cargo test -p proc_A_dine01 --lib pixel_world::
cargo test -p proc_A_dine01 --lib water::
python tools/sim_review/playcheck.py
```

Save the matrix + one voxel screenshot (`v` key) + one flat screenshot as "before" in `debug_runs/close_voxel_water/`. Note which tests fail now — that is the baseline, not a regression.

---

## Lane V — voxel ground (do first)

### Lead hypothesis (from code read, high confidence)

`PixelWorldTerrainField::revision` (`src/render/pixel_world/field.rs` ~L126–163) mixes a **water** fingerprint (`surface_stamp`: wet count, pool stage, outlet/breach/overtop flows) into the **terrain** stamp. Every time the reservoir changes enough, the revision bumps, and `sync_pixel_world_terrain` (`src/render/pixel_world/terrain.rs` ~L295) **despawns every ground chunk** and rebuilds them at `CHUNKS_PER_FRAME = 48`. During fill/flood this repeats continuously → ground vanishes and flickers. The earlier "fix" (coarser quanta, 12→48 chunks/frame) only reduced how often it happens.

A second, smaller cause: DESTROY-T2b priority remesh (same file ~L307) despawns chunks before their replacement is built → one-frame holes.

The new GPU frustum cull (`gpu_frustum_cull.rs`) is **fire-only** — rule it out quickly, don't chase it first.

### V0 — confirm (read-only, ~15 min)

- Log once per bump of `field.revision` with the stamp parts that changed (temporary `info!`, removed before commit), or count `revision` in the sim dump.
- Run DamFlood in voxel view: if revision bumps line up with flicker, hypothesis confirmed.
- Quick rule-outs: `TERRAIN_GPU_BAKE_SPIKE=0` (bake not the cause if flicker persists); camera orbit with MMB (frustum/near-plane issue if ground clips by angle, not by time).

### V1 — split terrain revision from water revision

- `PixelWorldTerrainField` keeps a **terrain-only** stamp/revision (grid + static depth + dense terrain).
- Water state that the voxel view needs goes in its own revision (e.g. `water_revision`), read only by water consumers (`water_gpu.rs` and the water half of `build_terrain_chunk`).
- If `build_terrain_chunk` still builds a water mesh (`(ground, _water)` — it's currently discarded), remove the water half so ground meshing never depends on water.
- **Done when:** filling/draining the reservoir causes zero ground chunk despawns (assert in a unit test: changing surface water does not bump the terrain revision).

### V2 — swap-on-ready remesh (no holes)

- In `sync_pixel_world_terrain`: never despawn an old chunk until its replacement exists. Build into new entities; when one is spawned, despawn the old entity for the same `(cx, cy)`. Same for the T2b priority path.
- Keep `CHUNKS_PER_FRAME`; it only affects how fast the rebuild finishes, not visibility.
- **Done when:** a full revision rebuild never shows missing ground (eye check + unit test: during a rebuild, every coord always has ≥ 1 live chunk).

### V3 — verify lane V

- `cargo test -p proc_A_dine01 --lib pixel_world::`
- `playcheck.py`, then voxel view through fill → strike → breach: no vanishing/flicker; orbit 360° with MMB.
- Debug perf avg/p95 not worse than the baseline.
- Update `playtest_dam_flood_notes_v1.md` item 2 with the real root cause; mark V rows here DONE.

---

## Lane W — water (after V lands)

### State

DF-3 removed the CPU water mesh and CPU raster water paint. Both views now draw from one GPU image, `SurfaceWaterGpuField` (`src/systems/water/gpu_field.rs`):
- flat: `src/render/pipelines/gpu_water_field_draw.rs` + `assets/shaders/water/water_field.wgsl`
- voxel: `src/render/pixel_world/water_gpu.rs` + `assets/shaders/water/water_field_3d.wgsl`

The GPU path was never eye-accepted ("capture Accept open" in the DF plan).

### W0 — find where it breaks (read-only, ~30 min)

Check each link in order; the first one that fails is the bug:

1. **Sim:** `sim_run_dump.json` water section — is ROI depth non-zero, is `wet_texels > 0`? (If not, it's a solver/scenario bug, not render — stop and report.)
2. **Pack/upload:** `pack_composite_water_field_rgba8` returns non-empty; `book.wired` stays true. Watch for `wired` flapping false (the empty-pack branch) — `water_gpu.rs` hides the mesh whenever `wired && wet_texels > 0` fails.
3. **Re-upload:** the reused-image path sets `img.data` + `asset_usage`. Check the render world really re-uploads (VFX_CAPTURE=1 PNG of the field, or `RTT_SPRITE_TRACE=1`). Bevy 0.20 may need `images.get_mut` to fire `Modified` — confirm.
4. **Draw:** is the entity spawned, on the right `RenderLayers`, with a valid bind group? Check pipeline errors in the log (`wgpu` validation), texture format vs. shader sample type (RGBA8 Unorm vs. `texture_2d<f32>`).
5. **Shader:** decode of depth from RGBA8 matches the packer (scale/offset); vertex displacement in `water_field_3d.wgsl` uses the same `height_scale()` as the ground.

Record the failing link here before coding.

### W1 — fix the failing link

Bounded fix at that link only. Typical shapes: keep `wired` sticky when the ROI is temporarily empty; fix the re-upload trigger; fix format/sample-type mismatch; fix the depth decode.

### W2 — fallback switch (only if W1 can't land in one slice)

Restore visible water behind **one** switch rather than leaving it blank: e.g. env `WATER_GPU_FIELD=0` → previous CPU MeshBuf path from `09cda60b^` for the voxel view only. Register it as a `ScaffoldContract` (`src/gui/representation_governance.rs`) with an exit row in this plan. Do not keep both paths drawing at once.

### W3 — verify lane W

- `cargo test -p proc_A_dine01 --lib water::`
- `playcheck.py`; eye-check both views (flat, `v`): reservoir visible at rest, rises during fill, spillway jets, breach wave downstream; zoom out (water LOD `water_grid_step_for_zoom`).
- Matrix: `s2_downstream_ratio`, `s7_wave` not worse than the baseline.
- Close DF-3 "capture Accept" in `plan_dam_flood_benchmark_v1.md`; update notes item 3.

---

## Handoff brief (paste to the executing agent)

> Execute `src/dev/plan_close_voxel_water_v1.md` lane **V** only (V0→V3), on master, one commit per slice. Confirm the hypothesis in V0 before editing; if it's wrong, write the real cause into the plan and stop. Finish with the V3 checks and report results (tests, playcheck matrix, before/after screenshots in `debug_runs/close_voxel_water/`). Don't touch water render files beyond removing the water dependency from the terrain revision.

Then, after V is accepted:

> Execute lane **W** (W0→W3) of the same plan. Find the failing link in W0 and record it before coding. Use W2 only if W1 can't land in one slice.

## Status

| Row | State |
|-----|-------|
| Baseline | DONE — 42/42 pixel_world+water lib tests green |
| V0 confirm | DONE — worse than guessed: `surface.is_changed()` in `dense_changed` rebuilt the field every sim tick → every ground chunk respawned per tick |
| V1 split revisions | DONE — `ground_revision` (ground only); per-tick trigger removed; tests `close_v1_*` |
| V1b WESL port (found in V3) | DONE — **real cause of "no ground"**: Bevy 0.20 passes `.wgsl` raw to wgpu, so `#import` failed to parse and the ground + voxel water materials never compiled. Ported `ground_mask` / `water_field_3d` to `.wesl` (`import …;`, `@if`, material group 3), ground color input → location 5. Guard test `close_v1b_no_naga_oil_directives_in_wgsl` |
| V2 swap-on-ready | DONE — old chunk despawns in the same flush its rebuild spawns; T2b priority no longer pre-despawns |
| V3 verify | DONE — voxel capture shows ground (dam scene, frame 600). Flicker over time not checkable from a still: operator eye check pending |
| W0 locate | DONE — Sonnet trace; suspects: voxel water flat Aabb (frustum cull), flat-view draw order, wet_texels threshold, bed range drift after V1 |
| W1 fix | OPEN |
| W2 fallback | CONDITIONAL |
| W3 verify | OPEN |

## Round 2 findings (2026-11-19, operator playtest + traces)

| Row | State |
|-----|-------|
| W1 voxel water render fixes | DONE `77e685b4` — no frustum cull on displaced grid; bed range matches texture; wet count matches shader discard |
| W2a solver mass balance | IN PROGRESS (uncommitted) — inflow only into wet north cells (was smeared over whole edge → fed banks); south edge free-drains (was dam-outlet sink ≈40 vs 220 m³/s inflow → runaway flood); barrier east-branch `continue` skipped north flux; conservation uses applied boundary volume |
| W2b buildings stretch with flood | IN PROGRESS — `build_site` top used `ground.max(water)`; now ground only |
| G1 grey ground | IN PROGRESS — A/B: `TERRAIN_GPU_BAKE_SPIKE=0` textures whole map; bake only fills part of the world image. Bake now **opt-in**. Fix the bake coverage later, then re-default ON |
| T1 no live trees (any view) | OPEN — sim has 100k live stand cells, 0 dead; lost in render. Live draw prob = `mean_density·0.9`, burnt ≈ independent → density≈0 suspected (`landscape_grammar_lg2.rs:445`, `vegetation_visual_extract.rs:188`). Next: runtime check of `row.modifiers.mean_density` |
| D1 dam missing | OPEN — scene seeds no barrier (`scene_seed.rs:71`); scenario step `InstallBarrier` expected to add it. Verify the step fires and the dam site/mesh spawns |
| **W3 single water system** | OPEN — operator requirement: **one** water, blue + semi-transparent. Today: static river (`WorldStaticWaterDepth`, dark, never moves) and live ROI solver (light sheet) are two fluids; composite pack lets ROI override static (`gpu_field.rs:202-218`). Target: seed the live solver from static depth inside the ROI (one fluid), static only outside the ROI with matched look/edge fade; one material colour ramp shared by flat + voxel shaders |

## Round 3 — plan review of this session's changes (2026-11-19, operator asked: no hacks)

Checked against `plan_dam_flood_benchmark_v1.md` (§0 no-cheat, §4 authority, DF-2/4/8/9, §8 rejects),
`plan_water_depth_truth_v1.md`, `plan_disaster_proxy_v1.md`, `plan_final_gpu_presentation_v1.md`.

### Keep (plan-aligned root-cause fixes)

| Change | Plan basis |
|:---|:---|
| WESL port, `ground_revision`, swap-on-ready (`fe3c6f27`) | FINAL-GPU: ground columns textured from the bake/raster image; Bevy 0.20 contract |
| Voxel water no-cull / bed encode / wet threshold (`77e685b4`) | FINAL-GPU: water only via the GPU field shader |
| Solver: barrier east-branch `continue` skipped z-flux | DF-2 correctness (barrier must leak/overtop along the river) |
| Solver: inflow into the wet channel; open (free-drain) downstream boundary; conservation from applied boundary volume | DF-2 "inflow/outflow boundaries", §4 "mass conservation is a hard invariant" |
| ROI starts from `WorldStaticWaterDepth`; no synthetic carve over a covered DEM | WATER-DEPTH-TRUTH "one physical water: bed+depth"; W4 authored bed+depth corridor is already in the world DEM |
| `build_site` stands on ground, not water top | §4 render reads snapshots; flood must not reshape buildings |

### Revert / redo properly

| Change | Problem | Plan-correct fix |
|:---|:---|:---|
| GPU terrain bake made opt-in | **Violates FINAL-GPU** (Simulation `CpuRaster` forbidden; bake default ON) | Revert. Fix the real bug: bake only fills part of the world image (rest grey) |
| `ELEVATION_NORM_METRES = 672` (new constant) | Third vertical contract: voxel implies 672 m, weather RON says `elevation_range_m: 1200` | **Operator decision:** one vertical scale for sim water bed, voxel display and weather lapse |
| Flood proxy skips `Dam`/`WaterPlant` archetypes | Hard-coded archetype list | DF-8 + §8: dam damage is the `DamBody` voxel path; whole-site demolish is **rejected** as the dam damage model. Gate the flood-foundation proxy on the same rule as placement (`building_may_occupy_water` / hydraulic structure), not an archetype list |
| Dam erosion: dropped outlet/spillway from `flow_proxy` | Direction right, model still wrong | DF-8: erosion ∝ (flow velocity − critical velocity) by `DamKind` at the breach/overtop face; controlled outlet works don't erode |
| `try_insert` in `presentation/mod.rs` | Hides the symptom; cause was the proxy despawning the dam | Revert once the proxy is fixed |

### Found cheats in the scene (pre-existing, §0 violations)

- `force_dam_flood_sites_operational_system` (`scenario/dam_flood/mod.rs:260`) skips the construction funnel ("clears the construction material stall"). §0: forbidden to "skip construction/damage funnels". S1 Build should commit + build the dam through the real funnel.
- The dam site is committed at seed time (`engine/test_harness.rs:~1694`), i.e. the dam exists in S0 River, while the script's `InstallBarrier` is a separate crest write. Barrier crest should come from `DamBody` (DF-4.4 "crest profile per tile → water barrier"), not a parallel script step.
- `dam_integrity_mean` is a book default (1.0), not aggregated from `DamBody` — matrix row can't fail.
- Pre-existing failing test: `gui::hud::dam_ops_card::tests::balance_text_names_inflow_release_and_held` ("Presets").

## Status table (2026-11-19, end of session) — evidence = dam scene playcheck captures

| Area | Item | State | Commit / evidence |
|:---|:---|:---|:---|
| Voxel ground | Shaders compile on Bevy 0.20 (WESL) | DONE | `fe3c6f27` |
| Voxel ground | No per-tick full remesh; swap-on-ready | DONE | `fe3c6f27` |
| Heights | One vertical contract (`WorldGenParams::elevation_range_m`) | DONE | `124d55de` |
| Ground look | Bake covers whole world; one material resolution; sand bed under water | DONE (valley) | `7f7e3fb6`, `d644c2f6` |
| Ground look | Plateau still grey (family default material is grey) | **OPEN** | r7 capture |
| Water sim | One water (ROI seeded from static depth, real bed) | DONE | `124d55de` |
| Water sim | Mass conservation on real terrain (donor cap + limiter) | DONE — mass err ~1e-7 | `b4c405c0` |
| Water sim | S0 gauge balance < 0.02 | FLAKY — 0.0016 in one run, 0.21 in the long run | r6 / r7 |
| Water sim | Reservoir fill in test time (solver dt fixed 1 s; time skip doesn't speed water) | **OPEN** | |
| Water look | One blue semi-transparent ramp, flat + voxel | DONE — but river reads pale over sand | `b4c405c0` |
| Dam | Barrier = dam footprint; crest = height above bed | DONE | `b4c405c0`, `3a4434fe` |
| Dam | Body solid (no water inside); no erosion before crest | DONE | `a82c1495` |
| Dam | Erosion from real face velocity per DamKind | DONE | `3a4434fe` |
| Dam | Flood proxy never demolishes dams / water-legal sites | DONE | `82715e4f` |
| Dam | Built for real (S1 commit + logistics, no forced Operational) | PARTIAL — still UnderConstruction at tick 4208 | `5cc4bb65` |
| Dam scene | Strike failsafe / breach-on-impact / fake book values removed | DONE | `d9d0db62` |
| Dam scene | Strike → blast → DamBody → breach end to end | **UNVERIFIED** (script never reached S5: dam unbuilt) | |
| Vegetation | Live stands no longer all Grass | DONE (shrubs) | `2be652c8` |
| Vegetation | Young/old forest stages appear | **OPEN** | r6/r7: no codes 3/4 |
| Fire | Tree flicker near fire | DONE | `605776ab` |
| Fire | Voxel flames/embers/smoke layer | PARTIAL — layer landed, particle frame has 0 rows in voxel view | `605776ab` |
| Fire | "Burns slower" | EXPLAINED — timber/shrub burn longer than grass by config, spread speed unchanged | agent measurement |
| Tests | Pre-existing failures (also fail on clean HEAD) | OPEN | dam_ops_card "Presets"; scenario wave1/wave3 + ecs12 (CommitConstructionSiteEvent not registered in test app); ai_faction_builds_supplies; voxel_massing; 2 construction integration (fuel_breaks resource) |

## Status update (2026-11-19, later)

| Area | Item | State | Commit / evidence |
|:---|:---|:---|:---|
| Dam | Built for real: S1 commit → logistics → Operational | **DONE** | `5cc4bb65`, `a94c0ec8`, auto-supply `…S3` — capture: Operational, 7000 concrete delivered, script reached S5 |
| Dam | Crest holds reservoir (stage 302 m, head 56 m) | DONE | r12_late |
| Dam ops card | "Crest" shows max cell crest (bank height under dam ends, 578 m) — misleading | OPEN (display) | r12_late |
| Dam scene | Strike → blast → DamBody → breach | IN RUN: integrity 0.80, breach flow 49 m³/s after strike — not yet checked visually | r12_late |
| Fire | Particles in voxel view | DONE (seen in r10) — fire_heat overlay + 3D footprint | `164cbbe5` |
| Culling | Voxel trees + fire culling off by default, `PIXEL_WORLD_CULL` switch | DONE | `e296c3f9` |
| Vegetation | Mature start + forest floodplain | LANDED — forest codes (3/4) still not seen in dumps | `7ee2e280` |
| Ground look | Plateau grey with bake on (green with bake off) | **OPEN** — two bake fixes landed (`74f53b65` bind group, `196e912c` re-arm union), still grey | r12_late |

## Operator report (2026-11-19, late) — queued after grey bake + forest

| Id | Item | Notes |
|:---|:---|:---|
| OP-1 | Can't click / select the dam | Dam ops card says "click dam, then use keys"; picking may not hit the voxel dam footprint (site pick vs PixelWorld ground ray) |
| OP-2 | A black line appears somewhere in the water | Unlocated — likely voxel water grid seam (ROI edge / grid step LOD / bed texture edge) or barrier row |
| OP-3 | Water does not react correctly with the dam | Reservoir holds (stage 302 m, head 56 m) but visual: water vs dam footprint (crest per tile, displaced cells, flat grid over bank cells under dam ends) |
| OP-4 | Dam ops card "Crest" shows max cell crest (bank height) | Show the crest over the channel (span invert + height) |
| OP-5 | A stand-in for every tree stage, always visible, both views | Grass / shrub / young / old / burning / burnt / dead / regrowth each get a voxel mesh (3D) and a symbol (2D) with slight per-instance variation (species, rand); no stage may draw nothing at any LOD |
| OP-6 | Fire effects in sync | Ground burn tint, flame/ember particles, smoke and the burning tree stage must follow one burning state + one clock (today they look out of sync) |

## On deck (operator, 2026-11-19 late)

| Id | Item | Notes |
|:---|:---|:---|
| OP-7 | Road bridge at (152..162, 110) | Scene play-prop road + bridge (fixture `road_bridge_tile` (160,110)). Renders as a thin dark glitchy strip that follows the ground INTO the river (no deck height), not clickable, no truck drives across it |
| OP-8 | Missile leaves a yellow streak | Trail segments (`BallisticPresentFrame.trail_segments` drawn as columns in voxel units layer) persist / read as a streak |
| OP-9 | Freeze on missile impact | Hitch at impact — measure with HITCH_MS (blast execute / ignition / crater mask / remesh) |
| OP-10 | Extra water downstream of the dam | Following the river past the dam, more water appears at one point (ROI south edge 96 → static water beyond / composite pack seam) |
| OP-11 | Reservoir fills unevenly behind the dam | Uneven / wonky build-up upstream after the dam is built |
| OP-12 | S3 drown premise unmet | No forest cell submerged 14 days → no drowned/dead stands for S9; evaluate drowning from solver depth per forest cell |

## Todo (operator playtest 2026-11-19 night, on the uncommitted OP-10/11/12 water + valley rework)

Background agents stopped. Work in this order, one at a time, each verified in one dam playcheck capture.

| Id | Item | State |
|:---|:---|:---|
| T-1 | Big square in the water + water runs in a weird pattern (also before the dam) | open — suspect solver ROI square vs static water outside it; check against new valley profile |
| T-2 | Water rework (OP-10/11/12) uncommitted: valley terrain, level pool from depth, ghost south boundary, time-skip on sim clock | review after T-1; not committed |
| T-3 | Missile crater: craters set MASK_DAMAGED, ground shader only reacts to DESTROYED → no bowl / no scorch | open — shallow bowl + scorch for damaged tiles |
| T-4 | Land not rendering cleanly: voxel terrain outline hard to see, high ground reads as flat solid green | fix in progress — ground shader was unlit (bake albedo only, mesh normals + wall shade ignored); add face shading |
| T-5 | Trees stopped drawing (REGRESSION — worked after d99e89d7 OP-5) — floodplain is a bare dark-green rectangle | agent bisecting; prime suspect uncommitted ValleyProfile rework |
| T-9 | Fire broken / unfinished: S9 burn 0 (was 34%), OP-6 sync never visually confirmed mid-burn | likely same root as T-5 (no stands = no fuel); then verify flames/smoke/stage sync in a mid-burn capture |
| T-10 | Day/night: voxel sun on the tactical day clock (8 min sim day), lit ground + shadow maps | done in tree, verify in capture |
| T-11 | Dam removed (demolish funnel despawn) kept blocking water — crest stayed in the field | fixed: `release_barrier_of_removed_dams_system` (test `removed_dam_releases_its_barrier`) |
| T-12 | Partial dam damage: water must flow through the damaged columns, erosion must widen that notch, all visible (dam mesh per column) | done in tree (DW-3), verify in capture |
| T-13 | Missile explosion must deform land (crater bowl in terrain truth → ground mesh + water bed), scorch/fry visible | agent: terrain deform request drained by terrain owner |
| T-14 | Second missile test: strike into a random wooded area (crater + fire + tree damage) | agent, after T-13 |
| T-15 | Flood goes "around" the dam (geometry not closing) | re-check after the Lake-stamp fix (whole floodplain was Lake in matrices) |
| T-16 | Hitch: upd_control_relax up to 236 ms at frames 2400–2750 (OP-9 impact freeze itself not reproduced: no hitch >150 ms after frame 2900) | open |
| T-6 | OP-9 freeze on missile impact | open — HITCH_MS=150 capture through the strike |
| T-7 | Verify blue water look, short missile tail, bridge deck (OP-7/8) in a capture | open |
| T-8 | OP-7 rest: roads not clickable, trucks don't drive roads | on deck (larger feature) |

## Design: destruction changes the world (operator 2026-11-19 night)

Operator ask: a missile crater deforms the land and throws dirt into a rim; a destroyed building or
dam stops being a building and its voxels become rubble that is part of the world; dam debris
washes downstream and scatters; water collides with every changed shape (craters, rubble, damaged
dam columns, standing buildings); partial damage is local and visible (water through the broken
columns, erosion widening that notch).

One authority, request-driven (AGENTS.md: one writer per domain, request queues to the owner):

- **Ground delta (terrain owner).** A per-tile height delta in metres on top of the authored DEM,
  owned by the terrain domain and drained from a `GroundDeformRequest` queue. Producers: blast
  (crater bowl: dig; ejecta: raise a rim with the dug volume), demolish/collapse (site footprint ->
  rubble heap with the structure's volume x a bulking factor), dam columns (failed column ->
  rubble), water (sediment pick-up / deposit). Volume-conserving where it is a move (crater ->
  rim, rubble -> downstream).
- **Readers.** PixelWorld field ground (mesh + trees + pick), water solver bed (`bed_m`, so flow
  routes around/through the new shape), ecology (rubble/crater cells: no stand, burnt), tactical
  bake (rubble/dirt family tint). No reader writes back.
- **Structures in water.** Every standing site footprint is an obstacle to the water solver at its
  drawn height (dam = per-column crest = integrity x height; other buildings = their height);
  destroyed -> obstacle removed and the rubble heap enters through the ground delta.
- **Debris transport.** Water picks rubble/loose dirt from a cell when face velocity exceeds a
  threshold and deposits it where velocity drops (simple capacity model), through the same queue.

Phases:
| Id | Slice | Owner |
|:---|:---|:---|
| DW-1 | Ground delta layer + request queue; crater bowl + ejecta rim from blasts; field ground + water bed follow | agent |
| DW-2 | Site destroy -> rubble heap (footprint volume) through DW-1; building stops drawing | DONE in tree: `construction/rubble.rs` (sites + failed dam columns) |
| DW-3 | Dam per-column crest from integrity (water flows through the damaged notch, erosion widens it); dam mesh per column; failed column -> rubble | main — DONE in tree: `BarrierColumnRequest` + `set_barrier_column_fractions`, `publish_dam_column_crests_system`, erosion global leak removed, dam columns drawn per integrity (rubble stub). Tests `dw3_*`. Left: blast.rs partial-damage global leak (after DW-1 agent), failed column -> rubble heap via DW-1 |
| DW-4 | Standing buildings as water obstacles at their height | after DW-3 |
| DW-5 | Debris/sediment wash-down (velocity pick-up / deposit) | DONE in tree: `systems/water/sediment.rs` -> `GroundDeformOp::Transfer` |
| DW-6 | Second missile test into a wooded area (crater + rim + fire + tree damage) | DONE in tree: dam_flood_df1 `StrikeOrder(138,132)` after S9 |

### Integration with the existing destruction program (checked 2026-11-19 night)

DW is not a new program: it extends **PLAN-DESTROY-PRESENTATION-v1** (`plan_destruction_presentation_v1.md`, 4 tiers)
at **Tier 1 (sim truth)**. Mapping and corrections:

| Existing (ADOPTED) | DW relation |
|:---|:---|
| Tier 1 sim truth: construction funnel / `execute_demolish_at_tile` / `BlastCraterBook` sole writer / dam `column_integrity` | DW-1 ground delta is a new Tier-1 truth owned by terrain, fed only by these funnels (blast execute, demolish, dam columns). No new writer of sites. |
| Tier 2 DESTROY-T2 mask (`MASK_DESTROYED` same-frame collapse) | Keep for buildings vanishing same frame. Craters no longer use DESTROYED (pit to world floor); the bowl comes from DW-1 heights. `plan_destroy_t2_mask_v1.md` row "Blast crater -> MASK_DESTROYED" is stale. |
| Tier 3 catch-up remesh (`CHUNKS_PER_FRAME`, T2b priority) | DW-1 height changes ride the same remesh (ground_revision), priority chunks first. |
| Tier 4 DEBRIS-FX particle burst on demolish | Keep: the burst covers the moment; DW-2 rubble heap is what stays. DEFER "rigid-body debris -> particles + CPU rubble occupancy" is honoured: rubble is height in the ground delta, not bodies. |
| GPU-B `TerrainScorchField` (scorch authority, fed by `ChunkFireOverlay`) | **Correction:** the voxel ground shader's mask-DAMAGED scorch tint (this session) is a second scorch path. Replace it by sampling `TerrainScorchField` in the ground shader, so fire burn + blast scorch show the same in 2D and 3D. Blasts already ignite their disk -> fire -> scorch field. |
| DISASTER-PROXY (flood depth -> foundation integrity -> demolish) | DW-4 obstacle walls feed water; flood damage to buildings stays DISASTER-PROXY (no second force path). |
| STRUCT-ROLE / STRUCT-CASCADE | DW-2 rubble volume per site can use role counts (load columns) later; first cut uses footprint x floors. |
| `VoxelOccupationBook` (REOPENED, stub) | Not used by DW; rubble is ground height. Revisit when X2 lands. |

### Audit of all destruction plans vs code (2026-11-19 night) — what is really done

DONE: DESTROY-T2 wiring, T2.1, T2b, BLAST-PROFILE, fire->site collapse (FireCollapseQueue), GPU-B scorch, DW-1..5 in code (DW-1..5 not yet eye-verified).
PARTIAL / not as claimed:
- STRUCT-ROLE / STRUCT-CASCADE / DISASTER-PROXY flood: only dams have `SiteStructuralRoles` -> ordinary buildings never take flood damage or cascade (doc says ADOPTED).
- DEBRIS-FX: burst only on demolish (not blast, not dam column failure).
- STAGE-RING: Tier 0 only (doc honest). Voxel occupation X2: reopened stub (doc honest).
- PT damaged/ruined tile states: driven only by readiness + fire heat, never by blast/flood `DestructibleDamage`.
- Stale docs: plan_destroy_t2_mask_v1 shader path (.wgsl -> .wesl); ballistics_blast_contract omits DW-1.

Game-feel gaps, ranked (new todo D-1..D-8):
| Id | Gap | Plan |
|:---|:---|:---|
| D-1 | Buildings show no damage before they vanish (`DestructibleDamage` / `damage_stage` read by no renderer) | voxel site mesh + 2D stamp read damage stage: cracked/dark/partly collapsed columns (same per-column idea as the dam) |
| D-2 | Ordinary buildings: no structural roles -> no flood damage, no cascade | default roles from footprint (perimeter = load wall, interior = floor) so DISASTER-PROXY + CASCADE reach them |
| D-3 | Blast has no smoke / dust / debris burst (audio only) | blast execute -> DEBRIS-FX burst + lingering smoke via the fire particle spine |
| D-4 | Craters / rubble don't affect movement, roads, ecology | readers of GroundDeltaBook: road blocked under rubble > 1 m, nav cost on slope, no stand on rubble/crater (burnt) |
| D-5 | Fire on buildings only a heat tint; no burning frames | fire visuals on sites from the same burning state (OP-6 one clock) |
| D-6 | Missiles / fire don't hurt units or vehicles | separate program (unit damage shells DEFERRED in destruction plan) — leave |
| D-7 | No repair / rebuild on rubble | later (economy) |
| D-8 | Collapse is one-frame vanish; no staged sequence | after D-1 |

## CLOSE — final status (2026-11-20)

Evidence: dam scene playcheck capture (frame 5000, 6500 ticks) + `cargo test --lib` + per-slice unit tests.

| Area | Item | Status | Evidence |
|:---|:---|:---|:---|
| Ground | Lit voxel ground (StandardMaterial ext), sun shadows, day-night on the sim day clock | DONE | capture: relief + shadows read; `voxel_day_pose` tests |
| Ground | Water-line stamp turned the authored floodplain into Lake (trees + fire gone) | DONE | dump `floodplain_water` Dry 5994; trees in capture |
| Ground | Box valley -> organic walls / head / swells (natural shorelines) | DONE | capture: organic reservoir shoreline |
| Trees | Tree chunks keyed on ground revision (no rebuild per water tick) | DONE | — |
| Fire | S9 dead-over-live burn | DONE | s9 = 1.18 PASS |
| Fire | Burning ground glows in 3D from GPU-B scorch field; burning buildings flame/smoke on the fire clock (D-5) | DONE (code) | `site_fire` tests |
| Water | Moire on water: swirl (flow-normalised phase) + zigzag (foam stripes on every fast cell) | DONE (code) | fixed-direction phase, stripes fade before pixel size, foam only on turbulent water |
| Roads | T-8 roads clickable (no road selection exists in 2D either) + trucks follow the road network | OPEN | agent hit the usage limit; resume after reset |
| Water | Reservoir fills / drowns forest (S3), wave reaches G3 (S7) | DONE | s3 = 752, s7 = 1.6 PASS |
| Water | S2 downstream ratio now min over impound (was overwritten by full-reservoir spill) | DONE | s2 PASS; ~0 is by design (scenario shuts every outlet; min release scales with outlet opening) |
| Water | Hydro plant flood-collapsed once every building got roles | DONE | s4 = 16.2 MW PASS |
| Destruction | DW-1 craters + rims (terrain ground delta), DW-2 rubble, DW-3 dam per-column crest + notch mesh, DW-4 buildings wall water, DW-5 debris wash, DW-6 woodland strike | DONE (code, unit-tested) | `dw1_*`..`dw5_*`; capture `blast_craters_tracked` |
| Destruction | Removed dam kept blocking water | DONE | `removed_dam_releases_its_barrier` |
| Destruction | D-1 visible damage (voxel broken columns + 2D damaged/ruined stamps) | DONE (code) | `dw3_*`, stamp tests |
| Destruction | D-2 roles for all buildings -> flood + cascade reach them; D-8 staged collapse (flood) | DONE (code) | `d2_d8_*` |
| Destruction | D-3 blast + collapse dust (persistent, correct row convention); old one-frame debris burst removed | DONE (code) | `blast_dust` tests |
| Destruction | D-4 rubble/crater block roads, clear stands | DONE (code) | `debris_block`, `stand_ground` tests |
| Destruction | Unit-movement cost from craters | DEFER | no sim movement-cost field exists (`LogisticsTile` stub) |
| Destruction | D-6 unit/vehicle damage, D-7 repair/rebuild | DEFER | separate programs (unit damage shells DEFERRED in destruction plan) |
| Destruction | Blast at/over gate stays instant collapse | BY DESIGN | instantaneous cause |
| Perf | T-16 `upd_control_relax` hitch: per-pass BTreeMap of ~100k tiles replaced by a dense grid built from the overlays | DONE (code) | relax tests; re-measure perf.debug in next capture |
| Docs | Stale destruction docs (T2 mask shader path/crater row, blast contract, presentation table, struct/disaster/debris statuses) | DONE | — |
| Tests | Lib suite failures | see commit note | baseline comparison vs HEAD |
