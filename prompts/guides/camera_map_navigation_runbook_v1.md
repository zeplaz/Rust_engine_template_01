# Camera / map navigation runbook (v1)

**Purpose:** World-space (game view) pan, zoom, edge scroll, and optional 2D “rotate the map” — **without hardcoded keys**; extend [`InputBindings`](../../src/gui/input_bindings.rs) and the Options → Key bindings UI.

**Depends on:** [`gui_runbook_v1.md`](gui_runbook_v1.md), [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md).

## Goals

1. **Pan** — keyboard axes + middle-mouse / drag if applicable; optional **screen-edge scroll** when pointer is near viewport border (sensitivity in options or config).
2. **Zoom** — wheel or bound keys; align with existing editor preview patterns where sensible (see `world_preview.rs` / map editor) but **game view** is the target.
3. **Rotate** (2D strategic map) — single bound “rotate step” or hold-to-rotate; persist angle on a camera or map root component.
4. **Config** — new fields on `InputBindings` (serde default + RON path), exposed in `options_keybindings_ui`, with **no** `KeyCode::` scattered outside defaults/tests per `input_bindings.rs` policy.

## Execution steps

1. **Inventory** — locate Bevy cameras / ortho used for the main map; note conflicts with egui focus (don’t pan when egui wants the mouse).
2. **Bindings design** — minimal set: e.g. `map_pan_up/down/left/right`, `map_zoom_in/out`, `map_rotate_ccw/cw`, `map_pan_mouse_button` (optional enum) or document mouse in runbook only if not key-bindable.
3. **Options UI** — add rows to keybindings panel; migration: missing RON keys use `Default`.
4. **Systems** — `Res<InputBindings>` + `Input`/`MouseWheel`/cursor position; clamp zoom/rotation; frame delta–scaled movement when sim unpaused if needed.
5. **Verification** — load a small test scene (`--test weather` / `--test fire` once CLI exists); confirm pan/zoom/rotate survive save-load of `input_bindings.ron` and respect egui capture.

**Implementation (2026-05-09):** [`MainWorldCamera`](../../src/gui/map_camera.rs) + [`MapCameraPlugin`](../../src/gui/map_camera.rs); bindings on [`InputBindings`](../../src/gui/input_bindings.rs); wheel zoom via [`AccumulatedMouseScroll`](https://docs.rs/bevy/latest/bevy/input/mouse/struct.AccumulatedMouseScroll.html); motion pan via [`AccumulatedMouseMotion`](https://docs.rs/bevy/latest/bevy/input/mouse/struct.AccumulatedMouseMotion.html).

## Out of scope (v1)

- Rebasing save games on camera pose.
- Touch / gamepad (future extension).

## Related todos

- CLI test entrypoints and small gen worlds (`main` / `EnginePlugin` startup args).
- Terrain and ecology debug overlays may assume usable navigation.
