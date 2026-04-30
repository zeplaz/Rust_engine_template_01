# Bevy asset terrain paired runbook `v1`

> **STATUS:** **Paired** with [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md). Granular **policy, on-disk audit, and integration gates** for terrain registries on the Bevy 0.18 asset path. Does **not** replace terrain **U3** Rust steps — it **front-runs** naming/extensions and **verifies** after U3/U5 milestones.

Version: `v1.0.1`
Audience: agents keeping [`bevy_asset_config_migration_matrix_v1.md`](../matrix/assets/bevy_asset_config_migration_matrix_v1.md) aligned with terrain material/tag/rule work.

---

## How to use this doc (loop protocol)

Per-phase packs live at [`../matrix/assets/runbook/`](../matrix/assets/runbook/README.md). One atomic step per iteration; re-read §1 before each step.

```mermaid
flowchart LR
  inv[Read §1 invariants] --> step[Open A1 or A2 or A3 pack]
  step --> edit[Edit Touch list 1-3 paths]
  edit --> check[cargo check -p proc_A_dine01]
  check --> gate{Doc-only step?}
  gate -->|yes| docOK[Re-read touched markdown]
  gate -->|no| test[cargo test named_test]
  docOK --> matrix[Matrix / cross-link update]
  test --> matrix
```

---

## 1. Invariants (re-read every loop)

1. Terrain on-disk layout stays **locked** per [`material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) §2 (JSON registries, RON rules).
2. **Target `AssetLoader` extensions** are exactly those in [`bevy_asset_config_migration_matrix_v1.md`](../matrix/assets/bevy_asset_config_migration_matrix_v1.md) **Terrain registry** table — change only with `ASK:` and a matching edit in terrain [`u3_steps_v1.md`](../matrix/terrain_biome/runbook/u3_steps_v1.md).
3. Every committed terrain example that declares it **must** include top-level `schema_version` where that format uses versioning (JSON examples today).
4. **No second load path** for the same file: `AssetServer` + loaders only; F8 / asset editor still file-edit only (same as terrain §1).
5. Determinism of **configs**: same bytes on disk + same engine ⇒ same parsed structs (no ambient randomness in loaders).

---

## 2. Anchor file set (≤5 paths per step)

1. This runbook §§1, 2, 4, 8.
2. [`bevy_asset_config_migration_matrix_v1.md`](../matrix/assets/bevy_asset_config_migration_matrix_v1.md) **Terrain registry** subsection.
3. [`material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) §2.
4. Current pack under [`../matrix/assets/runbook/`](../matrix/assets/runbook/README.md).
5. The single file in the step `Touch` list.

---

## 3. Atomic step schema

Same as [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) §3. Replace phase id with **`A<phase>-S<NN>`** (e.g. `A1-S01`).

**Doc-only steps:** `Verify` may be `cargo check -p proc_A_dine01` only (must stay green); no `cargo test` required if the step names **Doc-only** in **Goal**.

---

## 4. Phase index

| Phase | Pack | Focus | Status |
|:---:|:---|:---|:---:|
| **A1** | [`a1_steps_v1.md`](a1_steps_v1.md) | Cross-links + terrain README + matrix pointers | Applied |
| **A2** | [`a2_steps_v1.md`](a2_steps_v1.md) | Example file audits (`schema_version`, keys) | **Applied** (audits 2026-04-30 — examples compliant; matrix footnote added) |
| **A3** | [`a3_steps_v1.md`](a3_steps_v1.md) | Integration gates after terrain U3 / U5 | Pending |

Update **Status** in this table when each phase’s **phase-close** step completes.

---

## 5. Loop protocol (per-step)

1. Read §1.
2. Open one step in `a<N>_steps_v1.md`.
3. Edit only listed paths.
4. `cargo check -p proc_A_dine01` every step.
5. Run named `cargo test` when the step provides one.
6. Apply **Matrix update** in the step.
7. Drop step context; re-anchor §1.

**Halt:** two consecutive failures on check or test for one step — stop and surface (same spirit as terrain §6).

---

## 6. Sync gates (terrain)

| Terrain milestone | This runbook action |
|:---|:---|
| **U3-S04** (registry `AssetLoader`) | Run **A3-S01**; Terrain table **Loader / Asset** column → **Partial** / **Applied** per `src/` |
| **U5-S02** (`MaterialUnificationPlugin`) | **A3** close; confirm single registration ( **A3-S02** ) |
| **A2** (example audits) | Doc-only; complete before or in parallel with **U3** — does not require Rust |
| **U7** (asset hash / invalidation) | Revisit Bevy matrix + serialization paired book; `ASK:` if extensions change |

---

## 7. Glossary

| Term | Meaning |
|:---|:---|
| **Terrain registry** | Material + tag JSON + rules RON under `assets/config/terrain/` |
| **Extension** | Filename suffix the `AssetLoader` registers (e.g. `.material_registry.json`) |
| **Paired** | This runbook + terrain runbook §8b point at each other |

---

## 8. Cross-links

| Doc | Purpose |
|:---|:---|
| [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) | Terrain U3–U7 execution |
| [`terrain_paired_runbooks_queue_v1.md`](terrain_paired_runbooks_queue_v1.md) | Q0–Q6 queue for all paired books |
| [`material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) | On-disk layout §2 |
| [`material_tag_rule_system_v1.md`](../designer_questions/terrain_world/material_tag_rule_system_v1.md) | Designer narrative |
| [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md) §12 | Paired norms |

---

## 9. Prompt fragment

> Read this file §§1, 4, 6. Run **one** step from [`../matrix/assets/runbook/`](../matrix/assets/runbook/README.md). Do not mark **A3** **Applied** until terrain **U5-S02** (or equivalent) is **Applied** per material matrix §10. On coupling questions, use `ASK:` in [`terrain_world/implementation_questions_v1.md`](../designer_questions/terrain_world/implementation_questions_v1.md).
