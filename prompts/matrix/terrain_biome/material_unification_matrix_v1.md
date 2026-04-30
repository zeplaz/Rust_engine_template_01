# Material / tag / rule unification — integration matrix `v1`

**STATUS:** ⏳ **Spec active — U3 scaffolding Applied** (`src/terrain/material/*`); `ChunkCellMatrix` / pipeline passes still ahead per **U4+**. Existing biome / world-gen stack is unchanged; new registry, tag, rule layers slot **on top** without parallel data models.

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
| **MaterialId(u16)** (runtime variant) | `MaterialId` | [`src/terrain/material/registry.rs`](../../../src/terrain/material/registry.rs) | Partial — JSON + Bevy `AssetLoader` (*.material_registry.json); plugin registration **U5** |
| **MaterialDef + MaterialRegistry** | `MaterialDef`, `MaterialRegistry`, `MaterialRegistryLoader` | [`src/terrain/material/registry.rs`](../../../src/terrain/material/registry.rs) + `assets/config/terrain/material_registry.json` | Partial — same as `MaterialId` row |
| **TagId(u16) + TagRegistry + TagSet** | `TagId`, `TagSet`, `TagRegistry` | [`src/terrain/material/tags.rs`](../../../src/terrain/material/tags.rs) + `assets/config/terrain/tag_registry.json` | Partial — JSON load path only; Bevy loader **U5** |
| **MaterialRule + RuleSet + resolver** | `MaterialRule`, `RuleSet`, `resolve_material(...)` | [`src/terrain/material/rules.rs`](../../../src/terrain/material/rules.rs) + [`src/terrain/material/resolver.rs`](../../../src/terrain/material/resolver.rs) + `material_rules.ron` | Applied |
| **ChunkCellMatrix** (SoA per-chunk grid) | `ChunkCellMatrix` | [`src/terrain/generation/cell_matrix.rs`](../../../src/terrain/generation/cell_matrix.rs) | Partial — alloc + `idx` only (**U4-S01**); passes **U4-S02+** |
| **Multi-pass tagging** | *(proposed)* | `src/terrain/generation/passes.rs` | Pending |
| **MaterializedChunk** (ECS component) | *(proposed)* | `src/terrain/material/runtime.rs` | Pending |
| **Material plugin (assets + hot reload)** | *(proposed)* `MaterialUnificationPlugin` | `src/systems/terrain/material_plugin.rs` | Pending |
| **Preview color mapping** | `update_world_preview_texture` (today: hardcoded RGBA per `TerrainClass`) | [`src/gui/editor/world_preview.rs`](../../../src/gui/editor/world_preview.rs) | Partial — convert to `MaterialDef.preview_color` lookup |
| **Tilemap renderer** | `bevy_ecs_tilemap` (not in `Cargo.toml`) | n/a | Pending — deferred behind Bevy 0.18 migration plan |

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
| 2. Threshold tags | fields + `BiomeTuning` (+ new `tag_tuning` 📎) | `TagSet` cells (e.g. `lowland`, `wet`, `hot`) | new (data-driven from JSON) | Pending |
| 3. Biome / family | fields + `BiomeTuning` | `BiomeWeights`, `MaterialFamily` (`TerrainClass`), biome tags | existing `classify_biome` | Pending — wire into pipeline (no second classifier) |
| 4. Hydrology / erosion | hydrology events | `flooded`, `eroded`, `silted` tags | future hydrology pass | Pending |
| 5. Agent / scenario overlay | scenario data, agent commands | scenario tags | new | Pending |
| 6. Materialize | family, weights, tags, rules, registry | `MaterializedChunk.materials: Vec<MaterialId>` | new `resolve_material` | Pending |

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
| `Chunk { coord }` | Component | chunk loader | preview, sim | Partial (chunk story still spec-ahead per `chunks_streaming_v1.md`) |
| `ChunkCellMatrix` | Component (or `Resource` for global preview) | Pass 1 | passes 2–6 | Pending |
| `MaterializedChunk { materials }` | Component | Pass 6 | preview, tilemap, sim | Pending |
| `MaterialUnificationPlugin` | Plugin | app build | system schedule | Pending |

---

## 6. Preview integration matrix

| Mode | Source | Color mapping | Status |
|:---|:---|:---|:---:|
| Height / Moisture / Temperature | existing tile components | grey / blue / red ramps | Applied (`world_preview.rs`) |
| Biome | `TerrainClass` | hardcoded RGBA per variant | Applied — to be replaced by `MaterialDef.preview_color` (P3 below) |
| Regions | region index | placeholder | Partial |
| **Tag overlay** | `TagSet` mask | single-tag highlight | Pending — new `PreviewMode::Tag(TagId)` |
| **Material color** | `MaterialId → MaterialDef.preview_color` | data-driven | Pending |
| **Direct-sample** | `ChunkCellMatrix` | runs without spawned tiles | Pending — pair `composite_style_preview_integration_matrix_v1.md` **P3** |

---

## 7. Hot reload matrix

| Asset | Loader | Trigger | Re-run scope | Status |
|:---|:---|:---|:---|:---:|
| `WorldGenTuningOverlay` | manual + future watcher | F8 button / file watch | re-derive `BiomeTuning` + tag thresholds; rerun passes 2–6 | Partial (manual today) |
| `MaterialRegistry` | Bevy `AssetLoader` (`MaterialRegistryLoader`, `*.material_registry.json`) | file watch | rebuild name→id; rerun pass 6 only | Partial — loader type exists; `init_asset` / registration **U5** |
| `TagRegistry` | Bevy `AssetLoader` | file watch | rebuild name→id; rerun passes 2–6 | Pending |
| `RuleSet` | Bevy `AssetLoader` | file watch | re-sort rules; rerun pass 6 only | Pending |

**Invariant:** F8 panel + asset editor **only edit files**. The engine watches. No second mutation path.

---

## 8. Determinism / save matrix

| Topic | Decision | Status |
|:---|:---|:---:|
| Save wire format | `MaterialDef.name` strings (not raw `MaterialId`) | Pending — pair `serialization_hybrid_migration_matrix_v1.md` |
| Schema versions | `schema_version: u32` per file | Pending |
| Chunk hash | `hash(world_seed, chunk_xy, registry_version, rules_version, tuning_version)` | Pending |
| Rule order | file order = `rule_index` (deterministic tie-break) | Pending |

---

## 9. UI exposure matrix

| Surface | Current | Target | Status |
|:---|:---|:---|:---:|
| F8 egui panel | `WorldGenParams`, `BiomeTuning` subset, `PreviewMode` | + `PreviewMode::Tag`, registry summary | Partial |
| Asset editor World Gen page | Overview / Noise / Biome / Full JSON tabs | separate **Materials**, **Tags**, **Rules** nav pages (`terrain_registry_pages.py`) | Applied |
| Inspector (`bevy-inspector-egui`) | n/a | optional — view registries as tables | Pending |

---

## 10. Migration phases (U-series)

| Phase | Deliverable | Phase in this plan | Status |
|:---:|:---|:---|:---:|
| **U0** | This matrix + paired designer doc + cross-links | Phase 1 (markdown) | Applied |
| **U1** | `material_registry.example.json`, `tag_registry.example.json`, `material_rules.example.ron` | Phase 2 (assets) | Applied |
| **U2** | Asset-editor **Materials**, **Tags**, **Rules** pages + nav + repo_paths | Phase 3 (Python) | Applied |
| **U3** | Rust scaffolding: `MaterialId`, `MaterialDef`, `MaterialRegistry`, `TagId`, `TagRegistry`, `TagSet`, `MaterialRule`, `RuleSet`, `resolve_material` (loader + assets) | follow-up plan | **Applied** |
| **U4** | `ChunkCellMatrix` + multi-pass pipeline (pass 1–3 wired; 4–5 stubs) | follow-up plan | Pending |
| **U5** | `MaterializedChunk` + `MaterialUnificationPlugin` + preview color via `MaterialDef.preview_color` | follow-up plan | Pending |
| **U6** | Optional `bevy_ecs_tilemap` adapter behind feature flag; GPU compute preview | follow-up plan | Pending |
| **U7** *(optional)* | Invalidation graph (`ChunkDependency` / hashes), partial chunk rebuild policy, multi-layer tilemaps (terrain/overlay/resources), optional weighted rule scoring, debug rule-trace — see §§13–18 | after U5 | Pending |

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

**ECS sketch:** chunk component or child entity holding `source_noise_id`, `ruleset_hash`, `material_registry_hash`, `tag_registry_hash`, `tuning_hash` — compared each frame to loaded assets; mismatches set `ChunkDirty` flags.

---

## 14. Tag expansion pass detail (beyond §3 table)

| Pass | Outline name | Repo anchor | Status |
|:---:|:---|:---|:---:|
| 1 | Raw field tagging | §3 pass 2 + `BiomeTuning` / `tag_tuning` | Pending |
| 2 | Derived combo tags (`wet`+`lowland`→…) | Data-driven combo table **`ASK:`** | Pending |
| 3 | Spatial: clusters / edges / gradients | Neighbor reads — couple with `chunks_streaming_v1.md` ghost bands | Pending |
| 4 | Agent / LLM overlay | Pass 5 in §3; audited writes + authority (**§47**, **items 71–74**) | Pending |

---

## 15. Multi-layer tilemap / render stack

| Layer | Z (convention 📎) | Data source | Visibility | Status |
|:---|:---:|:---|:---|:---:|
| Terrain | 0 | `MaterializedChunk` → texture index | always (game) | Pending |
| Overlay / debug | 10 | `ChunkCellMatrix` fields or `TagSet` mask | designer toggle | Pending |
| Resources | 20 | Separate pass / rules **`ASK:`** | gameplay toggle | Pending |

**Constraint:** shared tile grid extent per chunk; layers may **not** share one `TileStorage` if the renderer requires independent textures — engine chooses one `TilemapId` per layer per chunk vs compositing **ASK:** **item 64**.

---

## 16. Performance / threading / cache

| Mechanism | Role | Status |
|:---|:---|:---:|
| `ChunkDirty` bitmask | Skip untouched chunks each schedule pass | Pending |
| `ChunkCache` coord → `{matrix, materialized, hashes}` | In-memory hot path; optional disk **`ASK:`** | Pending |
| `TaskPool` / off-thread generation | Noise/field fill writes into `ChunkCellMatrix` buffer; main thread applies ECS | Pending |
| Diff updates | Only changed tile indices sent to `TileStorage` | Pending |

---

## 17. LLM / agent rule editor (non-binding policy)

| Concern | Guideline | Matrix / checklist |
|:---|:---|:---|
| Determinism | Same seed + same **committed** rule/tag/registry files ⇒ identical `ChunkCellMatrix` + materialize | §8, **item 71** |
| Authority | Agent edits **files** or append-only overlay; engine reload path identical to designer saves | `material_tag_rule` §8 |
| Telemetry | Distribution metrics drive suggestions; implementation may stub (**item 73**) | — |
| Audit | Log rule diffs (dev JSONL **item 72**) | — |

---

## 18. Extensibility — profiles & packs

| Mechanism | Intent | Status |
|:---|:---|:---:|
| **World profile** | One handle selects bundle: `material_registry` + `tag_registry` + `material_rules` + `world_gen_tuning` path | Pending — **item 76** |
| **Biome / rules pack** | Drop-in folder merged at load or explicit `#include` **ASK:** **item 77** | Pending |
| **Modded `RuleSet`** | Stable rule `name` keys for overrides; numeric priority bands reserved for core vs mods **ASK:** | Pending |
