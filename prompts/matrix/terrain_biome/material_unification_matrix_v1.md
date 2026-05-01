# Material / tag / rule unification — integration matrix `v1`

**STATUS:** **U5 Applied** — `MaterializedChunk`, `MaterialUnificationPlugin`, registry-backed biome preview + `PreviewMode::Tag`; **U6 / U7 Applied (tilemap)** — optional `bevy_tilemap_adapter` → `TilemapAdapterPlugin` + `ChunkTilemaps` (terrain / overlay / resource layers, F8 toggles when feature on); **U7 Applied** — `ChunkDependency` / `ChunkDirty`, partial rebuild, `WorldProfile` loader, optional `dev_tools` `RuleTrace`. `generate_world` tile path unchanged; chunk ECS pipeline runs when `Chunk` + `ChunkCellMatrix` entities exist.

**Paired designer Q:** [`../../designer_questions/terrain_world/material_tag_rule_system_v1.md`](../../designer_questions/terrain_world/material_tag_rule_system_v1.md)
**Primary biome matrix:** [`terrain_biome_migration_matrix_v1.md`](terrain_biome_migration_matrix_v1.md)
**Layered preview matrix:** [`composite_style_preview_integration_matrix_v1.md`](composite_style_preview_integration_matrix_v1.md)
**Asset format matrix:** [`../assets/bevy_asset_config_migration_matrix_v1.md`](../assets/bevy_asset_config_migration_matrix_v1.md)
**Save format matrix:** [`../serialization/serialization_hybrid_migration_matrix_v1.md`](../serialization/serialization_hybrid_migration_matrix_v1.md)

Version: `v1.1.0` — additive §§13–18 (outline-inspired invalidation, passes, layers, perf, LLM policy, packs); phases **U7** optional.
Scope: One canonical chain `noise → ChunkCellMatrix → multi-pass tags → resolver → MaterialId → render/sim`. **No duplication** of existing types: `TerrainClass` is reused as `MaterialFamily`; `BiomeWeights`, `BiomeTuning`, `classify_biome`, `WorldGenTuningOverlay`, `WorldPreviewTexture` all kept and called by the new chain.

> **Prompt use:** verify Applied / Pending against `src/` with `rg`/`cargo check`. Use `ASK:` for unset numbers (registry caps, bitset width). Saves store **names**, not raw ids.

---

## 1. Concept ↔ symbol matrix

| Concept | Active symbol(s) | File | Status |
|:---|:---|:---|:---:|
| **MaterialFamily** (compile-time) | `TerrainClass` (16 variants) | [`src/terrain/biome.rs`](../../../src/terrain/biome.rs) | Applied — alias as `MaterialFamily` (no rename needed) |
| **Soft biome blend** | `BiomeWeights`, `BiomeId`, `BiomeId::primary` | [`src/terrain/biome.rs`](../../../src/terrain/biome.rs) | Applied |
| **Soil / surface mix** | `TerrainSurfaceMix`, `TileEnvironmentProfile` | [`src/terrain/biome.rs`](../../../src/terrain/biome.rs) | Applied |
| **Threshold tuning** | `BiomeTuning` + `world_gen_tuning.json` | [`src/terrain/biome.rs`](../../../src/terrain/biome.rs), [`src/terrain/generation/tuning_io.rs`](../../../src/terrain/generation/tuning_io.rs) | Applied |
| **Classifier** | `classify_biome(height, moisture, temperature) -> BiomeClassification` | [`src/terrain/biome.rs`](../../../src/terrain/biome.rs) | Applied |
| **Ecology profile** | `EcologicalSuitability`, `FloraType`, `CropType`, `FlowerType` | [`src/terrain/ecology.rs`](../../../src/terrain/ecology.rs) | Applied |
| **MaterialId(u16)** (runtime variant) | `MaterialId` | [`src/terrain/material/registry.rs`](../../../src/terrain/material/registry.rs) | **Applied** — JSON + `MaterialRegistryLoader` + `init_asset` via [`MaterialUnificationPlugin`](../../../src/systems/terrain/material_plugin.rs) |
| **MaterialDef + MaterialRegistry** | `MaterialDef`, `MaterialRegistry`, `MaterialRegistryLoader` | [`src/terrain/material/registry.rs`](../../../src/terrain/material/registry.rs) + example JSON | **Applied** — same plugin |
| **TagId(u16) + TagRegistry + TagSet** | `TagId`, `TagSet`, `TagRegistry`, `TagRegistryLoader` | [`src/terrain/material/tags.rs`](../../../src/terrain/material/tags.rs) | **Applied** — `*.tag_registry.json` loader + startup embed |
| **MaterialRule + RuleSet + resolver** | `MaterialRule`, `RuleSet`, `RuleSetLoader`, `resolve_material(...)` | [`src/terrain/material/rules.rs`](../../../src/terrain/material/rules.rs) + [`resolver.rs`](../../../src/terrain/material/resolver.rs) | **Applied** |
| **ChunkCellMatrix** (SoA per-chunk grid) | `ChunkCellMatrix` | [`src/terrain/generation/cell_matrix.rs`](../../../src/terrain/generation/cell_matrix.rs) | **Applied** — passes 1–6 (pass 5 stub) |
| **Multi-pass tagging** | `passes::{fill_fields, apply_threshold_tags, …}` | [`src/terrain/generation/passes/`](../../../src/terrain/generation/passes/) | **Applied** (pass 5 agent stub only) |
| **MaterializedChunk** (ECS component) | `MaterializedChunk` | [`src/terrain/material/runtime.rs`](../../../src/terrain/material/runtime.rs) | **Applied** — `size` + `materials`; filled by `materialize_chunks` |
| **Material plugin (assets + hot reload)** | `MaterialUnificationPlugin`, `TerrainRegistriesHandles` | [`src/systems/terrain/material_plugin.rs`](../../../src/systems/terrain/material_plugin.rs) | **Applied** |
| **Preview color mapping** | `preview_biome_rgba_for_tile` + `family_default_material_def` | [`src/gui/editor/world_preview.rs`](../../../src/gui/editor/world_preview.rs) | **Applied** — `MaterialDef.preview_color`; legacy LUT `#[deprecated]` |
| **Tilemap renderer** | `TilemapAdapterPlugin` + `ChunkTilemaps` → `bevy_ecs_tilemap` (`TileTextureIndex`) | [`src/render/tilemap_adapter.rs`](../../../src/render/tilemap_adapter.rs) | **Applied** — feature `bevy_tilemap_adapter`; `bevy_ecs_tilemap` **0.18.1** with `default-features = false` |

---

## 2. On-disk layout matrix

| File | Format | Designer-edited via | Engine consumer | Status |
|:---|:---|:---|:---|:---:|
| `assets/config/world_gen_tuning.json` | JSON | F8 egui panel + asset editor World Gen page | `WorldGenTuningOverlay` | Applied |
| `assets/config/world_gen_tuning.example.json` | JSON | committed seed | n/a | Applied |
| `assets/config/terrain/material_registry.json` | JSON | asset editor **Materials** page | *(proposed)* `Assets<MaterialRegistry>` | Pending |
| `assets/config/terrain/material_registry.example.json` | JSON | committed seed | n/a | Applied |
| `assets/config/terrain/tag_registry.json` | JSON | asset editor **Tags** page | *(proposed)* `Assets<TagRegistry>` | Pending |
| `assets/config/terrain/tag_registry.example.json` | JSON | committed seed | n/a | Applied |
| `assets/config/terrain/material_rules.ron` | RON | asset editor **Rules** page | *(proposed)* `Assets<RuleSet>` | Pending |
| `assets/config/terrain/material_rules.example.ron` | RON | committed seed | n/a | Applied |

**Format rule (locked):**

- **JSON** for flat designer-edited tables (registries) — matches existing asset-editor habits and `world_gen_tuning.json`.
- **RON** for the rule DSL — supports comments, tagged enums, nested predicates; aligns with [`bevy_asset_config_migration_matrix_v1.md`](../assets/bevy_asset_config_migration_matrix_v1.md) (**Terrain registry** subsection — target `AssetLoader` extensions) and [`serialization_hybrid_migration_matrix_v1.md`](../serialization/serialization_hybrid_migration_matrix_v1.md) ("move away from pure JSON for hand-edited heavy data").

---

## 3. Pipeline status matrix

| Pass | Inputs | Outputs | Calls | Status |
|:---:|:---|:---|:---|:---:|
| 1. Fields | seed, params | `ChunkCellMatrix.{elevation,moisture,temperature}` | `build_world_noise_kernels` + `fill_fields` (shared with `generate_world`) | **Applied** |
| 2. Threshold tags | fields + `BiomeTuning` (+ `threshold_tag_names` in tuning 📎 `tag_tuning` §42) | `TagSet` cells | `apply_threshold_tags` | **Applied** |
| 3. Biome / family | fields + `BiomeTuning` | `BiomeWeights`, `MaterialFamily` (`TerrainClass`), optional primary `BiomeId` tag | `classify_biome` via `classify_cells` | **Applied** |
| 4. Hydrology / erosion | hydrology events | `flooded`, `eroded`, `silted` tags | `apply_hydrology` + `terrain::generation::hydrology` (D8, priority-flood fill, accumulation); ECS uses `compute_hydrology_world` | **Applied** |
| 5. Agent / scenario overlay | scenario data, agent commands | scenario tags | `apply_agent_overlay` stub | **Partial** (stub) |
| 6. Materialize | family, weights, tags, rules, registry | `MaterializedChunkData.materials` | `resolve_material` via `materialize` | **Applied** |

---

## 4. Resolver contract matrix

| Property | Specification | Status |
|:---|:---|:---:|
| Signature | `resolve_material(family, &BiomeWeights, TagSet, &RuleSet, &MaterialRegistry, &TagRegistry) -> MaterialId` | Applied |
| Tie-break | priority desc, then `rule_index` asc (file order) — strict deterministic | Applied |
| Fallback | registry-declared family default; never silent `MaterialId(0)` (panics if family empty) | Applied |
| Memoization | optional cache key `(family, tag_set, quantize8(weights))` | 📎 **§44** |
| Pre-sort | rules sorted at load; resolver assumes sorted slice | Applied |

---

## 5. ECS integration matrix

| ECS item | Kind | Spawn site | Reader | Status |
|:---|:---|:---|:---|:---:|
| `Chunk { coord }` | Component | chunk loader | preview, `materialize_chunks` | Partial (no streaming loader yet; type in [`chunk.rs`](../../../src/terrain/generation/chunk.rs)) |
| `ChunkCellMatrix` | Component | paired with `Chunk` | passes 1–6 | **Applied** |
| `MaterializedChunk { size, materials }` | Component | `materialize_chunks` | preview | **Applied** |
| `MaterialUnificationPlugin` | Plugin | [`engine_with_worldgen.rs`](../../../src/engine/engine_with_worldgen.rs) | `materialize_chunks` schedule | **Applied** |

---

## 6. Preview integration matrix

| Mode | Source | Color mapping | Status |
|:---|:---|:---|:---:|
| Height / Moisture / Temperature | existing tile components | grey / blue / red ramps | Applied (`world_preview.rs`) |
| Biome | `TerrainClass` + `MaterialRegistry` | `MaterialDef.preview_color` (per chunk cell when `MaterializedChunk` covers tile; else family default) | **Applied** |
| Regions | region index | placeholder | Partial |
| **Tag overlay** | `TagSet` mask | `PreviewMode::Tag(TagId)` → `TAG_OVERLAY_HIGHLIGHT` | **Applied** |
| **Material color** | `MaterialId → MaterialDef.preview_color` | data-driven | **Applied** |
| **Direct-sample** | `ChunkCellMatrix` | runs without spawned tiles | Pending — pair `composite_style_preview_integration_matrix_v1.md` **P3** |

---

## 7. Hot reload matrix

| Asset | Loader | Trigger | Re-run scope | Status |
|:---|:---|:---|:---|:---:|
| `WorldGenTuningOverlay` | manual + future watcher | F8 button / file watch | re-derive `BiomeTuning` + tag thresholds; rerun passes 2–6 | Partial (manual today) |
| `MaterialRegistry` | `MaterialRegistryLoader`, `*.material_registry.json` | file watch | rebuild name→id; rerun pass 6 only | **Applied** |
| `TagRegistry` | `TagRegistryLoader`, `*.tag_registry.json` | file watch | rebuild name→id; rerun passes 2–6 | **Applied** |
| `RuleSet` | `RuleSetLoader`, `*.material_rules.ron` | file watch | re-sort rules; rerun pass 6 only | **Applied** |

**Invariant:** F8 panel + asset editor **only edit files**. The engine watches. No second mutation path.

---

## 8. Determinism / save matrix

| Topic | Decision | Status |
|:---|:---|:---:|
| Save wire format | `MaterialDef.name` strings (not raw `MaterialId`) | Decided — awaits backlog wave **S** / `serialization_hybrid_migration_matrix_v1.md` |
| Schema versions | `schema_version: u32` per file by default; registry of schemas when many schemas share lifecycle / optimization constraints | Decided — finalize in wave **S** |
| Chunk hash | `hash(world_seed, chunk_xy, registry_version, rules_version, tuning_version)`; keep footprint reliable and proportional to scheduling priority + rebuild cost | Decided |
| Rule order | logical clean ordering; debug traces expose tie-breaks so questionable priorities can be tuned | Decided — tie debug via `RuleTrace` (`dev_tools`) |

---

## 9. UI exposure matrix

| Surface | Current | Target | Status |
|:---|:---|:---|:---:|
| F8 egui panel | `WorldGenParams`, `BiomeTuning`, `PreviewMode` inc. **`Tag`**, tag combo | + registry-backed biome colors | **Applied** |
| Asset editor World Gen page | Overview / Noise / Biome / Full JSON tabs | separate **Materials**, **Tags**, **Rules** nav pages (`terrain_registry_pages.py`) | Applied |
| Inspector (`bevy-inspector-egui`) | n/a | optional — view registries as tables | Confirmed — Planned |

---

## 10. Migration phases (U-series)

| Phase | Deliverable | Phase in this plan | Status |
|:---:|:---|:---|:---:|
| **U0** | This matrix + paired designer doc + cross-links | Phase 1 (markdown) | Applied |
| **U1** | `material_registry.example.json`, `tag_registry.example.json`, `material_rules.example.ron` | Phase 2 (assets) | Applied |
| **U2** | Asset-editor **Materials**, **Tags**, **Rules** pages + nav + repo_paths | Phase 3 (Python) | Applied |
| **U3** | Rust scaffolding: `MaterialId`, `MaterialDef`, `MaterialRegistry`, `TagId`, `TagRegistry`, `TagSet`, `MaterialRule`, `RuleSet`, `resolve_material` (loader + assets) | follow-up plan | **Applied** |
| **U4** | `ChunkCellMatrix` + multi-pass pipeline (pass 1–4, 6 wired; pass 5 stub) | follow-up plan | **Applied** |
| **U5** | `MaterializedChunk` + `MaterialUnificationPlugin` + preview via `MaterialDef.preview_color` + `PreviewMode::Tag` | follow-up plan | **Applied** |
| **U6** | Optional `bevy_ecs_tilemap` adapter (multi-layer: terrain z0 / overlay z10 / resources z20; feature-gated); GPU compute preview out of scope | follow-up plan | **Applied** (ECS + F8 layer toggles with `bevy_tilemap_adapter`) |
| **U7** *(optional)* | Invalidation (`ChunkDependency` / `ChunkDirty`), partial chunk rebuild after `AssetEventSystems`, multi-layer tilemaps, `WorldProfile` / `default.world_profile.ron`, optional `dev_tools` `RuleTrace` — see §§13–18 | after U5 | **Applied** |

---

## 11. Cross-doc links

| Doc | Purpose |
|:---|:---|
| [`material_tag_rule_system_v1.md`](../../designer_questions/terrain_world/material_tag_rule_system_v1.md) | Designer narrative + open questions |
| [`composite_style_worldgen_v1.md`](../../designer_questions/terrain_world/composite_style_worldgen_v1.md) | Layered fields + preview |
| [`composite_style_preview_integration_matrix_v1.md`](composite_style_preview_integration_matrix_v1.md) | Preview path + P1–P5 |
| [`terrain_biome_migration_matrix_v1.md`](terrain_biome_migration_matrix_v1.md) | Canonical `TerrainClass` / `BiomeWeights`; its **Cross-matrix: procedural pipeline downstream** points to **this** doc §§13–18 |
| [`bevy_asset_config_migration_matrix_v1.md`](../assets/bevy_asset_config_migration_matrix_v1.md) | RON/JSON format direction |
| [`serialization_hybrid_migration_matrix_v1.md`](../serialization/serialization_hybrid_migration_matrix_v1.md) | Save name-vs-id policy |
| [`08_world_gen_desktop_tool_v1.md`](../../designer_questions/tools_ui/spec/08_world_gen_desktop_tool_v1.md) | Asset editor World Gen page |
| [`implementation_questions_v1.md`](../../designer_questions/terrain_world/implementation_questions_v1.md) | Engineering checklist (items **42–78**) |
| [`procedural_world_pipeline_reference_outline_v1.md`](../../designer_questions/terrain_world/procedural_world_pipeline_reference_outline_v1.md) | Non-authoritative brainstorm (outline → questions + matrix rows) |
| [`../../guides/terrain_unification_runbook_v1.md`](../../guides/terrain_unification_runbook_v1.md) | **Execution orchestrator** for Rust phases U3–U7 (invariants, loop protocol, halt rules) |
| [`../../guides/world_assets_tools_rulebook_v1.md`](../../guides/world_assets_tools_rulebook_v1.md) | **World tools parity** — `world_generator` vs main, production checklist, tests, U6/U7 gate |
| [`runbook/README.md`](runbook/README.md) | Index of per-phase atomic step packs (`u3_steps_v1.md` … `u7_steps_v1.md`) |
| [`../../guides/system_runbook_authoring_meta_v1.md`](../../guides/system_runbook_authoring_meta_v1.md) | Meta-runbook (authoring template for sibling system runbooks); **§12** paired norms |
| [`../../guides/bevy_asset_terrain_runbook_v1.md`](../../guides/bevy_asset_terrain_runbook_v1.md) | **Paired** Bevy terrain registry policy + gates (A1–A3); step packs [`runbook/README.md`](runbook/README.md) |
| [`../../guides/terrain_paired_runbooks_queue_v1.md`](../../guides/terrain_paired_runbooks_queue_v1.md) | **Paired** terrain-adjacent runbooks — Q0–Q6 creation queue + sync gates |
| [`../../designer_questions/terrain_world/llm_world_evolution_reference_outline_v1.md`](../../designer_questions/terrain_world/llm_world_evolution_reference_outline_v1.md) | Non-authoritative LLM rule-evolution / memory-tiers outline |

---

## 12. Prompt fragment (subsequent agent pass)

1. Read this matrix + paired [`material_tag_rule_system_v1.md`](../../designer_questions/terrain_world/material_tag_rule_system_v1.md).
2. **U1** example assets are committed under `assets/config/terrain/` — copy to active filenames when bootstrapping.
3. **U2** asset-editor pages live in `src/utils/asset_tools/src/pages/terrain_registry_pages.py` + `main_window.py` nav entries.
4. Plan **U3–U5** as a fresh implementation plan; do not silently introduce a second classifier — pass 3 must call existing `classify_biome`.
5. Update **Status** rows here as code lands. Saves remain **name-based** until the registry stabilizes.
6. For hot-reload granularity, partial rebuild, and multi-layer rendering, reconcile §§13–18 with code and **`implementation_questions_v1.md` 49–78**.

---

## 13. Invalidation / partial rebuild (outline-inspired)

| Dependency change | Typical re-run (chunk-scoped) | Full world regen 📎 |
|:---|:---|:---|
| `MaterialRegistry` file | Rebuild name→`MaterialId`; pass **6**; optionally atlas/texture remap only | If stable id contract breaks (`ASK:` **item 52**) |
| `TagRegistry` file | Name→`TagId`; passes **2–6** | Rare |
| `RuleSet` file | Re-sort; pass **6** (tags unchanged) | If rules reference removed tag ids |
| `world_gen_tuning` / `tag_tuning` | Passes **2–6** (field thresholds) | n/a |
| `ChunkCellMatrix` content (noise seed) | Passes **1–6** | — |
| **`tag_pipeline_version` bump** | From pass index *N* downward | Designer-defined |

**ECS (U7):** chunk components `ChunkDependency` + `ChunkDirty`; asset `AssetEvent` → `PostUpdate` marking after `AssetEventSystems`; `rebuild_dirty_chunks` runs lowest pass through pass 6; hashes refreshed on rebuild.

---

## 14. Tag expansion pass detail (beyond §3 table)

**Ordering intent:** biome / family classification remains canonical before higher-level tag tuning consumes combinations; combo tags derive from logical simulation concepts and may feed later resource / plant-growth probability tables.

| Pass | Outline name | Repo anchor | Status |
|:---:|:---|:---|:---:|
| 1 | Raw field tagging | §3 pass 2 + `BiomeTuning` / `tag_tuning` | Pending |
| 2 | Derived combo tags (`wet`+`lowland`→…) | Data-driven combo table is canonical; derive logical systems (wet lowlands, rocky edges, etc.) for downstream resource / ecology tables | Decided — author data rows |
| 3 | Spatial: clusters / edges / gradients | Neighbor reads + ghost bands required; spatial pass may consume all type/combination inputs for rocks, crystals, landforms, and plant-pattern phenomena | Decided — pair with chunk streaming |
| 4 | Agent / LLM overlay | Pass 5 in §3; audited writes + authority (**§47**, **items 71–74**); unresolved choices bubble to designer TODOs, not invented behavior | Pending |

---

## 15. Multi-layer tilemap / render stack

| Layer | Z (convention 📎) | Data source | Visibility | Status |
|:---|:---:|:---|:---|:---:|
| Terrain | 0 | `MaterializedChunk` → `TileTextureIndex` via [`TilemapAdapterPlugin`](../../../src/render/tilemap_adapter.rs) + [`ChunkTilemaps`](../../../src/render/tilemap_adapter.rs) | feature `bevy_tilemap_adapter` + `TilemapLayerVisibility` | **Applied** |
| Overlay / debug | 10 | `ChunkCellMatrix` + [`PreviewMode`](../../../src/gui/editor/world_gen_ui.rs) | F8 “Tilemap layers” when feature on | **Partial** |
| Resources | 20 | [`MaterializedResources`](../../../src/terrain/material/runtime.rs) (mirrors terrain until **`ASK:` §62**) | F8 toggle | **Partial** |

**Constraint:** shared tile grid extent per chunk; layers may **not** share one `TileStorage` if the renderer requires independent textures — engine chooses one `TilemapId` per layer per chunk vs compositing **ASK:** **item 64**.

---

## 16. Performance / threading / cache

| Mechanism | Role | Status |
|:---|:---|:---:|
| `ChunkDirty` bitmask | Skip untouched chunks; marking only on `AssetEvent` / tuning / noise bucket change | **Applied** |
| `ChunkCache` coord → `{matrix, materialized, hashes}` | In-memory hot path; optional disk cache is allowed when footprint / scheduling cost justifies it | Decided — implement later |
| `TaskPool` / off-thread generation | Noise/field fill writes into `ChunkCellMatrix` buffer; multithread aggressively, with LOD + scheduling-priority trees that adjust under load | Decided — implement later |
| Diff updates | Only changed tile indices sent to `TileStorage`; smooth-transition semantics still need a one-line renderer contract | Partial — `TODO:` designer brief §4 |
| Partial rebuild dispatcher | Lowest dirty pass → pass 6 per chunk; `PostUpdate` after asset events | **Applied** |

---

## 17. LLM / agent rule editor (non-binding policy)

| Concern | Guideline | Matrix / checklist |
|:---|:---|:---|
| Determinism | Same seed + same **committed** rule/tag/registry files ⇒ identical `ChunkCellMatrix` + materialize | §8, **item 71** |
| Authority | Agent edits **files** or append-only overlay; engine reload path identical to designer saves | `material_tag_rule` §8 |
| Telemetry | Distribution metrics drive suggestions; implementation may stub (**item 73**) | — |
| Audit | Log rule diffs (dev JSONL **item 72**) | **Partial** — `RuleTrace` (`--features dev_tools`) only |

---

## 18. Extensibility — profiles & packs

| Mechanism | Intent | Status |
|:---|:---|:---:|
| **World profile** | One `*.world_profile.ron` bundles registry + tags + rules paths (+ optional tuning path) | **Applied** — see `assets/config/terrain/profiles/default.world_profile.ron`, `WorldProfileLoader` |
| **Biome / rules pack** | Drop-in folder merged at load or explicit `#include` **ASK:** **item 77**; loaded biome packs must expose on/off switches via menu item + config key | Decided — UX key/name pending |
| **Modded `RuleSet`** | Stable rule `name` keys for overrides; numeric priority bands reserved for core vs mods **ASK:** | Pending |
