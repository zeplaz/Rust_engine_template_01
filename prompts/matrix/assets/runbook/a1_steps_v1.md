# A1 — Doc wiring (fine-grained)

> Pair [`../../../guides/bevy_asset_terrain_runbook_v1.md`](../../../guides/bevy_asset_terrain_runbook_v1.md). **Many steps are doc-only;** `cargo check -p proc_A_dine01` still runs every time.

---

### A1-S01 Terrain table in Bevy matrix

**Goal:** Ensure the **Terrain registry assets** subsection exists with three rows and the drift **Rule** line.

**Anchor reads:** orchestrator §§1–2 · [`../bevy_asset_config_migration_matrix_v1.md`](../bevy_asset_config_migration_matrix_v1.md).

**Touch:** [`../bevy_asset_config_migration_matrix_v1.md`](../bevy_asset_config_migration_matrix_v1.md) — add or verify the subsection immediately above **Sub-questions**.

**Verify:** `cargo check -p proc_A_dine01`

**Matrix update:** none (editing the matrix file is the step).

**Definition of done:**
- [ ] Subsection present; extensions column matches material matrix §2 spirit.
- [ ] Build passes.

---

### A1-S02 Material matrix §2 pointer

**Goal:** RON format bullet cites Bevy matrix **Terrain registry** for loader extensions.

**Anchor reads:** orchestrator §1 · [`../terrain_biome/material_unification_matrix_v1.md`](../terrain_biome/material_unification_matrix_v1.md) §2.

**Touch:** [`../terrain_biome/material_unification_matrix_v1.md`](../terrain_biome/material_unification_matrix_v1.md) — **Format rule** second bullet only.

**Verify:** `cargo check -p proc_A_dine01`

**Definition of done:**
- [ ] Link text names "Terrain registry" subsection explicitly.

---

### A1-S03 Terrain README on disk

**Goal:** `assets/config/terrain/README.md` lists examples and `schema_version` rule.

**Anchor reads:** orchestrator §1 · [`material_unification_matrix_v1.md`](../terrain_biome/material_unification_matrix_v1.md) §2.

**Touch:** [`../../../../assets/config/terrain/README.md`](../../../../assets/config/terrain/README.md) — create or trim to stay one screen.

**Verify:** `cargo check -p proc_A_dine01`

**Definition of done:**
- [ ] File exists at repo root path `assets/config/terrain/README.md`.

---

### A1-S04 Designer narrative cross-links

**Goal:** `material_tag_rule_system` lists paired Bevy runbook + terrain README.

**Anchor reads:** [`material_tag_rule_system_v1.md`](../../../designer_questions/terrain_world/material_tag_rule_system_v1.md) Cross-links.

**Touch:** that file only — two bullet lines.

**Verify:** `cargo check -p proc_A_dine01`

**Definition of done:**
- [ ] Bullets present; paths valid from designer doc.

---

### A1-S05 Terrain orchestrator §8b Bevy row

**Goal:** Bevy paired row uses a **markdown link** to the Bevy orchestrator (not bare path).

**Anchor reads:** [`../../../guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) §8b.

**Touch:** [`../../../guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) — one table cell only.

**Verify:** `cargo check -p proc_A_dine01`

**Definition of done:**
- [ ] Clickable link to `bevy_asset_terrain_runbook_v1.md`.

---

### A1-S06 Phase close — queues + material matrix §11

**Goal:** Mark Pair A as started; cross-doc row points at Bevy orchestrator + assets `runbook/`.

**Anchor reads:** [`../../../guides/terrain_paired_runbooks_queue_v1.md`](../../../guides/terrain_paired_runbooks_queue_v1.md) · [`../terrain_biome/material_unification_matrix_v1.md`](../terrain_biome/material_unification_matrix_v1.md) §11 · [`../../../guides/bevy_asset_terrain_runbook_v1.md`](../../../guides/bevy_asset_terrain_runbook_v1.md) §4.

**Touch:**
- [`../../../guides/terrain_paired_runbooks_queue_v1.md`](../../../guides/terrain_paired_runbooks_queue_v1.md) — inventory note under **A** (or opening paragraph).
- [`../terrain_biome/material_unification_matrix_v1.md`](../terrain_biome/material_unification_matrix_v1.md) — §11 row if missing.
- [`../../../guides/bevy_asset_terrain_runbook_v1.md`](../../../guides/bevy_asset_terrain_runbook_v1.md) — §4 **A1** → **Applied**.

**Verify:** `cargo check -p proc_A_dine01`

**Definition of done:**
- [ ] A1 row **Applied** in Bevy orchestrator §4.
- [ ] Queue + material §11 consistent.

---

## Open carries

None — `prompts/matrix/README.md` `assets/` row already lists this `runbook/`.
