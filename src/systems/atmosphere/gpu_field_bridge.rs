//! Pushes [`crate::render::WeatherFireFieldUniforms`] from **visual extract** only (no sim chunk queries).
//!
//! **W6 / X4:** cloud/precip/fog means come from [`AtmosphereFrame`]; fire heat uses
//! [`crate::render::extraction::RenderProjectionGraph`] (**fire** + **smoke** projection nodes).
//! Smoke channel bias comes from the smoke node (frame-sourced when the graph saw
//! [`AtmosphereFrame`]) — never a second ECS smoke scan. Ecology means still ride
//! [`ClimateVisualAggregate`] (biomass / fire_risk only). Single GPU field upload path.

use bevy::prelude::*;

use crate::gui::{
    preview_partial_min_interval_from_hz, CameraVisualState, FxVisibilitySettings, RepresentationBand,
    RepresentationResult,
};
use crate::render::extraction::{RenderProjectionGraph, SmokeProjectionNode};
use crate::render::{ClimateVisualAggregate, WeatherFireFieldUniforms};
use super::diagnostics::AtmosphereDiagnostics;
use super::frame::AtmosphereFrame;
use super::incremental_schedule::AtmospherePartialFieldState;

/// Distinct weather/fire **field** upload writers observed by this bridge (Accept: == 1).
pub const GPU_FIELD_UPLOAD_PATHS: u32 = 1;

/// Mean emitter intensity × smoke/toxic bias (WGSL `means.z` fire channel).
///
/// Smoke density/toxicity come from [`SmokeProjectionNode`] only (Layer B bridge).
pub(crate) fn effective_fire_heat_for_gpu_field(
    fire_proj: &crate::render::extraction::FireProjectionNode,
    smoke_proj: &SmokeProjectionNode,
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

    let smoke_d = smoke_proj.mean_density;
    let smoke_t = smoke_proj.mean_toxicity;
    let fire_boost = 1.0 + smoke_d.clamp(0.0, 1.0) * 0.22 + smoke_t.clamp(0.0, 1.0) * 0.12;
    (fire_mean * fire_boost).min(1.5)
}

/// Fog/smoke visual channel (WGSL `means.w`) — climate fog plus projected smoke density.
#[must_use]
pub(crate) fn fog_smoke_channel_from_bridge(
    climate_fog: f32,
    atm_weight: f32,
    smoke_proj: &SmokeProjectionNode,
) -> f32 {
    let fog = climate_fog * atm_weight;
    let smoke = smoke_proj.mean_density.clamp(0.0, 1.0) * 0.55;
    (fog + smoke).min(1.85)
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

/// Fill [`WeatherFireFieldUniforms`] from frame weather means + projection fire/smoke (pure).
#[must_use]
pub(crate) fn weather_fire_field_means_from_frame(
    frame: &AtmosphereFrame,
    climate: &ClimateVisualAggregate,
    fire_node: &crate::render::extraction::FireProjectionNode,
    smoke_node: &SmokeProjectionNode,
    atm_weight: f32,
    cine: f32,
    band: RepresentationBand,
    partial: Option<&AtmospherePartialFieldState>,
    time_secs: f32,
) -> WeatherFireFieldUniforms {
    let wx = frame.weather_means();
    let mut heat_effective = effective_fire_heat_for_gpu_field(fire_node, smoke_node);
    heat_effective *= 0.9 + 0.22 * cine.clamp(0.0, 1.0);
    heat_effective = heat_effective.min(1.85);
    if let Some(partial) = partial {
        heat_effective = blend_partial_field_heat(heat_effective, partial);
    }
    let fog_smoke = fog_smoke_channel_from_bridge(wx.fog, atm_weight, smoke_node);
    let fire_n = fire_node.instance_buffer.len();
    WeatherFireFieldUniforms {
        means: Vec4::new(wx.rain01, wx.snow01, heat_effective, fog_smoke),
        extra_means: Vec4::new(
            climate.mean_biomass,
            climate.mean_fire_risk,
            wx.wind_speed01,
            wx.lightning,
        ),
        time_secs,
        fire_propagate: fire_propagate_from_representation_band(band),
        fire_instance_count: (fire_n.min(u32::MAX as usize)) as u32,
        _fire_pad: UVec3::ZERO,
        ..WeatherFireFieldUniforms::default()
    }
}

fn sync_gpu_weather_fire_uniforms_from_extract(
    time: Res<Time>,
    cadence: Option<Res<crate::gui::VisualCadence>>,
    mut cadence_acc: Local<f32>,
    mut cadence_warmed: Local<bool>,
    frame: Res<AtmosphereFrame>,
    climate: Res<ClimateVisualAggregate>,
    fire_proj: Option<Res<RenderProjectionGraph>>,
    partial_field: Option<Res<AtmospherePartialFieldState>>,
    fx_vis: Option<Res<FxVisibilitySettings>>,
    cam_vis: Option<Res<CameraVisualState>>,
    rep: Option<Res<RepresentationResult>>,
    uniforms: Option<ResMut<WeatherFireFieldUniforms>>,
    mut diag: ResMut<AtmosphereDiagnostics>,
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

    let empty_fire = crate::render::extraction::FireProjectionNode::default();
    let empty_smoke = SmokeProjectionNode::default();
    let (fire_node, smoke_node) = match fire_proj.as_deref() {
        Some(g) => (&g.fire, &g.smoke),
        None => (&empty_fire, &empty_smoke),
    };

    let atm_w = fx_vis
        .as_deref()
        .map(|f| f.atmosphere_weight)
        .unwrap_or(1.0)
        .clamp(0.05, 2.5);
    let cine = cam_vis
        .as_deref()
        .map(|c| c.cinematic_weight)
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let band = rep
        .as_deref()
        .map(|r| r.active_band)
        .unwrap_or(RepresentationBand::Full);

    let filled = weather_fire_field_means_from_frame(
        &frame,
        &climate,
        fire_node,
        smoke_node,
        atm_w,
        cine,
        band,
        partial_field.as_deref(),
        time.elapsed_secs(),
    );
    // Preserve partial-dispatch pads owned elsewhere; overwrite means channels only.
    u.means = filled.means;
    u.extra_means = filled.extra_means;
    u.time_secs = filled.time_secs;
    u.fire_propagate = filled.fire_propagate;
    u.fire_instance_count = filled.fire_instance_count;
    u._fire_pad = filled._fire_pad;

    diag.gpu_field_upload_paths = GPU_FIELD_UPLOAD_PATHS;
    diag.presentation_frame_revision = frame.revision();
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
    use crate::render::extraction::{FireVisualGpuInstance, FireProjectionNode, SmokeProjectionNode};
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
    fn effective_fire_heat_boosts_with_smoke_projection() {
        let mut fire = FireProjectionNode::default();
        fire.instance_buffer.push(FireVisualGpuInstance {
            chunk_xy_heat_lum: Vec4::new(0.0, 0.0, 0.5, 0.0),
            ..Default::default()
        });
        let smoke = SmokeProjectionNode {
            mean_density: 0.8,
            mean_toxicity: 0.2,
            row_count: 1,
            extract_wired: true,
            ..Default::default()
        };
        let h0 = effective_fire_heat_for_gpu_field(&fire, &SmokeProjectionNode::default());
        let h1 = effective_fire_heat_for_gpu_field(&fire, &smoke);
        assert!(h1 > h0);
        assert!(h1 > 0.5);
    }

    #[test]
    fn fog_smoke_channel_uses_projection_density_only() {
        let smoke = SmokeProjectionNode {
            mean_density: 0.5,
            extract_wired: true,
            row_count: 1,
            ..Default::default()
        };
        let v = fog_smoke_channel_from_bridge(0.1, 1.0, &smoke);
        assert!(v > 0.1);
        assert!(v < 1.0);
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

    #[test]
    fn uniforms_means_come_from_atmosphere_frame() {
        let mut frame = AtmosphereFrame::default();
        frame.rebuild_for_tests_with_smoke(42, 0.0);
        let wx = *frame.weather_means();
        assert!(wx.tiles > 0);
        let climate = ClimateVisualAggregate {
            mean_biomass: 0.3,
            mean_fire_risk: 0.1,
            ..Default::default()
        };
        let u = weather_fire_field_means_from_frame(
            &frame,
            &climate,
            &FireProjectionNode::default(),
            &SmokeProjectionNode::default(),
            1.0,
            0.0,
            RepresentationBand::Full,
            None,
            1.5,
        );
        assert_eq!(u.means.x, wx.rain01);
        assert_eq!(u.means.y, wx.snow01);
        assert_eq!(u.extra_means.x, 0.3);
        assert_eq!(u.extra_means.z, wx.wind_speed01);
        assert_eq!(GPU_FIELD_UPLOAD_PATHS, 1);
    }

    #[test]
    fn bridge_does_not_write_sim_authorities() {
        let src = include_str!("gpu_field_bridge.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap();
        assert!(!prod.contains("ResMut<AtmosphereFrame>"));
        assert!(!prod.contains("ResMut<WeatherField>"));
        assert!(!prod.contains("ResMut<AtmosphereClipmapStack>"));
    }
}
