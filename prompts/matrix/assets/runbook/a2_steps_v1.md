# A2 — Example asset audits (one file per step)

> Pair [`../../../guides/bevy_asset_terrain_runbook_v1.md`](../../../guides/bevy_asset_terrain_runbook_v1.md). **Doc-only:** no new `cargo test` required unless you add tests in a later PR.

**Status:** **Applied** — see orchestrator §4. Execution log at bottom.

---

### A2-S01 `material_registry.example.json`

**Goal:** Top-level `schema_version` (number) and `materials` array present; each material has `name`, `family`, `tags`, `preview_color`; **`properties` keys use namespace convention** when non-empty (see `material_tag_rule_system_v1.md` §4.1).

**Anchor reads:** [`../bevy_asset_config_migration_matrix_v1.md`](../bevy_asset_config_migration_matrix_v1.md) Terrain table · example file.

**Touch:** [`../../../../assets/config/terrain/material_registry.example.json`](../../../../assets/config/terrain/material_registry.example.json) — fix only if audit fails.

**Verify:** `cargo check -p proc_A_dine01`

**Matrix update:** none.

**Definition of done:**
- [ ] Human or agent reread file; violations fixed or `ASK:` filed.

---

### A2-S02 `tag_registry.example.json`

**Goal:** `schema_version` + `tags` array; each tag has `name`, `category`.

**Touch:** [`../../../../assets/config/terrain/tag_registry.example.json`](../../../../assets/config/terrain/tag_registry.example.json).

**Verify:** `cargo check -p proc_A_dine01`

**Definition of done:**
- [ ] Audit clean or `ASK:`.

---

### A2-S03 `material_rules.example.ron`

**Goal:** Top-level `schema_version` and `rules` list; each rule has `required`, `forbidden`, `result`, `priority` (names align with [`u3_steps_v1.md`](../../terrain_biome/runbook/u3_steps_v1.md) when coded).

**Touch:** [`../../../../assets/config/terrain/material_rules.example.ron`](../../../../assets/config/terrain/material_rules.example.ron).

**Verify:** `cargo check -p proc_A_dine01`

**Definition of done:**
- [ ] Audit clean or `ASK:`.

---

### A2-S04 Bevy matrix **Loader / Asset** column truth

**Goal:** Terrain table **Loader / Asset** column stays **Pending** until terrain **U3-S04** lands; add one footnote row under the table if needed.

**Touch:** [`../bevy_asset_config_migration_matrix_v1.md`](../bevy_asset_config_migration_matrix_v1.md).

**Verify:** `cargo check -p proc_A_dine01`

**Definition of done:**
- [ ] **Pending** is explicit; footnote references terrain U3-S04.

---

### A2-S05 Phase close

**Goal:** [`../../../guides/bevy_asset_terrain_runbook_v1.md`](../../../guides/bevy_asset_terrain_runbook_v1.md) §4 — **A2** → **Applied**.

**Touch:** that file §4 only.

**Verify:** `cargo check -p proc_A_dine01`

**Definition of done:**
- [ ] A2 **Applied**.

---

## Open carries

- Optional: JSON Schema files for registries (`ASK:`) — not required for A2 close.

---

## Execution log

| Step | Result |
|:---|:---|
| A2-S01 | Pass — `schema_version` **2**; namespaced `properties` (`facts.*`, `sim.*`, …); materials retain `name`, `family`, `tags`, `preview_color`. |
| A2-S02 | Pass — `schema_version` 1; fourteen tags with `name`, `category`. |
| A2-S03 | Pass — `schema_version` 1; four rules with `required`, `forbidden`, `result`, `priority`. |
| A2-S04 | Pass — Bevy matrix footnote under Terrain table; **Loader / Asset** remains **Pending** until U3-S04. |
| A2-S05 | Pass — orchestrator §4 **A2** → **Applied**; `cargo check -p proc_A_dine01` green. |
