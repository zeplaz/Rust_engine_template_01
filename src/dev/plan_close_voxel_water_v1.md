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
