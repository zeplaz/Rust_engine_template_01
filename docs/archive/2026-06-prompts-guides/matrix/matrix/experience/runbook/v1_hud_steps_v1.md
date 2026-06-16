# VA1 — HUD panel state machine `v1`

> **Pair:** [`../../../guides/visual_aidv2_runbook_v1.md`](../../../guides/visual_aidv2_runbook_v1.md) · row **VISUAL-AID-V2-01**.

### V1-S01 panel_state module

**Goal:** Add `HudPanelState` enum and width helpers in `panel_state.rs`.

**Touch:** `src/gui/hud/panel_state.rs`, `src/gui/hud/mod.rs`

**Verify:** `cargo test -p proc_A_dine01 hud_panel_state -- --nocapture`

**Board:** VISUAL-AID-V2-01 → Partial

### V1-S02 migrate dock_shell + transmission

**Goal:** Replace `tray_expanded` / `expanded` / layout bools with state machine.

**Touch:** `src/gui/hud/dock_shell.rs`, `src/gui/hud/transmission.rs`

**Verify:** `cargo check -p proc_A_dine01`

### V1-S03 ESC + pin + click-outside

**Goal:** ESC and click-outside collapse unpinned panels; pin toggles Pinned.

**Touch:** `src/gui/hud/panel_state.rs`, `src/gui/input_bindings.rs` (if needed)

**Verify:** `cargo test -p proc_A_dine01 hud_panel_state -- --nocapture`

**Board:** VISUAL-AID-V2-01 → Done when predicate green
