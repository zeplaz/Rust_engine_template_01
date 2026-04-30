# Terrain unification runbook — step packs

> **STATUS:** Index of atomic-step packs that drive **Rust phases U3–U7** of the material / tag / rule unification. Pair with the orchestrator at [`../../../guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) and the matrix at [`../material_unification_matrix_v1.md`](../material_unification_matrix_v1.md).

Version: `v1.0.1`

**Paired runbooks:** terrain couples to serialization, assets, preview, and streaming (see orchestrator §8b). Author those via [`../../../guides/system_runbook_authoring_meta_v1.md`](../../../guides/system_runbook_authoring_meta_v1.md) using the shared **Q0–Q6** queue in [`../../../guides/terrain_paired_runbooks_queue_v1.md`](../../../guides/terrain_paired_runbooks_queue_v1.md). Keep §8 cross-links bidirectional.

---

## How to use

1. Read the orchestrator first: [`terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) §§1–6 (invariants, anchor set, schema, loop protocol, halt rules).
2. Open the **single** step pack for the active phase. Do not skip ahead.
3. Execute exactly **one atomic step** per loop iteration; verify with `cargo check -p proc_A_dine01` + the named test in the step.
4. Flip the matching Status row in [`../material_unification_matrix_v1.md`](../material_unification_matrix_v1.md) before moving on.

---

## Step packs

| Phase | Pack | Scope |
|:---:|:---|:---|
| **U3** | [`u3_steps_v1.md`](u3_steps_v1.md) | `MaterialId` / `MaterialDef` / registries / `TagSet` / `MaterialRule` / `RuleSet` / `resolve_material` + JSON & RON `AssetLoader` |
| **U4** | [`u4_steps_v1.md`](u4_steps_v1.md) | `ChunkCellMatrix` + multi-pass pipeline (1 fields, 2 threshold tags, 3 `classify_biome`, 4–5 stubs, 6 materialize) |
| **U5** | [`u5_steps_v1.md`](u5_steps_v1.md) | `MaterializedChunk`, `MaterialUnificationPlugin`, preview color via `MaterialDef.preview_color`, `PreviewMode::Tag` |
| **U6** *(optional)* | [`u6_steps_v1.md`](u6_steps_v1.md) | Feature-gated `bevy_ecs_tilemap` adapter |
| **U7** | [`u7_steps_v1.md`](u7_steps_v1.md) | Invalidation graph, partial rebuild, multi-layer stack, debug, world profile / packs (mirrors matrix §§13–18, checklist **49–78**) |

---

## Invariants reminder (full list in orchestrator §1)

- `ChunkCellMatrix` (not `MigrationMatrix`).
- `TerrainClass` is reused as `MaterialFamily` — no duplicate enum, no second classifier.
- JSON for `material_registry` / `tag_registry`; **RON** for `material_rules`.
- Saves use `MaterialDef.name`, not raw `MaterialId`.
- Engine reload path is `Assets<T>` watchers; F8 panel + asset-editor only edit files.
- Determinism: same seed + same committed config files ⇒ identical world.

---

## Sequencing

Finish U*N* (matrix §10 status **Applied**) before starting U*N+1*. **U6** is optional and may be skipped or feature-gated permanently. **U7** only begins after U5 is **Applied**.
