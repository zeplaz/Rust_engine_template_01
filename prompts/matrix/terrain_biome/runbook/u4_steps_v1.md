# U4 — `ChunkCellMatrix` + multi-pass pipeline `v1`

> **Pair:** orchestrator [`../../../guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) · matrix [`../material_unification_matrix_v1.md`](../material_unification_matrix_v1.md) §3 · designer [`../../../designer_questions/terrain_world/material_tag_rule_system_v1.md`](../../../designer_questions/terrain_world/material_tag_rule_system_v1.md). **Pre-req:** U3 must be **Applied** in matrix §10.

**Phase goal:** Build the per-chunk SoA grid and the pass pipeline that ends in `MaterialId` per cell. Pass 3 calls **only** the existing `classify_biome(...)` — no second classifier.

**Anchor set (always):** orchestrator §§1–2 · matrix §3 · this pack · the single edited file.

**Halt rules:** orchestrator §6.

---

## U4-S01 define `ChunkCellMatrix` (SoA)

**Goal:** Land the per-chunk SoA grid type with no behavior beyond construction.

**Anchor reads:** orchestrator §§1–2 · matrix §3 · [`src/terrain/biome.rs`](../../../../src/terrain/biome.rs) (for `BiomeWeights` / `TerrainClass`) · `src/terrain/material/tags.rs`.

**Touch:**
- New `src/terrain/generation/cell_matrix.rs`:
  - `pub struct ChunkCellMatrix { pub size: UVec2, pub elevation: Vec<f32>, pub moisture: Vec<f32>, pub temperature: Vec<f32>, pub tags: Vec<TagSet>, pub family: Vec<TerrainClass>, pub weights: Vec<BiomeWeights> }`
  - `impl ChunkCellMatrix { pub fn new(size: UVec2) -> Self; pub fn idx(&self, x: u32, y: u32) -> usize; }`
- [`src/terrain/generation/mod.rs`](../../../../src/terrain/generation/mod.rs) — declare `pub mod cell_matrix;`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 chunk_cell_matrix_alloc -- --nocapture` (asserts vec lengths == `size.x * size.y`).

**Matrix update:** §1 row `ChunkCellMatrix` → **Partial**.

**Definition of done:**
- [ ] Build + test pass.
- [ ] No methods beyond `new`/`idx` yet (passes are separate steps).

---

## U4-S02 pass 1 — field fill from existing noise

**Goal:** Populate `elevation`/`moisture`/`temperature` for one chunk using the existing noise stack (no new generators).

**Anchor reads:** orchestrator §§1–2 · matrix §3 row 1 · [`src/terrain/generation/terrain_noise.rs`](../../../../src/terrain/generation/terrain_noise.rs) · `cell_matrix.rs`.

**Touch:**
- New `src/terrain/generation/passes/mod.rs` (declare submodules).
- New `src/terrain/generation/passes/p1_fields.rs`:
  - `pub fn fill_fields(matrix: &mut ChunkCellMatrix, chunk_xy: IVec2, params: &WorldGenParams, tuning: Option<&NoiseSamplingTuning>)`
  - Calls existing fBm builders from `terrain_noise.rs`; writes into the three field vectors only.
- [`src/terrain/generation/mod.rs`](../../../../src/terrain/generation/mod.rs) — `pub mod passes;`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 pass1_fields_deterministic -- --nocapture` (same seed twice ⇒ same elevation vec).

**Matrix update:** §3 row **Pass 1** → **Applied**.

**Definition of done:**
- [ ] Build + test pass.
- [ ] No new noise types introduced.

---

## U4-S03 pass 2 — threshold tags

**Goal:** Convert raw fields into baseline tags (`lowland`, `wet`, `hot`, …) driven by `BiomeTuning` + (optional) `tag_tuning`.

**Anchor reads:** orchestrator §§1–2 · matrix §3 row 2 · [`src/terrain/biome.rs`](../../../../src/terrain/biome.rs) (`BiomeTuning`) · `cell_matrix.rs`.

**Touch:**
- New `src/terrain/generation/passes/p2_threshold_tags.rs`:
  - `pub fn apply_threshold_tags(matrix: &mut ChunkCellMatrix, tuning: &BiomeTuning, tag_registry: &TagRegistry)`
  - Threshold table is **data-driven**: read from `BiomeTuning` for now (📎 dedicated `tag_tuning` is impl Q **§42**, deferred to U7).
- `passes/mod.rs` — `pub mod p2_threshold_tags;`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 pass2_threshold_tags_lowland_wet -- --nocapture` (synthetic field values produce expected tag bits).

**Matrix update:** §3 row **Pass 2** → **Applied**.

**Definition of done:**
- [ ] Build + test pass.
- [ ] No tag literals hardcoded — all looked up via `tag_registry.name_to_id`.

---

## U4-S04 pass 3 — `classify_biome` (single classifier)

**Goal:** Fill `family` and `weights` arrays by calling the existing `classify_biome(...)` exactly once per cell.

**Anchor reads:** orchestrator §§1–2 (esp. invariant 3) · matrix §3 row 3 · [`src/terrain/biome.rs`](../../../../src/terrain/biome.rs) (`classify_biome`).

**Touch:**
- New `src/terrain/generation/passes/p3_classify.rs`:
  - `pub fn classify_cells(matrix: &mut ChunkCellMatrix, tuning: &BiomeTuning, tag_registry: &TagRegistry)`
  - For each cell: call `classify_biome(elevation, moisture, temperature)` → write `family` and `weights`; add biome-derived tags (e.g. `marine`, `coastal`, `boreal`) by mapping `BiomeId` → tag name via `tag_registry`.
- `passes/mod.rs` — `pub mod p3_classify;`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 pass3_uses_classify_biome_only -- --nocapture` (test that result for synthetic input matches direct `classify_biome` call).

**Matrix update:** §3 row **Pass 3** → **Applied**.

**Definition of done:**
- [ ] Build + test pass.
- [ ] **Invariant check:** the file imports `classify_biome` and calls **no other** classifier function.

---

## U4-S05 pass 4 + 5 stubs (hydrology / agent overlay)

**Goal:** Reserve no-op functions so downstream code can call them without `cfg`-gating; full implementation lands later.

**Anchor reads:** orchestrator §§1–2 · matrix §3 rows 4–5 · checklist items **55–58, 71–74** (`implementation_questions_v1.md`).

**Touch:**
- New `src/terrain/generation/passes/p4_hydrology.rs` — `pub fn apply_hydrology(_matrix: &mut ChunkCellMatrix) { /* TODO: items 55-58 */ }`.
- New `src/terrain/generation/passes/p5_agent_overlay.rs` — `pub fn apply_agent_overlay(_matrix: &mut ChunkCellMatrix) { /* TODO: items 71-74 */ }`.
- `passes/mod.rs` — declare both modules.

**Verify:**
- `cargo check -p proc_A_dine01` (no test required — these are intentional no-ops).

**Matrix update:** §3 rows **Pass 4**, **Pass 5** → **Partial** (stub).

**Definition of done:**
- [ ] Build passes.
- [ ] Both functions compile to no-ops; doc comments cite the impl-question numbers.

---

## U4-S06 pass 6 — materialize via `resolve_material`

**Goal:** Fill `MaterializedChunk.materials` (defined here as a local struct; `MaterializedChunk` ECS component lands in U5).

**Anchor reads:** orchestrator §§1–2 · matrix §3 row 6 · `src/terrain/material/resolver.rs` (from U3).

**Touch:**
- New `src/terrain/generation/passes/p6_materialize.rs`:
  - `pub struct MaterializedChunkData { pub materials: Vec<MaterialId> }` (plain data; ECS wrapper added in U5-S01).
  - `pub fn materialize(matrix: &ChunkCellMatrix, rules: &RuleSet, registry: &MaterialRegistry, tag_registry: &TagRegistry) -> MaterializedChunkData`.
  - Iterates cells; calls `resolve_material(family, &weights, tags, rules, registry, tag_registry)`.
- `passes/mod.rs` — `pub mod p6_materialize;`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 materialize_uses_resolver_e2e_small_chunk -- --nocapture` (4×4 chunk with example registries/rules; assert at least one `loam_wet` and one `basalt_dense`).

**Matrix update:** §3 row **Pass 6** → **Applied**; §1 row `ChunkCellMatrix` → **Applied**.

**Definition of done:**
- [ ] Build + test pass.
- [ ] No new resolver introduced — calls `resolve_material` from U3.

---

## U4-S07 phase close — matrix flip

**Goal:** Promote U4 status.

**Anchor reads:** orchestrator §4 · matrix §10.

**Touch:**
- [`prompts/matrix/terrain_biome/material_unification_matrix_v1.md`](../material_unification_matrix_v1.md) — §10 row **U4** → **Applied**.
- [`prompts/guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) §4 phase index **U4** → **Applied**.

**Verify:**
- `cargo test -p proc_A_dine01 pass -- --nocapture` (run all `pass*` tests + `materialize_*`).

**Definition of done:**
- [ ] All U4 tests green.
- [ ] U4 row Applied in both files.
- [ ] Invariant 3 (single classifier) re-checked by reading `p3_classify.rs` once more.

---

## Open carries (📎)

- Hydrology / agent overlay full implementations: U7 + designer follow-ups.
- Cross-chunk neighbor reads for spatial tags (impl Q §57): U7-S04 spatial pass.
- `tag_tuning` separation from `BiomeTuning` (impl Q §42): U7.
