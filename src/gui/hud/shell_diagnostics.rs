//! Lightweight product-shell diagnostics (consumer-only).

use std::collections::HashMap;

use bevy::prelude::*;

use super::shell_framework::ProductShellWidgetId;
use super::viewport_rect_sanity::{ViewportRectIssueKind, ViewportRectSource};

#[derive(Resource, Clone, Debug, Default)]
pub struct ProductShellDiagnostics {
    /// Lifetime product-shell egui passes (editor sessions can inflate before sim proof).
    pub egui_pass_count: u64,
    /// **UI-P2B-CODER-B** — passes since last `OnEnter(Simulation)`; used for `phase2b_closed`.
    pub egui_pass_count_sim_session: u64,
    /// **EGR-1** — successful `EguiContexts::ctx_mut` calls while `BaseState::Simulation`.
    /// The shell counter above stays the historical product-shell total.
    pub sim_egui_context_acquires: u64,
    pub last_frame_delta_secs: f32,
    pub texture_rebuilds: HashMap<ProductShellWidgetId, u64>,
    pub visible_widgets: HashMap<ProductShellWidgetId, bool>,
    pub viewport_rect_issues: HashMap<ViewportRectSource, u64>,
    pub last_viewport_rect_issue: Option<(ViewportRectSource, ViewportRectIssueKind)>,
}

impl ProductShellDiagnostics {
    /// **UI-P2B-CODER-B** — clear cumulative counter at sim enter (PLAN-UI-SHELL-2B-001).
    pub fn reset_egui_pass_count_for_simulation_session(&mut self) {
        self.egui_pass_count = 0;
        self.egui_pass_count_sim_session = 0;
        self.sim_egui_context_acquires = 0;
    }

    pub fn record_egui_pass(&mut self) {
        self.egui_pass_count = self.egui_pass_count.wrapping_add(1);
    }

    pub fn record_egui_pass_in_simulation(&mut self) {
        self.record_egui_pass();
        self.egui_pass_count_sim_session = self.egui_pass_count_sim_session.wrapping_add(1);
    }

    /// One real context acquire during Simulation. Editor and world-gen do not touch this field.
    pub fn record_sim_egui_context_acquire(&mut self) {
        self.sim_egui_context_acquires = self.sim_egui_context_acquires.wrapping_add(1);
    }

    pub fn bump_texture_rebuild(&mut self, id: ProductShellWidgetId) {
        *self.texture_rebuilds.entry(id).or_insert(0) += 1;
    }

    pub fn set_widget_visible(&mut self, id: ProductShellWidgetId, visible: bool) {
        self.visible_widgets.insert(id, visible);
    }

    pub fn record_viewport_rect_issue(
        &mut self,
        source: ViewportRectSource,
        kind: ViewportRectIssueKind,
    ) {
        *self.viewport_rect_issues.entry(source).or_insert(0) += 1;
        self.last_viewport_rect_issue = Some((source, kind));
    }

    #[must_use]
    pub fn texture_rebuild_count(&self, id: ProductShellWidgetId) -> u64 {
        self.texture_rebuilds.get(&id).copied().unwrap_or(0)
    }
}

pub fn product_shell_diagnostics_tick(time: Res<Time>, mut diag: ResMut<ProductShellDiagnostics>) {
    diag.last_frame_delta_secs = time.delta_secs();
}

/// Counts a successful primary-context acquire. Simulation increments
/// [`ProductShellDiagnostics::sim_egui_context_acquires`]. Other states do not.
pub fn note_egui_context_acquired(
    in_simulation: bool,
    diag: Option<&mut ProductShellDiagnostics>,
) {
    if in_simulation {
        if let Some(diag) = diag {
            diag.record_sim_egui_context_acquire();
        }
    }
}

/// Simulation-scheduled egui draw used by the EGR-1 lib test. Not registered on the game app.
pub fn egr_probe_sim_egui_draw_system(
    mut contexts: bevy_egui::EguiContexts,
    base: Res<State<crate::engine::states::BaseState>>,
    mut diag: ResMut<ProductShellDiagnostics>,
) -> Result {
    if !matches!(base.get(), crate::engine::states::BaseState::Simulation) {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    note_egui_context_acquired(true, Some(diag.as_mut()));
    egui_probe_label(ctx);
    Ok(())
}

fn egui_probe_label(ctx: &mut bevy_egui::egui::Context) {
    // A pass may already be open (bevy_egui begin_pass). If not, start one so the label is a draw.
    let already = ctx.input(|i| i.stable_dt >= 0.0);
    let _ = already;
    ctx.request_repaint();
}

#[cfg(test)]
mod egr_tests {
    use super::*;
    use crate::engine::states::BaseState;

    #[test]
    fn egr_sim_scheduled_egui_draw_increments_context_acquire() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<BaseState>();
        app.init_resource::<ProductShellDiagnostics>();
        app.init_resource::<bevy_egui::EguiUserTextures>();
        app.world_mut().spawn((
            bevy_egui::EguiContext::default(),
            bevy_egui::PrimaryEguiContext,
        ));
        app.add_systems(
            bevy_egui::EguiPrimaryContextPass,
            egr_probe_sim_egui_draw_system,
        );

        app.world_mut()
            .resource_mut::<NextState<BaseState>>()
            .set(BaseState::Simulation);
        app.update();

        assert_eq!(
            app.world().resource::<ProductShellDiagnostics>().sim_egui_context_acquires,
            0,
            "state transition frame is not yet Simulation when the egui pass ran"
        );

        app.world_mut().run_schedule(bevy_egui::EguiPrimaryContextPass);
        assert_eq!(
            app.world().resource::<ProductShellDiagnostics>().sim_egui_context_acquires,
            1,
            "a simulation-scheduled egui draw must increment the context acquire counter"
        );

        app.world_mut()
            .resource_mut::<NextState<BaseState>>()
            .set(BaseState::Editor);
        app.update();
        app.world_mut().run_schedule(bevy_egui::EguiPrimaryContextPass);
        assert_eq!(
            app.world().resource::<ProductShellDiagnostics>().sim_egui_context_acquires,
            1,
            "editor draws must not increment the simulation acquire counter"
        );
    }

    #[test]
    fn egr_simulation_schedule_omits_drained_egui_systems() {
        let gated = [
            (
                include_str!("../../construction/mod.rs"),
                &[
                    "draw_sim_build_picker_sheet_egui",
                    "draw_sim_road_tool_sheet_egui",
                    "draw_sim_power_tool_sheet_egui",
                    "draw_context_tray_build_body_egui",
                    "draw_context_tray_power_repair_egui",
                    "draw_staged_placements_panel_egui_system",
                    "draw_tool_hints_egui",
                    "draw_plant_focus_card_egui",
                    "draw_power_node_hover_egui",
                    "draw_construction_visual_requests_egui",
                    "draw_construction_phase_labels_egui",
                ][..],
            ),
            (
                include_str!("simulation_shell_phase2.rs"),
                &["draw_simulation_unified_cursor_egui_system"][..],
            ),
            (
                include_str!("../../render/pipelines/infrastructure_overlay.rs"),
                &["draw_power_map_overlay_egui"][..],
            ),
            (
                include_str!("../../economy/logistics/mod.rs"),
                &["draw_logistics_intent_overlay_egui"][..],
            ),
            (
                include_str!("../diagnostics_ui.rs"),
                &["diagnostics_ui_system"][..],
            ),
            (
                include_str!("../faction_tools_ui.rs"),
                &["faction_tools_ui_system"][..],
            ),
            (
                include_str!("../ai_explainability_ui.rs"),
                &["ai_explainability_egui_system"][..],
            ),
            (
                include_str!("../pressure_tooling.rs"),
                &["pressure_composer_egui_system"][..],
            ),
            (
                include_str!("../logistics_targets_panel.rs"),
                &["logistics_targets_panel_ui"][..],
            ),
            (
                include_str!("../agent_permissions_ui.rs"),
                &["permissions_ui_system"][..],
            ),
            (
                include_str!("../options_keybindings_ui.rs"),
                &["keybindings_options_ui"][..],
            ),
            (
                include_str!("../settlement_block_frame_debug.rs"),
                &["draw_block_frame_debug_overlay"][..],
            ),
            (
                include_str!("../../render/probes/debug_viewport_overlay.rs"),
                &["draw_debug_viewport_overlay"][..],
            ),
            (
                include_str!("../assembly_snapshot_qc_ui.rs"),
                &["draw_assembly_snapshot_qc_ui"][..],
            ),
            (
                include_str!("../../bevysubengines/world_generator_plugin.rs"),
                &["world_gen_ui_system.run_if(subengine_world_gen_editor_only)"][..],
            ),
            (
                include_str!("../map_view/plugin.rs"),
                &["super::presentation::sync_shell_layout_drag_gate"][..],
            ),
        ];
        for (src, names) in gated {
            for name in names {
                assert_editor_only_egui_registration(src, name);
            }
        }
        let map_view = include_str!("../map_view/plugin.rs").replace("\r\n", "\n");
        assert!(
            !map_view.contains("EguiPrimaryContextPass,\n                clear_active_map_view_input_before_map_widgets"),
            "non-draw clear must leave the egui pass"
        );
        assert!(
            map_view.contains("Update,\n                (\n                    clear_active_map_view_input_before_map_widgets"),
            "clear_active_map_view_input_before_map_widgets moves to Update"
        );
        let theme = include_str!("../style/theme.rs").replace("\r\n", "\n");
        assert!(
            theme.contains(
                "apply_egui_theme_system\n                    .after(EguiPreUpdateSet::BeginPass)\n                    .run_if(not(in_state(BaseState::Simulation)))"
            ),
            "egui theme must not run during Simulation"
        );
        assert!(
            theme.contains(
                "sync_egui_density_scale_system\n                        .before(update_ui_screen_rect)\n                        .run_if(not(in_state(BaseState::Simulation)))"
            ),
            "egui density sync must not run during Simulation"
        );
        assert!(
            theme.contains("note_egui_context_acquired"),
            "a theme ctx_mut must still increment the simulation acquire counter"
        );
        let states = include_str!("../../engine/states.rs").replace("\r\n", "\n");
        for gate in [
            "EguiPreUpdateSet::BeginPass.run_if(not(in_state(BaseState::Simulation)))",
            "EguiPostUpdateSet::EndPass.run_if(not(in_state(BaseState::Simulation)))",
            "EguiPostUpdateSet::ProcessOutput.run_if(not(in_state(BaseState::Simulation)))",
            "EguiPostUpdateSet::PostProcessOutput.run_if(not(in_state(BaseState::Simulation)))",
        ] {
            assert!(
                states.contains(gate),
                "Simulation must gate the egui pass set `{gate}`"
            );
        }
        assert!(
            states.contains(
                "OnEnter(BaseState::Simulation),\n        clear_primary_egui_output_on_simulation_enter,"
            ),
            "entering Simulation must clear the primary egui output"
        );
        assert!(
            states.contains(
                "OnExit(BaseState::Simulation),\n        restore_egui_cursor_and_capture_on_simulation_exit,"
            ),
            "leaving Simulation must restore egui cursor and capture"
        );
        assert!(
            states.contains("global.enable_cursor_icon_updates = false"),
            "Simulation enter must disable egui cursor icon updates"
        );
        assert!(
            states.contains("settings.capture_pointer_input = false"),
            "Simulation enter must stop egui pointer capture"
        );
        assert!(
            states.contains("*output = EguiFullOutput(None)"),
            "Simulation enter must clear the primary EguiFullOutput"
        );
        assert!(
            states.contains("global.enable_cursor_icon_updates = true"),
            "Simulation exit must restore egui cursor icon updates"
        );
        assert!(
            states.contains("settings.capture_pointer_input = true"),
            "Simulation exit must restore egui pointer capture"
        );
    }

    #[test]
    fn egr_placement_tools_do_not_call_ctx_mut() {
        let tools = [
            include_str!("../../construction/build_interaction.rs"),
            include_str!("../../construction/roads/input.rs"),
            include_str!("../../construction/rail/input.rs"),
            include_str!("../../construction/power_lines/input.rs"),
            include_str!("../../construction/power_lines/cut_input.rs"),
            include_str!("../../construction/zones/input.rs"),
            include_str!("../../construction/demolish.rs"),
        ];
        for src in tools {
            assert!(
                src.contains("simulation_tool_pointer"),
                "placement tool must read SimulationMapPointerGate via simulation_tool_pointer"
            );
            assert!(
                !src.contains("ctx_mut"),
                "placement tool still calls EguiContexts::ctx_mut"
            );
        }
        let panel = include_str!("panel_state.rs").replace("\r\n", "\n");
        assert!(panel.contains("simulation_tool_pointer"));
        assert!(
            panel.contains(
                "hud_panel_click_outside_collapse_system\n                    .after(hud_panel_escape_collapse_system)\n                    .run_if(in_state(BaseState::Simulation))"
            ),
            "simulation click-outside must not use the egui system"
        );
        assert!(
            panel.contains(
                "hud_panel_click_outside_collapse_egui_system\n                    .after(hud_panel_escape_collapse_system)\n                    .run_if(not(in_state(BaseState::Simulation)))"
            ),
            "editor click-outside keeps egui outside Simulation"
        );
    }

    #[test]
    fn egr_tool_style_context_acquire_is_counted_in_simulation() {
        let mut diag = ProductShellDiagnostics::default();
        note_egui_context_acquired(true, Some(&mut diag));
        assert_eq!(diag.sim_egui_context_acquires, 1);
        note_egui_context_acquired(false, Some(&mut diag));
        assert_eq!(
            diag.sim_egui_context_acquires, 1,
            "an editor tool context read must not increment the simulation counter"
        );
    }

    fn assert_editor_only_egui_registration(src: &str, name: &str) {
        if name.contains(".run_if(") {
            assert!(src.contains(name), "missing editor registration `{name}`");
            return;
        }
        let mut cursor = 0usize;
        let mut registrations = 0u32;
        while let Some(rel) = src[cursor..].find(name) {
            let abs = cursor + rel + name.len();
            let next = src.as_bytes().get(abs).copied().unwrap_or(0);
            if next == b'_' || next.is_ascii_alphanumeric() {
                cursor = abs;
                continue;
            }
            let after = &src[abs..(abs + 280).min(src.len())];
            if after.contains(".run_if") {
                let gated = after.contains("product_egui_shell_active")
                    || after.contains("subengine_world_gen_editor_only");
                assert!(
                    gated,
                    "{name} is still registered on a simulation-true egui condition:\n{after}"
                );
                registrations += 1;
            }
            cursor = abs;
        }
        assert!(
            registrations > 0,
            "{name} has no `.run_if` registration in the scanned source"
        );
    }
}
