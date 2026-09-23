PS C:\dev\github\Rust_engine_template_01> cargo run -p proc_A_dine01 --release -- --test visual --stay-open
   Compiling proc_A_dine01 v0.1.1 (C:\dev\github\Rust_engine_template_01)
warning: unused import: `bevy::diagnostic::FrameCount`
 --> src\dev\mig_a_adoption.rs:7:5
  |
7 | use bevy::diagnostic::FrameCount;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `build_mig_program_close_witness_body`
  --> src\dev\mig_a_audit.rs:10:30
   |
10 |     build_mig_a_rollup_json, build_mig_program_close_witness_body, mig_a18_frame_perf_witness_enabled,
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Vec2`
  --> src\dev\tactical_map_debug.rs:17:24
   |
17 | use bevy::math::{Mat4, Vec2};
   |                        ^^^^

warning: unused variable: `ortho`
   --> src\gui\debug\ui_layout_tree_debug.rs:321:5
    |
321 |     ortho: Res<crate::gui::MainWorldCameraOrthoTrace>,
    |     ^^^^^ help: if this is intentional, prefix it with an underscore: `_ortho`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: variable does not need to be mutable
   --> src\gui\tactical\sim_map_rtt.rs:356:5
    |
356 |     mut tex: ResMut<SimulationMapTexture>,
    |     ----^^^
    |     |
    |     help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

warning: unreachable pattern
   --> src\dev\tactical_map_debug.rs:182:9
    |
182 |         _ => json!({ "kind": "other", "debug": format!("{target:?}") }),
    |         ^ no value can reach this
    |
note: multiple earlier patterns match some of the same values
   --> src\dev\tactical_map_debug.rs:182:9
    |
169 |         RenderTarget::Image(ImageRenderTarget { handle, scale_factor }) => json!({
    |         --------------------------------------------------------------- matches some of the same values
...
179 |         RenderTarget::Window(_) => json!({ "kind": "window" }),
    |         ----------------------- matches some of the same values
180 |         RenderTarget::TextureView(_) => json!({ "kind": "texture_view" }),
    |         ---------------------------- matches some of the same values
181 |         RenderTarget::None { .. } => json!({ "kind": "none", "void_suspect": "CAMERA_RTT_TARGET_NONE" }),
    |         ------------------------- matches some of the same values
182 |         _ => json!({ "kind": "other", "debug": format!("{target:?}") }),
    |         ^ collectively making this unreachable
    = note: `#[warn(unreachable_patterns)]` (part of `#[warn(unused)]`) on by default

warning: type `TacticalMapDebugState` is more private than the item `write_tactical_map_debug_witness`
   --> src\dev\tactical_map_debug.rs:437:1
    |
437 | / pub fn write_tactical_map_debug_witness(
438 | |     mut state: ResMut<TacticalMapDebugState>,
439 | |     q: TacticalMapDebugInputs,
440 | |     main_cam: Query<
...   |
451 | |     hud_cam: Query<(&Camera, &RenderTarget), With<crate::gui::SimulationHudUiCamera>>,
452 | | ) {
    | |_^ function `write_tactical_map_debug_witness` is reachable at visibility `pub`
    |
note: but type `TacticalMapDebugState` is only usable at visibility `pub(self)`
   --> src\dev\tactical_map_debug.rs:25:1
    |
 25 | struct TacticalMapDebugState {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = note: `#[warn(private_interfaces)]` on by default

warning: type `TacticalMapDebugInputs<'_, '_>` is more private than the item `write_tactical_map_debug_witness`
   --> src\dev\tactical_map_debug.rs:437:1
    |
437 | / pub fn write_tactical_map_debug_witness(
438 | |     mut state: ResMut<TacticalMapDebugState>,
439 | |     q: TacticalMapDebugInputs,
440 | |     main_cam: Query<
...   |
451 | |     hud_cam: Query<(&Camera, &RenderTarget), With<crate::gui::SimulationHudUiCamera>>,
452 | | ) {
    | |_^ function `write_tactical_map_debug_witness` is reachable at visibility `pub`
    |
note: but type `TacticalMapDebugInputs<'_, '_>` is only usable at visibility `pub(self)`
   --> src\dev\tactical_map_debug.rs:42:1
    |
 42 | struct TacticalMapDebugInputs<'w, 's> {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `RTT_LAYER` is never used
  --> src\dev\tactical_map_debug.rs:23:7
   |
23 | const RTT_LAYER: usize = crate::gui::SIMULATION_MAP_RTT_RENDER_LAYER;
   |       ^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `assemble_stage_tick_test_app` is never used
   --> src\construction\site_stage_tick.rs:104:4
    |
104 | fn assemble_stage_tick_test_app() -> App {
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `proc_A_dine01` (lib) generated 10 warnings (run `cargo fix --lib -p proc_A_dine01` to apply 5 suggestions)
    Finished `release` profile [optimized] target(s) in 2m 42s
     Running `target\release\proc_A_dine01.exe --test visual --stay-open`
2026-08-12T22:29:20.442056Z ERROR bevy_asset::server: Path not found: C:\dev\github\Rust_engine_template_01\assets\assets/staging/city_c8_pilot_merge_run001/model.glb
2026-08-12T22:29:20.442415Z ERROR bevy_asset::server: Path not found: C:\dev\github\Rust_engine_template_01\assets\assets/models/modules/prop_fence_lod0_run001/model.glb
2026-08-12T22:29:20.442796Z ERROR bevy_asset::server: Path not found: C:\dev\github\Rust_engine_template_01\assets\assets/models/modules/prop_light_lod0_run001/model.glb
2026-08-12T22:29:20.442912Z ERROR bevy_asset::server: Path not found: C:\dev\github\Rust_engine_template_01\assets\assets/models/modules/prop_vent_lod0_run001/model.glb


