//! Pending blueprint queue panel — emits [`ConstructionQueueIntent`] only.

use bevy::prelude::*;
use bevy_egui::egui;

use crate::gui::hud::{
    capture_shell_layout, draw_virtualized_rows, shell_widget_runs_egui_with_budget,
    floating_unanchored_default_pos, show_product_shell_window, HudDockRegistry, HudLayoutStore,
    HudWidgetId,
    PendingHudLayoutCommit, ProductShellUpdateBudget, ShellWidgetDiagnostics, ShellWindowHost,
    WorldInteractionDiagnostics,
};

use super::blueprint_preset::blueprint_collection_from_pending;
use super::construction_queue_intent::{ConstructionQueueIntent, ConstructionQueuePanelView};
use super::pending_construction::PendingConstructionQueue;

pub fn draw_pending_construction_queue_egui(
    ctx: &mut egui::Context,
    view: &ConstructionQueuePanelView,
    dock: &mut HudDockRegistry,
    layout_store: &mut HudLayoutStore,
    intents: &mut MessageWriter<ConstructionQueueIntent>,
    pending: &PendingConstructionQueue,
    preset_ron: &mut Option<String>,
    update_budget: &mut ProductShellUpdateBudget,
    now_secs: f32,
    widget_timing: Option<&mut ShellWidgetDiagnostics>,
    world_interaction: Option<&WorldInteractionDiagnostics>,
    pending_layout: &mut PendingHudLayoutCommit,
    wave_s_imported: Option<&crate::io::save::WaveSImportedBlueprints>,
) {
    let mut open = dock.slot(HudWidgetId::ConstructionQueue).visible;
    if view.total_count == 0 && !open {
        return;
    }
    if !shell_widget_runs_egui_with_budget(
        dock,
        HudWidgetId::ConstructionQueue,
        open,
        Some(update_budget),
        now_secs,
    ) {
        return;
    }
    if let Some(response) = show_product_shell_window(
        ctx,
        ShellWindowHost {
            id: HudWidgetId::ConstructionQueue,
            title: "Pending blueprints",
            default_pos: floating_unanchored_default_pos(
                ctx,
                HudWidgetId::ConstructionQueue,
                [300.0, 240.0],
            ),
            default_size: [300.0, 240.0],
            min_size: [260.0, 180.0],
        },
        layout_store,
        dock,
        &mut open,
        |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "Ghost validity: {}  commit {}  (terrain {:.0}% · logistics {:.0}%)",
                    if view.ghost_valid { "ok" } else { "blocked" },
                    if view.commit_allowed { "allowed" } else { "blocked" },
                    view.terrain_score * 100.0,
                    view.logistics_score * 100.0,
                ))
                .small()
                .weak(),
            );
            if !view.errors.is_empty() {
                ui.label(
                    egui::RichText::new(format!("Errors: {}", view.errors.join(", ")))
                        .color(egui::Color32::from_rgb(220, 96, 96)),
                );
            }
            if !view.warnings.is_empty() {
                ui.label(
                    egui::RichText::new(format!("Warnings: {}", view.warnings.join(", ")))
                        .weak(),
                );
            }
            if let Some(hint) = &view.path_snap_hint {
                ui.label(egui::RichText::new(hint).small().color(egui::Color32::from_rgb(120, 200, 160)));
            }
            if !view.path_required_actions.is_empty() {
                ui.label(
                    egui::RichText::new(format!(
                        "Required: {}",
                        view.path_required_actions.join(" · ")
                    ))
                    .color(egui::Color32::from_rgb(220, 180, 96)),
                );
            }
            ui.label(
                egui::RichText::new(format!(
                    "Projected throughput hint: {:.0}% logistics · corridor phase compat stub (**BQ-132**)",
                    view.logistics_score * 100.0
                ))
                .small()
                .weak(),
            );
            if let Some(world) = world_interaction {
                ui.label(format!(
                    "Throughput {:.0}% · queue latency {:.1} ms · map hover latency {:.1} ms",
                    world.construction_throughput_hint * 100.0,
                    world.construction_queue_latency_ms,
                    world.map_interaction_latency_ms
                ));
                if world.optimistic_hover_active {
                    ui.label(format!(
                        "Hover highlight {:.0}% · tooltip {}",
                        world.hover_highlight_strength * 100.0,
                        if world.tooltip_pending { "pending" } else { "ready" }
                    ));
                }
            }
            ui.add(
                egui::ProgressBar::new(view.pending_count as f32 / view.total_count.max(1) as f32)
                    .text(format!("Queue {} / {}", view.pending_count, view.total_count)),
            );
            ui.label(
                egui::RichText::new(
                    "Enter commits the active ghost when valid. Shift+Enter approves the queue, then commits approved rows and the ghost.",
                )
                .small()
                .weak(),
            );
            ui.horizontal(|ui| {
                if ui.button("Approve all").clicked() {
                    intents.write(ConstructionQueueIntent::ApproveAll);
                }
                if ui.button("Approve factories").clicked() {
                    intents.write(ConstructionQueueIntent::ApproveFactories);
                }
                if ui.button("Clear unapproved").clicked() {
                    intents.write(ConstructionQueueIntent::ClearUnapproved);
                }
                if ui.button("Clear all").clicked() {
                    intents.write(ConstructionQueueIntent::ClearAll);
                }
                if ui.button("Export presets (RON)").clicked() {
                    let collection = blueprint_collection_from_pending(pending);
                    *preset_ron = ron::ser::to_string(&collection).ok();
                }
                if let Some(imported) = wave_s_imported.and_then(|w| w.collection.as_ref()) {
                    if ui
                        .button(format!(
                            "Import Wave S presets ({})",
                            imported.presets.len()
                        ))
                        .clicked()
                    {
                        *preset_ron = ron::ser::to_string(imported).ok();
                    }
                }
            });
            if let Some(ron) = preset_ron.as_ref() {
                ui.label(egui::RichText::new(ron).monospace().small());
            }
            let mut remove_index = None;
            let entries = &view.entries;
            draw_virtualized_rows(
                ui,
                "pending_construction_queue",
                22.0,
                140.0,
                entries.len(),
                |ui, row| {
                    let entry = &entries[row];
                    let mut approved = entry.approved;
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut approved, "Approved").changed() {
                            intents.write(ConstructionQueueIntent::SetApproved {
                                index: row,
                                approved,
                            });
                        }
                        ui.label(format!(
                            "{} @ ({},{}) rot {} mirror {}",
                            entry.label,
                            entry.origin_x,
                            entry.origin_z,
                            entry.rotation_quarter_turns,
                            entry.mirror_x
                        ));
                        if ui.small_button("Cancel").clicked() {
                            remove_index = Some(row);
                        }
                    });
                },
            );
            if let Some(index) = remove_index {
                intents.write(ConstructionQueueIntent::Remove { index });
            }
            ui.label(
                egui::RichText::new(format!(
                    "Pending {} · total {}",
                    view.pending_count, view.total_count
                ))
                .small()
                .weak(),
            );
        },
        widget_timing,
    ) {
        capture_shell_layout(layout_store, HudWidgetId::ConstructionQueue, &response, Some(pending_layout));
    }
    dock.slot_mut(HudWidgetId::ConstructionQueue).visible = open;
}
