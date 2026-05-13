//! **Canonical CPU fire visual snapshot** — one ECS pass from sim → `FireVisualFrame` → derived overlay + GPU upload (`base_fire2_smoke.md`).
//!
//! ## Contract (two CPU concepts)
//! 1. **`FireVisualFrame`** — per-frame render snapshot: [`FireVisualGpuInstance`] rows (`FireVisualProxy`) + [`ChunkFireHeat`] chunk table. **Only** this module’s `extract_fire_visual_frame` reads [`ChunkSurfaceFire`] for visuals.
//! 2. **`SharedOverlayFieldBuffers`** — **derived** chunk heat map for minimap / preview; filled **only** from `FireVisualFrame::chunk_heat` (no second ECS scan).
//!
//! GPU: `ExtractResource` copies `FireVisualFrame` to the render world; [`crate::render::gpu_weather_fire_field::prepare_fire_visual_gpu_storage`] uploads `instances` to storage. Render/compute must **not** read ECS or overlay ECS for fire rows—only the frame / GPU buffer.

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

use std::collections::HashMap;

use crate::render::overlay_field_buffers::chunk_fire_heat_maps_differ;
use crate::render::sim_visual_extract::{ChunkFireHeat, FireVisualGpuInstance, SimFireEmitterVisualExtract};
use crate::render::SharedOverlayFieldBuffers;
use crate::render::light::{LightCategory, RequestLocalLight};
use crate::render::lighting::{
    build_fire_light_clusters, FireLightCluster, FireLightEmission as VisFireLightSample, FireLightType,
};
use crate::systems::atmosphere::AtmosphereDiagnostics;
use crate::systems::ecology::ChunkEcology;
use crate::systems::fire::{
    ChunkFuelProfile, ChunkSmokeField, ChunkSurfaceFire, FireLightEmission as SimFireLightEmission,
};
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::MaterializedChunk;

use super::fire_emission_profile::infer_fire_emission_profile;

/// Canonical **CPU** fire visual snapshot for the frame (proxy rows + chunk heat). Not sim truth.
#[derive(Resource, Default, Debug, Clone, ExtractResource)]
pub struct FireVisualFrame {
    pub instances: Vec<FireVisualGpuInstance>,
    pub chunk_heat: Vec<ChunkFireHeat>,
}

/// Chunk-scale **regional** hints for fog / atmosphere (from `FireVisualFrame::instances`; no ECS scan).
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct FireAtmosphereAggregate {
    pub smoke_density: f32,
    pub smoke_color: Vec3,
    pub heat_energy: f32,
    pub ember_density: f32,
    pub visibility_loss: f32,
}

/// Frame-local cluster rows between cluster build and light emit (not entities).
#[derive(Resource, Default, Debug)]
struct FireClusterScratch {
    clusters: Vec<FireLightCluster>,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FireVisualFrameSet {
    /// One ECS pass → [`FireVisualFrame`].
    BuildProfiles,
    /// Greedy merge → [`FireClusterScratch`].
    BuildClusters,
    /// Regional averages → [`FireAtmosphereAggregate`].
    BuildAtmosphere,
    /// `RequestLocalLight` messages for pooled lights.
    EmitLights,
    /// Future: GPU smoke / volume extract (buffer only).
    EmitSmoke,
    /// Future: particle burst hints (buffer only).
    EmitParticles,
}

pub struct FireVisualFramePlugin;

impl Plugin for FireVisualFramePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FireVisualFrame>()
            .init_resource::<SimFireEmitterVisualExtract>()
            .init_resource::<FireAtmosphereAggregate>()
            .init_resource::<FireClusterScratch>()
            .configure_sets(
                Update,
                (
                    FireVisualFrameSet::BuildClusters.after(FireVisualFrameSet::BuildProfiles),
                    FireVisualFrameSet::BuildAtmosphere.after(FireVisualFrameSet::BuildClusters),
                    FireVisualFrameSet::EmitLights.after(FireVisualFrameSet::BuildAtmosphere),
                    FireVisualFrameSet::EmitSmoke.after(FireVisualFrameSet::EmitLights),
                    FireVisualFrameSet::EmitParticles.after(FireVisualFrameSet::EmitSmoke),
                ),
            )
            .add_systems(
                Update,
                (
                    extract_fire_visual_frame,
                    sync_shared_overlay_from_frame,
                    sync_sim_fire_emitter_visual_from_frame,
                    sync_atmosphere_diag_fire_instance_count,
                )
                    .chain()
                    .in_set(FireVisualFrameSet::BuildProfiles),
            )
            .add_systems(
                Update,
                build_fire_clusters_into_scratch.in_set(FireVisualFrameSet::BuildClusters),
            )
            .add_systems(
                Update,
                aggregate_fire_atmosphere_from_frame.in_set(FireVisualFrameSet::BuildAtmosphere),
            )
            .add_systems(
                Update,
                emit_fire_light_requests_from_cluster_scratch.in_set(FireVisualFrameSet::EmitLights),
            )
            .add_systems(
                Update,
                fire_visual_emit_smoke_stub.in_set(FireVisualFrameSet::EmitSmoke),
            )
            .add_systems(
                Update,
                fire_visual_emit_particles_stub.in_set(FireVisualFrameSet::EmitParticles),
            );
    }
}

fn sync_shared_overlay_from_frame(
    frame: Res<FireVisualFrame>,
    mut shared: ResMut<SharedOverlayFieldBuffers>,
) {
    let mut next = HashMap::new();
    for h in &frame.chunk_heat {
        let e = next.entry(h.chunk).or_insert(0.0);
        *e = f32::max(*e, h.heat);
    }
    if chunk_fire_heat_maps_differ(&shared.chunk_fire_heat, &next) {
        shared.chunk_fire_heat = next;
        shared.bump();
    }
}

fn sync_sim_fire_emitter_visual_from_frame(
    frame: Res<FireVisualFrame>,
    mut sim_fire: ResMut<SimFireEmitterVisualExtract>,
) {
    sim_fire.instances.clear();
    sim_fire.instances.reserve(frame.instances.len());
    for row in &frame.instances {
        sim_fire.instances.push(row.to_fire_emitter_gpu());
    }
}

fn sync_atmosphere_diag_fire_instance_count(
    frame: Res<FireVisualFrame>,
    mut diag: ResMut<AtmosphereDiagnostics>,
) {
    diag.last_emitter_extract_count = frame.instances.len();
}

fn extract_fire_visual_frame(
    mut frame: ResMut<FireVisualFrame>,
    q: Query<(
        &Chunk,
        &ChunkCellMatrix,
        &ChunkSurfaceFire,
        &SimFireLightEmission,
        Option<&ChunkSmokeField>,
        Option<&ChunkFuelProfile>,
        Option<&ChunkEcology>,
        Option<&ChunkWeather>,
        Option<&MaterializedChunk>,
    )>,
) {
    frame.instances.clear();
    frame.chunk_heat.clear();
    for (chunk, matrix, fire, em, smoke, prof, eco, wx, mat_chunk) in &q {
        let profile = infer_fire_emission_profile(
            chunk, fire, em, smoke, eco, prof, wx, matrix, mat_chunk,
        );
        frame
            .instances
            .push(FireVisualGpuInstance::from(&profile));
        frame.chunk_heat.push(ChunkFireHeat {
            chunk: profile.chunk_coord,
            heat: profile.heat,
            smoke: profile.smoke_density,
        });
    }
}

fn build_fire_clusters_into_scratch(
    frame: Res<FireVisualFrame>,
    mut scratch: ResMut<FireClusterScratch>,
) {
    scratch.clusters.clear();
    let samples: Vec<VisFireLightSample> = frame
        .instances
        .iter()
        .map(FireVisualGpuInstance::cluster_emission)
        .collect();
    scratch.clusters = build_fire_light_clusters(&samples);
}

fn aggregate_fire_atmosphere_from_frame(
    mut agg: ResMut<FireAtmosphereAggregate>,
    frame: Res<FireVisualFrame>,
) {
    if frame.instances.is_empty() {
        *agg = FireAtmosphereAggregate::default();
        return;
    }
    let n = frame.instances.len() as f32;
    let mut total_smoke = 0f32;
    let mut mean_color = Vec3::ZERO;
    let mut heat_energy = 0f32;
    let mut ember = 0f32;
    let mut vis = 0f32;
    for row in &frame.instances {
        total_smoke += row.smoke_ember_vis_priority.x;
        mean_color += row.smoke_color_toxic.xyz();
        heat_energy += row.heat() * row.luminosity().max(0.01);
        ember += row.smoke_ember_vis_priority.y;
        vis += row.smoke_ember_vis_priority.z;
    }
    agg.smoke_density = (total_smoke / n).clamp(0.0, 1.0);
    agg.smoke_color = (mean_color / n.max(1.0)).clamp(Vec3::ZERO, Vec3::ONE);
    agg.heat_energy = heat_energy;
    agg.ember_density = (ember / n).clamp(0.0, 1.0);
    agg.visibility_loss = (vis / n).clamp(0.0, 1.0);
}

const LUMINOSITY_TO_POINTLIGHT: f32 = 24_000.0;

fn emit_fire_light_requests_from_cluster_scratch(
    scratch: Res<FireClusterScratch>,
    mut writer: MessageWriter<RequestLocalLight>,
) {
    for cluster in &scratch.clusters {
        writer.write(cluster_to_request(cluster));
    }
}

fn cluster_to_request(cluster: &FireLightCluster) -> RequestLocalLight {
    let color = match cluster.dominant_type {
        FireLightType::Forest => Color::srgb(1.0, 0.42, 0.18),
        FireLightType::Fuel => Color::srgb(1.0, 0.55, 0.25),
        FireLightType::Chemical => Color::srgb(0.4, 1.0, 0.3),
        FireLightType::Electrical => Color::srgb(0.6, 0.8, 1.0),
        FireLightType::Structure => Color::srgb(1.0, 0.35, 0.12),
    };

    let intensity = cluster.total_luminosity.max(0.05)
        * LUMINOSITY_TO_POINTLIGHT
        * (cluster.member_count as f32).sqrt().max(1.0);
    let range = 80.0 + cluster.radius * 1.8;
    let priority = cluster.total_heat + cluster.member_count as f32 * 0.25;
    let flicker_phase = cluster.centroid.x * 0.013;
    let flicker_strength = 0.03 + cluster.total_heat * 0.05;

    RequestLocalLight {
        position: cluster.centroid,
        color,
        intensity,
        range,
        priority,
        category: LightCategory::Fire,
        flicker_phase,
        flicker_strength,
    }
}

/// Stub: route [`FireVisualFrame`] → smoke volume / GPU when that path exists.
fn fire_visual_emit_smoke_stub(_frame: Res<FireVisualFrame>) {}

/// Stub: route frame → particle burst requests when Hanabi bridge lands.
fn fire_visual_emit_particles_stub(_frame: Res<FireVisualFrame>) {}
