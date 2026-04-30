# Layered procedural fields + fast pixel preview (composite-style mental model) `v1`

**Audience:** designers + implementers aligning **multi-layer world generation** and **map preview** with this repo.  
**Mental model:** **composite** layering uses **independent scalar fields** composed with rules — this document adopts that *pattern* without tying the naming to any commercial game.

**Paired implementation matrices:** [`../../matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md`](../../matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md) · **Material / tag / rules:** [`material_tag_rule_system_v1.md`](material_tag_rule_system_v1.md) + [`../../matrix/terrain_biome/material_unification_matrix_v1.md`](../../matrix/terrain_biome/material_unification_matrix_v1.md)

**Code touchpoints (verify with `rg`, not assumptions):**

- World gen params & spawn: [`src/terrain/generation/world_generator_enhanced.rs`](../../../src/terrain/generation/world_generator_enhanced.rs) — `WorldGenParams`, `WorldMarker`, `TileMarker`, `Height`, `Moisture`, `Temperature`, `TerrainType`, `WORLD_GEN_TUNING_JSON_PATH`, `generate_world`
- Noise & tuning: [`src/terrain/generation/terrain_noise.rs`](../../../src/terrain/generation/terrain_noise.rs) — `NoiseSamplingTuning`, `build_fbm_perlin`, `build_height_noise`, `TerrainNoiseProfile`
- JSON overlay: [`src/terrain/generation/tuning_io.rs`](../../../src/terrain/generation/tuning_io.rs) — `WorldGenTuningOverlay`, `load_overlay`
- Biome resolution: [`src/terrain/biome.rs`](../../../src/terrain/biome.rs) — `TerrainClass`, `BiomeWeights`, `BiomeTuning`, `classify_biome`
- Plugins & UI: [`src/terrain/generation/world_generation_plugin.rs`](../../../src/terrain/generation/world_generation_plugin.rs) — `WorldGenerationInGamePlugin`, `WorldGenerationToolsUiPlugin`, `WorldGenToolsPlugin`; [`src/gui/editor/world_gen_ui.rs`](../../../src/gui/editor/world_gen_ui.rs) — `WorldGenUiState`, `PreviewMode`, `world_gen_ui_system`; [`src/gui/editor/world_preview.rs`](../../../src/gui/editor/world_preview.rs) — `WorldPreviewTexture`, `update_world_preview_texture`, `WorldPreviewPlugin`
- Example overlay: [`assets/config/world_gen_tuning.example.json`](../../../assets/config/world_gen_tuning.example.json)

---

## 1. Scope & references

- **Layer composition:** base terrain is not “one noise texture” — treat **elevation**, **moisture**, **temperature**, and optional **auxiliary** masks as **separate samplers** before a single **tile resolver** runs.
- **Preview ≠ simulation:** preview is a **cheap raster** (fewer octaves, optional parallel fill); it must not run hydrology, cities, or combat.
- **Pair with:**
  - **[`chunks_streaming_v1.md`](chunks_streaming_v1.md)** — chunk grid, interest sets, ghost bands; **chunk-local determinism** must match whatever becomes the authoritative `ChunkId` story.
  - **[`simulation_lod_v1.md`](simulation_lod_v1.md)** — preview **LOD pyramid** (zoom levels) is **orthogonal** to sim LOD tiers but should reuse the **same chunk index space** where possible.
  - **[`hydrology_v1.md`](hydrology_v1.md)** — which layers **re-run** at runtime vs stay **baked** at gen.
  - **[`tile_sprites_v1.md`](tile_sprites_v1.md)** — preview **RGBA** is a stand-in; final art may use palettes / atlases keyed off `TerrainClass` + overlays.

---

## 2. Layer stack (provisional)

| Layer | Role | In repo today | Notes |
|:---|:---|:---:|:---|
| **Elevation** | Land/ocean macro shape, feeds water & alpine rules | ✅ | `Height`, `build_height_noise`, `sample_height_field` |
| **Moisture** | Biome humidity field | ✅ | `Moisture`, fBm·Perlin in `generate_world` |
| **Temperature** | Biome thermal field | ✅ | `Temperature`, fBm·Perlin |
| **Aux / shaping** | Coast detail, domain warp, micro-breakup | **Partial** | `domain_warp_strength`, `terrain_detail_mix`, warp/detail channels in `NoiseSamplingTuning` |
| **Resource fields** | Patch-based deposit noise per resource | ❌ **Proposed** | No `ResourceField` / patch sampler in `world_generator_enhanced` yet |
| **Threat / bases** | Placement mask for hostile pockets | ❌ **Proposed** | No dedicated field; tie to future strategic/factions specs if needed |

---

## 3. Determinism rules

- **World seed:** `WorldGenParams.seed` drives RNG and noise seeds in [`generate_world`](../../../src/terrain/generation/world_generator_enhanced.rs).
- **Per-layer offsets:** today, moisture uses `params.seed.wrapping_add(1)`, temperature `wrapping_add(2)`; warp/detail use `NoiseSamplingTuning::{warp_seed_offset, detail_seed_offset}`. **Target:** all channels document explicit offsets (see matrix **P1**).
- **Chunk-local seeds (target):** `chunk_seed = hash(world_seed, chunk_x, chunk_y)` when generation moves to **chunk tasks** — must match checksum tests and item **33** in [`implementation_questions_v1.md`](implementation_questions_v1.md).
- **Schema/version:** any change to resolver order or field list bumps a **versioned** header (pair [`serialization` matrix](../../matrix/serialization/serialization_hybrid_migration_matrix_v1.md) when saves exist).

---

## 4. Tile resolver

**Target pipeline (conceptual):**

1. Sample scalar fields at world XY (or chunk-local + offset).
2. If elevation below water band → water / shore classes (align `BiomeTuning` sea / shore thresholds).
3. Else derive **biome weights** / dominant class from moisture + temperature (+ height), then optional **aux** masks.
4. If any **resource** mask exceeds threshold → overlay or override tile (design choice: see **📎** below).
5. Optional **threat** tint for preview only, or a gameplay layer.

**Repo today:** per-tile `TerrainType` comes from `classify_biome` in [`src/terrain/biome.rs`](../../../src/terrain/biome.rs) using the same height/moisture/temperature values written in `generate_world` — this is the canonical **resolver** until resource patches land.

**📎 Open (designer / implementation):**

- Resources as **separate ECS component** vs **overriding** `TerrainClass`.
- Patch shape: single threshold vs dual-threshold “blob” with richness curve.
- Order of application vs **hydrology** (rivers may cut through patches).

---

## 5. Resource patches & spawn bias (proposed)

- **Density noise** per resource type; **threshold** creates blobs; **richness** from curve of density.
- **Spawn bias:** scale density or max richness by falloff from **spawn / starting area** vector — curve parameters 📎 **ASK** (no defaults in this doc).
- **Clustering:** optional second noise multiplied as a mask (designer tuning).

---

## 6. Pixel preview pipeline

**Desired (fast path):** for each preview pixel, map to world coords, sample fields (possibly with **fewer octaves** than full gen), resolve color (water depth gradient, biome base, resource blend, threat tint).

**Repo today:** [`update_world_preview_texture`](../../../src/gui/editor/world_preview.rs) iterates **ECS** tiles (`TileMarker` + `Height`/`Moisture`/`Temperature`/`TerrainType`) and writes **one texel per tile** from `Transform` — it is **not** a standalone software sampler and requires **entities spawned** after `GenerateWorldEvent`. Document this asymmetry in the matrix.

**Parameters 📎 ASK:** pixels-per-world-tile, preview max resolution, whether preview uses same `noise_octaves` as full gen or a dedicated `preview_octaves` knob.

---

## 7. Hierarchical preview / LOD

- **LOD0 / LOD1 / LOD2:** successive **2×** (or 📎) downsamples of the same logical world window for zoomed-out map UI.
- **Invariant:** each level must read from the **same deterministic field stack** (no ad-hoc unrelated noise).
- **Coupling:** chunk grid from [`chunks_streaming_v1.md`](chunks_streaming_v1.md); preview tiles may align to chunk boundaries for **cache keys** (see §9).

---

## 8. Multithread / GPU paths

- **CPU parallel:** row- or pixel-parallel fill (e.g. `rayon`) over a flat RGBA buffer, then upload one `bevy::prelude::Image` — see [`bevy_0_18_migration_plan`](../../matrix/engine_bevy/bevy_0_18_migration_plan.md) for general render/compute constraints.
- **GPU (future):** compute shader sampling for live sliders; **deferred** until render stack decision.

---

## 9. Cached chunk previews (proposed)

- **Key:** `(chunk_x, chunk_y, seed_hash, tuning_hash, preview_lod)`.
- **Invalidate:** seed change, schema bump, overlay reload, or slider commit — exact policy 📎 **ASK**.
- **Persistence:** optional — if previews are saved, pair **serialization** matrix for format and eviction.

---

## 10. Reflect-driven UI (target) & current UI

- **Target:** `#[derive(Reflect)]` `WorldSettings` (or extend `WorldGenParams`) with `#[reflect(ui)]` on tunables for inspector-driven panels.
- **Current:** `world_gen_ui_system` exposes sliders/radios for regions, terrain, `NoiseSamplingTuning`, subset of `BiomeTuning`, preview modes; JSON round-trip via `WORLD_GEN_TUNING_JSON_PATH` and [`WorldGenTuningOverlay`](../../../src/terrain/generation/tuning_io.rs).
- **Desktop tool:** [`tools_ui/spec/08_world_gen_desktop_tool_v1.md`](../tools_ui/spec/08_world_gen_desktop_tool_v1.md) — same keys as overlay.

---

## 11. Open designer questions (📎)

Feed answers back into [`implementation_questions_v1.md`](implementation_questions_v1.md) § **Layered fields & fast preview** (items **35–41**).

| # | Topic |
|:---:|:---|
| L1 | Chunk size for **authoritative** gen vs **preview** downsampling — single source or two scales? → **impl §35** |
| L2 | Preview **octave budget** vs gameplay gen — shared or separate tunables? → **§36** |
| L3 | Max **resource patch** types per world; ordering when two masks overlap → **§37** |
| L4 | **Threat** layer: preview-only tint vs gameplay placement → **§38** |
| L5 | **LOD** pyramid depth (2 vs 3 levels) and max texture size per level → **§39** |
| L6 | **Spawn** point: fixed scenario vs first pass on height — affects bias fields → **§41** |

---

## Cross-links

- Pipeline stub: [`spec/01_world_generation_pipeline.md`](spec/01_world_generation_pipeline.md)
- Terrain / hydro bind: [`spec/02_terrain_hydrology_worldgen.md`](spec/02_terrain_hydrology_worldgen.md)
- Primary biome matrix: [`../../matrix/terrain_biome/terrain_biome_migration_matrix_v1.md`](../../matrix/terrain_biome/terrain_biome_migration_matrix_v1.md)
- Material registry + tag + rule chain: [`material_tag_rule_system_v1.md`](material_tag_rule_system_v1.md) · [`../../matrix/terrain_biome/material_unification_matrix_v1.md`](../../matrix/terrain_biome/material_unification_matrix_v1.md)
