# M5 — Snapshot save / load `v1`

> **STATUS:** **Applied** (engineering, 2026-05). Matrix §3 and orchestrator §4 match. **Pair:** orchestrator [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) · matrix [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md) §3 row **M5**. **Pre-req:** **M4** **Applied**.
>
> **Note:** v1 save/load is **palette buttons** in the map editor (single `MapEditorMapSnapshotIoRequest` message). **Main menu “Open in editor”** from a snapshot file is still optional / future — do not block M5 closure on it.

**Halt rules:** orchestrator §6. **Names not ids** for materials/biomes in DTOs. Align with [`../serialization/serialization_hybrid_migration_matrix_v1.md`](../serialization/serialization_hybrid_migration_matrix_v1.md) — if row missing, **`ASK:`** and keep **Partial**.

---

### M5-S01 map snapshot DTO

**Goal:** Add **`serde`** snapshot struct (width, height, cells: height + **terrain family name** (string registry key) + optional road flags) in a **single** module file; **no** stable `u16` material ids in v1 save.

**Anchor reads:**

1. [`../serialization/serialization_hybrid_migration_matrix_v1.md`](../serialization/serialization_hybrid_migration_matrix_v1.md) — schema policy.
2. [`../../../../src/terrain/biome.rs`](../../../../src/terrain/biome.rs).

**Touch:**

- New: `src/terrain/editor/map_snapshot.rs` *(max 1 file; register `mod` in `src/terrain/mod.rs` if required — second Touch)*

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 map_snapshot -- --nocapture` *(or add unit test for roundtrip empty grid)*

**Matrix update:** **M5** → **Partial**.

**Definition of done:**

- [x] Unit test: **serialize → deserialize** (`map_snapshot_v1_round_trips_ron`).

---

### M5-S02 save from editor

**Goal:** **Save map** button writes JSON or RON to path (dialog **ASK:** — v1 may use fixed **`saves/maps/last.ron`**).

**Anchor reads:**

1. [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs).
2. Snapshot module from **M5-S01**.

**Touch:**

- [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs)
- Snapshot module file

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 map_snapshot -- --nocapture`

**Matrix update:** none.

**Definition of done:**

- [x] File appears on disk after save from **Editor** (palette → `assets/saves/maps/last.ron`).

---

### M5-S03 load hydrate + open in editor

**Goal:** Deserializing a snapshot **rebuilds tile ECS** in the editor (fixed dev path acceptable for v1).

**Anchor reads:**

1. [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs) *(hydrate in `map_editor_map_snapshot_io`)*.
2. [`../../../../src/terrain/generation/world_generator_enhanced.rs`](../../../../src/terrain/generation/world_generator_enhanced.rs) *(spawn patterns only if extending)*.

**Touch:**

- [`../../../../src/gui/editor/map_editor/mod.rs`](../../../../src/gui/editor/map_editor/mod.rs)

**Verify:**

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 map_snapshot -- --nocapture`

**Matrix update:** none.

**Definition of done:**

- [x] **Load** from palette hydrates grid + road markers without panic (`assets/saves/maps/last.ron`). **Main menu** entry remains **future** (optional M5.x).

---

### M5-S04 phase close — M5 Applied

**Goal:** **M5** **Applied**; cross-link **G4** / serialization matrix if a new row was added.

**Anchor reads:** matrix §3 · orchestrator §4 · serialization matrix.

**Touch:**

- [`../map_editor_matrix_v1.md`](../map_editor_matrix_v1.md)
- [`../../../guides/map_editor_runbook_v1.md`](../../../guides/map_editor_runbook_v1.md) §4.

**Verify:** `cargo check -p proc_A_dine01`

**Matrix update:** **M5** → **Applied**.

**Definition of done:**

- [x] **M5** **Applied**.
