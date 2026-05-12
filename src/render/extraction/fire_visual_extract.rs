//! **Single** fire visual extraction pass: sim ECS → transient buffers → consumers (`base_fire2_smoke.md`).
//!
//! Downstream systems (lights, smoke, fog, particles) must **not** re-query [`ChunkSurfaceFire`] etc.;
//! they read [`FireVisualExtractBuffer`] and [`FireAtmosphereAggregate`] only.

use bevy::prelude::*;

use crate::render::light::{LightCategory, RequestLocalLight};
use crate::render::lighting::{
    build_fire_light_clusters, FireLightCluster, FireLightEmission as ClusterEmission, FireLightType,
};
use crate::systems::ecology::ChunkEcology;
use crate::systems::fire::{ChunkFuelProfile, ChunkSmokeField, ChunkSurfaceFire, FireLightEmission};
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::MaterializedChunk;

use super::fire_emission_profile::{infer_fire_emission_profile, CombustionClass, FireEmissionProfile};

/// Full rewrite each tick — **derived**, not sim truth.
#[derive(Resource, Default, Debug)]
pub struct FireVisualExtractBuffer {
    pub emissions: Vec<FireEmissionProfile>,
}

/// Chunk-scale **regional** hints for fog / atmosphere (from emission buffer; no ECS scan).
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
pub enum FireExtractSet {
    /// One ECS pass → [`FireVisualExtractBuffer`].
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

pub struct FireVisualExtractPlugin;

impl Plugin for FireVisualExtractPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FireVisualExtractBuffer>()
            .init_resource::<FireAtmosphereAggregate>()
            .init_resource::<FireClusterScratch>()
            .configure_sets(
                Update,
                (
                    FireExtractSet::BuildClusters.after(FireExtractSet::BuildProfiles),
                    FireExtractSet::BuildAtmosphere.after(FireExtractSet::BuildClusters),
                    FireExtractSet::EmitLights.after(FireExtractSet::BuildAtmosphere),
                    FireExtractSet::EmitSmoke.after(FireExtractSet::EmitLights),
                    FireExtractSet::EmitParticles.after(FireExtractSet::EmitSmoke),
                ),
            )
            .add_systems(
                Update,
                rewrite_fire_visual_extract_buffer.in_set(FireExtractSet::BuildProfiles),
            )
            .add_systems(
                Update,
                build_fire_clusters_into_scratch.in_set(FireExtractSet::BuildClusters),
            )
            .add_systems(
                Update,
                aggregate_fire_atmosphere_from_buffer.in_set(FireExtractSet::BuildAtmosphere),
            )
            .add_systems(
                Update,
                emit_fire_light_requests_from_cluster_scratch.in_set(FireExtractSet::EmitLights),
            )
            .add_systems(Update, fire_visual_emit_smoke_stub.in_set(FireExtractSet::EmitSmoke))
            .add_systems(
                Update,
                fire_visual_emit_particles_stub.in_set(FireExtractSet::EmitParticles),
            );
    }
}

fn rewrite_fire_visual_extract_buffer(
    mut buf: ResMut<FireVisualExtractBuffer>,
    q: Query<(
        &Chunk,
        &ChunkCellMatrix,
        &ChunkSurfaceFire,
        &FireLightEmission,
        Option<&ChunkSmokeField>,
        Option<&ChunkFuelProfile>,
        Option<&ChunkEcology>,
        Option<&ChunkWeather>,
        Option<&MaterializedChunk>,
    )>,
) {
    buf.emissions.clear();
    for (chunk, matrix, fire, em, smoke, prof, eco, wx, mat_chunk) in &q {
        buf.emissions.push(infer_fire_emission_profile(
            chunk, fire, em, smoke, eco, prof, wx, matrix, mat_chunk,
        ));
    }
}

fn build_fire_clusters_into_scratch(
    buf: Res<FireVisualExtractBuffer>,
    mut scratch: ResMut<FireClusterScratch>,
) {
    scratch.clusters.clear();
    let samples: Vec<ClusterEmission> = buf
        .emissions
        .iter()
        .map(|p| ClusterEmission {
            position: p.world_pos,
            heat: p.heat,
            luminosity: p.luminosity,
            smoke_density: p.smoke_density,
            radius: p.influence_radius,
            priority: p.extract_priority,
            fire_type: combustion_class_to_fire_light_type(p.combustion_class),
        })
        .collect();
    scratch.clusters = build_fire_light_clusters(&samples);
}

fn aggregate_fire_atmosphere_from_buffer(
    mut agg: ResMut<FireAtmosphereAggregate>,
    buf: Res<FireVisualExtractBuffer>,
) {
    if buf.emissions.is_empty() {
        *agg = FireAtmosphereAggregate::default();
        return;
    }
    let n = buf.emissions.len() as f32;
    let mut total_smoke = 0f32;
    let mut mean_color = Vec3::ZERO;
    let mut heat_energy = 0f32;
    let mut ember = 0f32;
    let mut vis = 0f32;
    for e in &buf.emissions {
        total_smoke += e.smoke_density;
        mean_color += e.smoke_color;
        heat_energy += e.heat * e.luminosity.max(0.01);
        ember += e.ember_rate;
        vis += e.visibility_reduction;
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

fn combustion_class_to_fire_light_type(c: CombustionClass) -> FireLightType {
    match c {
        CombustionClass::Vegetation => FireLightType::Forest,
        CombustionClass::Hydrocarbon => FireLightType::Fuel,
        CombustionClass::Electrical => FireLightType::Electrical,
        CombustionClass::Chemical => FireLightType::Chemical,
        CombustionClass::Structural => FireLightType::Structure,
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

/// Stub: route [`FireVisualExtractBuffer`] → smoke volume / GPU when that path exists.
fn fire_visual_emit_smoke_stub(_buf: Res<FireVisualExtractBuffer>) {}

/// Stub: route buffer → particle burst requests when Hanabi bridge lands.
fn fire_visual_emit_particles_stub(_buf: Res<FireVisualExtractBuffer>) {}
