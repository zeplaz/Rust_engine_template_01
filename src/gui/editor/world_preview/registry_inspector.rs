//! Read-only registry tables for Wave P preview tooling (consumer-only).


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

#[allow(dead_code)] // Registry list is not a WorldGen egui panel anymore.
pub fn world_preview_registry_inspector() {}

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
