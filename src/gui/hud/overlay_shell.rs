//! Overlay shell — toggle groups + legend widgets (mock consumers only).

use bevy::prelude::*;


use super::overlay_framework::OverlayFrameworkState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OverlayToggleGroup {
    Threat,
    Logistics,
    Recon,
    Utility,
}

#[derive(Resource, Clone, Debug)]
pub struct OverlayShellState {
    pub groups: [bool; 4],
    pub legend_open: bool,
}

impl Default for OverlayShellState {
    fn default() -> Self {
        Self {
            groups: [false, false, false, false],
            legend_open: false,
        }
    }
}

impl OverlayShellState {
    pub fn group_mut(&mut self, group: OverlayToggleGroup) -> &mut bool {
        match group {
            OverlayToggleGroup::Threat => &mut self.groups[0],
            OverlayToggleGroup::Logistics => &mut self.groups[1],
            OverlayToggleGroup::Recon => &mut self.groups[2],
            OverlayToggleGroup::Utility => &mut self.groups[3],
        }
    }
}

#[must_use]
pub fn mock_overlay_channel_descriptors() -> Vec<crate::strategic::OverlayChannelDescriptor> {
    super::overlay_framework::default_overlay_channel_runtimes()
        .into_iter()
        .map(|row| row.descriptor)
        .collect()
}

pub struct OverlayShellPlugin;

impl Plugin for OverlayShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OverlayFrameworkState>()
            .init_resource::<OverlayShellState>()
            .add_plugins(super::info_tabs::HudInfoTabPlugin);
    }
}

