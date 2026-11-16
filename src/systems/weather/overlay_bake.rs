//! # LGT-10 — Static baking + light probes for weather overlay (V4-LGT10)
//!
//! Dual-cite: NEW **LGT-10** (gea4v2 12.5.3 *Static Lighting and Baking* p205-206 ·
//! gea4v2 12.5.4 *Dynamic Lighting and Light Probes* p206) · LEGACY keyword
//! **`raster_lighting`** (gea_v2_e4 §12.5 *Lighting with Triangle Rasterization*
//! p198 — baking/probes live under that chapter). No dedicated `bake` /
//! `light_probe` keyword hit in the legacy matrix — cite the §12.5 chapter
//! anchor (prefer gea_v2_e4 over gea_e3 `advanced_lighting_gi`).
//!
//! ## Contract (GEA → engine)
//!
//! | Book idea | Engine surface |
//! |:---|:---|
//! | Static bake of slow fields (12.5.3) | [`bake_weather_overlay_from_field`] sole writer → GPU [`OverlayChannel::Wind`] + CPU probe lattice |
//! | Light probes for dynamic objects (12.5.4) | [`sample_overlay_bake_probe_at_pos_m`] / [`sample_overlay_bake_probe_at_uv`] read the lattice only |
//! | Voxels in scope | Voxel / PixelWorld movers sample the **same** probe lattice — no second bake path |
//!
//! Debt not taken: GPU lightmap atlases · spherical-harmonic irradiance probes ·
//! second writer on `OverlayChannel::Wind` · sampling `WeatherField` from probe helpers.
//!
//! Tripwires: `lgt10_*` (no GPU).

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use crate::gui::{OverlayChannel, OverlayFieldFrame};
use crate::systems::sim_control::{SimStepStamp, SimTick, SimTimeMicros};

use super::field::WeatherField;

/// Baked RGBA probe sample for a dynamic object (cloud/rain/wind from the slow field bake).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverlayBakeProbeSample {
    pub cloud: f32,
    pub rain01: f32,
    /// Unit wind direction in world XY (from baked B/A); zero when calm / not ready.
    pub wind_dir: Vec2,
    pub ready: bool,
}

/// Evidence that the wind/cloud overlay channel was published (dump / matrix / projection).
///
/// Holds the **CPU probe lattice** (LGT-10) mirroring the GPU Wind bake so dynamic
/// objects can sample without a second bake path or GPU readback.
#[derive(Resource, Debug, Clone, Default)]
pub struct WeatherOverlayBakeState {
    pub revision: u64,
    pub field_revision: u64,
    pub dims: UVec2,
    pub published: bool,
    pub mean_cloud: f32,
    pub mean_rain01: f32,
    /// World extent (metres) of the baked lattice — for probe UV mapping.
    pub extent_m: Vec2,
    /// Cell size (metres) at bake time.
    pub cell_size_m: f32,
    /// CPU mirror of baked RGBA8 (R=cloud G=rain01 B/A=wind dir). Empty when not published.
    pub probe_rgba: Vec<u8>,
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

fn decode_probe_texel(rgba: &[u8], o: usize) -> OverlayBakeProbeSample {
    let cloud = rgba[o] as f32 / 255.0;
    let rain01 = rgba[o + 1] as f32 / 255.0;
    let u = rgba[o + 2] as f32 / 255.0;
    let v = rgba[o + 3] as f32 / 255.0;
    // Bake encodes unit dir as (dir * 0.5 + 0.5); invert for dynamic consumers.
    let wind_dir = Vec2::new(u * 2.0 - 1.0, v * 2.0 - 1.0);
    let wind_dir = if wind_dir.length_squared() > 1e-6 {
        wind_dir.normalize()
    } else {
        Vec2::ZERO
    };
    OverlayBakeProbeSample {
        cloud,
        rain01,
        wind_dir,
        ready: true,
    }
}

/// Sample the baked probe lattice at normalised UV `[0,1]²` (bilinear).
///
/// Dynamic-object path (gea4v2 12.5.4) — reads **only** the CPU lattice written by
/// [`bake_weather_overlay_from_field`]; never re-samples [`WeatherField`].
#[must_use]
pub fn sample_overlay_bake_probe_at_uv(
    bake: &WeatherOverlayBakeState,
    uv: Vec2,
) -> OverlayBakeProbeSample {
    if !bake.published || bake.probe_rgba.is_empty() {
        return OverlayBakeProbeSample::default();
    }
    let w = bake.dims.x.max(1) as usize;
    let h = bake.dims.y.max(1) as usize;
    let n = w * h;
    if bake.probe_rgba.len() < n * 4 {
        return OverlayBakeProbeSample::default();
    }
    let u = uv.x.clamp(0.0, 1.0);
    let v = uv.y.clamp(0.0, 1.0);
    let fx = u * (w.saturating_sub(1) as f32);
    let fy = v * (h.saturating_sub(1) as f32);
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;

    let s00 = decode_probe_texel(&bake.probe_rgba, (y0 * w + x0) * 4);
    let s10 = decode_probe_texel(&bake.probe_rgba, (y0 * w + x1) * 4);
    let s01 = decode_probe_texel(&bake.probe_rgba, (y1 * w + x0) * 4);
    let s11 = decode_probe_texel(&bake.probe_rgba, (y1 * w + x1) * 4);

    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let cloud = lerp(lerp(s00.cloud, s10.cloud, tx), lerp(s01.cloud, s11.cloud, tx), ty);
    let rain01 = lerp(
        lerp(s00.rain01, s10.rain01, tx),
        lerp(s01.rain01, s11.rain01, tx),
        ty,
    );
    let wind = Vec2::new(
        lerp(
            lerp(s00.wind_dir.x, s10.wind_dir.x, tx),
            lerp(s01.wind_dir.x, s11.wind_dir.x, tx),
            ty,
        ),
        lerp(
            lerp(s00.wind_dir.y, s10.wind_dir.y, tx),
            lerp(s01.wind_dir.y, s11.wind_dir.y, tx),
            ty,
        ),
    );
    let wind_dir = if wind.length_squared() > 1e-6 {
        wind.normalize()
    } else {
        Vec2::ZERO
    };
    OverlayBakeProbeSample {
        cloud,
        rain01,
        wind_dir,
        ready: true,
    }
}

/// Sample the baked probe lattice at world XY metres (dynamic object / voxel mover).
#[must_use]
pub fn sample_overlay_bake_probe_at_pos_m(
    bake: &WeatherOverlayBakeState,
    pos_m: Vec2,
) -> OverlayBakeProbeSample {
    if !bake.published || bake.extent_m.x <= 0.0 || bake.extent_m.y <= 0.0 {
        return OverlayBakeProbeSample::default();
    }
    let uv = Vec2::new(
        (pos_m.x / bake.extent_m.x).clamp(0.0, 1.0),
        (pos_m.y / bake.extent_m.y).clamp(0.0, 1.0),
    );
    sample_overlay_bake_probe_at_uv(bake, uv)
}

/// Raster `next` cloud (R), rain01 (G), wind dir u (B), wind dir v (A) into OverlayChannel::Wind.
///
/// **Sole writer** of [`OverlayChannel::Wind`] and the CPU probe lattice (LGT-10).
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
        state.probe_rgba.clear();
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
        state.probe_rgba.clear();
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
            img.data = Some(rgba.clone());
        }
        h
    } else {
        images.add(weather_overlay_image(dims, rgba.clone()))
    };

    overlay.fields.insert(OverlayChannel::Wind, handle);
    overlay.stamp = SimStepStamp::new(
        tick.map(|t| t.0).unwrap_or(0),
        sim_time.map(|t| t.0).unwrap_or(field.prev_micros),
    );
    state.revision = state.revision.wrapping_add(1);
    state.field_revision = field.revision;
    state.dims = dims;
    state.extent_m = field.extent_m;
    state.cell_size_m = field.cell_size_m;
    state.probe_rgba = rgba;
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
    use super::super::field::WeatherTerrain;
    use super::super::tuning::{WeatherPresetId, WeatherTuning};

    fn storm_field() -> WeatherField {
        let tuning = WeatherTuning::default();
        let n = 4usize;
        WeatherField::new(
            42,
            Vec2::splat(2000.0),
            &tuning,
            Some(WeatherPresetId::StormFront),
            WeatherTerrain::neutral(n),
            0,
        )
    }

    #[test]
    fn bake_publishes_wind_channel() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Image>>()
            .init_resource::<OverlayFieldFrame>()
            .init_resource::<WeatherOverlayBakeState>();

        app.insert_resource(storm_field());
        app.add_systems(Update, bake_weather_overlay_from_field);
        app.update();

        let state = app.world().resource::<WeatherOverlayBakeState>();
        assert!(state.published, "expected wind overlay bake");
        assert!(state.revision >= 1);
        assert!(!state.probe_rgba.is_empty(), "CPU probe lattice must ship with bake");
        let overlay = app.world().resource::<OverlayFieldFrame>();
        assert!(overlay.fields.contains_key(&OverlayChannel::Wind));
    }

    /// LGT-10: dynamic-object probe samples the baked lattice (not live field).
    #[test]
    fn lgt10_probe_samples_baked_lattice_for_dynamic_objects() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Image>>()
            .init_resource::<OverlayFieldFrame>()
            .init_resource::<WeatherOverlayBakeState>()
            .insert_resource(storm_field())
            .add_systems(Update, bake_weather_overlay_from_field);
        app.update();

        let bake = app.world().resource::<WeatherOverlayBakeState>().clone();
        assert!(bake.published);
        assert_eq!(bake.probe_rgba.len(), (bake.dims.x * bake.dims.y * 4) as usize);

        let centre = sample_overlay_bake_probe_at_uv(&bake, Vec2::splat(0.5));
        assert!(centre.ready);
        assert!((0.0..=1.0).contains(&centre.cloud));
        assert!((0.0..=1.0).contains(&centre.rain01));

        let at_pos = sample_overlay_bake_probe_at_pos_m(&bake, bake.extent_m * 0.5);
        assert!(at_pos.ready);
        assert!((at_pos.cloud - centre.cloud).abs() < 1e-3);

        // Unpublished → not ready (dynamic object must not invent irradiance).
        let empty = WeatherOverlayBakeState::default();
        assert!(!sample_overlay_bake_probe_at_uv(&empty, Vec2::splat(0.5)).ready);
    }

    /// LGT-10: bake skips when field revision unchanged (static bake cadence).
    #[test]
    fn lgt10_bake_skips_unchanged_field_revision() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Image>>()
            .init_resource::<OverlayFieldFrame>()
            .init_resource::<WeatherOverlayBakeState>()
            .insert_resource(storm_field())
            .add_systems(Update, bake_weather_overlay_from_field);
        app.update();
        let rev1 = app.world().resource::<WeatherOverlayBakeState>().revision;
        app.update();
        let rev2 = app.world().resource::<WeatherOverlayBakeState>().revision;
        assert_eq!(rev1, rev2, "static bake must not republish on unchanged field.revision");
    }
}
