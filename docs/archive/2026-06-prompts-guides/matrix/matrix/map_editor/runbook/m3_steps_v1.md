# M3 — Terrain brushes `v1`

> **Pair:** orchestrator [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) · matrix [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md) §3 row **M3**. **Pre-req:** **M2** **Applied**.

**Halt rules:** orchestrator §6. Do not add a second **`TerrainClass`** resolver.

---

### M3-S01 tile pick from camera

**Goal:** Under **`BaseState::Editor`**, resolve **world (x,z)** (or tile index) from cursor + primary camera for tiles with **`TileMarker`**.

**Anchor reads:**

1. [`../../../../src/terrain/generation/world_generator_enhanced.rs`](../../../../src/terrain/generation/world_generator_enhanced.rs) — `TileMarker`, `Transform` layout.
2. [`../../../../src/gui/editor/world_preview.rs`](../../../../src/gui/editor/world_preview.rs) — existing preview camera if reusable.
3. [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs).

**Touch:**

- [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs)

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 --lib -- --quiet`

**Matrix update:** **M3** → **Partial**.

**Definition of done:**

- [ ] Pick returns **None** off-map; documented coordinate convention.

---

### M3-S02 height brush apply

**Goal:** Adjust **`Height`** and **`Transform`** Y for picked tile(s) within brush radius; use same vertical scale convention as generator (**`height * 20`** in spawn — match in place).

**Anchor reads:**

1. [`../../../../src/terrain/generation/world_generator_enhanced.rs`](../../../../src/terrain/generation/world_generator_enhanced.rs) — `Height` component.

**Touch:**

- [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs)

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 --lib -- --quiet`

**Matrix update:** none.

**Definition of done:**

- [ ] Slider or wheel changes height without panicking.

---

### M3-S03 biome repaint brush

**Goal:** Set **`TerrainType(TerrainClass)`** on tiles in radius; document whether **`classify_biome`** is skipped (manual override).

**Anchor reads:**

1. [`../../../../src/terrain/biome.rs`](../../../../src/terrain/biome.rs).

**Touch:**

- [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs)

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 --lib -- --quiet`

**Matrix update:** none.

**Definition of done:**

- [ ] Manual repaint does not call **`classify_biome`** unless step explicitly adds sync.

---

### M3-S04 phase close — M3 Applied

**Goal:** Flip **M3** **Applied** in matrix + orchestrator §4.

**Anchor reads:** matrix §3 · orchestrator §4.

**Touch:**

- [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md)
- [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) §4.

**Verify:** `cargo check -p proc_A_dine01`

**Matrix update:** **M3** → **Applied**.

**Definition of done:**

- [ ] **M3** **Applied**.
