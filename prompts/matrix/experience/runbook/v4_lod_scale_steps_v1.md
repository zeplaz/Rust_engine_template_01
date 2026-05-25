# VA4 — Band-driven building visual `v1`

> **VISUAL-AID-V2-04**. Pre-req: VA3.

### V4-S01 representation flags

**Goal:** `building_visual_simplified` on `RepresentationResult` from `WorldLodBand`.

**Touch:** `src/gui/representation_policy.rs`

**Verify:** `cargo test -p proc_A_dine01 representation_band_building -- --nocapture`

### V4-S02 registry upload policy

**Goal:** Band gates mesh simplification without duplicate LOD enum.

**Touch:** `src/gui/representation_policy.rs` (consumer wiring)

**Board:** VISUAL-AID-V2-04 → Done or InProgress with witness
