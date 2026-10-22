//! Read-only registry tables for Wave P preview tooling (consumer-only).



#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub enum PreviewRegistryInspectorHost {
    #[default]
    EguiSidebar,
    DesktopAssetTool,
}

impl PreviewRegistryInspectorHost {
    #[must_use]
    #[allow(dead_code)] // egui-retired; kept for Bevy port / export
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
