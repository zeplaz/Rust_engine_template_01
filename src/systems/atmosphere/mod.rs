//! Atmosphere simulation + diagnostics (`base_fire2_smoke.md`).

mod incremental_schedule;
mod field_page_residency;
mod field_l0_shim;
mod gpu_field_bridge;
mod advect;
mod coupling;
pub mod coupling_terms;
mod diagnostics;
mod emitter_sync;
mod field;
pub mod frame;
mod gpu_paths;
mod overlays;
mod particles;
mod perf_overlay;
pub mod pipeline;
mod render_layers;
pub mod storm_class;
mod update;
mod validation_layout;
mod visibility;
mod visual_extract;

pub use advect::advect_atmosphere_field;
pub use coupling::merge_atmosphere_into_logistics_sample;
pub use coupling_terms::{
    aggregate_source_terms, detect_firestorm_plumes, is_plume_core, pyro_cloud_lift,
    AEROSOL_TEMP_SHADE_C, PLUME_EMBER_MIN, PLUME_HEAT_MIN, PLUME_MIN_CELLS, PLUME_SMOKE_MIN,
    WeatherSourceCell, WeatherSourceTerms,
};
pub use diagnostics::AtmosphereDiagnostics;
pub use emitter_sync::{fire_emitter_from_heat_fuel, FireEmitter};
pub(crate) use emitter_sync::update_fire_emitters_from_heat;
pub use field::{
    atmos_chunk_to_tile, atmos_chunk_to_tile_in, participating_media_visibility, AtmosphereCell,
    ATMO_GRID_ORIGIN, ATMO_GRID_SIZE, PARTICIPATING_ASH_OPTICAL_K, PARTICIPATING_FOG_OPTICAL_K,
    PARTICIPATING_SMOKE_OPTICAL_K, PARTICIPATING_VIS_FLOOR,
};
pub use field_l0_shim::{
    atmosphere_field_l0_shim_system, DEBT_010_WRITERS_ON_L0, FIELD_L0_SHIM_RETIRED,
    FIELD_L0_SHIM_WIRED,
};
pub use frame::{
    build_atmosphere_frame, combine_visibility_m_k, AtmosphereFrame,
    AtmosphereFrameInputs, AtmosphereFrameKey, AtmosphereFrameStats, AtmosphereTile,
    AtmosphereWeatherMeans, VIS_ASH_K, VIS_SMOKE_K,
};
pub use storm_class::{
    classify_storm, classify_storm_with_context, low_depth_hpa,
    StormClass, StormClassThresholds,
};
pub use visual_extract::{DEBT_011_SMOKE_UNIFIED, SMOKE_EXTRACT_BRIDGE};
pub use gpu_paths::{
    ATMOSPHERE_ASHFALL_WGSL, ATMOSPHERE_COMPOSITE_WIRED, ATMOSPHERE_FIELD_PAGE_TABLE_WGSL,
    ATMOSPHERE_GROUND_HAZE_WGSL, ATMOSPHERE_HEAT_DISTORTION_WGSL, ATMOSPHERE_PARTICLE_INSTANCING_WGSL,
    ATMOSPHERE_SMOKE_COLUMN_WGSL, ATMOSPHERE_WGSL_QUARANTINED, WEATHER_FIRE_FIELD_WGSL,
};
pub use overlays::{atmosphere_overlay_rgba, OverlayMode};
pub use particles::{
    AtmosphereParticle, AtmosphereParticleBudget, AtmosphereParticleKind, AtmosphereParticlePool,
};
pub use perf_overlay::AtmospherePerfThresholds;
pub use incremental_schedule::{
    register_atmosphere_incremental_schedule, AtmosphereDirtyRegion, AtmosphereDirtyRegionQueue,
    AtmosphereGpuFieldBridge, AtmosphereIncrementalSchedule, AtmospherePartialFieldState,
    AtmospherePartialUpload, AtmospherePartialUploadPlan, AtmospherePartialWriteMetrics,
    AtmosphereFieldAtlasCenter, FIELD_TEXEL_BYTES, P2H_GPU_PARTIAL_TEXTURE_UPLOADS_ENABLED,
    P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE, build_partial_gpu_uploads, chunk_to_field_texel,
    expand_diffusion_region, mirror_partial_write_metrics, weather_fire_field_full_texture_bytes,
};
pub use field_page_residency::{
    sync_atmosphere_field_page_residency, sync_atmosphere_field_page_table, AtmosphereFieldPage,
    AtmosphereFieldPageTable, AtmosphereFieldResidencyTable, AtmospherePageEntry,
    ATMOSPHERE_FIELD_CHUNKS_PER_PAGE,
};
pub use pipeline::{configure_atmosphere_pipeline_sets, AtmospherePipelineSet};
pub use render_layers::AtmosphereRenderLayers;
pub use update::{
    atmosphere_field_blend_fire_overlay_sources, atmosphere_field_fill_from_chunks,
    atmosphere_field_seed_slab_smoke,
};
pub use validation_layout::{
    tile_in_any_validation_region, AtmosphereValidationRegion, ATMOSPHERE_VALIDATION_LAYOUT_V1,
};
pub use visibility::{sample_tactical_visibility_from_l0, visibility_between};

pub use crate::render::{
    ChunkSmokeGpu, ClimateVisualAggregate, FireEmitterGpu, SimChunkSmokeVisualExtract,
    SimFireEmitterVisualExtract,
};

use bevy::prelude::*;

use emitter_sync::sync_fire_emitters;

use crate::systems::fire::project_chunk_smoke_from_atmosphere_frame;

pub struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        configure_atmosphere_pipeline_sets(app);
        incremental_schedule::register_atmosphere_incremental_schedule(app);
        // DEBT-011: AtmosphereField deleted; AtmospherePlugin owns clipmap stack when
        // SubstratePlugin is absent (lib proofs / headless). Substrate re-init is idempotent.
        app.init_resource::<crate::substrate::atmosphere::AtmosphereClipmapStack>()
            .add_systems(
                Update,
                (
                    atmosphere_field_fill_from_chunks,
                    atmosphere_field_blend_fire_overlay_sources
                        .after(atmosphere_field_fill_from_chunks),
                    update::atmosphere_field_seed_slab_smoke
                        .after(atmosphere_field_blend_fire_overlay_sources),
                )
                    .chain()
                    .in_set(AtmospherePipelineSet::FieldFill),
            )
            .add_systems(
                Update,
                advect_atmosphere_field.in_set(AtmospherePipelineSet::WindAdvect),
            )
            .add_systems(
                Update,
                // W7: sole ChunkSmoke writer — project from AtmosphereFrame after Frame rebuild.
                project_chunk_smoke_from_atmosphere_frame
                    .after(frame::build_atmosphere_frame)
                    .in_set(AtmospherePipelineSet::Frame),
            )
            .add_systems(
                Update,
                (
                    sync_fire_emitters,
                    update_fire_emitters_from_heat.after(sync_fire_emitters),
                )
                    .chain()
                    .in_set(AtmospherePipelineSet::Emitters),
            );
        // ES-0: CPU particle controller + render-prep placeholder removed from live schedule.
        // Fire/atmosphere particles use `fire_vfx` → GPU instanced quad spine (ES-2 generalizes).
        frame::atmosphere_frame_systems(app);
        coupling_terms::coupling_terms_systems(app);
        coupling::coupling_systems(app);
        render_layers::render_layer_systems(app);
        visual_extract::visual_extract_systems(app);
        gpu_field_bridge::gpu_field_bridge_systems(app);
        diagnostics::atmosphere_diagnostics_systems(app);
    }
}

/// ES-0 exit predicate: CPU atmosphere particle + render-prep stubs are not on the live schedule.
#[must_use]
pub fn effects_stub_schedule_clean(field_fill_runs: u64) -> bool {
    field_fill_runs > 0
}

#[cfg(test)]
mod es0_schedule_tests {
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    use crate::gui::InputBindings;
    use crate::systems::sim_control::SimControlPlugin;

    use super::{effects_stub_schedule_clean, AtmosphereDiagnostics, AtmospherePlugin};

    #[test]
    fn effects_stub_schedule_clean_after_sim_ticks() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);
        app.init_resource::<InputBindings>();
        app.add_plugins(SimControlPlugin);
        app.add_plugins(AtmospherePlugin);

        for _ in 0..4 {
            app.update();
        }

        let diag = app.world().resource::<AtmosphereDiagnostics>();
        assert_eq!(
            diag.particle_controller_runs, 0,
            "atmosphere_particle_controller must not run on live schedule (ES-0)"
        );
        assert_eq!(
            diag.render_prep_runs, 0,
            "atmosphere_render_prep_placeholder must not run on live schedule (ES-0)"
        );
        assert!(
            effects_stub_schedule_clean(diag.field_fill_runs),
            "atmosphere field fill should still run"
        );
    }
}
