//! Pushes [`crate::render::WeatherFireFieldUniforms`] from **visual extract** only (no sim chunk queries).
//!
//! Fire heat in the GPU field mean uses [`crate::render::extraction::RenderProjectionGraph`] (fire node) + [`SimChunkSmokeVisualExtract`].
//! Weather / ecology means come from [`ClimateVisualAggregate`](crate::render::ClimateVisualAggregate)
//! (filled in the same [`AtmospherePipelineSet::VisualExtract`](super::pipeline::AtmospherePipelineSet) chain).

use bevy::prelude::*;

use crate::gui::{
    preview_partial_min_interval_from_hz, CameraVisualState, FxVisibilitySettings, RepresentationBand,
    RepresentationResult,
};
use crate::render::extraction::RenderProjectionGraph;
use crate::render::{ClimateVisualAggregate, SimChunkSmokeVisualExtract, WeatherFireFieldUniforms};
use super::incremental_schedule::AtmospherePartialFieldState;

/// Mean emitter intensity × smoke/toxic bias (WGSL `means.z` fire channel).
pub(crate) fn effective_fire_heat_for_gpu_field(
    fire_proj: &crate::render::extraction::FireProjectionNode,
    smoke_ex: &SimChunkSmokeVisualExtract,
) -> f32 {
    let fire_mean = if !fire_proj.instance_buffer.is_empty() {
        let n = fire_proj.instance_buffer.len() as f32;
        fire_proj
            .instance_buffer
            .iter()
            .map(|i| i.heat())
            .sum::<f32>()
            / n
    } else if !fire_proj.chunk_heat.is_empty() {
        let n = fire_proj.chunk_heat.len() as f32;
        fire_proj.chunk_heat.iter().map(|h| h.heat).sum::<f32>() / n
    } else {
        0.0
    };

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

#[must_use]
pub(crate) fn blend_partial_field_heat(base: f32, partial: &AtmospherePartialFieldState) -> f32 {
    if partial.cells.is_empty() {
        return base;
    }
    let bias = partial.mean_cell_heat();
    let alpha = (partial.partial_writes.min(8) as f32) * 0.04;
    (base * (1.0 - alpha) + bias * alpha).clamp(0.0, 1.85)
}

#[must_use]
pub(crate) const fn fire_propagate_from_representation_band(band: RepresentationBand) -> f32 {
    match band {
        RepresentationBand::Full => 0.45,
        RepresentationBand::Tactical => 0.35,
        RepresentationBand::Strategic => 0.22,
        RepresentationBand::OverlayOnly => 0.12,
        RepresentationBand::Dormant => 0.0,
    }
}

fn sync_gpu_weather_fire_uniforms_from_extract(
    time: Res<Time>,
    cadence: Option<Res<crate::gui::VisualCadence>>,
    mut cadence_acc: Local<f32>,
    mut cadence_warmed: Local<bool>,
    climate: Res<ClimateVisualAggregate>,
    fire_proj: Option<Res<RenderProjectionGraph>>,
    smoke_ex: Res<SimChunkSmokeVisualExtract>,
    partial_field: Option<Res<AtmospherePartialFieldState>>,
    fx_vis: Option<Res<FxVisibilitySettings>>,
    cam_vis: Option<Res<CameraVisualState>>,
    rep: Option<Res<RepresentationResult>>,
    uniforms: Option<ResMut<WeatherFireFieldUniforms>>,
) {
    let Some(mut u) = uniforms else {
        return;
    };

    if let Some(c) = cadence.as_deref() {
        let interval = preview_partial_min_interval_from_hz(c.atmosphere_hz);
        *cadence_acc += time.delta_secs();
        let allow = !*cadence_warmed || *cadence_acc >= interval;
        if !allow {
            return;
        }
        *cadence_warmed = true;
        if *cadence_acc >= interval {
            *cadence_acc -= interval;
        } else {
            *cadence_acc = 0.0;
        }
    } else {
        *cadence_warmed = false;
        *cadence_acc = 0.0;
    }

    let empty_node = crate::render::extraction::FireProjectionNode::default();
    let mut heat_effective = match fire_proj.as_deref() {
        Some(g) => effective_fire_heat_for_gpu_field(&g.fire, &smoke_ex),
        None => effective_fire_heat_for_gpu_field(&empty_node, &smoke_ex),
    };

    // View Representation: macro/cinematic emphasis on GPU field means (no ECS sim readback).
    let cine = cam_vis
        .as_deref()
        .map(|c| c.cinematic_weight)
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    heat_effective *= 0.9 + 0.22 * cine;
    heat_effective = heat_effective.min(1.85);
    if let Some(partial) = partial_field.as_deref() {
        heat_effective = blend_partial_field_heat(heat_effective, partial);
    }

    let atm_w = fx_vis
        .as_deref()
        .map(|f| f.atmosphere_weight)
        .unwrap_or(1.0)
        .clamp(0.05, 2.5);

    let fire_n = fire_proj
        .as_deref()
        .map(|g| g.fire.instance_buffer.len())
        .unwrap_or(0);
    u.fire_instance_count = (fire_n.min(u32::MAX as usize)) as u32;
    u._fire_pad = UVec3::ZERO;

    let band = rep
        .as_deref()
        .map(|r| r.active_band)
        .unwrap_or(RepresentationBand::Full);
    u.fire_propagate = fire_propagate_from_representation_band(band);

    u.means = Vec4::new(
        climate.mean_rain,
        climate.mean_snow,
        heat_effective,
        climate.mean_fog_density * atm_w,
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
    use super::incremental_schedule::apply_partial_field_updates_tick;
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
            .after(FireVisualFrameSet::ProjectGpu)
            .after(publish_climate_visual_aggregate)
            .after(publish_sim_visual_extract)
            .after(apply_partial_field_updates_tick),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::extraction::{FireVisualGpuInstance, FireProjectionNode};
    use crate::render::sim_visual_extract::ChunkSmokeGpu;
    use crate::systems::atmosphere::incremental_schedule::AtmospherePartialFieldState;

    #[test]
    fn partial_field_blends_into_gpu_heat() {
        let mut partial = AtmospherePartialFieldState::default();
        partial.cells.insert(IVec2::ZERO, 0.9);
        partial.partial_writes = 2;
        let blended = blend_partial_field_heat(0.2, &partial);
        assert!(blended > 0.2);
    }

    #[test]
    fn effective_fire_heat_boosts_with_smoke_extract() {
        let mut fire = FireProjectionNode::default();
        fire.instance_buffer.push(FireVisualGpuInstance {
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

    #[test]
    fn fire_propagate_scales_down_by_representation_band() {
        let full = fire_propagate_from_representation_band(RepresentationBand::Full);
        let tac = fire_propagate_from_representation_band(RepresentationBand::Tactical);
        let ovr = fire_propagate_from_representation_band(RepresentationBand::OverlayOnly);
        assert!(full > tac);
        assert!(tac > ovr);
        assert_eq!(0.0, fire_propagate_from_representation_band(RepresentationBand::Dormant));
    }
}
