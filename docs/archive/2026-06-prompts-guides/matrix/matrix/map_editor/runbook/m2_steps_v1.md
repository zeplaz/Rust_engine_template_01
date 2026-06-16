# M2 — Editor shell + modes `v1`

> **Pair:** orchestrator [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) · matrix [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md) §3 row **M2**. **Pre-req:** **M1** **Applied**.

**Halt rules:** orchestrator §6.

---

### M2-S01 map editor plugin scaffold

**Goal:** Add **`MapEditorPlugin`** registering systems with **`run_if(in_state(BaseState::Editor))`**.

**Anchor reads:**

1. [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) §1.
2. [`../../../../src/engine/engine_with_worldgen.rs`](../../../../src/engine/engine_with_worldgen.rs).
3. [`../../../../src/engine/states.rs`](../../../../src/engine/states.rs).

**Touch:**

- [`../../../../src/engine/engine_with_worldgen.rs`](../../../../src/engine/engine_with_worldgen.rs)
- New: `src/gui/editor/map_editor/mod.rs` *(or single `map_editor.rs` — max 3 files)*

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 --lib -- --quiet`

**Matrix update:** **M2** → **Partial**.

**Definition of done:**

- [ ] Plugin builds; systems do not run in **MainMenu** / **Simulation** unless explicitly shared.

---

### M2-S02 wire InGameEditorState and tool resource

**Goal:** Initialize **`State<InGameEditorState>`** (or **`NextState`** transitions) and a **`MapEditorTool`** resource (brush radius, mode enum).

**Anchor reads:**

1. [`../../../../src/engine/states.rs`](../../../../src/engine/states.rs).
2. [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) §1.

**Touch:**

- [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs) *(create if missing)*
- [`../../../../src/engine/states.rs`](../../../../src/engine/states.rs) *(only if new state init required in plugin build)*

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 --lib -- --quiet`

**Matrix update:** none.

**Definition of done:**

- [ ] Mode can switch (e.g. **Select** vs **Terrain**) from code or placeholder UI.

---

### M2-S03 temp-egui tool palette window

**Goal:** **`TEMP-EGUI`** labelled panel lists modes + **Play** (**→ Simulation**) + **Exit** (**→ MainMenu**).

**Anchor reads:**

1. [`../../../guides/gui_runbook_v1.md`](../../../guides/gui_runbook_v1.md) §1.
2. [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs).

**Touch:**

- [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs)

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 --lib -- --quiet`

**Matrix update:** none.

**Definition of done:**

- [ ] Panel comment or window title includes **`TEMP-EGUI`**.

---

### M2-S04 phase close — M2 Applied

**Goal:** Mark **M2** **Applied** in matrix §3 and orchestrator §4.

**Anchor reads:**

1. [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md) §3.
2. [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) §4.

**Touch:**

- [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md)
- [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md)

**Verify:**

- `cargo check -p proc_A_dine01`

**Matrix update:** **M2** → **Applied**.

**Definition of done:**

- [ ] **M2** **Applied**.
