# VA6 — Strategic icon rendering `v1`

> **VISUAL-AID-V2-06**. Pre-req: VA4 + VA5 + stable FULL_APP.

### V6-S01 projection graph slice

**Goal:** Macro/Strategic icon instance buffer via `RenderProjectionGraph` + `RepresentationResult`.

**Touch:** `src/render/extraction/render_projection_graph.rs`, `src/gui/strategic_icon_instances.rs` (scaffold)

**Verify:** `cargo test -p proc_A_dine01 strategic_icon -- --nocapture`

**Board:** VISUAL-AID-V2-06 → Done when `macro_icon_instance_count > 0` witness (or deferred `ASK:`)
