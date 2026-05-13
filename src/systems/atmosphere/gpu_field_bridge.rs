//! Pushes [`crate::render::WeatherFireFieldUniforms`] from **visual extract** only (no sim chunk queries).
//!
//! Fire heat in the GPU field mean uses [`crate::render::FireVisualFrame`] + [`SimChunkSmokeVisualExtract`].
//! Weather / ecology means come from [`ClimateVisualAggregate`](crate::render::ClimateVisualAggregate)
//! (filled in the same [`AtmospherePipelineSet::VisualExtract`](super::pipeline::AtmospherePipelineSet) chain).

use bevy::prelude::*;

use crate::render::extraction::FireVisualFrame;
use crate::render::{ClimateVisualAggregate, SimChunkSmokeVisualExtract, WeatherFireFieldUniforms};

/// Mean emitter intensity × smoke/toxic bias (WGSL `means.z` fire channel).
pub(crate) fn effective_fire_heat_for_gpu_field(
    fire_visual: &FireVisualFrame,
    smoke_ex: &SimChunkSmokeVisualExtract,
) -> f32 {
    let n_emit = fire_visual.instances.len().max(1) as f32;
    let fire_mean = fire_visual
        .instances
        .iter()
        .map(|i| i.heat())
        .sum::<f32>()
        / n_emit;

    let (smoke_d, smoke_t) = if smoke_ex.instances.is_empty() {
        (0.0f32, 0.0f32)
    } else {
        let n = smoke_ex.instances.len() as f32;
        let d: f32 = smoke_ex
            .instances
            .iter()
            .map(|s| s.density_tox_vis.x)
            .sum::<f32>()
            / n;
        let t: f32 = smoke_ex
            .instances
            .iter()
            .map(|s| s.density_tox_vis.y)
            .sum::<f32>()
            / n;
        (d, t)
    };
    let fire_boost = 1.0 + smoke_d.clamp(0.0, 1.0) * 0.22 + smoke_t.clamp(0.0, 1.0) * 0.12;
    (fire_mean * fire_boost).min(1.5)
}

fn sync_gpu_weather_fire_uniforms_from_extract(
    time: Res<Time>,
    climate: Res<ClimateVisualAggregate>,
    fire_visual: Option<Res<FireVisualFrame>>,
    smoke_ex: Res<SimChunkSmokeVisualExtract>,
    uniforms: Option<ResMut<WeatherFireFieldUniforms>>,
) {
    let Some(mut u) = uniforms else {
        return;
    };

    let heat_effective = match fire_visual.as_deref() {
        Some(f) => effective_fire_heat_for_gpu_field(f, &smoke_ex),
        None => effective_fire_heat_for_gpu_field(
            &FireVisualFrame::default(),
            &smoke_ex,
        ),
    };

    let fire_n = fire_visual
        .as_deref()
        .map(|f| f.instances.len())
        .unwrap_or(0);
    u.fire_instance_count = (fire_n.min(u32::MAX as usize)) as u32;
    u._fire_pad = UVec3::ZERO;

    u.means = Vec4::new(
        climate.mean_rain,
        climate.mean_snow,
        heat_effective,
        climate.mean_fog_density,
    );
    u.extra_means = Vec4::new(
        climate.mean_biomass,
        climate.mean_fire_risk,
        climate.mean_wind_speed,
        climate.mean_lightning_risk,
    );
    u.time_secs = time.elapsed_secs();
}

pub fn gpu_field_bridge_systems(app: &mut App) {
    use crate::render::extraction::FireVisualFrameSet;
    use super::pipeline::AtmospherePipelineSet;
    use super::visual_extract::{
        publish_climate_visual_aggregate, publish_sim_visual_extract,
        sync_weather_precip_sample_from_climate_aggregate,
    };

    app.add_systems(
        Update,
        (
            publish_climate_visual_aggregate,
            publish_sim_visual_extract,
            sync_weather_precip_sample_from_climate_aggregate.after(publish_sim_visual_extract),
        )
            .chain()
            .in_set(AtmospherePipelineSet::VisualExtract),
    )
    .add_systems(
        Update,
        sync_gpu_weather_fire_uniforms_from_extract
            .after(FireVisualFrameSet::BuildProfiles)
            .after(publish_climate_visual_aggregate)
            .after(publish_sim_visual_extract),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::extraction::FireVisualFrame;
    use crate::render::sim_visual_extract::FireVisualGpuInstance;
    use crate::render::ChunkSmokeGpu;

    #[test]
    fn effective_fire_heat_boosts_with_smoke_extract() {
        let mut fire = FireVisualFrame::default();
        fire.instances.push(FireVisualGpuInstance {
            chunk_xy_heat_lum: Vec4::new(0.0, 0.0, 0.5, 0.0),
            ..Default::default()
        });
        let mut smoke = SimChunkSmokeVisualExtract::default();
        smoke.instances.push(ChunkSmokeGpu {
            chunk_xy: Vec4::ZERO,
            density_tox_vis: Vec4::new(0.8, 0.2, 0.0, 0.0),
        });
        let h0 = effective_fire_heat_for_gpu_field(&fire, &SimChunkSmokeVisualExtract::default());
        let h1 = effective_fire_heat_for_gpu_field(&fire, &smoke);
        assert!(h1 > h0);
        assert!(h1 > 0.5);
    }
}
