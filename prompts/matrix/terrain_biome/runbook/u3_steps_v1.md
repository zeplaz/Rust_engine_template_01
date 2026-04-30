# U3 — Rust scaffolding (registries + AssetLoaders) `v1`

> **Pair:** orchestrator [`../../../guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) §§1–6 · matrix [`../material_unification_matrix_v1.md`](../material_unification_matrix_v1.md) §§1, 2, 4, 7, 10 · designer [`../../../designer_questions/terrain_world/material_tag_rule_system_v1.md`](../../../designer_questions/terrain_world/material_tag_rule_system_v1.md).

**Phase goal:** Land the Rust types and asset-loaders for `MaterialRegistry`, `TagRegistry`, `RuleSet`, and the deterministic `resolve_material` resolver. **No `ChunkCellMatrix` yet** (that is U4).

**Anchor set (always):** orchestrator §§1–2 · matrix §§1, 2, 4, 7, 10 · this pack · the single edited file.

**Halt rules:** see orchestrator §6. Two consecutive build/test failures on a single step ⇒ stop.

---

## U3-S01 add `ron` dependency

**Goal:** Add the `ron` crate so `material_rules.ron` can be parsed.

**Anchor reads:** orchestrator §§1–2 · this pack · [`Cargo.toml`](../../../../Cargo.toml).

**Touch:**
- [`Cargo.toml`](../../../../Cargo.toml) — add `ron = "0.8"` under `[dependencies]` (pin verified at run time; `ASK:` if `cargo` rejects the version).

**Verify:**
- `cargo check -p proc_A_dine01`

**Matrix update:** none (preparatory).

**Definition of done:**
- [ ] `cargo check` passes.
- [ ] `Cargo.lock` updated and committed.
- [ ] No other deps added.

---

## U3-S02 wire `terrain::material` module

**Goal:** Create the empty module that the next steps populate.

**Anchor reads:** orchestrator §§1–2 · [`src/terrain/mod.rs`](../../../../src/terrain/mod.rs).

**Touch:**
- [`src/terrain/mod.rs`](../../../../src/terrain/mod.rs) — add `pub mod material;` after `pub mod ecology;`.
- New file `src/terrain/material/mod.rs` containing only `// material/tag/rule unification — see prompts/guides/terrain_unification_runbook_v1.md` and `pub mod registry;` placeholders gated behind `#[allow(unused)]` until U3-S03 lands the file.

**Verify:**
- `cargo check -p proc_A_dine01` (compiles even with empty submodules — keep `mod.rs` minimal).

**Matrix update:** none.

**Definition of done:**
- [ ] Build passes.
- [ ] No new public re-exports added yet.

---

## U3-S03 `MaterialId` + `MaterialDef` + `MaterialRegistry`

**Goal:** Land the runtime registry types and a JSON loader path that mirrors [`src/terrain/generation/tuning_io.rs`](../../../../src/terrain/generation/tuning_io.rs).

**Anchor reads:** orchestrator §§1–2 · matrix §§1, 2, 4 · [`assets/config/terrain/material_registry.example.json`](../../../../assets/config/terrain/material_registry.example.json) · [`src/terrain/generation/tuning_io.rs`](../../../../src/terrain/generation/tuning_io.rs).

**Touch:**
- New `src/terrain/material/registry.rs` with:
  - `pub struct MaterialId(pub u16);` (newtype, `Copy + Eq + Hash`).
  - `pub struct MaterialDef { name: String, family: TerrainClass, tags: Vec<String>, properties: serde_json::Value, preview_color: [u8; 4] }` (`tags` stays `Vec<String>` here; interning happens against `TagRegistry` at load — kept separate from `TagSet` per matrix §1).
  - `pub struct MaterialRegistry { schema_version: u32, materials: Vec<MaterialDef>, name_to_id: HashMap<String, MaterialId> }` plus `pub fn load_from_json(path: &str) -> std::io::Result<Self>` (mirrors `tuning_io::load_overlay`).
- `src/terrain/material/mod.rs` — declare `pub mod registry;` and re-export `MaterialId`, `MaterialDef`, `MaterialRegistry`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 material_registry_loads_example -- --nocapture` (inline `#[cfg(test)] mod tests` in `registry.rs` reading the example JSON).

**Matrix update:** [`material_unification_matrix_v1.md`](../material_unification_matrix_v1.md) §1 rows for `MaterialId(u16)` and `MaterialDef + MaterialRegistry` → **Partial** (loader exists; AssetLoader follows in S04).

**Definition of done:**
- [ ] Build + test pass.
- [ ] `name_to_id` rebuilt deterministically from input order.
- [ ] No `unwrap()` in load path; errors map to `std::io::Error` like `tuning_io.rs`.

---

## U3-S04 `MaterialRegistry` `AssetLoader` (Bevy 0.18)

**Goal:** Expose `MaterialRegistry` as a Bevy `Asset` with a JSON `AssetLoader` (`.material_registry.json`).

**Anchor reads:** orchestrator §§1–2 · matrix §7 (hot reload) · `src/terrain/material/registry.rs` (just authored).

**Touch:**
- `src/terrain/material/registry.rs` — derive `Asset` and `TypePath`; add `pub struct MaterialRegistryLoader;` implementing `bevy::asset::AssetLoader` for `.material_registry.json` extension.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 material_registry_loader_extension -- --nocapture` (asserts `extensions()` contains `"material_registry.json"`).

**Matrix update:** §7 row `MaterialRegistry` → **Partial**; §1 rows promoted by S03 stay **Partial** until plugin registration in U5.

**Definition of done:**
- [ ] Loader compiles against Bevy 0.18 `AssetLoader` trait (`load(&self, reader, settings, ctx)` shape).
- [ ] No plugin registration here (deferred to U5).

---

## U3-S05 `TagId` + `TagRegistry` + `TagSet`

**Goal:** Add tag interning and a fixed-width `TagSet`.

**Anchor reads:** orchestrator §§1–2 · matrix §§1, 7 · designer doc §3 (📎 §43 representation choice) · [`assets/config/terrain/tag_registry.example.json`](../../../../assets/config/terrain/tag_registry.example.json).

**Touch:**
- New `src/terrain/material/tags.rs` with:
  - `pub struct TagId(pub u16);` (`Copy + Eq + Hash`).
  - `pub struct TagSet([u64; 4]);` (default 256-tag cap; **`ASK:`** if more than 256 tags ever needed — drives §43).
  - `impl TagSet { pub fn insert(&mut self, TagId); pub fn contains(&self, TagId) -> bool; pub fn union(&self, &Self) -> Self; pub fn intersects_all(&self, &Self) -> bool; }`
  - `pub struct TagRegistry { schema_version: u32, tags: Vec<TagDef>, name_to_id: HashMap<String, TagId> }` + `load_from_json`.
- `src/terrain/material/mod.rs` — `pub mod tags;` re-exporting `TagId`, `TagSet`, `TagRegistry`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 tag_registry_loads_example -- --nocapture`
- `cargo test -p proc_A_dine01 tag_set_set_ops -- --nocapture` (insert/contains/union round-trip).

**Matrix update:** §1 row `TagId(u16) + TagRegistry + TagSet` → **Partial**.

**Definition of done:**
- [ ] Build + both tests pass.
- [ ] `TagSet` width documented in module-level comment with `ASK:` link to impl Q §43.

---

## U3-S06 `MaterialRule` + `RuleSet` (RON)

**Goal:** Parse `material_rules.ron` and store rules in deterministic order.

**Anchor reads:** orchestrator §§1–2 · matrix §§1, 4 · [`assets/config/terrain/material_rules.example.ron`](../../../../assets/config/terrain/material_rules.example.ron).

**Touch:**
- New `src/terrain/material/rules.rs`:
  - `pub struct MaterialRule { required: Vec<String>, forbidden: Vec<String>, family_filter: Option<TerrainClass>, result_name: String, priority: i32 }` (file form uses names; resolver step post-resolves to ids — keeps saves name-stable).
  - `pub struct RuleSet { schema_version: u32, rules: Vec<MaterialRule> }` + `pub fn load_from_ron(path: &str) -> std::io::Result<Self>`.
  - **Sort on load**: `priority` desc, then file index asc (record `rule_index` before sort).
- `src/terrain/material/mod.rs` — `pub mod rules;`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 material_rules_load_example_ron -- --nocapture` (asserts at least one rule loaded with expected name).
- `cargo test -p proc_A_dine01 rule_set_sort_stable_under_equal_priority -- --nocapture`.

**Matrix update:** §1 row `MaterialRule + RuleSet + resolver` → **Partial** (resolver lands in S07).

**Definition of done:**
- [ ] Build + both tests pass.
- [ ] Sort is **stable** for equal priority (no shuffle).

---

## U3-S07 `resolve_material` resolver

**Goal:** Pure deterministic function selecting `MaterialId` from `(family, weights, tag_set, &RuleSet, &MaterialRegistry)`.

**Anchor reads:** orchestrator §§1–2 · matrix §4 (resolver contract) · `registry.rs`, `tags.rs`, `rules.rs` (just authored).

**Touch:**
- New `src/terrain/material/resolver.rs`:
  - `pub fn resolve_material(family: TerrainClass, weights: &BiomeWeights, tags: TagSet, rules: &RuleSet, registry: &MaterialRegistry, tag_registry: &TagRegistry) -> MaterialId`
  - Iterate sorted rules; first rule whose `family_filter` matches (or is `None`) and whose `required` tags are all in `tags` and `forbidden` are all absent ⇒ returns `registry.name_to_id[result_name]`.
  - Fallback: registry-declared default per family (`MaterialDef.name` for the matching family); never `MaterialId(0)` silently — return `Err` or panic in debug per matrix §4.
- `src/terrain/material/mod.rs` — `pub mod resolver;` re-exporting `resolve_material`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 resolver_priority_tiebreak_deterministic -- --nocapture` (two rules same priority, asserts file-order winner).
- `cargo test -p proc_A_dine01 resolver_falls_back_to_family_default -- --nocapture`.

**Matrix update:** §1 row `MaterialRule + RuleSet + resolver` → **Applied**; §4 rows → **Applied**.

**Definition of done:**
- [ ] Build + both tests pass.
- [ ] No second classifier introduced (no call to anything other than `classify_biome` will be added later in U4 pass 3).
- [ ] Function is `pub` and takes only borrowed inputs (so it is trivially testable without ECS).

---

## U3-S08 phase close — matrix flip

**Goal:** Promote U3 status in the matrix and orchestrator phase index.

**Anchor reads:** orchestrator §4 · matrix §10.

**Touch:**
- [`prompts/matrix/terrain_biome/material_unification_matrix_v1.md`](../material_unification_matrix_v1.md) — §10 row **U3** → **Applied**.
- [`prompts/guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) — §4 phase index **U3** → **Applied**.

**Verify:**
- `cargo check -p proc_A_dine01` (sanity).
- `cargo test -p proc_A_dine01 material_ -- --nocapture` (run all `material_*` tests; ensure full green).

**Matrix update:** see Touch.

**Definition of done:**
- [ ] All U3 tests green.
- [ ] U3 row Applied in both files.
- [ ] No invariant from orchestrator §1 violated.
- [ ] Commit suggestion surfaced to human (do **not** auto-commit).

---

## Open carries (📎)

These remain `ASK:` and do **not** block U3 close:

- §43 `TagSet` width if total tags > 256.
- §44 resolver memoization key (defer to U7).
- §47 agent write authority on tags (U4 pass 5 stub).
- §48 `weight_predicate` schema (currently rules use `required`/`forbidden` only — extension is U7).
