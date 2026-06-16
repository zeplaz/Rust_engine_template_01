# G1 — Hydrology gap remediation `v1` (FULL — historical atomic steps)

> **Archived:** Copy preserved 2026-05-01. **Routing:** G1 is **Applied**; active pointer + verification snapshot: [`../../../matrix/gap_remediation/runbook/g1_hydrology_steps_v1.md`](../../../matrix/gap_remediation/runbook/g1_hydrology_steps_v1.md).

---

# G1 — Hydrology gap remediation `v1`

> **Pair:** orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md) · hunt guide [`../../../guides/implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md) · terrain matrix [`../../terrain_biome/material_unification_matrix_v1.md`](../../terrain_biome/material_unification_matrix_v1.md) §3 pass 4. **Pre-req:** terrain U4-U7 are treated as Applied; G1 fixes the remaining p4 hydrology gap, not the closed U-phase history.

**Phase goal:** Replace parallel / stub hydrology paths with one canonical flow implementation, then feed both chunk p4 tags and legacy ECS river/lake visuals from that implementation.

**Anchor set (always):** orchestrator §§1-2 · gap hunt §§1-4 · terrain matrix §§1, 3, 10 · this pack · the single edited file.

**Halt rules:** orchestrator §6.

---

## G1-S01 `apply_hydrology` signature + subengine routing

**Goal:** Create the canonical p4 hydrology entry point and mark the legacy subengine no-op loops as out-of-scope rather than silently pretending to generate rivers/lakes.

**Anchor reads:** orchestrator §§1-2 · gap hunt §2E · [`../../../guides/world_assets_tools_rulebook_v1.md`](../../../guides/world_assets_tools_rulebook_v1.md) §1 · [`../../terrain_biome/material_unification_matrix_v1.md`](../../terrain_biome/material_unification_matrix_v1.md) §3.

**Touch:**
- [`../../../../src/terrain/generation/passes/p4_hydrology.rs`](../../../../src/terrain/generation/passes/p4_hydrology.rs):
  - Replace the stub-only body with `pub struct HydrologyParams { pub water_line: f32, pub river_threshold: f32, pub erosion_slope_threshold: f32, pub silt_moisture_threshold: f32 }`.
  - Add `impl Default for HydrologyParams` using conservative defaults derived from `BiomeTuning::default()` where possible.
  - Add `pub fn apply_hydrology_with_params(matrix: &mut ChunkCellMatrix, params: &HydrologyParams)`.
  - Keep `pub fn apply_hydrology(matrix: &mut ChunkCellMatrix)` as the default wrapper.
- [`../../../../src/bevysubengines/world_generator_plugin.rs`](../../../../src/bevysubengines/world_generator_plugin.rs):
  - Replace lines around the river/lake no-op loops with one explicit comment: legacy subengine is not the designer-facing path per `world_assets_tools_rulebook_v1.md` §1; hydrology behavior lives in `terrain::generation::passes::p4_hydrology` / the G1 pack.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 apply_hydrology_signature_compiles -- --nocapture`

**Matrix / routing update:** none yet; p4 remains **Partial** until G1-S06.

**Definition of done:**
- [x] Build + test pass.
- [x] `p4_hydrology.rs` is no longer an empty stub.
- [x] `world_generator_plugin.rs` no longer contains no-op river/lake loops that imply real generation.
- [x] No runtime hydrology behavior is invented beyond the signature/default scaffolding.

---

## G1-S02 ECS path delegates to canonical hydrology result

**Goal:** Extract river/lake computation out of `world_generator_enhanced.rs` inline loops so the ECS visual path can later consume the same hydrology result as p4.

**Anchor reads:** orchestrator §§1-2 · [`../../../../src/terrain/generation/world_generator_enhanced.rs`](../../../../src/terrain/generation/world_generator_enhanced.rs) around `generate_rivers` / `generate_lakes` · [`../../../../src/terrain/generation/passes/p4_hydrology.rs`](../../../../src/terrain/generation/passes/p4_hydrology.rs).

**Touch:**
- [`../../../../src/terrain/generation/world_generator_enhanced.rs`](../../../../src/terrain/generation/world_generator_enhanced.rs):
  - Extract `compute_hydrology(height_grid, params) -> HydrologyResult { rivers: Vec<Vec<(u32, u32)>>, lakes: Vec<LakeRegion> }`.
  - Move the current greedy-descent river path and circular lake mask into this result-producing helper as a transitional implementation.
  - Rewrite `generate_rivers` and `generate_lakes` to consume `HydrologyResult` instead of computing paths inline.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 generate_rivers_uses_compute_hydrology -- --nocapture`

**Matrix / routing update:** none yet; transitional implementation still uses current behavior.

**Definition of done:**
- [x] Build + test pass.
- [x] River/lake entity spawning is separated from river/lake path computation.
- [x] The transitional output is deterministic for a fixed seed.
- [x] No new file beyond the single touch path is needed.

---

## G1-S03 D8 flow + priority-flood algorithm

**Goal:** Replace the transitional greedy river path with deterministic D8 flow direction, priority-flood depression filling, upstream accumulation, and lake-basin extraction.

**Anchor reads:** orchestrator §§1-2 · [`../../../../src/terrain/generation/cell_matrix.rs`](../../../../src/terrain/generation/cell_matrix.rs) · [`../../../../src/terrain/generation/world_generator_enhanced.rs`](../../../../src/terrain/generation/world_generator_enhanced.rs) hydrology helper from G1-S02.

**Touch:**
- New [`../../../../src/terrain/generation/hydrology/mod.rs`](../../../../src/terrain/generation/hydrology/mod.rs):
  - `pub mod flow;`
  - Re-export `HydrologyResult`, `LakeRegion`, `FlowParams`, and `compute_hydrology`.
- New [`../../../../src/terrain/generation/hydrology/flow.rs`](../../../../src/terrain/generation/hydrology/flow.rs):
  - D8 neighbor selection with deterministic tie-break order.
  - Priority-flood depression filling (Barnes-style min-heap) over row-major height grids.
  - Upstream accumulation from flow directions.
  - River extraction from accumulation threshold.
  - Lake-basin extraction from filled depressions / low basins.
- [`../../../../src/terrain/generation/mod.rs`](../../../../src/terrain/generation/mod.rs):
  - Add `pub mod hydrology;`.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 flow_accumulation_deterministic -- --nocapture`
- `cargo test -p proc_A_dine01 priority_flood_no_pits -- --nocapture`
- `cargo test -p proc_A_dine01 river_threshold_count -- --nocapture`

**Matrix / routing update:** none yet; algorithm exists but p4 tags are not connected until G1-S04.

**Definition of done:**
- [x] Build + tests pass.
- [x] Same height grid + params produces byte-identical `HydrologyResult`.
- [x] Greedy descent in `world_generator_enhanced.rs` is removed or routed through the new helper.
- [x] Algorithm handles flat basins deterministically instead of random walking.

---

## G1-S04 Hydrology tags into p4

**Goal:** Use the canonical hydrology result in pass 4 to set terrain tags such as `flooded`, `eroded`, and `silted`.

**Anchor reads:** orchestrator §§1-2 · terrain matrix §3 pass 4 row · [`../../../../src/terrain/material/tags.rs`](../../../../src/terrain/material/tags.rs) · [`../../../../src/terrain/generation/hydrology/flow.rs`](../../../../src/terrain/generation/hydrology/flow.rs).

**Touch:**
- [`../../../../src/terrain/generation/passes/p4_hydrology.rs`](../../../../src/terrain/generation/passes/p4_hydrology.rs):
  - Call `hydrology::compute_hydrology` over the chunk-local elevation grid.
  - Set `flooded` on lake/water-basin cells.
  - Set `eroded` on river/high-flow cells above slope threshold.
  - Set `silted` on low-gradient wet cells above moisture threshold.
- [`../../../../assets/config/terrain/tag_registry.example.json`](../../../../assets/config/terrain/tag_registry.example.json):
  - Ensure `flooded`, `eroded`, and `silted` names exist with stable examples.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 p4_hydrology_sets_flooded_tag -- --nocapture`

**Matrix / routing update:** terrain matrix §3 pass 4 remains **Partial** until G1-S06; note tag wiring as partial progress in PR summary.

**Definition of done:**
- [x] Build + test pass.
- [x] Hydrology tags are written through `TagSet`; no stringly-typed runtime tags.
- [x] Example tag registry contains the three hydrology tags.
- [x] No p4 behavior depends on legacy ECS entity hierarchy.

---

## G1-S05 ECS visuals consume p4 output

**Goal:** Drive legacy ECS `RiverMarker` / `LakeMarker` visuals from the canonical hydrology result and tag semantics instead of a separate greedy/circular implementation.

**Anchor reads:** orchestrator §§1-2 · [`../../../../src/terrain/generation/world_generator_enhanced.rs`](../../../../src/terrain/generation/world_generator_enhanced.rs) hydrology helpers · [`../../../../src/terrain/generation/passes/p4_hydrology.rs`](../../../../src/terrain/generation/passes/p4_hydrology.rs).

**Touch:**
- [`../../../../src/terrain/generation/world_generator_enhanced.rs`](../../../../src/terrain/generation/world_generator_enhanced.rs):
  - Switch river/lake spawning to consume the canonical hydrology output from `terrain::generation::hydrology`.
  - Map lake cells to `LakeMarker` groups and high-flow river paths to `RiverMarker` groups.
  - Keep `apply_shallow_water_visual` as visual-only; terrain classification decisions come from hydrology output/tag semantics.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 enhanced_generator_consumes_p4_hydrology_tags -- --nocapture`

**Matrix / routing update:** none yet; close in G1-S06 after smoke/test status is recorded.

**Definition of done:**
- [x] Build + test pass.
- [x] No random-walk or circle-only lake generation remains in `world_generator_enhanced.rs`.
- [x] ECS visuals are a consumer of hydrology data, not a second hydrology model.
- [x] `WorldGeneratorSubenginePlugin` remains legacy / out-of-scope per tooling rulebook.

---

## G1-S06 phase close

**Goal:** Mark G1 hydrology remediation complete across routing docs once the code path and tests are green.

**Anchor reads:** orchestrator §4 · gap hunt §2E and §4.5 · terrain matrix §3.

**Touch:**
- [`../../terrain_biome/material_unification_matrix_v1.md`](../../terrain_biome/material_unification_matrix_v1.md):
  - §3 pass 4 `Hydrology / erosion` row **Partial** → **Applied**.
  - Mention canonical D8 / priority-flood hydrology and tags.
- [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md):
  - §4 row **G1** → **Applied**.
- [`../../../guides/implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md):
  - §2E baseline hydrology bullets → closed/route-to-G1 note.

**Verify:**
- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 hydrology_ -- --nocapture`
- `cargo test -p proc_A_dine01 p4_hydrology_ -- --nocapture`

**Matrix / routing update:** G1 **Applied**; terrain matrix p4 hydrology row **Applied**.

**Definition of done:**
- [x] Build + tests pass.
- [x] G1 row Applied in the orchestrator.
- [x] Terrain matrix p4 hydrology row Applied.
- [x] Gap-hunt baseline no longer lists G1 hydrology as an open example.
