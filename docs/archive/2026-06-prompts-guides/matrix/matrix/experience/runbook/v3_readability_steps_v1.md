# VA3 — Tile readability clamp `v1`

> **VISUAL-AID-V2-03**.

### V3-S01 TileReadabilityConfig

**Goal:** Resource + `screen_pixels_per_tile` from ortho + viewport.

**Touch:** `src/gui/tile_readability.rs`, `src/gui/map_camera.rs`

**Verify:** `cargo test -p proc_A_dine01 tile_readability -- --nocapture`

### V3-S02 LodInputs bias

**Goal:** Floor zoom in `compute_world_representation_frame` when below min px/tile.

**Touch:** `src/gui/world_representation.rs`

**Board:** VISUAL-AID-V2-03 → Done
