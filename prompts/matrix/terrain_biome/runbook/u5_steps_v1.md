# U5 — `MaterializedChunk` + `MaterialUnificationPlugin` + preview hookup `v1`

> **Pair:** orchestrator [`../../../guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) · matrix [`../material_unification_matrix_v1.md`](../material_unification_matrix_v1.md) §§5, 6, 7, 9, 10. **Pre-req:** U4 must be **Applied**.

**Phase goal:** Wire the U3+U4 building blocks into ECS and the editor preview. **No tilemap renderer yet** — that is U6.

**Anchor set (always):** orchestrator §§1–2 · matrix §§5–7 · this pack · the single edited file.

**Halt rules:** orchestrator §6.

---

## U5-S01 `MaterializedChunk` ECS component

**Goal:** Promote `MaterializedChunkData` (from U4-S06) into a Bevy ECS `Component`.

**Anchor reads:** orchestrator §§1–2 · matrix §5 · `src/terrain/generation/passes/p6_materialize.rs`.

**Touch:**
- New `src/terrain/material/runtime.rs`:
  - `#[derive(Component, Clone, Debug)] pub struct MaterializedChunk { pub size: UVec2, pub materials: Vec<MaterialId> }`.
  - `From<MaterializedChunkData>` impl (consumes the data form into the component).
- `src/terrain/material/mod.rs` — `pub mod runtime;` re-exporting `MaterializedChunk`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 materialized_chunk_from_data -- --nocapture`.

**Matrix update:** §5 row `MaterializedChunk` → **Partial** (component only; spawn site lands in S02).

**Definition of done:**
- [ ] Build + test pass.
- [ ] No system spawns yet — purely the type.

---

## U5-S02 `MaterialUnificationPlugin` registration

**Goal:** Bevy plugin that registers all assets/loaders and a single system that runs passes 1→6 for any newly-spawned `Chunk`.

**Anchor reads:** orchestrator §§1–2 · matrix §§5, 7 · `src/terrain/material/registry.rs`, `tags.rs`, `rules.rs` (loaders) · `src/terrain/generation/passes/`.

**Touch:**
- New `src/systems/terrain/material_plugin.rs`:
  - `pub struct MaterialUnificationPlugin;`
  - `impl Plugin for MaterialUnificationPlugin { fn build(&self, app: &mut App) { … } }`
  - In `build`: `app.init_asset::<MaterialRegistry>()`, `init_asset::<TagRegistry>()`, `init_asset::<RuleSet>()`, register the three `AssetLoader`s (created in U3-S04 for `MaterialRegistry`; mirror loaders for `TagRegistry` JSON and `RuleSet` RON in this step if not yet present — bundle into this same step since they are trivial and identical in shape).
  - System `materialize_chunks` querying `(Entity, &Chunk, &ChunkCellMatrix), Without<MaterializedChunk>` and inserting `MaterializedChunk` once registries are loaded.
- [`src/systems/terrain/mod.rs`](../../../../src/systems/terrain/mod.rs) — declare `pub mod material_plugin;` (create `mod.rs` if absent).
- Register the plugin in the engine bootstrap (find via `rg "DefaultPlugins"` — single insertion site).

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 material_plugin_app_boot -- --nocapture` (build a minimal `App` with the plugin and tick once; assert no panic and assets registered).

**Matrix update:** §1 `Material plugin` row → **Applied**; §7 rows for the three assets → **Applied**.

**Definition of done:**
- [ ] Build + boot test pass.
- [ ] Loaders for `TagRegistry` (JSON) and `RuleSet` (RON) shipped in the same step.
- [ ] No second mutation path: F8/asset-editor stay file-edit only.

---

## U5-S03 preview color via `MaterialDef.preview_color`

**Goal:** Replace the hardcoded RGBA in `world_preview.rs` `PreviewMode::Biome` arm with a registry lookup.

**Anchor reads:** orchestrator §§1–2 · matrix §6 row `Biome` · [`src/gui/editor/world_preview.rs`](../../../../src/gui/editor/world_preview.rs) (lines around `biome_to_color`) · `MaterialRegistry` from U3.

**Touch:**
- [`src/gui/editor/world_preview.rs`](../../../../src/gui/editor/world_preview.rs):
  - Add `Res<MaterialRegistry>` (via `Assets<MaterialRegistry>` + a single `Handle<MaterialRegistry>` resource initialized by the plugin).
  - When `PreviewMode::Biome`: look up the dominant `MaterialId` for the tile via `MaterializedChunk` if present; fall back to `family_to_default_material(family, &registry).preview_color`.
  - Keep the legacy `biome_to_color` function as `#[deprecated(note = "use MaterialDef.preview_color")]` for one cycle.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 preview_uses_material_def_color -- --nocapture` (synthetic registry returns the expected RGBA for one family).

**Matrix update:** §6 row `Biome` → **Applied**; §6 row `Material color` → **Applied**.

**Definition of done:**
- [ ] Build + test pass.
- [ ] No new color literals introduced; all colors come from registry.

---

## U5-S04 add `PreviewMode::Tag(TagId)`

**Goal:** New preview overlay highlighting cells whose `TagSet` contains a chosen `TagId`.

**Anchor reads:** orchestrator §§1–2 · matrix §6 row `Tag overlay` · [`src/gui/editor/world_gen_ui.rs`](../../../../src/gui/editor/world_gen_ui.rs) (`enum PreviewMode`).

**Touch:**
- [`src/gui/editor/world_gen_ui.rs`](../../../../src/gui/editor/world_gen_ui.rs):
  - Add variant `Tag(TagId)` to `PreviewMode`.
  - Add a UI control (egui combo box) populated from the loaded `TagRegistry` to pick the active tag.
- [`src/gui/editor/world_preview.rs`](../../../../src/gui/editor/world_preview.rs):
  - Add a match arm rendering `[255, 220, 0, 255]` where the cell `TagSet` contains the chosen `TagId`, transparent otherwise (RGBA configurable later via overlay tuning — no new tuning struct in this step).

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 preview_tag_overlay_highlights_match -- --nocapture`.

**Matrix update:** §6 row `Tag overlay` → **Applied**.

**Definition of done:**
- [ ] Build + test pass.
- [ ] Existing preview modes still compile and behave unchanged.

---

## U5-S05 phase close — matrix flip

**Goal:** Promote U5 status across docs.

**Anchor reads:** orchestrator §4 · matrix §10.

**Touch:**
- [`prompts/matrix/terrain_biome/material_unification_matrix_v1.md`](../material_unification_matrix_v1.md) — §10 row **U5** → **Applied**; §9 row `F8 egui panel` → **Applied** (now exposes registry summary + tag preview).
- [`prompts/guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) §4 phase index **U5** → **Applied**.

**Verify:**
- `cargo test -p proc_A_dine01 material_ -- --nocapture`
- `cargo test -p proc_A_dine01 preview_ -- --nocapture`
- `cargo run -p proc_A_dine01 --bin world_generator` (smoke run; close window after seeing preview render with material colors — manual gate, surface to human).

**Definition of done:**
- [ ] All U5 tests green.
- [ ] U5 row Applied in both files.
- [ ] Manual smoke confirmed by human (mark in PR description).

---

## Open carries (📎)

- Tilemap renderer integration (`bevy_ecs_tilemap`) → U6 (optional).
- Multi-layer overlay/resource layers → U7-S04.
- Inspector hook for registries (matrix §9 row `Inspector`) → deferred until `bevy-inspector-egui` re-pinned.
