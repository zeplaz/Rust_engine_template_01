//! Stage 6 consumer DTOs — residency / streaming display only.

use bevy::prelude::*;
use bevy_egui::egui;
use serde::{Deserialize, Serialize};

use crate::gui::style::{muted_text, UiPalette};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StreamedChunkDiagnosticDto {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub residency_ring: u8,
    pub ghost_band: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResidencyOverlayConsumerDto {
    pub schema_version: u32,
    pub resident_chunks: u32,
    pub ghost_chunks: u32,
    pub utility_channel_mask: u32,
    pub paged_atlas_pages: u32,
    pub chunks: Vec<StreamedChunkDiagnosticDto>,
}

impl ResidencyOverlayConsumerDto {
    pub const CURRENT_SCHEMA: u32 = 1;
}

/// Test / menu scaffold only — sim HUD uses [`crate::gui::hud::residency_overlay_consumer_from_frame`].
#[must_use]
pub fn mock_residency_overlay_consumer() -> ResidencyOverlayConsumerDto {
    ResidencyOverlayConsumerDto {
        schema_version: ResidencyOverlayConsumerDto::CURRENT_SCHEMA,
        resident_chunks: 12,
        ghost_chunks: 4,
        utility_channel_mask: 0b1011,
        paged_atlas_pages: 2,
        chunks: vec![StreamedChunkDiagnosticDto {
            chunk_x: 0,
            chunk_y: 0,
            residency_ring: 0,
            ghost_band: false,
        }],
    }
}

pub fn draw_stage6_residency_consumer_panel(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    dto: &ResidencyOverlayConsumerDto,
) {
    muted_text(
        ui,
        palette,
        format!(
            "resident={} ghost={} utility_mask={:#x} atlas_pages={} (**BQ-134**)",
            dto.resident_chunks,
            dto.ghost_chunks,
            dto.utility_channel_mask,
            dto.paged_atlas_pages,
        ),
    );
}
