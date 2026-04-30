# Terrain paired runbooks — authoring queue `v1`

> **STATUS:** **Markdown queue only.** Plans creation of execution runbooks that stay aligned with [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) (U3–U7). Use [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md) §12 for norms.

Version: `v1.0.1`

---

## Inventory (planned paths)

| ID | Pair | Planned orchestrator | Primary matrix / anchor |
|:---|:---|:---|:---|
| **S** | Serialization / hybrid wire | `prompts/guides/serialization_terrain_runbook_v1.md` | [`serialization_hybrid_migration_matrix_v1.md`](../matrix/serialization/serialization_hybrid_migration_matrix_v1.md) |
| **A** | Bevy assets & hot-reload | `prompts/guides/bevy_asset_terrain_runbook_v1.md` | [`bevy_asset_config_migration_matrix_v1.md`](../matrix/assets/bevy_asset_config_migration_matrix_v1.md) |
| **P** | Preview / composite UI | `prompts/guides/world_preview_runbook_v1.md` | [`composite_style_preview_integration_matrix_v1.md`](../matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md) |
| **C** | Chunk streaming / neighbors | `prompts/guides/chunk_streaming_terrain_runbook_v1.md` | [`terrain_world/chunks_streaming_v1.md`](../designer_questions/terrain_world/chunks_streaming_v1.md) + material matrix §§13–16 |

Status of each file: **not created** until Q6 passes for that row — **except pair A (Bevy terrain): orchestrator + `matrix/assets/runbook/` committed; run A1–A3 there.** §8b in the terrain orchestrator lists the same four; keep paths in sync.

---

## Discrete steps per pair (Q0–Q6)

Run **one pair at a time**. Resume at the last completed `Qn`. Any open decision becomes `ASK:` in the paired matrix or [`terrain_world/implementation_questions_v1.md`](../designer_questions/terrain_world/implementation_questions_v1.md) — do not block the terrain Rust run on prose.

| Step | Action |
|:---:|:---|
| **Q0** | Read terrain [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) §§1, 8b and this row's primary matrix; list blocking questions. |
| **Q1** | Resolve or defer `ASK:` (human). Deferred items go into matrix/checklist with an owner. |
| **Q2** | Author partner **orchestrator** from meta §7.1; §8 must link terrain + step-pack README path (even if README is placeholder). |
| **Q3** | Author **step packs** (meta §7.2–7.3) **or** placeholder `runbook/README.md` per meta §10 if phases unclear. |
| **Q4** | **Cross-links:** partner matrix + terrain §8b path string match; partner §8 lists terrain. |
| **Q5** | Add a **sync gate** table (below) to the partner orchestrator §4 or §8 (one small table). |
| **Q6** | Meta §15 self-check; confirm bidirectional §8 links. |

---

## Sync gates (fill in partner orchestrator at Q5)

Copy this table into each **partner** orchestrator once steps exist; trim rows that do not apply yet.

| Terrain phase | Partner concern | Joint rule |
|:---:|:---|:---|
| **U3** | S, A | Wire format + loaders agree on registry/rule/tag asset names and `schema_version`. |
| **U5** | P | Preview modes and material lookup ship together; no second color table. |
| **U4+** | C | Cross-chunk tag or field reads match streaming ghost-band policy. |
| **U7** | A, S, C | Invalidation hashes and partial regen agree with save/load and chunk lifecycle. |

---

## Suggested order

**A** (assets) or **S** (serialization) first if U3 is imminent — they catch loader and wire-format drift early. **P** when **U5** is on the critical path. **C** when **U4** spatial passes need neighbors.

---

## Cross-links

| Doc | Role |
|:---|:---|
| [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) §8b | Terrain-side paired table |
| [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md) §12 | Paired norms + quality gate |
| [`../matrix/terrain_biome/material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) | Terrain phase truth |
