//! **COD-SIM-HUD-TRAY-BUILD-001** — context tray Build tab body (egui overlay).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::EguiContexts;

use crate::construction::{
    draw_r4_corridor_tray_legend, draw_staged_placements_panel_body,
    parametric_active_ghost_hud_line, ActiveBuildTool, BuildCommandActor, BuildGhostState,
    BuildPlacementPreview, BuildStripState, BuildTool, BuildingDefinitionRegistry,
    ConstructionHistory, PendingConstructionQueue, SiteStubOverlayState, StagedPlacementBook,
    StagedPlacementMode, ToolContext,
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
    pub pending: Res<'w, PendingConstructionQueue>,
    pub site_stub: Res<'w, SiteStubOverlayState>,
    pub corridor_book: Option<Res<'w, CorridorConstructionBook>>,
    pub occupation: Option<Res<'w, TileOccupationBook>>,
    pub history: ResMut<'w, ConstructionHistory>,
}

#[must_use]
pub fn context_tray_build_tab_wired() -> bool {
    ContextTrayTab::Build.label() == "Build"
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
    true
}

#[must_use]
pub fn peek_shows_modifiers_wired() -> bool {
    !TRAY_PEEK_MODIFIERS.is_empty()
}

pub fn draw_context_tray_build_body_egui(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    mut params: ContextTrayBuildDrawParams,
    mut events: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation) {
        return Ok(());
    }
    if params.tray.active_tab != ContextTrayTab::Build {
        return Ok(());
    }
    if params.tray.panel_state == HudPanelState::Collapsed {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    apply_sim_hud_egui_theme(ctx, &params.palette);

    let body_h = match params.tray.panel_state {
        HudPanelState::Peek => CONTEXT_TRAY_PEEK_BODY_H_PX,
        HudPanelState::Expanded | HudPanelState::Pinned => CONTEXT_TRAY_BODY_H_PX.max(156.0),
        HudPanelState::Collapsed => return Ok(()),
    };

    let is_build =
        matches!(params.tool.tool, BuildTool::Building(_)) && params.strip.active != ToolContext::None;
    let screen_h = ctx.input(|i| i.screen_rect().height());
    let anchor_y = screen_h - CONTEXT_TRAY_TAB_H_PX - body_h;

    if params.tray.panel_state == HudPanelState::Peek {
        if !is_build {
            return Ok(());
        }
        egui::Area::new(egui::Id::new("context_tray_build_peek"))
            .fixed_pos(egui::pos2(CONTEXT_TRAY_LEFT_INSET_PX, anchor_y))
            .show(ctx, |ui| {
                ui.set_width(400.0);
                ui.label(caption_text(&params.palette, TRAY_PEEK_MODIFIERS));
            });
        return Ok(());
    }

    let site_legend = params.site_stub.preset_id.is_some() || !params.site_stub.zone_labels.is_empty();

    egui::Area::new(egui::Id::new("context_tray_build_body"))
        .fixed_pos(egui::pos2(CONTEXT_TRAY_LEFT_INSET_PX, anchor_y))
        .show(ctx, |ui| {
            ui.set_min_width(280.0);
            picker_sheet_frame(&params.palette).show(ui, |ui| {
                if site_legend {
                    ui.label(title_text(&params.palette, TRAY_LEGEND_TITLE));
                    ui.label(caption_text(&params.palette, TRAY_LEGEND_FOOTPRINT));
                    ui.label(caption_text(&params.palette, TRAY_LEGEND_YARD));
                    ui.label(caption_text(
                        &params.palette,
                        "Yard · Rail · Svc · Park · Load",
                    ));
                    ui.separator();
                }
                if let Some(book) = params.corridor_book.as_ref() {
                    draw_r4_corridor_tray_legend(ui, &params.tool, book);
                }
                ui.label(title_text(&params.palette, TRAY_STAGING_TITLE));
                ui.checkbox(&mut params.staging.enabled, "Stage placements");
                if let Some(line) = parametric_active_ghost_hud_line(
                    &params.tool,
                    &params.registry,
                    &params.ghost,
                    &params.preview,
                ) {
                    ui.label(caption_text(&params.palette, &line));
                }
                if params.staging.enabled || !params.book.rows.is_empty() {
                    draw_staged_placements_panel_body(
                        ui,
                        &mut params.book,
                        params.actor.0,
                        &mut events,
                        &mut params.history,
                        params.occupation.as_deref(),
                    );
                } else {
                    ui.label(caption_text(&params.palette, TRAY_STAGING_EMPTY));
                }
                ui.separator();
                ui.label(title_text(&params.palette, TRAY_QUEUE_TITLE));
                let n = params.pending.pending_count();
                let first = params
                    .pending
                    .entries
                    .first()
                    .map(|e| e.label.as_str())
                    .unwrap_or("—");
                ui.label(data_text(
                    &params.palette,
                    &tray_queue_summary(n, first),
                ));
            });
        });
    Ok(())
}
