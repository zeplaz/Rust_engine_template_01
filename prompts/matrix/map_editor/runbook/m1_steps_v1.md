# M1 — State machine + menu routing `v1`

> **Pair:** orchestrator [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) · matrix [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md) §3 row **M1**.

**Anchor set:** orchestrator §§1–2 · matrix §§2–3 · this pack.

**Halt rules:** orchestrator §6. Additionally — do not remove **Enter simulation** without `ASK:`; **add** editor route alongside.

---

### M1-S01 full-ready open map editor

**Goal:** From **`WorldGenFlowState::FullReady`**, user can choose **Open in map editor** which sets **`BaseState::Editor`** and idles world-gen flow (**`WorldGenFlowState::Idle`**).

**Anchor reads:**

1. [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) §§1, 4.
2. [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md) §2.
3. [`../../../../src/gui/editor/world_gen_ui.rs`](../../../../src/gui/editor/world_gen_ui.rs).
4. [`../../../../src/engine/states.rs`](../../../../src/engine/states.rs).

**Touch:**

- [`../../../../src/gui/editor/world_gen_ui.rs`](../../../../src/gui/editor/world_gen_ui.rs)

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 hydrology -- --nocapture` *(smoke; no new test required if none — substitute `cargo test -p proc_A_dine01 --lib -- --quiet`)*

**Matrix update:** **M1** row → **Partial** (until **M1-S04**).

**Definition of done:**

- [ ] Build passes.
- [ ] Test passes.
- [ ] New button visible only in **FullReady**; keeps existing **Enter world** path.
- [ ] No invariant from orchestrator §1 broken.

---

### M1-S02 main menu open saved map in editor

**Goal:** **Load** UI offers **Open in editor** that sets **`BaseState::Editor`** and **`WorldGenFlowState::Idle`** (still stub deserialize until **M5**).

**Anchor reads:**

1. [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) §1.
2. [`../../../../src/gui/main_menu.rs`](../../../../src/gui/main_menu.rs).
3. [`../../../../src/engine/states.rs`](../../../../src/engine/states.rs).

**Touch:**

- [`../../../../src/gui/main_menu.rs`](../../../../src/gui/main_menu.rs)

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 --lib -- --quiet`

**Matrix update:** none (still **M1** Partial).

**Definition of done:**

- [ ] Build passes.
- [ ] Second button on load panel; does **not** enter **Simulation** until **M5** hydrates.

---

### M1-S03 editor session exit to main menu

**Goal:** From **`BaseState::Editor`**, a single **Exit to main menu** action returns **`BaseState::MainMenu`** and **`WorldGenFlowState::Idle`** (implementation may live in **M2** palette if no file exists yet — then **Touch** only the new editor UI file created in **M2-S01** and mark this step **blocked** until **M2-S01** in a note; prefer implementing a minimal `egui::Window` in **`MapEditorPlugin`** in this step if **M2** not started).

**Anchor reads:**

1. [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) §1.
2. [`../../../../src/engine/states.rs`](../../../../src/engine/states.rs).

**Touch:**

- `ASK:` if no editor UI file — **default:** create `src/gui/editor/map_editor_ui.rs` with one **Exit to main menu** button only (≤3 files: new file + `src/gui/editor/mod.rs` + `src/terrain/generation/world_generation_plugin.rs` or `engine_with_worldgen` plugin registration). *Keep Touch ≤3 — use new file + `mod.rs` + one registration site.*

**Note for agent:** Prefer **`src/gui/editor/map_editor_ui.rs`** + `pub mod map_editor_ui` in [`../../../../src/gui/editor/mod.rs`](../../../../src/gui/editor/mod.rs) + register system in the smallest existing plugin that already runs egui (e.g. extend **`WorldGenToolsPlugin`** chain or add **`MapEditorPlugin`** in **`world_generation_plugin.rs`**). **Single registration site only.**

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 --lib -- --quiet`

**Matrix update:** **M1** remains **Partial**.

**Definition of done:**

- [ ] Build passes.
- [ ] User can leave **Editor** without entering **Simulation**.

---

### M1-S04 phase close — M1 Applied

**Goal:** Flip **M1** to **Applied** in matrix §3 and orchestrator §4; confirm **FullReady → Editor**, load **Open in editor** stub, **Exit** from editor.

**Anchor reads:**

1. [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md) §3.
2. [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) §4.

**Touch:**

- [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md)
- [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md)

**Verify:**

- `cargo check -p proc_A_dine01`

**Matrix update:** **M1** → **Applied**.

**Definition of done:**

- [ ] **M1** row **Applied** in matrix and orchestrator phase index.
