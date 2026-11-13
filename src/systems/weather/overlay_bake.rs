//! W3 — bake `WeatherField` cloud/wind into [`OverlayFieldFrame`] (single writer for Wind channel).

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use crate::gui::{OverlayChannel, OverlayFieldFrame};
use crate::systems::sim_control::{SimStepStamp, SimTick, SimTimeMicros};

use super::field::{WeatherField, WeatherTerrain};
use super::tuning::{WeatherPresetId, WeatherTuning};

/// Evidence that the wind/cloud overlay channel was published (dump / matrix / projection).
#[derive(Resource, Debug, Clone, Default)]
pub struct WeatherOverlayBakeState {
    pub revision: u64,
    pub field_revision: u64,
    pub dims: UVec2,
    pub published: bool,
    pub mean_cloud: f32,
    pub mean_rain01: f32,
}

fn weather_overlay_image(dims: UVec2, rgba: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: dims.x.max(1),
            height: dims.y.max(1),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage |= TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
    image
}

/// Raster `next` cloud (R), rain01 (G), wind dir u (B), wind dir v (A) into OverlayChannel::Wind.
pub fn bake_weather_overlay_from_field(
    field: Res<WeatherField>,
    tick: Option<Res<SimTick>>,
    sim_time: Option<Res<SimTimeMicros>>,
    mut images: ResMut<Assets<Image>>,
    mut overlay: ResMut<OverlayFieldFrame>,
    mut state: ResMut<WeatherOverlayBakeState>,
    mut last_field_rev: Local<u64>,
) {
    if !field.is_ready() {
        state.published = false;
        return;
    }
    if *last_field_rev == field.revision && state.published {
        return;
    }
    *last_field_rev = field.revision;

    let dims = field.dims.max(UVec2::ONE);
    let n = (dims.x * dims.y) as usize;
    let snap = &field.next;
    if snap.cloud.len() < n || snap.rain_mm_h.len() < n || snap.wind_mps.len() < n {
        state.published = false;
        return;
    }

    let mut rgba = vec![0u8; n * 4];
    let mut sum_cloud = 0.0_f32;
    let mut sum_rain = 0.0_f32;
    for i in 0..n {
        let cloud = snap.cloud[i].clamp(0.0, 1.0);
        let rain01 = (snap.rain_mm_h[i] / 10.0).clamp(0.0, 1.0);
        let w = snap.wind_mps[i];
        let wx = w[0];
        let wy = w[1];
        let speed = (wx * wx + wy * wy).sqrt();
        let (u, v) = if speed > 0.05 {
            ((wx / speed) * 0.5 + 0.5, (wy / speed) * 0.5 + 0.5)
        } else {
            (0.5, 0.5)
        };
        sum_cloud += cloud;
        sum_rain += rain01;
        let o = i * 4;
        rgba[o] = (cloud * 255.0) as u8;
        rgba[o + 1] = (rain01 * 255.0) as u8;
        rgba[o + 2] = (u.clamp(0.0, 1.0) * 255.0) as u8;
        rgba[o + 3] = (v.clamp(0.0, 1.0) * 255.0) as u8;
    }

    let existing = overlay.fields.get(&OverlayChannel::Wind).cloned();
    let reuse = existing.as_ref().is_some_and(|h| {
        images
            .get(h)
            .is_some_and(|img| img.width() == dims.x && img.height() == dims.y)
    });
    let handle = if reuse {
        let h = existing.expect("reuse implies existing handle");
        if let Some(mut img) = images.get_mut(&h) {
            img.data = Some(rgba);
        }
        h
    } else {
        images.add(weather_overlay_image(dims, rgba))
    };

    overlay.fields.insert(OverlayChannel::Wind, handle);
    overlay.stamp = SimStepStamp::new(
        tick.map(|t| t.0).unwrap_or(0),
        sim_time.map(|t| t.0).unwrap_or(field.prev_micros),
    );
    state.revision = state.revision.wrapping_add(1);
    state.field_revision = field.revision;
    state.dims = dims;
    state.published = true;
    let nf = n.max(1) as f32;
    state.mean_cloud = sum_cloud / nf;
    state.mean_rain01 = sum_rain / nf;
}

/// Sync [`WeatherProjectionNode`] from bake evidence (no parallel weather extract).
pub fn sync_weather_projection_node(
    bake: Res<WeatherOverlayBakeState>,
    fence: Option<Res<crate::render::CommittedVisualSnapshotFence>>,
    mut graph: ResMut<crate::render::extraction::RenderProjectionGraph>,
) {
    let tick = fence.map(|f| f.fire.tick).unwrap_or(bake.revision);
    graph.weather.sync_from_bake(&bake, tick);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bake_publishes_wind_channel() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Image>>()
            .init_resource::<OverlayFieldFrame>()
            .init_resource::<WeatherOverlayBakeState>();

        let tuning = WeatherTuning::default();
        let n = 4usize;
        let field = WeatherField::new(
            42,
            Vec2::splat(2000.0),
            &tuning,
            Some(WeatherPresetId::StormFront),
            WeatherTerrain::neutral(n),
            0,
        );
        app.insert_resource(field);
        app.add_systems(Update, bake_weather_overlay_from_field);
        app.update();

        let state = app.world().resource::<WeatherOverlayBakeState>();
        assert!(state.published, "expected wind overlay bake");
        assert!(state.revision >= 1);
        let overlay = app.world().resource::<OverlayFieldFrame>();
        assert!(overlay.fields.contains_key(&OverlayChannel::Wind));
    }
}
