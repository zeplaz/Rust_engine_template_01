//! Shared `run_if` helpers so **player shell** (Bevy) and **dev tooling** stay separated.

use crate::engine::states::BaseState;
use crate::engine::AppState;
use bevy::prelude::*;

#[must_use]
pub fn in_simulation_or_editor(base: Res<State<BaseState>>) -> bool {
    matches!(base.get(), BaseState::Simulation | BaseState::Editor)
}

/// GPU Bevy minimap chrome sync in PostUpdate — skip duplicate Egui-pass sync in Simulation.
pub fn minimap_chrome_egui_sync_active(base: Res<State<BaseState>>) -> bool {
    !matches!(base.get(), BaseState::Simulation)
}

/// PLAY-01 Phase 2B: floating product shell (dock windows, side rail, editor chrome).
///
/// Simulation keeps Bevy chrome only. Editor product dock/toolbox run under this gate.
///
/// **World-gen chrome is exclusive:** while `AppState::WorldGen`, product shell stays off.
/// Session controls live on the Bevy `WorldGenShellRoot`.
#[must_use]
pub fn product_shell_active(
    base: Res<State<BaseState>>,
    app: Res<State<AppState>>,
) -> bool {
    product_shell_states_active(*base.get(), *app.get())
}

/// Const form for shell_framework / tests.
#[must_use]
pub const fn product_shell_base_active(base: BaseState) -> bool {
    matches!(base, BaseState::Editor)
}

#[must_use]
pub fn product_shell_states_active(base: BaseState, app: AppState) -> bool {
    matches!(base, BaseState::Editor) && app != AppState::WorldGen
}

/// Map editor TEMP-EGUI chrome (palette, scenario tools, editor minimap).
///
/// Must **not** run during world-gen / preview / gameplay — `bridge_ux_to_legacy` maps
/// `AppState::WorldGen` → `BaseState::Editor`, which would otherwise stack editor tools on
/// World Generator + Preview chrome.
#[must_use]
pub fn map_editor_chrome_active(
    base: Res<State<BaseState>>,
    app: Res<State<AppState>>,
) -> bool {
    map_editor_chrome_states_active(*base.get(), *app.get())
}

#[must_use]
pub const fn map_editor_chrome_states_active(base: BaseState, app: AppState) -> bool {
    matches!(base, BaseState::Editor)
        && !matches!(
            app,
            AppState::WorldGen | AppState::InGame | AppState::Paused
        )
}

/// 2B-03 — left status rail (duplicate of Bevy context rail in sim).
#[must_use]
pub fn side_status_rail_egui_active(
    base: Res<State<BaseState>>,
    app: Res<State<AppState>>,
) -> bool {
    product_shell_active(base, app)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::states::BaseState;
    use crate::engine::AppState;

    #[test]
    fn product_shell_editor_only() {
        assert!(!product_shell_base_active(BaseState::Simulation));
        assert!(product_shell_base_active(BaseState::Editor));
    }

    #[test]
    fn product_shell_off_during_world_gen() {
        assert!(!product_shell_states_active(
            BaseState::Editor,
            AppState::WorldGen
        ));
        assert!(product_shell_states_active(
            BaseState::Editor,
            AppState::InGame
        ));
    }

    #[test]
    fn map_editor_chrome_off_during_world_gen_and_play() {
        assert!(!map_editor_chrome_states_active(
            BaseState::Editor,
            AppState::WorldGen
        ));
        assert!(!map_editor_chrome_states_active(
            BaseState::Editor,
            AppState::InGame
        ));
        assert!(map_editor_chrome_states_active(
            BaseState::Editor,
            AppState::Setup
        ));
    }
}
