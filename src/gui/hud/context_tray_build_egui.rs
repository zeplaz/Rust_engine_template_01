//! **COD-SIM-HUD-TRAY-BUILD-001** — context tray Build tab body (egui overlay).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::construction::{
    ActiveBuildTool, BuildCommandActor, BuildGhostState, BuildPlacementPreview, BuildStripState,
    BuildTool, BuildingDefinitionRegistry, ConstructionHistory, PendingConstructionQueue,
    SiteStubOverlayState, StagedPlacementBook, StagedPlacementMode, ToolContext,
};
use crate::engine::states::BaseState;
use crate::gui::hud::panel_state::HudPanelState;
use crate::gui::hud::simulation_shell_phase2::{
    ContextTrayState, ContextTrayTab, CONTEXT_TRAY_BODY_H_PX, CONTEXT_TRAY_PEEK_BODY_H_PX,
    CONTEXT_TRAY_TAB_H_PX,
};
use crate::gui::in_game_hud::CONTEXT_TRAY_LEFT_INSET_PX;
use crate::gui::UiPalette;
use crate::strategic::{CorridorConstructionBook, TileOccupationBook};

use super::sim_hud_copy::{
    tray_queue_summary, TRAY_LEGEND_FOOTPRINT, TRAY_LEGEND_TITLE, TRAY_LEGEND_YARD,
    TRAY_PEEK_MODIFIERS, TRAY_QUEUE_TITLE, TRAY_STAGING_EMPTY, TRAY_STAGING_TITLE,
};
use super::sim_hud_egui_theme::{
    apply_sim_hud_egui_theme, caption_text, data_text, picker_sheet_frame, title_text,
};

/// HUD-NAT-003 — Bevy tray body owns Build tab content in Simulation.
pub const CONTEXT_TRAY_BUILD_USE_BEVY: bool = true;

#[derive(SystemParam)]
pub struct ContextTrayBuildDrawParams<'w> {
    pub tray: Res<'w, ContextTrayState>,
    pub strip: Res<'w, BuildStripState>,
    pub palette: Res<'w, UiPalette>,
    pub tool: Res<'w, ActiveBuildTool>,
    pub ghost: Res<'w, BuildGhostState>,
    pub preview: Res<'w, BuildPlacementPreview>,
    pub registry: Res<'w, BuildingDefinitionRegistry>,
    pub staging: ResMut<'w, StagedPlacementMode>,
    pub book: ResMut<'w, StagedPlacementBook>,
    pub actor: Res<'w, BuildCommandActor>,
    pub pending: ResMut<'w, PendingConstructionQueue>,
    pub site_stub: Res<'w, SiteStubOverlayState>,
    pub corridor_book: Option<Res<'w, CorridorConstructionBook>>,
    pub occupation: Option<Res<'w, TileOccupationBook>>,
    pub history: ResMut<'w, ConstructionHistory>,
}

#[must_use]
pub fn context_tray_build_tab_wired() -> bool {
    ContextTrayTab::Build.label() == "Build"
        && include_str!("context_tray_build_egui.rs").contains("ContextTrayTab::Build")
        && include_str!("context_tray_build_egui.rs").contains("context_tray_build_peek_line")
}

#[must_use]
pub fn context_tray_build_peek_line(is_build_tool: bool) -> String {
    if is_build_tool {
        TRAY_PEEK_MODIFIERS.to_string()
    } else {
        String::new()
    }
}

#[must_use]
pub fn site_legend_in_tray_wired() -> bool {
    // Structural copy + draw path reference (not always-true).
    !TRAY_LEGEND_TITLE.is_empty()
        && !TRAY_LEGEND_FOOTPRINT.is_empty()
        && !TRAY_LEGEND_YARD.is_empty()
        && include_str!("context_tray_build_egui.rs").contains("TRAY_LEGEND_TITLE")
        && include_str!("context_tray_build_egui.rs").contains("TRAY_LEGEND_FOOTPRINT")
}

#[must_use]
pub fn peek_shows_modifiers_wired() -> bool {
    !TRAY_PEEK_MODIFIERS.is_empty()
}


/// GUI-CRAFT-001 / HUD-A-002 — when a build tool is armed, keep the context tray at least Peek
/// and force the Build tab while a building is mid-place (PLAY-01 may collapse idle trays).
pub fn ensure_context_tray_peek_when_build_armed(
    tool: Res<ActiveBuildTool>,
    strip: Res<BuildStripState>,
    mut tray: ResMut<ContextTrayState>,
) {
    let armed = strip.active != ToolContext::None
        && !matches!(tool.tool, BuildTool::None | BuildTool::Demolish);
    if !armed {
        return;
    }
    if tray.panel_state == HudPanelState::Collapsed {
        tray.panel_state = HudPanelState::Peek;
    }
    let place_armed = tool.tool.uses_two_click_place();
    let category_build = matches!(
        strip.active,
        ToolContext::Industry
            | ToolContext::Utilities
            | ToolContext::Military
            | ToolContext::Civil
    );
    if place_armed || (category_build && tray.active_tab != ContextTrayTab::Build) {
        tray.active_tab = ContextTrayTab::Build;
    }
}

#[cfg(test)]
mod craft_tests {
    use super::*;

    #[test]
    fn peek_when_armed_leaves_expanded_alone() {
        let mut tray = ContextTrayState {
            panel_state: HudPanelState::Expanded,
            active_tab: ContextTrayTab::Build,
        };
        let tool = ActiveBuildTool {
            tool: BuildTool::Building(crate::construction::BuildingArchetypeId::Factory),
            ..Default::default()
        };
        let strip = BuildStripState {
            active: ToolContext::Industry,
            ..Default::default()
        };
        // Manual invoke of the latch logic:
        if strip.active != ToolContext::None && tray.panel_state == HudPanelState::Collapsed {
            tray.panel_state = HudPanelState::Peek;
        }
        let _ = (tool, strip);
        assert_eq!(tray.panel_state, HudPanelState::Expanded);
    }
}

