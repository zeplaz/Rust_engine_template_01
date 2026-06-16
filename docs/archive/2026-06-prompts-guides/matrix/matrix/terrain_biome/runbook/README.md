# Terrain unification runbook — step packs

> **Maintenance first:** For **Applied** U3–U7 status, onboarding, and when to reopen a pack, read **[`../../../legacy_runbooks/terrain/terrain_u_applied_maintenance_v1.md`](../../../legacy_runbooks/terrain/terrain_u_applied_maintenance_v1.md)** — then return here for atomic `uN_steps_v1.md` execution detail.

> **STATUS:** Index of atomic-step packs that drive **Rust phases U3–U7** of the material / tag / rule unification. Pair with the orchestrator at [`../../../guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md), the matrix at [`../material_unification_matrix_v1.md`](../material_unification_matrix_v1.md), and **tooling parity / testing** at [`../../../guides/world_assets_tools_rulebook_v1.md`](../../../guides/world_assets_tools_rulebook_v1.md).

**Authoritative phase status:** U3–U7 are **Applied** per orchestrator [**§4 Phase index**](../../../guides/terrain_unification_runbook_v1.md#4-phase-index) and matrix [**§10**](../material_unification_matrix_v1.md) (mirror those docs when in doubt). This README describes **how to run** packs for maintenance, audits, or future extensions — not an open execution gap for U6/U7.

Version: `v1.0.2`

**Paired runbooks:** terrain couples to serialization, assets, preview, and streaming (see orchestrator [**§8b**](../../../guides/terrain_unification_runbook_v1.md#8b-paired-runbooks-planned)). Author those via [`../../../guides/system_runbook_authoring_meta_v1.md`](../../../guides/system_runbook_authoring_meta_v1.md) using the shared **Q0–Q6** queue in [`../../../guides/terrain_paired_runbooks_queue_v1.md`](../../../guides/terrain_paired_runbooks_queue_v1.md). **Before** spawning new paired orchestrators or expanding step packs, designers and leads should complete the short briefing [`../../../guides/rulebook_backlog_designer_brief_v1.md`](../../../guides/rulebook_backlog_designer_brief_v1.md) so backlog scope and priorities are explicit. Keep §8 cross-links bidirectional.

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
| **U4** | [`u4_steps_v1.md`](u4_steps_v1.md) | `ChunkCellMatrix` + multi-pass pipeline (1 fields, 2 threshold tags, 3 `classify_biome`, 4 hydrology, 5 stub, 6 materialize) |
| **U5** | [`u5_steps_v1.md`](u5_steps_v1.md) | `MaterializedChunk`, `MaterialUnificationPlugin`, preview color via `MaterialDef.preview_color`, `PreviewMode::Tag` |
| **U6** *(optional)* | [`u6_steps_v1.md`](u6_steps_v1.md) | Feature `bevy_tilemap_adapter` — terrain layer → `TileTextureIndex`; phase **Applied** per orchestrator §4 / matrix §10 (still **optional** to enable in a given build or team scope) |
| **U7** | [`u7_steps_v1.md`](u7_steps_v1.md) | Invalidation graph, partial rebuild, multi-layer stack, debug, world profile / packs — phase **Applied** per orchestrator §4 / matrix §10 (mirrors matrix §§13–18, checklist **49–78**); pairs with U6 when multi-layer / tilemap is in use |

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

Finish U*N* (matrix §10 status **Applied**) before starting U*N+1*. **U6** remains **optional** (feature-gated); **U7** historically **only begins after U5** is **Applied** — today both U6 and U7 are **Applied** in matrix §10 for the mainline trajectory; teams skipping **U6** still rely on **U7** for invalidation, profiles, and dev trace. Confirm **`world_generator` binary vs main** parity per [`../../../guides/world_assets_tools_rulebook_v1.md`](../../../guides/world_assets_tools_rulebook_v1.md) §1 before treating terrain tooling as closed for your team. For work **outside** these U-packs (serialization, preview, streaming, or other systems), use [**`rulebook_backlog_designer_brief_v1.md`**](../../../guides/rulebook_backlog_designer_brief_v1.md) then [`terrain_paired_runbooks_queue_v1.md`](../../../guides/terrain_paired_runbooks_queue_v1.md) and [`system_runbook_authoring_meta_v1.md`](../../../guides/system_runbook_authoring_meta_v1.md) §3.
