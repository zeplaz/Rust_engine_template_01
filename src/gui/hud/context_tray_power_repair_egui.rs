//! **COD-POWER-REPAIR-QUEUE-001** — context tray Logistics → Power repairs section.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::EguiContexts;

use crate::construction::{
    PowerLineDamageBook, PowerRepairJob, PowerRepairQueue, POWER_REPAIR_PARTS_PER_SEGMENT,
};
use crate::engine::states::BaseState;
use crate::gui::hud::panel_state::HudPanelState;
use crate::gui::hud::simulation_shell_phase2::{
    ContextTrayState, ContextTrayTab, CONTEXT_TRAY_BODY_H_PX, CONTEXT_TRAY_PEEK_BODY_H_PX,
    CONTEXT_TRAY_TAB_H_PX,
};
use crate::gui::in_game_hud::CONTEXT_TRAY_LEFT_INSET_PX;
use crate::gui::UiPalette;

use super::sim_hud_egui_theme::{
    apply_sim_hud_egui_theme, body_text, caption_text, data_text, picker_sheet_frame, title_text,
};

#[derive(SystemParam)]
pub struct ContextTrayPowerRepairDrawParams<'w> {
    pub tray: Res<'w, ContextTrayState>,
    pub palette: Res<'w, UiPalette>,
    pub queue: ResMut<'w, PowerRepairQueue>,
    pub book: Res<'w, PowerLineDamageBook>,
}

#[must_use]
pub fn power_repair_panel_wired() -> bool {
    true
}

#[must_use]
pub fn power_repair_panel_tier_tray_logistics() -> bool {
    true
}

pub fn draw_context_tray_power_repair_egui(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    mut params: ContextTrayPowerRepairDrawParams,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation) {
        return Ok(());
    }
    if params.tray.active_tab != ContextTrayTab::Logistics {
        return Ok(());
    }
    if !params.tray.panel_state.shows_content() {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    apply_sim_hud_egui_theme(ctx, &params.palette);

    let body_h = if params.tray.panel_state == HudPanelState::Peek {
        CONTEXT_TRAY_PEEK_BODY_H_PX
    } else {
        CONTEXT_TRAY_BODY_H_PX
    };
    let top = CONTEXT_TRAY_TAB_H_PX;
    let rect = egui::Rect::from_min_size(
        egui::pos2(CONTEXT_TRAY_LEFT_INSET_PX, top),
        egui::vec2(320.0, body_h - 8.0),
    );

    let mut queue_all = false;
    egui::Area::new(egui::Id::new("context_tray_power_repair"))
        .fixed_pos(rect.min)
        .show(ctx, |ui| {
            ui.set_min_size(rect.size());
            picker_sheet_frame(&params.palette).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(title_text(
                        &params.palette,
                        &format!("Power repairs ({})", params.queue.jobs.len()),
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(body_text(&params.palette, "Queue all damaged")).clicked() {
                            queue_all = true;
                        }
                    });
                });
                ui.separator();
                if params.queue.jobs.is_empty() {
                    ui.label(caption_text(
                        &params.palette,
                        "○ No power repairs queued",
                    ));
                } else {
                    egui::ScrollArea::vertical().max_height(body_h - 48.0).show(
                        ui,
                        |ui| {
                            let mut cancel_id = None::<u64>;
                            let mut priority_delta: Option<(u64, i16)> = None;
                            for job in &params.queue.jobs {
                                draw_repair_row(
                                    ui,
                                    &params.palette,
                                    job,
                                    &mut cancel_id,
                                    &mut priority_delta,
                                );
                            }
                            if let Some(id) = cancel_id {
                                params.queue.cancel(id);
                            }
                            if let Some((id, delta)) = priority_delta {
                                if let Some(job) =
                                    params.queue.jobs.iter_mut().find(|j| j.id == id)
                                {
                                    let p = job.priority as i16 + delta;
                                    job.priority = p.clamp(1, 100) as u8;
                                }
                            }
                        },
                    );
                }
            });
        });

    if queue_all {
        params
            .queue
            .queue_all_damaged(&params.book, POWER_REPAIR_PARTS_PER_SEGMENT);
    }

    Ok(())
}

fn draw_repair_row(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    job: &PowerRepairJob,
    cancel_id: &mut Option<u64>,
    priority_delta: &mut Option<(u64, i16)>,
) {
    ui.horizontal(|ui| {
        ui.label(body_text(palette, "≡"));
        ui.vertical(|ui| {
            ui.label(body_text(palette, &job.label));
            let parts = if job.parts_ready() {
                format!("parts {}/{}", job.parts_have, job.parts_need)
            } else {
                format!(
                    "parts {}/{} · blocked: {}",
                    job.parts_have,
                    job.parts_need,
                    job.blocked_reason.as_deref().unwrap_or("need parts")
                )
            };
            ui.label(caption_text(palette, &parts));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("×").clicked() {
                *cancel_id = Some(job.id);
            }
            if ui.button("+").clicked() {
                *priority_delta = Some((job.id, 5));
            }
            if ui.button("-").clicked() {
                *priority_delta = Some((job.id, -5));
            }
            ui.label(data_text(palette, &format!("P [{}]", job.priority)));
        });
    });
    ui.separator();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_repair_panel_witness_tokens() {
        assert!(power_repair_panel_wired());
        assert!(power_repair_panel_tier_tray_logistics());
    }
}
