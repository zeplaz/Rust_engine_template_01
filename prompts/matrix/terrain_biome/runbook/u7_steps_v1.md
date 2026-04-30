# U7 — Invalidation, multi-layer, debug, packs `v1`

> **Pair:** orchestrator [`../../../guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) · matrix [`../material_unification_matrix_v1.md`](../material_unification_matrix_v1.md) §§13–18, §10 row **U7** · checklist [`../../../designer_questions/terrain_world/implementation_questions_v1.md`](../../../designer_questions/terrain_world/implementation_questions_v1.md) items **49–78**. **Pre-req:** U5 must be **Applied** (U6 may remain Partial).

**Phase goal:** Land the production-quality glue: invalidation graph, partial chunk rebuild, multi-layer rendering, debug inspectors, and world profiles / packs.

**Anchor set:** orchestrator §§1–2 · matrix §§13–18 · checklist 49–78 · this pack · the single edited file.

**Halt rules:** orchestrator §6. Additionally — any change that would silently re-run the **whole** world when one chunk's deps changed ⇒ **halt** (matrix §13 invariant).

---

## U7-S01 `ChunkDependency` hashes

**Goal:** Per-chunk hash record for invalidation decisions.

**Anchor reads:** orchestrator §§1–2 · matrix §13 · checklist items **50, 53, 54**.

**Touch:**
- New `src/terrain/material/dependency.rs`:
  - `#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)] pub struct ChunkDependency { pub source_noise_id: u64, pub registry_hash: u64, pub rules_hash: u64, pub tags_hash: u64, pub tuning_hash: u64 }`.
  - Helper `pub fn hash_asset<T: Hash>(asset: &T) -> u64` using `std::hash::DefaultHasher` (deterministic for our use; **`ASK:`** if cross-process determinism required → switch to `xxh3`).
- `src/terrain/material/mod.rs` — declare `pub mod dependency;`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 chunk_dependency_hash_stable -- --nocapture` (same input ⇒ same hash within process).

**Matrix update:** §13 — note new dependency component in row context (no row flip yet).

**Definition of done:**
- [ ] Build + test pass.
- [ ] No global mutable state for hashes.

---

## U7-S02 `ChunkDirty` flag + version diff system

**Goal:** Each frame, compare each chunk's `ChunkDependency` against the currently loaded asset versions; mark mismatched chunks dirty.

**Anchor reads:** orchestrator §§1–2 · matrix §16 row `ChunkDirty` · `dependency.rs`.

**Touch:**
- `src/terrain/material/dependency.rs` (extend):
  - `#[derive(Component, Default, Debug)] pub struct ChunkDirty { pub passes: u8 }` (bitmask: `0b00111111` = passes 1–6 dirty).
- `src/systems/terrain/material_plugin.rs` (extend from U5):
  - Add system `mark_chunks_dirty_on_asset_change` listening to `AssetEvent<MaterialRegistry>`, `AssetEvent<TagRegistry>`, `AssetEvent<RuleSet>`, plus `WorldGenTuningOverlay` change events. Sets `ChunkDirty.passes` per matrix §13 decision table.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 dirty_marker_set_on_registry_change -- --nocapture` (synthetic `AssetEvent::Modified` for `MaterialRegistry` ⇒ all chunks have `ChunkDirty.passes & 0b00100000 != 0`).

**Matrix update:** §16 row `ChunkDirty bitmask` → **Applied**.

**Definition of done:**
- [ ] Build + test pass.
- [ ] System reads only `Changed<>` / `EventReader<>`; never iterates all chunks unconditionally.

---

## U7-S03 partial rebuild dispatcher

**Goal:** When `ChunkDirty.passes` is non-zero, re-run **only** the affected passes for that chunk.

**Anchor reads:** orchestrator §§1–2 · matrix §13 (decision table) · checklist item **51**.

**Touch:**
- `src/systems/terrain/material_plugin.rs` (extend):
  - System `rebuild_dirty_chunks` — for each `(Entity, &mut ChunkCellMatrix, &mut ChunkDirty, ...)` re-runs the smallest sufficient pass subset:
    - Registry change ⇒ pass 6 only.
    - Rules change ⇒ pass 6 only.
    - Tag registry change ⇒ passes 2–6.
    - Tuning change ⇒ passes 2–6.
    - Noise/seed change ⇒ passes 1–6.
  - Clears `ChunkDirty.passes` on completion; updates `ChunkDependency` with new hashes.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 partial_rebuild_registry_only_runs_pass6 -- --nocapture` (counter inside test passes asserts pass-1 runs only once across two registry edits).

**Matrix update:** §13 footnote “ECS sketch” — promote to row in §16: `Partial rebuild dispatcher` → **Applied**.

**Definition of done:**
- [ ] Build + test pass.
- [ ] No path re-runs full world when one chunk changed.
- [ ] Hashes refreshed after rebuild.

---

## U7-S04 multi-layer tilemap stack

**Goal:** Add overlay (z=10) and resource (z=20) layers as separate tilemaps sharing the chunk grid.

**Anchor reads:** orchestrator §§1–2 · matrix §15 · checklist items **63–66**.

**Touch:**
- `src/render/tilemap_adapter.rs` (extend, still gated by `bevy_tilemap_adapter`):
  - Spawn three tilemap entities per chunk: terrain (z=0), overlay (z=10), resource (z=20).
  - Overlay system writes from active `PreviewMode` (height/moisture/temperature/tag) — reuses `world_preview` color helpers.
  - Resource layer reads from a **new** `MaterializedResources` component (defined here as `pub struct MaterializedResources { pub ids: Vec<MaterialId> }`); no rules wired yet (📎 separate `resource_rules.ron` is impl Q **§62**, deferred).

**Verify:**
- `cargo check -p proc_A_dine01 --features bevy_tilemap_adapter`
- `cargo test -p proc_A_dine01 --features bevy_tilemap_adapter multi_layer_spawn_three_tilemaps -- --nocapture`.

**Matrix update:** §15 rows `Overlay / debug`, `Resources` → **Partial**; §10 row **U6** → **Applied** (multi-layer minimum reached).

**Definition of done:**
- [ ] Default build (feature off) clean.
- [ ] Feature-on test passes.
- [ ] Independent visibility toggles wired (boolean per layer in F8 panel).

---

## U7-S05 rule trace component (dev tools)

**Goal:** Optional per-cell record of the winning rule index, behind a `dev_tools` feature.

**Anchor reads:** orchestrator §§1–2 · matrix §17 row `Audit` · checklist item **75**.

**Touch:**
- [`Cargo.toml`](../../../../Cargo.toml) — add feature `dev_tools = []` (no extra deps).
- `src/terrain/material/runtime.rs` (extend, gated):
  - `#[cfg(feature = "dev_tools")] pub struct RuleTrace { pub winners: Vec<u32> }` storing `rule_index` per cell.
- `src/terrain/generation/passes/p6_materialize.rs` (extend, gated):
  - When `dev_tools` is on, also produce `RuleTrace` alongside `MaterializedChunkData`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo check -p proc_A_dine01 --features dev_tools`
- `cargo test -p proc_A_dine01 --features dev_tools rule_trace_records_winning_index -- --nocapture`.

**Matrix update:** §17 row `Audit` → **Partial** (component only; on-disk JSONL log deferred — checklist item **72**).

**Definition of done:**
- [ ] Both builds clean.
- [ ] Trace cost is zero in default build.

---

## U7-S06 world profile loader

**Goal:** A single `WorldProfile` asset bundles registry / tags / rules / tuning paths so a scenario can pick one.

**Anchor reads:** orchestrator §§1–2 · matrix §18 row `World profile` · checklist item **76**.

**Touch:**
- New `assets/config/terrain/profiles/default.ron` (committed example).
- New `src/terrain/material/profile.rs`:
  - `#[derive(Asset, TypePath, Deserialize)] pub struct WorldProfile { schema_version: u32, material_registry: String, tag_registry: String, material_rules: String, tuning: Option<String> }`.
  - RON `AssetLoader` for `.world_profile.ron`.
  - `pub fn apply_profile(commands: &mut Commands, asset_server: &AssetServer, profile: &WorldProfile) -> ProfileHandles` returning the three handles (registry/tags/rules).
- `src/systems/terrain/material_plugin.rs` — register the loader; expose a `WorldProfileSelector` resource.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 world_profile_loads_default_ron -- --nocapture`.

**Matrix update:** §18 row `World profile` → **Applied**.

**Definition of done:**
- [ ] Build + test pass.
- [ ] Saves continue to store `MaterialDef.name`, not raw ids — verified by an inline doc comment near `apply_profile`.

---

## U7-S07 phase close — matrix flip

**Goal:** Promote U7 + §§13–18 status across docs.

**Anchor reads:** orchestrator §4 · matrix §§10, 13–18.

**Touch:**
- [`prompts/matrix/terrain_biome/material_unification_matrix_v1.md`](../material_unification_matrix_v1.md):
  - §10 row **U7** → **Applied**.
  - §13 dependency table — leave as-is (rows describe behavior, not statuses).
  - §15 add status column entries → **Applied** (terrain), **Partial** (overlay, resources).
  - §16 → **Applied** for rows that have shipped (`ChunkDirty`, partial rebuild, off-thread fill if completed; otherwise leave **Partial**).
  - §17 row `Audit` → **Partial**.
  - §18 row `World profile` → **Applied**.
- [`prompts/guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) §4 — **U7** → **Applied**.

**Verify:**
- `cargo test -p proc_A_dine01 -- --nocapture` (full suite — surface failures to human; do not silently skip).

**Definition of done:**
- [ ] All tests green at default features.
- [ ] All tests green at `--features bevy_tilemap_adapter` and `--features dev_tools`.
- [ ] Matrix + orchestrator phase index updated.
- [ ] Open `📎` carries below logged in [`implementation_questions_v1.md`](../../../designer_questions/terrain_world/implementation_questions_v1.md) (no new numbers without `ASK:`).

---

## Open carries (📎) — promoted to checklist `ASK:`

- Resource ruleset (`resource_rules.ron`) — checklist item **62**.
- On-disk audit log (JSONL) — checklist item **72**.
- Cross-process deterministic hash (xxh3) — checklist item **50**.
- Modded `RuleSet` priority bands — checklist item **77**.
- Tag visualizer GPU path — checklist item **78**.
