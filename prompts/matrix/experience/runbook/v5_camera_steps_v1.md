# VA5 — Zoom visual bias + ortho scaffold `v1`

> **VISUAL-AID-V2-05**. Pre-req: VA4.

### V5-S01 ZoomVisualBias

**Goal:** Visual-only scale curve resource; no sim `Transform` mutation.

**Touch:** `src/gui/tile_readability.rs` or `src/gui/zoom_visual_bias.rs`, `src/gui/map_camera.rs`

**Verify:** `cargo test -p proc_A_dine01 zoom_visual_bias -- --nocapture`

### V5-S02 hybrid ortho scaffold

**Goal:** `ScaffoldContract` on ortho tilt tuning in `map_camera.rs`.

**Touch:** `src/gui/map_camera.rs`

**Board:** VISUAL-AID-V2-05 → Done or InProgress
