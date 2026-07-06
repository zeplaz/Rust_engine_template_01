//! Bevy 0.19 Core2d overlay pass ordering (replaces `RenderGraph` sub-graph edges).

use bevy::core_pipeline::{Core2d, Core2dSystems};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::RenderApp;

/// SDR Core2d overlay pipelines are compiled for this format (must match RTT targets).
pub const CORE2D_OVERLAY_SDR_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

/// Index into `[sdr, hdr]` overlay pipeline tables (`0` = SDR, `1` = HDR float).
#[inline]
#[must_use]
pub fn core2d_overlay_pipeline_hdr_index(target_format: TextureFormat) -> usize {
    usize::from(!matches!(
        target_format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Bgra8UnormSrgb
    ))
}

/// Ordered overlay raster passes chained after stock Core2d main pass.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Core2dOverlaySet {
    WaterSurface,
    WaterParticleRaster,
    TileDebug,
    FireParticleRaster,
    TerrainInstanced,
}

pub struct Core2dOverlayOrderPlugin;

impl Plugin for Core2dOverlayOrderPlugin {
    fn build(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.configure_sets(
            Core2d,
            (
                Core2dOverlaySet::WaterSurface,
                Core2dOverlaySet::WaterParticleRaster,
                Core2dOverlaySet::TileDebug,
                Core2dOverlaySet::FireParticleRaster,
                Core2dOverlaySet::TerrainInstanced,
            )
                .chain()
                .after(Core2dSystems::MainPass)
                .before(Core2dSystems::EarlyPostProcess),
        );
    }
}
