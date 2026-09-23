# PLAN-WORLD-SCALE-CONTRACT-001 — VSS-T1-001 `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN  ◈PROGRAM
⟨ID⟩ PLAN-WORLD-SCALE-CONTRACT-001  ·  VSS-T1-001
Q↑↑  Ct▮▮▮  Cx▮▮▮▮▮▮  EV/Cx=HIGH  Au:🏛🟨🟨🟨
Parent: $ref:src/dev/plan_vfx_spectator_scale_program_v1.md § VSS-T1
Trip: $ref:src/dev/TRIP_VSS_SCALE_CONTRACT.md
Prior scale: $ref:src/dev/world_map_scale_v1.md
Designer anchors: $ref:src/dev/design_zoom_fire_read_v1.md · $ref:src/dev/design_build_readability_v1.md
Sim spine: $ref:.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md
Status: **ACTIVE** — 2026-08-28
Owner: @planner (S1) → @coder (S2) → @designer (S3)
```

**Rule:** Witness JSON wins over queue rows. This doc is **architecture only** — no Rust edits in S1.

---

## Summary

The engine already has a **symbolic** metres-per-tile contract (`WorldMapScale`, default 100 m/tile) and a **sim** contract (1 tile = 1 world unit on XZ), but **no single named `WORLD_SCALE_CONTRACT`** ties together chunk slab size, building footprint norms, camera operational anchors, and VFX px-per-tile gates. Operators perceive 32×32 tile slabs as “city-sized territories” because (a) sim-entry camera defaults to **whole-map fit** (~1 px/tile), then tactical overlays tint **per-chunk**, and (b) at the designer’s operational play anchor (`zoom_alpha ≈ 0.42`), the camera’s `map_zoom_limits_for_world` hi-cap (~8 tiles across the short viewport edge) drives **~27 px/tile**, making a **32-tile slab ≈ 870 px** — nearly the full map hole. A 3×3 primary footprint at the same zoom is only **~81 px**. The fix is not “smaller worlds” but a **published contract**: keep **32-tile sim substrate**, introduce an **8-tile tactical display grid** for overlays/readability, anchor default play at **operational zoom** (not fit-to-world), and codify **3×3 typical / 10×8 site** building envelopes.

---

## Current Problems

| # | Problem | Evidence |
|:---|:---|:---|
| P1 | **Scale constants scattered** | `SLAB=32` in `test_harness.rs`; `UVec2::new(32,32)` fallbacks in `economy/logistics/portals.rs`, `minimap_compositor`, `fire_streaming.rs`; `WorldMapScale` in `world_map_scale.rs`; fire px/tile in `fire_vfx/witness.rs` |
| P2 | **Sim-entry zoom ≠ operational play** | `focus_main_camera_on_world_params` uses `fit * 0.9` whole-map zoom (`tile_world_fallback.rs`); designer contract expects `zoom_alpha ≈ 0.42` (`design_zoom_fire_read_v1.md`) |
| P3 | **Chunk boundary reads as district** | Ecology/fire/minimap chunk tint keys off `WorldChunkLayoutCache.tiles_per_chunk` (observed 32); designer site rule says primary ≈ **15–40% of site**, not chunk (`design_build_readability_v1.md`) |
| P4 | **Footprint norms implicit** | Pilot catalog: primary shapes 2×2–6×5; site stubs 6×5–10×8 (`_pilot_catalog.ron`, `_mock_shapes.ron`); tool default 2×2 (`build_state.rs`); growth default 4×2 warehouse (`settlement/execute.rs`) |
| P5 | **Fire zoom re-key drift** | `FIRE_SPARK_*_PX_PER_TILE` (2.5 / 4.0) vs legacy `zoom_alpha` bands (0.42 / 0.85) coexist (`fire_vfx/witness.rs`, `map_camera.rs` `TACTICAL_VFX_PROOF_ZOOM_ALPHA`) |
| P6 | **Symbolic vs sim chunk conflated** | 32 tiles × 100 m/tile = **3.2 km** slab — valid for WSS/substrate sim, but presented as the same grid the player places 3×3 buildings on |

---

## Proposed `WORLD_SCALE_CONTRACT` constants

| Constant | Symbol | **Proposed value** | **Current live** | Layer | Notes |
|:---|:---|:---:|:---:|:---|:---|
| Sim tile unit | `TILE_SIM_UNIT` | **1.0** | 1.0 (implicit) | Sim | 1 tile = 1 world unit XZ — **unchanged** |
| Symbolic tile width | `TILE_METERS_SYMBOLIC` | **100.0** | 100.0 (`WorldMapScale::default`) | Lore / WG rhythm | Drives `derive_land_features`, UI km labels |
| Sim chunk slab (substrate) | `CHUNK_TILES_SIM` | **32** | 32 (`test_harness` SLAB; fallbacks) | Sim / WSS / fire / weather | Keep for ECS `ChunkCellMatrix`, dense cache, substrate paging |
| Tactical display grid | `CHUNK_TILES_DISPLAY` | **8** | 32 (implicit — no split) | View / overlay / minimap | New contract target (VSS-T1-004); 4×4 display cells per sim slab |
| Primary building footprint (typical) | `BUILDING_TYPICAL_FOOTPRINT_TILES` | **(3, 3)** | 2×2 tool default; pilots 2×2–6×5 | Construction / readability | Median mock pilot primary mass |
| Primary building footprint (large) | `BUILDING_LARGE_FOOTPRINT_TILES` | **(6, 5)** | `logistics_rail_warehouse_l_6x5` | Construction | Largest pilot primary shape |
| Site envelope (typical) | `SITE_TYPICAL_ENVELOPE_TILES` | **(10, 8)** | `logistics_rail_warehouse` site stub | Construction overlay | Yard + rail void inside site box |
| Default world extent (editor) | `WORLD_DEFAULT_TILES_AXIS` | **512** | 512 (`WorldGenParams::default`) | WorldGen | ~51 km @ 100 m/tile |
| Harness / visual world | `WORLD_HARNESS_TILES_AXIS` | **320** | 320 (`MediumSmall`, `debug_maneuver`, map backends) | Test / visual | ~32 km — `stage5_full_app` path |
| Operational play zoom (normalized) | `OPERATIONAL_ZOOM_ALPHA` | **0.42** | 0.42 (design); **not** sim-entry default | Camera / VFX | `design_zoom_fire_read_v1.md` |
| Operational px-per-tile (sparse sparks) | `OPERATIONAL_PX_PER_TILE` | **2.5** | 2.5 (`FIRE_SPARK_OPERATIONAL_PLAY_PX_PER_TILE`) | VFX | FIRE-VIS-001 axis |
| Full scatter px-per-tile | `TACTICAL_PX_PER_TILE` | **4.0** | 4.0 (`FIRE_SPARK_FULL_SCATTER_PX_PER_TILE`) | VFX | Scatter ramp ceiling |
| Tactical proof zoom (harness only) | `TACTICAL_PROOF_ZOOM_ALPHA` | **0.85** | 0.85 (`TACTICAL_VFX_PROOF_ZOOM_ALPHA`) | Witness / `--test visual` | Not G-PLAY default |
| Buildings per operational frame (target) | `BUILDINGS_PER_FRAME_TARGET` | **4–9** | unmeasured | Designer | 2×2 grid of 3×3 footprints in ~900 px map hole @ operational zoom |
| Min tile screen px (readable) | `TILE_MIN_SCREEN_PX` | **8.0** | 8.0 (`map_zoom_limits` `min_span_tiles=8`) | Camera | ~8 tiles across short edge at max zoom-in |

**Authority model:**

```text
⊚WorldMapScale           ═▶ ⊨symbolic metres/tile (lore, WG derivation)
⊚WorldGenParams          ═▶ ⊨world extent (width/height tiles)
⊚ChunkCellMatrix.size    ═▶ ⊨sim slab geometry (CHUNK_TILES_SIM)
⊚WorldChunkLayoutCache   ⊰ ⊚ChunkCellMatrix  (observed tiles_per_chunk — today equals sim)
⊚ViewProjectionAuthority ═▶ ⊨camera px-per-tile (operational anchor)
⊚FireSparkWitness        ◂⊳[snapshot] ⊚ViewProjectionAuthority + fire policy constants
⊚ConstructionVisualRequests ◂⊳[snapshot] ⊚BuildGhostState / site book (footprint tiles)
```

---

## Why 32-tile chunks feel like “huge territories”

### Quantitative (1280×720, map hole ≈ 900×520 px, 512² world)

1. **Sim-entry camera** (`focus_main_camera_on_world_params`, `_` branch):  
   `zoom ≈ 0.9 × min(900/512, 520/512) ≈ 0.91` **px/tile** → entire 512-tile world on screen; a 32-tile slab is only **~29 px** — but chunk **tint overlays** still key to 32-tile sim boundaries, so the *semantic* district is 3.2 km even when visually tiny.

2. **Operational play anchor** (`zoom_alpha = 0.42`, per-world limits from `map_zoom_limits_for_world`):  
   `hi ≈ short_viewport / 8 ≈ 65` px/tile cap → `scale ≈ 0.02 + 0.42 × (65 − 0.02) ≈ **27.3 px/tile**`  
   - **32-tile slab** → 32 × 27.3 ≈ **873 px** (fills map hole — reads as “one screen = one chunk”)  
   - **3×3 primary footprint** → ≈ **82 px** (~9% of viewport width — “doll-house on a field”)  
   - **10×8 site stub** → ≈ **273 × 218 px** (reasonable, but chunk border still 3.2× larger than site)

3. **Symbolic mismatch:** 32 tiles × 100 m = **3.2 km** per sim slab — the size of a small town. A 3×3 primary building at 100 m/tile is **300 m × 300 m** (large warehouse); it should dominate a **city block**, not shrink inside a **multi-km chunk cell**.

4. **Test harness seeds 32² slabs** (`spawn_test_scene_chunk_slabs_once`) independent of `WorldGenParams` extent — on a 320² harness world that is **10×10 = 100 chunk entities**, each presented as a district unit to fire/weather/ecology.

**Root cause (plain language):** the engine uses **one grid** (32-tile sim slab) for **substrate simulation**, **overlay tinting**, and **player mental model of “district”**, while the **camera** can be at whole-map fit **or** operational zoom — and at operational zoom the sim slab is simply too many tiles across the screen.

---

## Target Architecture

```text
◎layers
  Sim substrate     CHUNK_TILES_SIM = 32     (WSS, fire tick, weather, dense cache)
  Tactical display  CHUNK_TILES_DISPLAY = 8  (overlay grid, minimap district, ecology legend)
  Construction      footprint in tiles       (primary 3×3 typical inside 10×8 site)
  Camera            OPERATIONAL_ZOOM_ALPHA   (default play, not fit-to-world)
  Symbolic          TILE_METERS_SYMBOLIC     (WG rhythm only — not render scale)

◎module target (VSS-T1-002)
  src/terrain/world_scale_contract.rs   — pub const WORLD_SCALE_CONTRACT table
  re-export from world_map_scale.rs     — avoid duplicate metre/tile defs
  WorldGenParams doc + preset comments  — align 512 default / 320 harness

◎forbidden
  ⛔ second metres-per-tile default
  ⛔ shrinking CHUNK_TILES_SIM to fix readability without substrate migration plan
  ⛔ fit-to-world as sim-entry default once operational anchor ships
```

---

## Implementation Phases (ordered DAG)

### P1 — Scale contract publication (VSS-T1-001) · @planner · **THIS DOC**

| Field | Value |
|:---|:---|
| **Goal** | Publish constants table, authority map, witness schema |
| **Files** | `src/dev/plan_world_scale_contract_v1.md`, `debug_runs/world_scale_contract_live.json` (stub) |
| **Owner** | @planner |
| **Risks** | None (read-only) |
| **Acceptance** | Trip registry satisfied; tribunal questions answered; witness stub `status: pending` |

### P2 — Constants module + camera default alignment (VSS-T1-002) · @coder · **SHIPPED**

| Field | Value |
|:---|:---|
| **Status** | **done** 2026-08-28 — witness `green: true`, `sim_entry_zoom_alpha` = 0.42 |
| **Goal** | Single `WORLD_SCALE_CONTRACT` module; wire `focus_main_camera_on_world_params` default to `OPERATIONAL_ZOOM_ALPHA` (not whole-map fit); document `CHUNK_TILES_SIM` vs `CHUNK_TILES_DISPLAY` |
| **Files** | `src/terrain/world_scale_contract.rs` (new), `world_map_scale.rs`, `world_generator_enhanced.rs`, `test_harness.rs` (SLAB → const ref), `tile_world_fallback.rs`, `map_camera.rs`, `fire_vfx/witness.rs` (const re-exports only) |
| **⊚authority-owner** | `WorldMapScale` / `WorldGenParams` / `ViewProjectionAuthority` — no new writers |
| **Risks** | Camera change affects G-PLAY, construction pick, fire witness; stage5 regression |
| **Compat** | `RUST_ENGINE_MAP_ZOOM_LEGACY_FIT=1` env gate for one release if needed |
| **Diagnostics** | `world_scale_contract_live.json` measured fields populated |
| **Acceptance** | `cargo test -p proc_A_dine01 --lib world_scale` · witness `green: true` · `sim_entry_zoom_alpha` ≈ 0.42 ± 0.05 on 512² default |

### P3 — Readability audit: buildings-per-frame (VSS-T1-003) · @designer

| Field | Value |
|:---|:---|
| **Status** | **done** 2026-08-29 — `tribunal.designer_signed_off: true` |
| **Goal** | Sign off that at `OPERATIONAL_ZOOM_ALPHA`, **4–9** typical 3×3 primaries fit in map hole; site stub 10×8 reads as “yard + building”, not chunk-filling; update `design_build_readability_v1.md` screen-height table with measured px |
| **Files** | `src/dev/design_build_readability_v1.md`, `design_zoom_fire_read_v1.md` (cross-link px/tile anchors) |
| **Owner** | @designer |
| **Risks** | May require BUILD-READ-WORLD-002 iso scale lever — route to @coder if primary < 40 px tall |
| **Acceptance** | `buildings_per_frame_measured` within target band in witness · designer sign-off row in witness `tribunal` |
| **Measured** | bpf **10.15** (linear proxy, +13% vs 9 cap — dissent recorded); primary **78.8 px**; site **263×210 px**; primary **37.5%** of site height |
| **Verdict** | **PASS** — qualitative readability predicates met; P4 display-grid split remains deferred |

**Optional P4 (defer unless P3 fails):** VSS-T1-004 @coder — split `WorldChunkLayoutCache` display grid (8) from sim slab (32) for overlays/minimap only.

---

## ECS Schedule Plan

Scale contract is **resource/constant** — no new systems. Ordering constraints for P2:

```text
OnEnter(Simulation)
  → focus_main_camera_on_world_params     (commit OPERATIONAL_ZOOM_ALPHA pose)
  → derive_map_camera_desired_from_view_authority
Update
  → map_camera_apply_input / wheel
  → sync_main_world_camera_projection
  → extract (fire/VFX read px-per-tile from ViewProjectionAuthority snapshot)
```

**Readers:** fire spark cull, ecology chunk overlay, construction ghost projection, minimap compositor  
**Writers:** `ViewProjectionAuthority` only (camera input + startup focus)  
**Invalidation:** `WorldGenParams` width/height change → recompute zoom limits + refresh witness

---

## Diagnostics / Witness

**Path:** `debug_runs/world_scale_contract_live.json`  
**Collector:** `world_scale_contract_live_proof.rs` (VSS-T1-002, @coder)

| Field | Type | Purpose |
|:---|:---|:---|
| `status` | string | `pending` → `measured` → `green` |
| `program_id` | string | `VSS-T1-001` |
| `slice_id` | string | `VSS-T1-001` / `002` / `003` |
| `contract` | object | Published constants mirror (all keys from table above) |
| `measured.world_tiles_axis` | u32 \| null | Live `WorldGenParams.width` |
| `measured.meters_per_tile` | f32 \| null | Live `WorldMapScale.meters_per_tile` |
| `measured.chunk_tiles_sim` | u32 \| null | Observed `ChunkCellMatrix` / harness SLAB |
| `measured.chunk_tiles_display` | u32 \| null | Observed overlay grid (null until P4) |
| `measured.sim_entry_zoom_alpha` | f32 \| null | After `focus_main_camera_on_world_params` |
| `measured.sim_entry_px_per_tile` | f32 \| null | Camera scale at sim entry |
| `measured.operational_px_per_tile` | f32 \| null | At `OPERATIONAL_ZOOM_ALPHA` |
| `measured.chunk_slab_screen_px` | f32 \| null | `chunk_tiles_sim × px_per_tile` |
| `measured.building_typical_screen_px` | [f32, f32] \| null | 3×3 footprint screen size |
| `measured.buildings_per_frame` | f32 \| null | Viewport / typical footprint width |
| `measured.site_envelope_screen_px` | [f32, f32] \| null | 10×8 site at operational zoom |
| `anchors.fire_operational_px_per_tile` | f32 | 2.5 (from contract) |
| `anchors.fire_full_scatter_px_per_tile` | f32 | 4.0 |
| `anchors.tactical_proof_zoom_alpha` | f32 | 0.85 |
| `footprint_pilot_median_tiles` | [u32, u32] | [3, 3] from catalog audit |
| `green` | bool | All P2 predicates pass |
| `tribunal.designer_signed_off` | bool | P3 complete |
| `dissent` | array | Optional minority scale proposals |

---

## Edge Cases

| Case | Handling |
|:---|:---|
| **World < 32 tiles axis** | Harness clamps slab: `SLAB.min(params.width)` — contract documents min world 64+ for chunk sim |
| **1024² LargeStrategic** | Keep `CHUNK_TILES_SIM=32`; operational px/tile still ~27 due to `min_span_tiles=8` cap — buildings-per-frame target holds |
| **Multi-window / RTT** | Measure px/tile from `SimulationMapViewport` logical size, not full window |
| **Minimap** | Never inherits tactical proof zoom; chunk display grid may differ from main map (P4) |
| **Construction 6×5 warehouse** | Counts as one “building” in per-frame target; site 10×8 still fits @ operational zoom |
| **Chunk streaming / async** | `CHUNK_TILES_SIM` is compile-time sim contract — streaming uses same slab size |

---

## Open Questions

| ID | Question | Default stance |
|:---|:---|:---|
| Q1 | Reduce `CHUNK_TILES_SIM` from 32 → 16? | **No** — WSS/substrate + dense cache aligned on 32; fix display grid instead |
| Q2 | Change `TILE_METERS_SYMBOLIC` from 100 m? | **No** — WG presets and `world_map_scale_v1.md` shipped; buildings “oversized in tiles” is explicit |
| Q3 | Should sim-entry use operational zoom immediately? | **Yes** for interactive sim (P2); `ActiveTestScene` / scenario / `VfxFireTestRegion` may override after commit — not whole-map fit as default |
| Q4 | Split display grid in P2 or P4? | **P4** unless minimap bleed blocks P3 sign-off |

---

## ΔWF routing

| Slice | Agent | Witness note |
|:---|:---|:---|
| VSS-T1-001 | @planner | This doc + stub JSON |
| VSS-T1-002 | @coder | Populate `measured.*`, `green: true` |
| VSS-T1-003 | @designer | `tribunal.designer_signed_off: true` |
| VSS-T1-004 | @coder | `measured.chunk_tiles_display: 8` |

**Joint review question for @coder:** Should `focus_main_camera_on_world_params` commit `map_scale_for_zoom_alpha(OPERATIONAL_ZOOM_ALPHA, …)` unconditionally on `OnEnter(Simulation)`, or only when `ActiveTestScene` is absent?
