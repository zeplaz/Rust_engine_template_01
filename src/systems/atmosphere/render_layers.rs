//! Layer toggles for volumetric / distortion passes (`base_fire2_smoke.md` §7).
//!
//! WGSL stubs live under `assets/shaders/` — see [`super::gpu_paths`] for `AssetServer` paths.

use bevy::prelude::*;

use crate::gui::{CameraVisualState, RepresentationBand, RepresentationResult};

use super::diagnostics::AtmosphereDiagnostics;
use super::gpu_paths::{
    ATMOSPHERE_ASHFALL_WGSL, ATMOSPHERE_GROUND_HAZE_WGSL, ATMOSPHERE_HEAT_DISTORTION_WGSL,
    ATMOSPHERE_SMOKE_COLUMN_WGSL,
};
use super::pipeline::AtmospherePipelineSet;

#[derive(Resource, Debug, Clone, Copy)]
pub struct AtmosphereRenderLayers {
    pub ground_haze: bool,
    pub smoke_columns: bool,
    pub heat_distortion: bool,
    pub ashfall: bool,
}

impl Default for AtmosphereRenderLayers {
    fn default() -> Self {
        Self {
            ground_haze: false,
            smoke_columns: false,
            heat_distortion: false,
            ashfall: false,
        }
    }
}

fn sync_atmosphere_render_layers_from_camera(
    cam: Option<Res<CameraVisualState>>,
    policy: Res<RepresentationResult>,
    mut layers: ResMut<AtmosphereRenderLayers>,
) {
    let Some(cam) = cam.as_deref() else {
        return;
    };
    // Presentation-only toggles until WGSL paths bind to these flags (`base_fire2_smoke.md` §7).
    layers.smoke_columns = cam.cinematic_weight > 0.4 || cam.strategic_weight > 0.65;
    layers.ground_haze = cam.strategic_weight > 0.45 || cam.cinematic_weight > 0.35;
    layers.heat_distortion = matches!(
        policy.active_band,
        RepresentationBand::Full | RepresentationBand::Tactical
    ) && cam.cinematic_weight < 0.8;
    layers.ashfall = cam.cinematic_weight > 0.52;
}

fn atmosphere_render_prep_placeholder(
    _layers: Res<AtmosphereRenderLayers>,
    mut diag: ResMut<AtmosphereDiagnostics>,
) {
    // Future: load `ATMOSPHERE_*_WGSL` pipelines when toggles flip on.
    let _paths = (
        ATMOSPHERE_GROUND_HAZE_WGSL,
        ATMOSPHERE_SMOKE_COLUMN_WGSL,
        ATMOSPHERE_HEAT_DISTORTION_WGSL,
        ATMOSPHERE_ASHFALL_WGSL,
    );
    diag.render_prep_runs = diag.render_prep_runs.wrapping_add(1);
}

pub fn render_layer_systems(app: &mut App) {
    app.init_resource::<AtmosphereRenderLayers>()
        .init_resource::<RepresentationResult>()
        .add_systems(
        Update,
        (
            sync_atmosphere_render_layers_from_camera,
            atmosphere_render_prep_placeholder,
        )
            .chain()
            .in_set(AtmospherePipelineSet::RenderPrep),
    );
}

#[cfg(test)]
mod tests {
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    use crate::gui::InputBindings;
    use crate::gui::RepresentationResult;
    use crate::systems::atmosphere::{AtmosphereDiagnostics, AtmospherePlugin};
    use crate::systems::sim_control::SimControlPlugin;

    #[test]
    fn render_prep_runs_each_sim_tick() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);
        app.init_resource::<InputBindings>();
        app.init_resource::<RepresentationResult>();
        app.add_plugins(SimControlPlugin);
        app.add_plugins(AtmospherePlugin);

        app.update();
        let r0 = app.world().resource::<AtmosphereDiagnostics>().render_prep_runs;
        app.update();
        let r1 = app.world().resource::<AtmosphereDiagnostics>().render_prep_runs;
        assert!(r1 > r0, "render_prep_runs should advance each frame");
    }
}
