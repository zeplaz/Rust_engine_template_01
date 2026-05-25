//! World-gen / preview chrome contract — regression gate for double-stack UI.
//!
//! **Incident:** Bevy simulation command shell spawned on `AppState::WorldGen` while egui
//! World Generator + Preview were open → duplicate labels, click steal, unstable load.
//!
//! Run: `cargo test -p proc_A_dine01 --lib world_gen_chrome`

#![allow(dead_code)] // Source-contract scanners; exercised by `cargo test --lib world_gen_chrome`.

/// Human-readable contract (also referenced by agent playbooks).
pub const WORLD_GEN_CHROME_CONTRACT: &str = r"
World-gen chrome exclusivity (must hold):
1. SimulationCommandShellRoot spawns ONLY on OnEnter(BaseState::Simulation).
2. SimulationCommandShellRoot MUST NOT spawn on OnEnter(AppState::WorldGen).
3. product_egui_shell_active is FALSE while AppState::WorldGen (even if BaseState::Editor).
4. World Generator egui window title must not duplicate an in-body section heading.
5. Map editor TEMP-EGUI (palette, scenario tools, editor minimap) must NOT run during AppState::WorldGen.
";

/// Source-level guard — catches re-adding the bad OnEnter hook.
#[must_use]
pub fn in_game_hud_source_forbids_world_gen_shell_spawn(source: &str) -> bool {
    let bad_on_enter = source.contains("OnEnter(crate::engine::AppState::WorldGen)")
        && source.contains("spawn_simulation_command_shell");
    let bad_on_exit = source.contains("OnExit(crate::engine::AppState::WorldGen)")
        && source.contains("despawn_simulation_command_shell");
  let lifecycle_fn = source.contains("fn register_sim_command_shell_lifecycle");
    let lifecycle_sim_only = !source.contains("register_sim_command_shell_lifecycle")
        || (source.contains("OnEnter(BaseState::Simulation), spawn_simulation_command_shell")
            && !source
                .split("fn register_sim_command_shell_lifecycle")
                .nth(1)
                .is_some_and(|tail| tail.contains("AppState::WorldGen")));
    !bad_on_enter && !bad_on_exit && lifecycle_sim_only && lifecycle_fn
}

#[must_use]
pub fn world_gen_ui_source_avoids_duplicate_window_title(source: &str) -> bool {
    // Window title + immediate section_heading with same label was the double-text bug.
    !source.contains(
        "Window::new(\"World Generator\")\n        .default_size",
    ) || !source.contains(
        "section_heading(ui, pal, CmdHeadingStyle::Gt, \"World Generator\")",
    )
}

#[must_use]
pub fn map_editor_source_uses_chrome_gate(source: &str) -> bool {
    source.contains("map_editor_chrome_active")
        && !source.contains("map_editor_palette_system.run_if(in_state(BaseState::Editor))")
        && !source.contains("scenario_editor_tools_entry_window.run_if(in_state(BaseState::Editor))")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::states::BaseState;
    use crate::engine::AppState;
    use crate::gui::ui_gates::{map_editor_chrome_states_active, product_egui_shell_states_active};

    #[test]
    fn contract_product_egui_shell_off_during_world_gen() {
        assert!(
            !product_egui_shell_states_active(BaseState::Editor, AppState::WorldGen),
            "REGRESSION: editor product egui must not run during WorldGen — stacks on preview chrome"
        );
        assert!(
            product_egui_shell_states_active(BaseState::Editor, AppState::InGame),
            "editor product egui should run in normal editor/in-game UX"
        );
    }

    #[test]
    fn regression_in_game_hud_does_not_spawn_shell_on_world_gen_enter() {
        const SOURCE: &str = include_str!("in_game_hud.rs");
        assert!(
            in_game_hud_source_forbids_world_gen_shell_spawn(SOURCE),
            "REGRESSION: in_game_hud.rs wires sim Bevy shell to AppState::WorldGen — \
             remove OnEnter/OnExit(WorldGen) for spawn_simulation_command_shell"
        );
    }

    #[test]
    fn regression_world_gen_ui_no_duplicate_title_heading() {
        const SOURCE: &str = include_str!("editor/world_gen_ui.rs");
        assert!(
            world_gen_ui_source_avoids_duplicate_window_title(SOURCE),
            "REGRESSION: world_gen_ui.rs duplicates 'World Generator' in window title and section_heading"
        );
    }

    #[test]
    fn regression_map_editor_gated_off_during_world_gen() {
        assert!(
            !map_editor_chrome_states_active(BaseState::Editor, AppState::WorldGen),
            "REGRESSION: map editor chrome must not run during WorldGen"
        );
        const SOURCE: &str = include_str!("editor/map_editor/mod.rs");
        assert!(
            map_editor_source_uses_chrome_gate(SOURCE),
            "REGRESSION: map_editor/mod.rs must gate TEMP-EGUI on map_editor_chrome_active"
        );
    }

    #[test]
    fn regression_register_sim_shell_lifecycle_is_simulation_only() {
        const SOURCE: &str = include_str!("in_game_hud.rs");
        assert!(
            SOURCE.contains("fn register_sim_command_shell_lifecycle"),
            "missing register_sim_command_shell_lifecycle — wire sim shell via single helper"
        );
        assert!(
            in_game_hud_source_forbids_world_gen_shell_spawn(SOURCE),
            "REGRESSION: sim shell lifecycle must not reference AppState::WorldGen"
        );
    }
}
