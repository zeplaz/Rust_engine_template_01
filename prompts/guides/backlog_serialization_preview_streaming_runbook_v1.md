# Backlog wave runbook — serialization, preview, chunk streaming `v1`

> **STATUS:** Planning orchestrator for the paired backlog wave after G3 GUI setup. Execution order is **S** (serialization) → **P** (preview / composite UI) → **C** (chunk streaming / neighbors).

Version: `v1.0.0`
Audience: agents and humans turning recorded backlog answers into paired runbooks and atomic steps.

---

## 1. Invariants

1. **Saves use names, not raw ids** unless the serialization matrix explicitly allows ids.
2. **Schema versions are explicit.** Default to per-file `schema_version`; promote to registry only when many schemas share lifecycle or optimization requires it.
3. **Preview is a consumer, not a source of truth.** It reads chunk/material/tag state and does not mutate runtime simulation.
4. **Streaming owns neighbor bands.** Spatial tag expansion and smooth chunk edges require ghost bands / neighbor reads.
5. **TaskPool work stays off-thread until ECS apply.** Worker tasks write buffers; main thread owns Bevy ECS mutation.
6. **Surface unresolved choices as `TODO:`** in [`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) §4.

---

## 2. Anchor set

Every backlog step reads:

1. This runbook §§1-5.
2. [`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) §§3-4.
3. [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) §8b.
4. [`../matrix/terrain_biome/material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) relevant section.
5. The owning matrix / source file for the active wave.

---

## 3. Wave index

| Wave | Focus | Primary anchors | Status |
|:---:|:---|:---|:---:|
| **S** | Serialization / save format | [`../matrix/serialization/serialization_hybrid_migration_matrix_v1.md`](../matrix/serialization/serialization_hybrid_migration_matrix_v1.md), terrain matrix §8 | Active after G3 setup |
| **P** | Preview / composite UI | terrain matrix §§6, 9, 15; [`world_assets_tools_rulebook_v1.md`](world_assets_tools_rulebook_v1.md) | Pending |
| **C** | Chunk streaming / neighbors | terrain matrix §§13, 16; `chunks_streaming_v1.md` if present | Pending — author missing matrix if needed |

---

## 4. Wave S — serialization

**Goal:** lock save wire format and schema-version strategy before runtime state expands.

**Known decisions:**
- Terrain material saves store `MaterialDef.name`, not raw `MaterialId`.
- Per-file `schema_version` is the default; registry is allowed when optimization / shared lifecycle warrants it.

**First actions:**
1. Audit [`../matrix/serialization/serialization_hybrid_migration_matrix_v1.md`](../matrix/serialization/serialization_hybrid_migration_matrix_v1.md) for terrain material/tag rows.
2. Promote one row into an atomic runbook step.
3. Bubble schema registry vs per-file decision if implementation evidence changes the default.

---

## 5. Wave P — preview / composite UI

**Goal:** make preview surfaces read canonical material/tag/chunk state without creating a second data path.

**Anchors:**
- [`../matrix/terrain_biome/material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) §§6, 9, 15.
- [`world_assets_tools_rulebook_v1.md`](world_assets_tools_rulebook_v1.md).

**Open item:** inspector / registry table view is **Confirmed - Planned**; choose whether it stays egui/F8, Bevy inspector, or desktop asset tool.

---

## 6. Wave C — chunk streaming / neighbors

**Goal:** support ghost bands, LOD, TaskPool generation, and smooth tile diff updates.

**Known decisions:**
- `ChunkCache` is in-memory hot path, with optional disk cache when footprint and rebuild cost justify it.
- `TaskPool` generation should be used aggressively for heavy terrain work.
- Spatial tag expansion requires ghost bands.

**Open item:** define the `TileStorage` diff update contract for smooth transitions; tracked in [`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) §4.

---

## 7. Prompt fragment

> Read [`prompts/guides/backlog_serialization_preview_streaming_runbook_v1.md`](backlog_serialization_preview_streaming_runbook_v1.md) §§1-6. Execute wave **S** before **P**, and **P** before **C**. Do not invent save schema, preview mutation authority, or chunk-streaming thresholds; surface unresolved choices in the backlog brief §4.
