//! Read-only registry tables for Wave P preview tooling (consumer-only).

use bevy_egui::egui;

use crate::gui::style::{muted_label, section_heading, CmdHeadingStyle, UiPalette, UiSpacing};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::material::{MaterialRegistry, TagRegistry};

pub fn world_preview_registry_inspector(
    ui: &mut egui::Ui,
    handles: &TerrainRegistriesHandles,
    materials: &bevy::prelude::Assets<MaterialRegistry>,
    tags: &bevy::prelude::Assets<TagRegistry>,
    palette: &UiPalette,
    _spacing: &UiSpacing,
) {
    section_heading(ui, palette, CmdHeadingStyle::None, "Registry inspector");
    muted_label(
        ui,
        palette,
        "Read-only view of canonical material and tag registries. Edits belong in asset files or the desktop asset tool.",
    );

    if let Some(reg) = materials.get(&handles.material_registry) {
        section_heading(ui, palette, CmdHeadingStyle::None, "Materials");
        egui::ScrollArea::vertical()
            .max_height(180.0)
            .id_salt("world_preview_material_registry_scroll")
            .show(ui, |ui| {
                egui::Grid::new("world_preview_material_registry_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("name");
                        ui.label("family");
                        ui.label("tags");
                        ui.end_row();
                        for material in &reg.materials {
                            ui.label(&material.name);
                            ui.label(format!("{}", material.family.0));
                            ui.label(material.tags.len().to_string());
                            ui.end_row();
                        }
                    });
            });
    }

    if let Some(reg) = tags.get(&handles.tag_registry) {
        section_heading(ui, palette, CmdHeadingStyle::None, "Tags");
        egui::ScrollArea::vertical()
            .max_height(160.0)
            .id_salt("world_preview_tag_registry_scroll")
            .show(ui, |ui| {
                egui::Grid::new("world_preview_tag_registry_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("name");
                        ui.label("category");
                        ui.end_row();
                        for tag in &reg.tags {
                            ui.label(&tag.name);
                            ui.label(&tag.category);
                            ui.end_row();
                        }
                    });
            });
    }
}
