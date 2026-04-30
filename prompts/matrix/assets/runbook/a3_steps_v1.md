# A3 — Integration gates (after terrain Rust)

> Pair [`../../../guides/bevy_asset_terrain_runbook_v1.md`](../../../guides/bevy_asset_terrain_runbook_v1.md). **Do not start** until terrain [**U3-S04**](../../terrain_biome/runbook/u3_steps_v1.md) (per-material loader step) is **Applied** in practice — i.e. code + matrix agree.

---

### A3-S01 Post–U3-S04 loader column

**Goal:** For each terrain registry row in [`../bevy_asset_config_migration_matrix_v1.md`](../bevy_asset_config_migration_matrix_v1.md), set **Loader / Asset** to **Partial** or **Applied** to match `src/` (not prose only).

**Anchor reads:** terrain [`material_unification_matrix_v1.md`](../terrain_biome/material_unification_matrix_v1.md) §10 · Bevy matrix Terrain table.

**Touch:** [`../bevy_asset_config_migration_matrix_v1.md`](../bevy_asset_config_migration_matrix_v1.md) — Terrain table column + short footnote with symbol names (`MaterialRegistry`, etc.).

**Verify:** `cargo check -p proc_A_dine01` — plus confirm loaders exist in `rg AssetLoader material_registry` (agent runs `rg`).

**Definition of done:**
- [ ] Matrix matches repo; mismatches are `ASK:`.

---

### A3-S02 Post–U5-S02 single registration

**Goal:** Document that `MaterialUnificationPlugin` (or successor) registers all three loaders **once** — footnote in Bevy matrix or material matrix §7.

**Anchor reads:** terrain [`material_unification_matrix_v1.md`](../terrain_biome/material_unification_matrix_v1.md) §7 · [`../../terrain_biome/runbook/u5_steps_v1.md`](../../terrain_biome/runbook/u5_steps_v1.md).

**Touch:** one of: Bevy matrix Designer linkage / Terrain rule block, **or** material matrix §7 — **one file only**.

**Verify:** `cargo check -p proc_A_dine01`

**Definition of done:**
- [ ] No duplicate loader registration paths implied by docs.

---

### A3-S03 Phase close

**Goal:** [`../../../guides/bevy_asset_terrain_runbook_v1.md`](../../../guides/bevy_asset_terrain_runbook_v1.md) §4 — **A3** → **Applied** (only after A3-S01–S02 truth).

**Touch:** that file §4 only.

**Verify:** `cargo check -p proc_A_dine01`

**Definition of done:**
- [ ] A3 **Applied**; terrain §6 sync gate table still accurate.

---

## Open carries

- **U7** asset hashing: revisit A3 or add **A4** pack later (`ASK:`) — do not inflate this file preemptively.
