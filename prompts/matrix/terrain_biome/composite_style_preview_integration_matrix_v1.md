# Layered world gen + pixel preview — integration matrix `v1`

**STATUS:** ⏳ **Spec active — partial implementation** (multi-layer noise + ECS tile preview exist; direct-sample preview, resource patches, chunk preview cache, GPU path **not** built).  
**Paired designer Q:** [`../../designer_questions/terrain_world/composite_style_worldgen_v1.md`](../../designer_questions/terrain_world/composite_style_worldgen_v1.md)  
**Primary terrain/biome matrix:** [`terrain_biome_migration_matrix_v1.md`](terrain_biome_migration_matrix_v1.md)  
**Material / tag / rule unification:** [`material_unification_matrix_v1.md`](material_unification_matrix_v1.md) · designer [`../../designer_questions/terrain_world/material_tag_rule_system_v1.md`](../../designer_questions/terrain_world/material_tag_rule_system_v1.md)

Version: `v1.0.0`  
Scope: Map **scalar fields**, **tuning JSON**, **preview paths**, **UI exposure**, and **migration phases** for a **composite-style** *layered* generator + *fast* map preview, without duplicating the whole biome consolidation table.

> **Prompt use:** Verify every **Applied** row against `src/` with `rg` / `cargo check`. Mark **Pending** until implemented. Use **`ASK:`** for unset numbers. Do not invent gameplay constants here.

---

## 1. Layer field matrix

| Layer | Active symbol(s) | Sampler / builder | Seed / offset (today) | Status |
|:---|:---|:---|:---|:---:|
| **Elevation** | `Height` | `build_height_noise`, `sample_height_field` | `WorldGenParams.seed` via `TerrainNoiseProfile` path | Applied |
| **Moisture** | `Moisture` | `build_fbm_perlin` | `params.seed.wrapping_add(1)` | Applied |
| **Temperature** | `Temperature` | `build_fbm_perlin` | `params.seed.wrapping_add(2)` | Applied |
| **Aux (warp / detail)** | (in height pipeline) | `build_fbm_perlin` warp + detail | `warp_seed_offset`, `detail_seed_offset` in `NoiseSamplingTuning` | Partial |
| **Resources** | — *(proposed)* `ResourceField` | — | — | Pending |
| **Threat / bases** | — | — | — | Pending |
| **Tags (semantic)** | Interned `TagId` / `TagSet` on `ChunkCellMatrix` | multi-pass tagging | **Pending** | Pair [`material_tag_rule_system_v1.md`](../../designer_questions/terrain_world/material_tag_rule_system_v1.md) |
| **Materials (resolved)** | `MaterialId` via `resolve_material` | rule resolver + registry | **Pending** | Pair [`material_unification_matrix_v1.md`](material_unification_matrix_v1.md) |

**Files:** [`src/terrain/generation/world_generator_enhanced.rs`](../../../src/terrain/generation/world_generator_enhanced.rs), [`src/terrain/generation/terrain_noise.rs`](../../../src/terrain/generation/terrain_noise.rs)

**Preview follow-up:** tag-overlay and material-color preview modes replace hardcoded `TerrainClass` RGBA in [`world_preview.rs`](../../../src/gui/editor/world_preview.rs) per [`material_unification_matrix_v1.md`](material_unification_matrix_v1.md) §6.

---

## 2. Tuning JSON ↔ Rust matrix

**Overlay path:** `assets/config/world_gen_tuning.json` — constant `WORLD_GEN_TUNING_JSON_PATH` in [`world_generator_enhanced.rs`](../../../src/terrain/generation/world_generator_enhanced.rs).  
**Serde type:** [`WorldGenTuningOverlay`](../../../src/terrain/generation/tuning_io.rs) — `noise_sampling: Option<NoiseSamplingTuning>`, `biome_tuning: Option<BiomeTuning>`.  
**Example:** [`assets/config/world_gen_tuning.example.json`](../../../assets/config/world_gen_tuning.example.json)

| JSON key | Rust type | Merged into | Status |
|:---|:---|:---|:---:|
| `noise_sampling` | `NoiseSamplingTuning` | `WorldGenParams.noise_sampling` | Applied |
| `biome_tuning` | `BiomeTuning` | `WorldGenParams.biome_tuning` | Applied |

**Gap (P1):** explicit **`moisture_seed_offset`** / **`temperature_seed_offset`** in `NoiseSamplingTuning` (today hardcoded `+1` / `+2` in `generate_world`) — **Pending**; goal is tunable parity without behavior drift when defaults match.

---

## 3. Preview pipeline matrix

| Path | File / symbol | Cost driver | Typical use | Status |
|:---|:---|:---|:---|:---:|
| **ECS scan → texture** | [`world_preview.rs`](../../../src/gui/editor/world_preview.rs) `update_world_preview_texture` | Iterates all `TileMarker` entities; writes texels from `Transform` | In-editor preview after **Generate World** | Applied |
| **Direct scalar sample → buffer** | *(proposed)* parallel raster, no tile entities | Per-pixel noise samples | Pre-gen / zoomed LOD / headless thumbs | Pending |
| **GPU compute** | *(future)* | GPU bandwidth | Live parameter drag | Pending |

**Asymmetry:** ECS path **requires spawned tiles**; direct path matches the *composite-style* “for each preview pixel, sample fields” model ([designer doc §6](../../designer_questions/terrain_world/composite_style_worldgen_v1.md)).

---

## 4. Resolution / preview LOD matrix

| Topic | Code / note | Status |
|:---|:---|:---:|
| Default preview size | `WorldPreviewTexture` default **512×512** (`width`/`height` fields); initialized from `WorldGenParams.width/height` in `init_world_preview_texture` | Applied |
| **1 px = N tiles** (downsample) | Not a first-class tunable yet — **ASK:** | Pending |
| **LOD pyramid** (LOD0/LOD1/LOD2) | Couple to [`simulation_lod_v1.md`](../../designer_questions/terrain_world/simulation_lod_v1.md) chunk grid — **Pending** | Pending |

---

## 5. Multithread / GPU matrix

| Strategy | Dependency | Notes | Status |
|:---|:---|:---|:---|
| Sequential CPU | None | Simplest | Applied (implicit in ECS scan) |
| **`rayon` parallel** | Cargo `rayon` | Pixel/chunk parallel fill for direct preview | Pending |
| **Compute shader** | Bevy render capabilities | See [`bevy_0_18_migration_plan.md`](../engine_bevy/bevy_0_18_migration_plan.md) | Pending |

---

## 6. Plugin integration points

| Plugin | Role | Preview / gen link | Status |
|:---|:---|:---|:---|
| `WorldPreviewPlugin` | Registers `WorldPreviewTexture`, `update_world_preview_texture`, `display_world_preview` | ECS-scan preview | Applied |
| `WorldGenUiPlugin` | `world_gen_ui_system`, `WorldGenUiState`, `PreviewMode` | Chooses preview mode | Applied |
| `WorldGenerationToolsUiPlugin` | Editor tooling aggregate (see `world_generation_plugin.rs`) | F8 tooling | Applied |
| **`WorldPreviewSamplerPlugin`** *(proposed)* | Direct-sample + optional `rayon` | Fast LOD previews | Pending |

**File:** [`src/terrain/generation/world_generation_plugin.rs`](../../../src/terrain/generation/world_generation_plugin.rs)

---

## 7. Cache & invalidation matrix

| Event | Action | Status |
|:---|:---|:---:|
| **Seed** change | Full preview rebuild | Applied (user must regenerate world + texture cleared by scan) |
| **`WorldGenTuningOverlay` reload** | Invalidate preview if tiles reflect old params — **currently** params update on reload but **tiles do not** until regen | Partial |
| **Per-chunk cache** *(proposed)* | `HashMap<ChunkCoord, PreviewTile>` + dirty flags | Pending |
| **Ghost / seam** | Cross-chunk sampling must respect neighbor continuity — pair [`chunks_streaming_v1.md`](../../designer_questions/terrain_world/chunks_streaming_v1.md) | Pending |

---

## 8. Reflect / UI exposure matrix (`WorldGenParams`)

Subset of fields — **Applied** if visible in [`world_gen_ui.rs`](../../../src/gui/editor/world_gen_ui.rs); **JSON only** if in `WorldGenTuningOverlay` but not necessarily all in egui.

| Field / group | Egui (`world_gen_ui_system`) | `world_gen_tuning.json` | Status |
|:---|:---:|:---:|:---:|
| `width`, `height`, `seed` | ✅ | — | Partial |
| `num_regions`, `region_method`, `region_iterations` | ✅ | — | Applied |
| `height_noise_profile`, `noise_scale`, octaves, lacunarity, persistence | ✅ | — | Applied |
| `height_curve_exponent`, `domain_warp_strength`, `terrain_detail_mix` | ✅ | — | Applied |
| `moisture_bias`, `temperature_bias` | ✅ | — | Applied |
| `noise_sampling` (*full struct*) | ✅ (collapsible) | ✅ `noise_sampling` | Applied |
| `biome_tuning` | ✅ **subset** of sliders | ✅ `biome_tuning` **full** struct | Partial |
| `river_count`, `lake_count`, `mountain_threshold`, `island_mode`, `island_falloff` | ✅ | — | Applied |
| `PreviewMode` | ✅ | — | Applied |

**Gap:** egui **Biome generator coupling** does not expose all `BiomeTuning` soft-weight fields (e.g. `marine_height_sensitivity`, `alpine_*`, `arid_*`, `wetland_*`, `boreal_*`, `temperate_*`) — desktop JSON editor is the full-field path today ([`08_world_gen_desktop_tool_v1.md`](../../designer_questions/tools_ui/spec/08_world_gen_desktop_tool_v1.md)).

---

## 9. Determinism / test matrix

| Test | Pair doc | Status |
|:---|:---|:---:|
| Same **seed** + params ⇒ same **tile set** / checksum | [`implementation_questions_v1.md`](../../designer_questions/terrain_world/implementation_questions_v1.md) items **33–34** | Partial |
| Same seed ⇒ same **preview image hash** (direct path, when exists) | This matrix §3 | Pending |
| Chunk load order | Item **33** | Partial |

---

## 10. Migration phases (checklist)

| Phase | Deliverable | Status |
|:---:|:---|:---:|
| **P1** | Add `moisture_seed_offset` / `temperature_seed_offset` to `NoiseSamplingTuning`; wire `generate_world`; extend example JSON + asset editor | Pending |
| **P2** | `ResourceField` + serializable patch layer; tile resolver extension | Pending |
| **P3** | `WorldPreviewSamplerPlugin` (or equivalent) + CPU parallel buffer + `Image` upload; LOD pyramid | Pending |
| **P4** | Chunk-keyed preview cache + dirty invalidation | Pending |
| **P5** | GPU compute preview (optional) | Pending |

---

## 11. Cross-doc links

| Doc | Purpose |
|:---|:---|
| [`terrain_biome_migration_matrix_v1.md`](terrain_biome_migration_matrix_v1.md) | Canonical `TerrainClass` / `BiomeWeights` / UI plugin table |
| [`composite_style_worldgen_v1.md`](../../designer_questions/terrain_world/composite_style_worldgen_v1.md) | Designer narrative + open questions |
| [`serialization_hybrid_migration_matrix_v1.md`](../serialization/serialization_hybrid_migration_matrix_v1.md) | If preview or chunk caches persist |
| [`08_world_gen_desktop_tool_v1.md`](../../designer_questions/tools_ui/spec/08_world_gen_desktop_tool_v1.md) | Python asset editor World Gen page |
| [`01_world_generation_pipeline.md`](../../designer_questions/terrain_world/spec/01_world_generation_pipeline.md) | Pipeline stub |
| [`material_unification_matrix_v1.md`](material_unification_matrix_v1.md) | Registry, tags, rules, `ChunkCellMatrix`, materialize pass |
| [`material_tag_rule_system_v1.md`](../../designer_questions/terrain_world/material_tag_rule_system_v1.md) | Designer narrative |

---

## 12. Prompt fragment (subsequent agent pass)

1. Read this matrix + paired [`composite_style_worldgen_v1.md`](../../designer_questions/terrain_world/composite_style_worldgen_v1.md).  
2. Implement **P1** without changing default noise output (offsets default to current `+1` / `+2` semantics).  
3. Add **direct-sample preview** prototype behind a feature flag or separate schedule before replacing ECS preview.  
4. Reconcile `WorldPreviewTexture` size with **downsample scale** 📎.  
5. Update **Applied / Pending** rows in this file when code lands.
