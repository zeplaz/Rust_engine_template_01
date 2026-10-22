# Interaction bug ledger (v1)

Owner: fleet `interaction` (see `fleet_charter_20261008_v1.md`). Scope: mouse picking, hover/select,
UI-vs-world input capture, menus / Escape / focus, keybindings, build placement ghosts, draw tools
(road / rail / power line / zone), strategic icon anchoring and click targets.

Severity: **H** = wrong world action or lost/duplicated command (misplaced build, click leaks through UI,
double commit, stuck state); **M** = wrong feedback / preview vs commit disagree in a reachable layout;
**L** = cosmetic, rare edge, or defensive.

Status: `open` · `fixed <sha>` · `justified` (left open with reason) · `wontfix`.

Regression suite: `cargo test -p proc_A_dine01 --lib interaction_regression` (module
`src/gui/input_interaction_regression.rs`) plus the per-bug tests named below.

| ID | Sev | Area | Repro | Status | Test |
|----|-----|------|-------|--------|------|
| INT-001 | H | picking | Draw tools (road/rail/power/zone/demolish/power hover) pick with the *desired-camera* projection that assumes RTT texture aspect == fill-rect aspect; buildings pick with the *rendered ortho*. When the RTT is clamped (`RTT_MAX_EXTENT` 4096 px — e.g. 5120 px-wide ultrawide) or lags a resize, the ortho letterboxes and draw-tool picks land up to `Δ = cursor_dx·(1 − view_px/logical)/zoom` world units from the drawn cursor/ghost. | fixed (working tree — `ConstructionMapProjection` + `cursor_world_on_map` → `TacticalMapPickFrame`) | `input_pick::camera_frame_matches_rendered_ortho_when_texture_letterboxes` |
| INT-002 | H | placement | Building / demolish cursor → tile uses `floor().max(0.0) as u32`: a cursor west/south of the world (camera may pan past the edge) clamps to column/row 0, so the ghost sticks to the map edge and a click places/demolishes there; no upper-bound reject in the pick. | fixed (working tree — `world_xy_to_tile_in_world` on build/demolish/zone/draw cursor paths) | `input_pick::tile_rejects_off_world` |
| INT-005 | H | placement | Two-click place FSM: first LMB on an armed building ghost should lock it (`BuildPlacementMode::Adjust`) without committing; second LMB commits. Headless harness observes `placement_mode` still `Place` after the first LMB. Not yet triaged whether the product FSM or the harness input injection is at fault. | fixed 19fe5290 (harness cause: pick holds the ghost until `building_intent` is set; `arm_factory` now sets it) | `int_005_first_lmb_locks_without_committing`, `int_005_locked_origin_survives_cursor_motion` |
| INT-006 | H | menus | Escape pause menu (`InGameMenuState::Pause`) did not set `SimulationMapPointerGate.chrome_blocks`, so world tools could still place under the overlay. | fixed (working tree — gate sync holds pointer while Pause) | — |
| INT-007 | H | cursor | Dual cursor painters (egui `draw_simulation_unified_cursor` + Bevy `SimPlayCrosshair`) + map wheel / logistics picks bypassing the gate. | fixed (working tree — sole authority `SimulationMapPointerGate.cursor_kind`; Bevy crosshair variants; egui painter retired no-op; wheel + logistics gated) | `triage_cursor_unify_001_*` |
| R-H4 | H | chrome | Editor dock / dev / F1 / F3 / logistics Bevy panels blocked map picks from panel *presence* alone (whole map blocked). | fixed 19fe5290 (cursor-in-rect via `ComputedNode`+`UiGlobalTransform`, `apply_bevy_panel_pointer_blocks_system`) | `panel_open_cursor_over_rect_blocks_placement`, `panel_open_cursor_outside_rect_reaches_placement`, `editor_dock_present_cursor_off_dock_reaches_placement` |
| R-H5 | H | ordering | Map consumers were not ordered after the panel blockers, so a click could pass through a panel depending on schedule order. | fixed 19fe5290 (`PointerGateSet::{Sync,Block,Publish}` chained; construction input + hud consumers `.after(Publish)`) | — |
