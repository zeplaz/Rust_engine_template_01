# U6 — Optional `bevy_ecs_tilemap` adapter `v1`

> **Pair:** orchestrator [`../../../guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) · matrix [`../material_unification_matrix_v1.md`](../material_unification_matrix_v1.md) §10 row **U6**, §15 row `Terrain` (multi-layer stack). **Pre-req:** U5 must be **Applied**. **U6 is optional** — phase may be skipped or left at status **Partial** indefinitely.

**Phase goal:** Provide a thin, **feature-gated** adapter that writes `MaterializedChunk.materials` into a `bevy_ecs_tilemap` `TileStorage`. Default build excludes the dependency.

**Anchor set:** orchestrator §§1–2 · matrix §10 + §15 · this pack · the single edited file.

**Halt rules:** orchestrator §6. Additionally — if `bevy_ecs_tilemap` has no Bevy 0.18-compatible release at execution time, **halt** and surface to human (do not pin pre-release without explicit `ASK:`).

---

## U6-S01 add feature flag + optional dependency

**Goal:** Add a `bevy_tilemap_adapter` feature that pulls in `bevy_ecs_tilemap`, off by default.

**Anchor reads:** orchestrator §§1–2 · [`Cargo.toml`](../../../../Cargo.toml).

**Touch:**
- [`Cargo.toml`](../../../../Cargo.toml):
  - Under `[features]`: add `bevy_tilemap_adapter = ["dep:bevy_ecs_tilemap"]`.
  - Under `[dependencies]`: add `bevy_ecs_tilemap = { version = "0.x", optional = true }` — **`ASK:`** for the exact version that pairs with Bevy 0.18 at execution time. If no compatible version exists, **halt**.

**Verify:**
- `cargo check -p proc_A_dine01` (feature off — should be unchanged).
- `cargo check -p proc_A_dine01 --features bevy_tilemap_adapter` (feature on — must also pass).

**Matrix update:** none (preparatory).

**Definition of done:**
- [ ] Default build clean.
- [ ] Feature-on build clean.
- [ ] No use of `bevy_ecs_tilemap` types outside `#[cfg(feature = "bevy_tilemap_adapter")]` yet.

---

## U6-S02 thin adapter writing `TileTextureIndex`

**Goal:** Implement a system (gated by the feature) that mirrors `MaterializedChunk` into a `bevy_ecs_tilemap::TileStorage`.

**Anchor reads:** orchestrator §§1–2 · matrix §15 row `Terrain` · `src/terrain/material/runtime.rs`.

**Touch:**
- New `src/render/tilemap_adapter.rs` (entire file gated by `#[cfg(feature = "bevy_tilemap_adapter")]`):
  - `pub struct TilemapAdapterPlugin;`
  - System `sync_terrain_layer` queries `Changed<MaterializedChunk>` and writes `TileTextureIndex(material_id.0 as u32)` into the matching tile entity.
  - **One layer only** (terrain). Overlay + resource layers are U7.
- [`src/render/mod.rs`](../../../../src/render/mod.rs) — `#[cfg(feature = "bevy_tilemap_adapter")] pub mod tilemap_adapter;`.

**Verify:**
- `cargo check -p proc_A_dine01 --features bevy_tilemap_adapter`
- `cargo test -p proc_A_dine01 --features bevy_tilemap_adapter tilemap_adapter_writes_terrain_layer -- --nocapture` (build a minimal `App`, spawn one chunk, assert tile entities receive expected `TileTextureIndex`).

**Matrix update:** §1 row `Tilemap renderer` → **Partial** (feature-gated); §15 row `Terrain` → **Partial**.

**Definition of done:**
- [ ] Default build still clean (no symbol leak).
- [ ] Feature-on test passes.
- [ ] No second sync path; the adapter is the only writer to terrain `TileTextureIndex`.

---

## U6-S03 phase close — matrix flip (Partial)

**Goal:** Mark U6 status as **Partial** (single layer, feature-gated). Promotion to **Applied** requires U7-S04 (multi-layer).

**Anchor reads:** orchestrator §4 · matrix §10.

**Touch:**
- [`prompts/matrix/terrain_biome/material_unification_matrix_v1.md`](../material_unification_matrix_v1.md) — §10 row **U6** → **Partial** (note: feature-gated terrain layer only).
- [`prompts/guides/terrain_unification_runbook_v1.md`](../../../guides/terrain_unification_runbook_v1.md) §4 phase index **U6** → **Partial**.

**Verify:**
- `cargo check -p proc_A_dine01` (default).
- `cargo check -p proc_A_dine01 --features bevy_tilemap_adapter`.

**Definition of done:**
- [ ] Both builds clean.
- [ ] Matrix + orchestrator updated to **Partial** (not **Applied** — U7 closes that).

---

## Open carries (📎)

- Overlay layer (z=10) and resource layer (z=20): U7-S04.
- Diff-based `TileTextureIndex` updates (only changed cells): U7-S03.
- GPU compute preview path: out of scope here, separate proposal.
