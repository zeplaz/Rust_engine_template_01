//! @orchestrator-status IN_PROGRESS
//! @orchestrator-owner render_pipeline_agent
//! @orchestrator-do-not-cleanup
//! Minimum on-screen tile size clamp (Visual Aid v2 VA3).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::map_camera::{MainWorldCameraOrthoTrace, MapCameraDesired};
use super::SimulationMapViewport;
use super::world_representation::{GlobalLodState, LodInputs};

/// Minimum pixels per world tile on the sim map viewport.
#[derive(Resource, Clone, Debug)]
pub struct TileReadabilityConfig {
    pub min_pixels_per_tile: f32,
    pub enabled: bool,
}

impl Default for TileReadabilityConfig {
    fn default() -> Self {
        Self {
            min_pixels_per_tile: 14.0,
            enabled: true,
        }
    }
}

/// Bundled inputs so world-representation systems stay within Bevy chain param limits.
#[derive(SystemParam)]
pub struct TileReadabilityInputs<'w> {
    pub config: Res<'w, TileReadabilityConfig>,
    pub ortho: Res<'w, MainWorldCameraOrthoTrace>,
    pub map_vp: Res<'w, SimulationMapViewport>,
}

/// Witness for VISUAL-AID-V2-03.
#[derive(Resource, Clone, Debug, Default)]
pub struct TileReadabilityWitness {
    pub clamp_active: bool,
    pub screen_pixels_per_tile: f32,
    pub zoom_floor_applied: f32,
}

/// Nonlinear visual scale bias (VA5) — visual layer only, not simulation transforms.
#[derive(Resource, Clone, Debug)]
pub struct ZoomVisualBias {
    pub min_scale: f32,
    pub max_scale: f32,
    pub curve_exp: f32,
    pub enabled: bool,
}

impl Default for ZoomVisualBias {
    fn default() -> Self {
        Self {
            min_scale: 0.75,
            max_scale: 2.2,
            curve_exp: 0.62,
            enabled: true,
        }
    }
}

impl ZoomVisualBias {
    #[must_use]
    pub fn visual_scale_from_zoom(&self, zoom: f32) -> f32 {
        if !self.enabled {
            return 1.0;
        }
        let z = zoom.abs().max(1e-4);
        z.powf(self.curve_exp).clamp(self.min_scale, self.max_scale)
    }
}

/// Estimate screen pixels per world tile from ortho trace + viewport width.
#[must_use]
pub fn screen_pixels_per_tile(
    ortho: &MainWorldCameraOrthoTrace,
    map_vp: &SimulationMapViewport,
    desired: &MapCameraDesired,
) -> f32 {
    if !map_vp.is_adequate_for_camera() {
        return f32::MAX;
    }
    let vp_w = map_vp.logical_size().x.max(1.0);
    let world_w = ortho.fixed_width.max(1e-3) * desired.scale.x.abs().max(1e-3);
    vp_w / world_w
}

/// Floor zoom so tiles stay readable when zooming out.
#[must_use]
pub fn readability_zoom_floor(config: &TileReadabilityConfig, screen_px_per_tile: f32, current_zoom: f32) -> f32 {
    if !config.enabled || screen_px_per_tile >= config.min_pixels_per_tile {
        return current_zoom;
    }
    let ratio = config.min_pixels_per_tile / screen_px_per_tile.max(1e-3);
    current_zoom * ratio
}

/// Apply readability floor to [`LodInputs::zoom_level`] (alpha 0..1).
pub fn apply_readability_to_lod_inputs(
    config: &TileReadabilityConfig,
    screen_px: f32,
    inputs: &mut LodInputs,
    zoom_alpha_floor: f32,
) {
    if !config.enabled {
        return;
    }
    if screen_px < config.min_pixels_per_tile {
        inputs.zoom_level = inputs.zoom_level.max(zoom_alpha_floor);
        inputs.screen_density = (screen_px / config.min_pixels_per_tile).clamp(0.25, 1.0);
    }
}

pub fn sync_tile_readability_witness(
    inputs: TileReadabilityInputs,
    desired: Res<MapCameraDesired>,
    mut global: ResMut<GlobalLodState>,
    mut witness: ResMut<TileReadabilityWitness>,
    mut va_witness: ResMut<crate::dev::VisualAidV2Witness>,
) {
    let px = screen_pixels_per_tile(inputs.ortho.as_ref(), inputs.map_vp.as_ref(), desired.as_ref());
    witness.screen_pixels_per_tile = px;
    witness.clamp_active = inputs.config.enabled && px < inputs.config.min_pixels_per_tile;
    witness.zoom_floor_applied = readability_zoom_floor(&inputs.config, px, desired.scale.x);
    global.readability_zoom_floor = if witness.clamp_active { 0.42 } else { 0.0 };
    global.readability_screen_density = if witness.clamp_active {
        (px / inputs.config.min_pixels_per_tile).clamp(0.25, 1.0)
    } else {
        1.0
    };
    va_witness.tile_readability_clamp_active = witness.clamp_active;
    va_witness.screen_pixels_per_tile = px;
}

/// Apply precomputed bias from [`GlobalLodState`] onto LOD inputs.
pub fn apply_tile_readability_lod_bias(global: &GlobalLodState, inputs: &mut LodInputs) {
    if global.readability_zoom_floor > 0.0 {
        inputs.zoom_level = inputs.zoom_level.max(global.readability_zoom_floor);
        inputs.screen_density = inputs.screen_density.min(global.readability_screen_density);
    }
}

pub struct TileReadabilityPlugin;

impl Plugin for TileReadabilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileReadabilityConfig>()
            .init_resource::<TileReadabilityWitness>()
            .init_resource::<ZoomVisualBias>()
            .add_systems(
                Update,
                (
                    sync_tile_readability_witness,
                    crate::gui::representation_policy::sync_visual_aidv2_representation_witness,
                )
                    .run_if(crate::gui::ui_gates::in_simulation_or_editor),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_readability_zoom_floor_when_too_small() {
        let config = TileReadabilityConfig {
            min_pixels_per_tile: 20.0,
            enabled: true,
        };
        let floor = readability_zoom_floor(&config, 10.0, 0.5);
        assert!(floor > 0.5);
    }

    #[test]
    fn zoom_visual_bias_clamps_curve() {
        let bias = ZoomVisualBias::default();
        let s = bias.visual_scale_from_zoom(0.2);
        assert!(s >= bias.min_scale);
    }
}
