# VFX + product — coder Phase 2 queue `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@coder` ×2 |
| **Supersedes** | Queued items in [`fire_particle_spark_coder_queue_v1.md`](../prompts/guides/ui/fire_particle_spark_coder_queue_v1.md) § W1/A1–B2 and [`water_surface_vfx_coder_queue_v1.md`](../prompts/guides/ui/water_surface_vfx_coder_queue_v1.md) § W1/W2 first pass |
| **Machine queue** | [`continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) |
| **Master plan** | [`coder_execution_plan_v1.md`](coder_execution_plan_v1.md) |

**Context:** Fire Phase A+B **code is largely landed**. Water W1/W2 **first pass landed** but track is **NOT CLOSED** — see [`stages/water_vfx_closure_plan_v1.md`](stages/water_vfx_closure_plan_v1.md) (ocean, bend/coast foam, river read, designer PASS). Witness JSON often shows `fire_spark_rows: 0` at **strategic** zoom — particles correctly cull (D-F09). Phase 2 fire = **prove, tune, integrate**; water = **closure track**.

**Design gates:** **None blocking.** Optional: Phase 4 icon atlas PNG; **post-implementation VFX review** vs reference mocks (below).

---

## Post-implementation VFX review (optional · `@designer`)

**Full brief:** [`vfx_post_implementation_review_v1.md`](../prompts/guides/ui/vfx_post_implementation_review_v1.md) · **record:** [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) (**D-VFX**) · captures → `assets/vfx/reference/review_captures/`

After **P2-VFX-VISUAL-001** / polish slices land, capture **tactical-zoom** stills and compare:

| Mock | Path |
|:---|:---|
| Fire sparks (blob vs pinpoint) | [`assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png`](../../assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png) |
| Water lake / river / ocean | [`assets/vfx/reference/water/water_surface_target_v1.png`](../../assets/vfx/reference/water/water_surface_target_v1.png) |

**Outcome:** PASS / TUNE tickets — does **not** block coder queue.

---

## Done (close these slices — do not re-implement)

| Slice | Evidence |
|:---|:---|
| FX-FIRE-SPARK-001 Phase A | `fire_particle_draw.wgsl` pinpoint + scatter |
| FX-FIRE-SPARK-002 | `fire_spark_compute.wgsl`, `gpu_spark_compute.rs`, `fire_spark_compute_enabled: true` |
| FX-FIRE-SPARK-003 | `fire_spark_*` fields in `stage5_full_app_live.json` |
| FX-FIRE-SPARK-005 | `ParticleClass::Spark` / `Ember` in `gpu_particles.rs` |
| FX-FIRE-SPARK-006 | `fire_particle_view_culled`, `is_tactical_fire_particle_view` |
| FX-WATER-SHADER-001 | `register_water_surface_draw` in `engine_with_worldgen.rs` |
| FX-WATER-SHADER-002 | `water_w1_green: true`, river segment/tile counts in witness |
| FX-WATER-PARTICLE-001 | `water_particle.wgsl` + `water_particle_draw.wgsl` exist |
| FX-WATER-PARTICLE-002 | `gpu_water_particles.rs` profiles + §7 caps |

**Partial:** FX-FIRE-SPARK-004 — `relink_core2d_transparent_overlay_order` exists; **smoke field vs spark** order still needs audit (see § P2-A2).

---

## Phase 2 — two coders (new primary work)

```text
Coder A — Visual proof + render compositing + tuning
  P2-VFX-VISUAL-001   Tactical-zoom visual harness (fire + water particles > 0)
  P2-FIRE-SPARK-010   Sparks above smoke overlay (weather field pass)
  P2-FIRE-SPARK-011   In-sim spark shower tuning (hot cells, motion read)
  P2-WATER-POLISH-001 River ribbon width + ocean tile coverage + GPU/CPU parity

Coder B — Witness gates + product lanes (disjoint files)
  P2-VFX-WITNESS-001  stage5 gates: rows > 0 at tactical zoom_alpha
  P2-WATER-WITNESS-002 water_particle_* counts in visual proof JSON
  UI-WP-LAYOUT-001    World preview D-01 shell (signed)
  IND-E01             Industrial concrete chain E2E
```

---

## P2-VFX-VISUAL-001 · Tactical visual proof (both coders — operator + small harness)

**Problem:** `debug_runs/stage5_full_app_live.json` shows `water_particle_strategic_culled: true`, `fire_spark_rows: 0` — **expected** at strategic zoom, not a shader bug.

**Goal:** Refresh proof with **tactical** `zoom_alpha ≥ 0.65` and visible fire + water on map.

```
Lane: P2-VFX-VISUAL-001
Read: src/dev/vfx_coder_phase2_queue_v1.md § P2-VFX-VISUAL-001
First: stage5 harness or visual test sets MapCameraDesired to tactical zoom before witness stamp
Do NOT: remove D-W09 / D-F09 strategic cull rules
Verify: cargo run -p proc_A_dine01 --release -- --test visual
        fire_spark_rows > 0 AND water_particle_river_streaks > 0 in refreshed JSON
```

| Step | Task | Files (≤3) |
|:---:|:---|:---|
| V-1 | Harness: optional `Stage5VfxProofProfile { tactical_zoom: true, seed_fire: true }` | `stage5_full_app_harness.rs` |
| V-2 | Seed 1–3 hot fire cells in fixture when `seed_fire` | test helper or `FireVisualFrame` inject |
| V-3 | Operator doc: zoom in before `--test visual` if harness unchanged | `debug_runs/README.md` one line |

**Accept:** Refreshed `stage5_full_app_live.json` with `fire_spark_rows > 0` **or** documented tactical visual run + screenshot path under `assets/vfx/reference/`.

---

## P2-FIRE-SPARK-010 · Sparks above smoke (Coder A)

**Gap:** Transparent order is water → water particles → **fire sparks**. **Chunk smoke / weather fire field** may still draw after sparks.

```
Lane: P2-FIRE-SPARK-010
Read: gpu_fire_particle_raster.rs relink_core2d_transparent_overlay_order
First: audit gpu_weather_fire_field / overlay pass vs FireParticleRasterPassLabel
Verify: smoky cell — sparks visible on top in --test visual
```

| Step | Task |
|:---:|:---|
| S-1 | Map full Core2d transparent subgraph (terrain, smoke RT, water, fire) |
| S-2 | Edge: `FireParticleRaster` after smoke composite (D-F10 A) |
| S-3 | Witness: `fire_sparks_above_smoke: true` in diagnostic JSON (optional field) |

---

## P2-FIRE-SPARK-011 · Spark motion + density tuning (Coder A)

**Goal:** Phase B compute **visible** on tactical map — shower, respawn, attractor pull.

| Step | Task | Files |
|:---:|:---|:---|
| M-1 | Tune `fire_spark_compute.wgsl` lifetime decay / respawn burst | WGSL |
| M-2 | Attractor count / mass from top-N hot `GpuParticleInstance` rows | `gpu_spark_compute.rs` |
| M-3 | Compare to `fire_spark_target_v1.png` + `elemental_sparks/` refs | visual |

---

## P2-WATER-POLISH-001 · River read + ocean coverage (Coder A)

**Goal:** Rivers “not missing” in **player** view — not only witness counts.

| Step | Task | D-W |
|:---:|:---|:---|
| R-1 | Widen river ribbon `half_width` or tile_kind priority vs lake mask | D-W01 |
| R-2 | Fix `ocean_tiles` sampling (witness shows `water_ocean_tiles: 0` on some worlds) | D-W04 |
| R-3 | CPU raster vs GPU overlay parity check on same catalog stamp | W1 |

---

## P2-VFX-WITNESS-001 · Tactical witness gates (Coder B)

**Goal:** CI-friendly predicates so Phase 2 does not regress silently.

| Step | Task | Files |
|:---:|:---|:---|
| W-1 | Lib test: `update_world_fire_particles` at `zoom_alpha = 0.8` → `spark_witness.rows > 0` | `gpu_particles.rs` tests |
| W-2 | Lib test: `update_world_water_particles` at tactical zoom → `witness.river_streaks > 0` | `gpu_water_particles.rs` tests |
| W-3 | Harness: when `tactical_vfx_proof` flag set, assert JSON fields | `stage5_full_app_harness.rs` |

---

## P2-WATER-WITNESS-002 · Water particle proof fields (Coder B)

Extend existing stamps (already partial in harness):

| Field | Green when |
|:---|:---|
| `water_particle_rows` | > 0 at tactical zoom |
| `water_particle_river_streaks` | > 0 when `water_river_segments > 0` |
| `water_particle_strategic_culled` | `false` at tactical zoom |
| `water_shader_motion_always_on` | always `true` |

---

## Product lanes (Coder B — parallel when VFX blocked)

### UI-WP-LAYOUT-001

```
Lane: UI-WP-LAYOUT-001
Read: ui_world_preview_coder_queue_v1.md
First: D-01 shell only (≤3 files) — window.rs
Verify: cargo test -p proc_A_dine01 --lib stage5
```

### IND-E01

```
Lane: IND-E01
Read: src/dev/industrial_activation_pipeline.md
First: concrete chain → industrial_activation_live.json production_green
Do NOT: gpu_particles, water_*, fire_* shaders
```

---

## Optional / P3 (~15 min each)

| ID | Task |
|:---|:---|
| UI-P3-M2-VERIFY | Confirm `minimap_compositor_live.json` `logistics_rows > 0` after visual |
| UI-P2A-F03 | `ops_zone_hover_token: true` |
| UI-P2A-P4-AUTH | `build_rail_authoritative: true` |

---

## Deferred (do not start without new design sign-off)

| ID | Lane |
|:---|:---|
| FX-WATER-WEATHER | Storm spray W3 |
| FX-WATER-RUNTIME | Flood/breach W4 |
| FX-FIRE-SPARK-007 | Point-list primitive (only if quads still too soft **after** P2 visual) |

---

## 4-day schedule (Phase 2)

| Day | Coder A | Coder B |
|:---|:---|:---|
| 1 | P2-VFX-VISUAL-001 harness zoom + fire seed | P2-VFX-WITNESS-001 unit tests |
| 2 | P2-FIRE-SPARK-010 smoke order | P2-WATER-WITNESS-002 + refresh JSON |
| 3 | P2-FIRE-SPARK-011 motion tune | UI-WP-LAYOUT-001 D-01 |
| 4 | P2-WATER-POLISH-001 river/ocean | IND-E01 or P3 witness tails |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Phase 2 queue after W1/W2 + fire B landed |
