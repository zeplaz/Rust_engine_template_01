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
