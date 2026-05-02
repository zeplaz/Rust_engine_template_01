# World map editor — migration / boundary matrix `v1`

> **STATUS:** Anchor matrix for **post-procedural** editing (brushes, roads, save/load). Phases **M1–M5** track [`../../guides/map_editor_runbook_v1.md`](../../guides/map_editor_runbook_v1.md). **Rust execution:** step packs in [`runbook/README.md`](runbook/README.md).

Version: `v1.0.0`

---

## 1. Scope

| In | Out (v1) |
|:---|:---|
| Route **FullReady** → **`BaseState::Editor`** (alternative to **Simulation**) | Full 3D sculpting, voxel layers |
| **egui** tool palette (`TEMP-EGUI` until Bevy UI parity) | LLM-assisted edit |
| Tiles backed by existing ECS (`TileMarker`, `Height`, `TerrainType`, …) | Regenerating whole world inside editor without explicit user action |
| Snapshot **save/load** (names not stable ids per meta save policy) | Multiplayer live sync |

---

## 2. Concept ↔ symbol (canonical)

| Concept | Symbol / location |
|:---|:---|
| World root | `WorldMarker` — `src/terrain/generation/world_generator_enhanced.rs` |
| Tile | `TileMarker`, `Height`, `Moisture`, `Temperature`, `TerrainType` |
| Biome class | `TerrainClass` — `src/terrain/biome.rs` (no parallel enum) |
| Base vs editor vs sim | `BaseState` — `src/engine/states.rs` |
| World-gen flow | `WorldGenFlowState` — `src/engine/states.rs` |
| In-game editor modes | `InGameEditorState` — `src/engine/states.rs` *(wire under Editor)* |

---

## 3. Phase index (M1–M5)

| Phase | Scope | Step pack | Status |
|:---:|:---|:---| :---: |
| **M1** | State machine + menus: FullReady → Editor; load → Editor stub split | [`runbook/m1_steps_v1.md`](runbook/m1_steps_v1.md) | Applied |
| **M2** | `MapEditorPlugin` shell, `run_if(Editor)`, mode resource, minimal palette | [`runbook/m2_steps_v1.md`](runbook/m2_steps_v1.md) | Pending |
| **M3** | Height brush + biome repaint on tiles | [`runbook/m3_steps_v1.md`](runbook/m3_steps_v1.md) | Pending |
| **M4** | Road / structure markers tile-aligned | [`runbook/m4_steps_v1.md`](runbook/m4_steps_v1.md) | Pending |
| **M5** | Snapshot serde, save from editor, open saved map in editor | [`runbook/m5_steps_v1.md`](runbook/m5_steps_v1.md) | Pending |

---

## 4. Cross-links

| Doc | Role |
|:---|:---|
| [`../../guides/map_editor_runbook_v1.md`](../../guides/map_editor_runbook_v1.md) | Execution orchestrator (invariants, loop, halt) |
| [`../../guides/system_runbook_authoring_meta_v1.md`](../../guides/system_runbook_authoring_meta_v1.md) | Meta-runbook (authoring rules) |
| [`../../guides/engine_architecture_human_map_v1.md`](../../guides/engine_architecture_human_map_v1.md) | Engine IA / state overview |
| [`../../guides/gui_runbook_v1.md`](../../guides/gui_runbook_v1.md) | `TEMP-EGUI` policy for editor panels |
| [`../serialization/serialization_hybrid_migration_matrix_v1.md`](../serialization/serialization_hybrid_migration_matrix_v1.md) | Save DTO versioning (**M5** must align) |
| [`../../guides/terrain_unification_runbook_v1.md`](../../guides/terrain_unification_runbook_v1.md) §8b | Paired: tile / material names on disk |

---

## 5. Sync gates

| Partner | Gate |
|:---|:---|
| **Serialization wave S / G4** | **M5** must not invent stable binary ids; use **names** for materials/biomes in snapshots until hybrid matrix rows say otherwise |
| **Terrain U3–U7** | Editor edits **ECS tiles**; chunk/materialize paths are **out of scope** for M1–M3 unless an explicit step adds invalidation |
| **World gen alpha** | **M1** must not break `GenerateWorldEvent` / preview / full pipeline |
