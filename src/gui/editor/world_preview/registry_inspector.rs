//! Read-only registry tables for Wave P preview tooling (consumer-only).

use bevy_egui::egui;

use super::registry_interchange::{
    material_registry_interchange_path, open_registry_interchange_in_desktop_shell,
    tag_registry_interchange_path,
};
use crate::gui::style::{
    muted_label, section_heading, CmdHeadingStyle, UiPalette, UiSpacing, widget_scroll_vertical_capped,
};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::material::{MaterialRegistry, TagRegistry};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PreviewRegistryInspectorHost {
    #[default]
    EguiSidebar,
    DesktopAssetTool,
}

impl PreviewRegistryInspectorHost {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::EguiSidebar => "egui sidebar",
            Self::DesktopAssetTool => "desktop asset tool",
        }
    }
}

pub fn world_preview_registry_inspector(
    ui: &mut egui::Ui,
    host: PreviewRegistryInspectorHost,
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
        &format!(
            "Read-only view of canonical material and tag registries (host: {}). Edits belong in asset files or the desktop asset tool.",
            host.label()
        ),
    );

    if host == PreviewRegistryInspectorHost::DesktopAssetTool {
        let material_path = material_registry_interchange_path();
        let tag_path = tag_registry_interchange_path();
        muted_label(
            ui,
            palette,
            &format!(
                "Interchange: {} | {}",
                material_path.display(),
                tag_path.display()
            ),
        );
        ui.horizontal(|ui| {
            if ui.button("Open material registry").clicked() {
                let _ = open_registry_interchange_in_desktop_shell(&material_path);
            }
            if ui.button("Open tag registry").clicked() {
                let _ = open_registry_interchange_in_desktop_shell(&tag_path);
            }
        });
    }

    if let Some(reg) = materials.get(&handles.material_registry) {
        section_heading(ui, palette, CmdHeadingStyle::None, "Materials");
        widget_scroll_vertical_capped("world_preview_material_registry_scroll", 180.0).show(ui, |ui| {
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
        widget_scroll_vertical_capped("world_preview_tag_registry_scroll", 160.0).show(ui, |ui| {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_inspector_host_is_egui_sidebar() {
        assert_eq!(
            PreviewRegistryInspectorHost::default(),
            PreviewRegistryInspectorHost::EguiSidebar
        );
    }
}
