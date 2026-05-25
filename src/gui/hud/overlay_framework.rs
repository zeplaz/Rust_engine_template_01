//! Generic overlay descriptors — opacity/blend controls (mock consumer only).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::strategic::{OverlayChannelDescriptor, StrategicOverlayType, UtilityChannel};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OverlayChannelRuntime {
    pub descriptor: OverlayChannelDescriptor,
    pub enabled: bool,
    pub opacity: f32,
    pub blend_weight: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct OverlayFrameworkState {
    pub channels: Vec<OverlayChannelRuntime>,
    pub legend_open: bool,
}

impl Default for OverlayFrameworkState {
    fn default() -> Self {
        Self {
            channels: default_overlay_channel_runtimes(),
            legend_open: false,
        }
    }
}

#[must_use]
pub fn default_overlay_channel_runtimes() -> Vec<OverlayChannelRuntime> {
    vec![
        OverlayChannelRuntime {
            descriptor: OverlayChannelDescriptor {
                utility: UtilityChannel::Threat,
                overlay: StrategicOverlayType::Threat,
                color_rgb: [220, 96, 96],
            },
            enabled: true,
            opacity: 0.85,
            blend_weight: 1.0,
        },
        OverlayChannelRuntime {
            descriptor: OverlayChannelDescriptor {
                utility: UtilityChannel::Logistics,
                overlay: StrategicOverlayType::LogisticsStress,
                color_rgb: [96, 180, 220],
            },
            enabled: true,
            opacity: 0.75,
            blend_weight: 0.9,
        },
        OverlayChannelRuntime {
            descriptor: OverlayChannelDescriptor {
                utility: UtilityChannel::Visibility,
                overlay: StrategicOverlayType::Recon,
                color_rgb: [120, 220, 120],
            },
            enabled: true,
            opacity: 0.7,
            blend_weight: 0.8,
        },
        OverlayChannelRuntime {
            descriptor: OverlayChannelDescriptor {
                utility: UtilityChannel::Congestion,
                overlay: StrategicOverlayType::Congestion,
                color_rgb: [220, 180, 80],
            },
            enabled: false,
            opacity: 0.6,
            blend_weight: 0.7,
        },
        OverlayChannelRuntime {
            descriptor: OverlayChannelDescriptor {
                utility: UtilityChannel::Instability,
                overlay: StrategicOverlayType::EwCoverage,
                color_rgb: [180, 120, 220],
            },
            enabled: false,
            opacity: 0.55,
            blend_weight: 0.65,
        },
    ]
}

pub fn draw_overlay_legend(ui: &mut bevy_egui::egui::Ui, channels: &[OverlayChannelRuntime]) {
    for row in channels {
        if !row.enabled {
            continue;
        }
        let color = bevy_egui::egui::Color32::from_rgb(
            row.descriptor.color_rgb[0],
            row.descriptor.color_rgb[1],
            row.descriptor.color_rgb[2],
        );
        ui.horizontal(|ui| {
            ui.colored_label(color, "■");
            ui.label(format!(
                "{:?} / {:?} · opacity {:.0}% · blend {:.2}",
                row.descriptor.utility,
                row.descriptor.overlay,
                row.opacity * 100.0,
                row.blend_weight
            ));
        });
    }
}
