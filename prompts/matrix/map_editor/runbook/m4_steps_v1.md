# M4 — Roads / structures (tile-aligned) `v1`

> **Pair:** orchestrator [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) · matrix [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md) §3 row **M4**. **Pre-req:** **M3** **Applied**.

**Halt rules:** orchestrator §6. Reuse existing **road** components if present; otherwise **`ASK:`** before inventing a parallel road model.

---

### M4-S01 audit existing road entities

**Goal:** Document in step PR description which **`Entity`** / components represent roads today; pick **one** spawn pattern for editor (-marker only v1).

**Anchor reads:**

1. [`../../../../src/entities/structure/components.rs`](../../../../src/entities/structure/components.rs) — road stubs.
2. [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) §1.

**Touch:**

- [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs) — comments only OR `ASK:` file if no road type — *if **Touch** is comments-only, also touch [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md) §5 sync gate note*

**Verify:**

- `cargo check -p proc_A_dine01`

**Matrix update:** **M4** → **Partial**.

**Definition of done:**

- [ ] Audit note committed (matrix §5 or pack footer).

---

### M4-S02 place road marker at tile

**Goal:** On click in **Road** mode, spawn child under world root (or attach marker component) at tile **(x,z)** — minimal visual (mesh optional **ASK:**).

**Anchor reads:**

1. [`../../../../src/terrain/generation/world_generator_enhanced.rs`](../../../../src/terrain/generation/world_generator_enhanced.rs) — `WorldMarker`.

**Touch:**

- [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs)

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 --lib -- --quiet`

**Matrix update:** none.

**Definition of done:**

- [ ] At least one road marker appears; no duplicate **WorldMarker**.

---

### M4-S03 phase close — M4 Applied

**Goal:** **M4** **Applied**.

**Anchor reads:** matrix §3 · orchestrator §4.

**Touch:**

- [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md)
- [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) §4.

**Verify:** `cargo check -p proc_A_dine01`

**Matrix update:** **M4** → **Applied**.

**Definition of done:**

- [ ] **M4** **Applied**.
