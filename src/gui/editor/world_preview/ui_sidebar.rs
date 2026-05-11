//! Left column: mobility profile + terrain tag pool (generation / highlight).

use bevy_egui::egui;

use crate::gui::editor::world_gen_hints as hints;
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::material::TagId;
use bevy::prelude::*;

#[inline]
fn tt(response: egui::Response, text: &'static str) -> egui::Response {
    response.on_hover_text(text)
}

pub fn world_preview_sidebar(
    ui: &mut egui::Ui,
    world_gen_ui_state: &mut WorldGenUiState,
    world_gen_params: &mut WorldGenParams,
    handles: &TerrainRegistriesHandles,
    tag_assets: &Assets<crate::terrain::material::TagRegistry>,
    mobility_assets: &Assets<crate::terrain::mobility::MobilityProfileRegistry>,
) {
    if let Some(mob) = mobility_assets.get(&handles.mobility_profiles) {
        if !mob.profiles.is_empty() {
            ui.label(egui::RichText::new("Mobility profile").strong());
            let n = mob.profiles.len();
            let idx = world_gen_ui_state.mobility_profile_index.min(n - 1);
            world_gen_ui_state.mobility_profile_index = idx;
            let mut sel = idx;
            egui::ComboBox::from_id_salt("world_preview_mobility_profile")
                .selected_text(mob.profiles[idx].id.as_str())
                .show_ui(ui, |ui| {
                    for (i, p) in mob.profiles.iter().enumerate() {
                        ui.selectable_value(&mut sel, i, p.id.as_str());
                    }
                });
            if sel != idx {
                world_gen_ui_state.mobility_profile_index = sel;
            }
            ui.add_space(8.0);
        }
    }

    if let Some(tag_reg) = tag_assets.get(&handles.tag_registry) {
        ui.label(egui::RichText::new("Terrain tag pool").strong());
        ui.small(
            "Unchecked names are not written onto chunks; Tags overlay only highlights cells carrying checked tags.",
        );
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .id_salt("world_preview_tag_pool_scroll")
            .show(ui, |ui| {
                for (i, t) in tag_reg.tags.iter().enumerate() {
                    let id = TagId(i as u16);
                    let mut on = world_gen_params.tag_pool.contains(id);
                    let r = ui.checkbox(&mut on, &t.name);
                    let r = tt(r, hints::TAG_POOL_ENTRY);
                    if r.changed() {
                        if on {
                            world_gen_params.tag_pool.insert(id);
                        } else {
                            world_gen_params.tag_pool.remove(id);
                        }
                    }
                }
            });
    }
}
