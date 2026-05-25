# VA2 — Footprint GPU tiles `v1`

> **Pair:** orchestrator § VA2 · **VISUAL-AID-V2-02**.

### V2-S01 footprint requests

**Goal:** `ConstructionVisualRequests.footprint_tiles` filled from ghost + preview.

**Touch:** `src/construction/visual_authority.rs`, `src/construction/build_interaction.rs`

**Verify:** `cargo test -p proc_A_dine01 footprint_tile -- --nocapture`

### V2-S02 GPU bridge

**Goal:** Bridge pushes `TileDebugInstance` rows to `WorldMain` with scaffold contract.

**Touch:** `src/construction/footprint_tile_instances.rs`, `src/gui/tile_debug_types.rs`, `assets/shaders/debug/tile_debug_instanced.wgsl`

**Verify:** `cargo check -p proc_A_dine01`

### V2-S03 demote egui hint

**Goal:** `build_footprint_overlay.rs` text-only when GPU path active.

**Touch:** `src/construction/build_footprint_overlay.rs`

**Board:** VISUAL-AID-V2-02 → Done
