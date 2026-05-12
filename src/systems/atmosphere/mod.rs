//! Atmosphere simulation + diagnostics (`base_fire2_smoke.md`).

mod gpu_field_bridge;
mod advect;
mod coupling;
mod diagnostics;
mod emitter_sync;
mod field;
mod gpu_paths;
mod overlays;
mod particles;
mod perf_overlay;
pub mod pipeline;
mod render_layers;
mod update;
mod validation_layout;
mod visibility;
mod visual_extract;

pub use advect::advect_atmosphere_field;
pub use coupling::merge_atmosphere_into_logistics_sample;
pub use diagnostics::AtmosphereDiagnostics;
pub use emitter_sync::{fire_emitter_from_heat_fuel, FireEmitter};
pub(crate) use emitter_sync::update_fire_emitters_from_heat;
pub use field::{AtmosphereCell, AtmosphereField, GlobalWind};
pub use gpu_paths::{
    ATMOSPHERE_ASHFALL_WGSL, ATMOSPHERE_GROUND_HAZE_WGSL, ATMOSPHERE_HEAT_DISTORTION_WGSL,
    ATMOSPHERE_PARTICLE_INSTANCING_WGSL, ATMOSPHERE_SMOKE_COLUMN_WGSL, WEATHER_FIRE_FIELD_WGSL,
};
pub use overlays::{atmosphere_overlay_rgba, OverlayMode};
pub use particles::{
    AtmosphereParticle, AtmosphereParticleBudget, AtmosphereParticleKind, AtmosphereParticlePool,
};
pub use perf_overlay::AtmospherePerfThresholds;
pub use pipeline::{configure_atmosphere_pipeline_sets, AtmospherePipelineSet};
pub use render_layers::AtmosphereRenderLayers;
pub use update::{atmosphere_field_blend_fire_overlay_sources, atmosphere_field_fill_from_chunks};
pub use validation_layout::{
    tile_in_any_validation_region, AtmosphereValidationRegion, ATMOSPHERE_VALIDATION_LAYOUT_V1,
};
pub use visibility::visibility_between;

pub use crate::render::{
    ChunkSmokeGpu, ClimateVisualAggregate, FireEmitterGpu, SimChunkSmokeVisualExtract,
    SimFireEmitterVisualExtract,
};

use bevy::prelude::*;

use emitter_sync::sync_fire_emitters;
use particles::atmosphere_particle_controller;

use crate::systems::fire::chunk_smoke_field_pull_from_advected_atmosphere;

pub struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        configure_atmosphere_pipeline_sets(app);
        app.init_resource::<AtmosphereField>()
            .init_resource::<GlobalWind>()
            .add_systems(
                Update,
                (
                    atmosphere_field_fill_from_chunks,
                    atmosphere_field_blend_fire_overlay_sources
                        .after(atmosphere_field_fill_from_chunks),
                )
                    .chain()
                    .in_set(AtmospherePipelineSet::FieldFill),
            )
            .add_systems(
                Update,
                (
                    advect_atmosphere_field,
                    chunk_smoke_field_pull_from_advected_atmosphere.after(advect_atmosphere_field),
                )
                    .chain()
                    .in_set(AtmospherePipelineSet::WindAdvect),
            )
            .add_systems(
                Update,
                (
                    sync_fire_emitters,
                    update_fire_emitters_from_heat.after(sync_fire_emitters),
                )
                    .chain()
                    .in_set(AtmospherePipelineSet::Emitters),
            )
            .init_resource::<AtmosphereParticleBudget>()
            .init_resource::<AtmosphereParticlePool>()
            .add_systems(
                Update,
                atmosphere_particle_controller.in_set(AtmospherePipelineSet::Particles),
            );
        coupling::coupling_systems(app);
        render_layers::render_layer_systems(app);
        visual_extract::visual_extract_systems(app);
        gpu_field_bridge::gpu_field_bridge_systems(app);
        diagnostics::atmosphere_diagnostics_systems(app);
    }
}
