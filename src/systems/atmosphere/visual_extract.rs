//! Publishes sim state into [`crate::render::extraction::sim_visual_extract`] resources (`base_gui_next.md` Stage 2).
//!
//! **LGT-04** (gea4v2 12.5.13 *Skies, Clouds, and Volumetrics* · frame-fed W6/X4 presentation):
//! precip / haze consumers read optical depth from LGT-05 [`participating_media_visibility`] —
//! volumetric *look* via density→visibility, not a second ray-march path (RND-14 cards stay).
//!
//! **W7:** smoke extract prefers [`AtmosphereFrame`] tracers (same authority as the smoke projection
//! node). [`ClimateVisualAggregate`] weather channels come from the frame; ecology means ride
//! [`EcologyVisualSnapshot`] (no ChunkEcology scan here).

use bevy::prelude::*;

use crate::render::{
    ChunkSmokeGpu, ClimateVisualAggregate, EcologyVisualSnapshot, SimChunkSmokeVisualExtract,
};
use crate::systems::fire::ChunkSmokeField;
use crate::systems::weather::WeatherPrecipVisualSample;
use crate::terrain::generation::Chunk;

use super::diagnostics::AtmosphereDiagnostics;
use super::frame::AtmosphereFrame;

const SMOKE_EXTRACT_EPS: f32 = 1e-4;

/// [`ClimateVisualAggregate`] for GPU / precip overlay.
///
/// Weather from [`AtmosphereFrame::weather_means`]; ecology means from [`EcologyVisualSnapshot`]
/// (sole ecology scan lives in `publish_ecology_visual_snapshot`).
pub(crate) fn publish_climate_visual_aggregate(
    frame: Res<AtmosphereFrame>,
    ecology: Option<Res<EcologyVisualSnapshot>>,
    mut out: ResMut<ClimateVisualAggregate>,
) {
    let wx = frame.weather_means();
    let (bio, frisk, ne) = ecology
        .as_deref()
        .map(|e| (e.mean_biomass, e.mean_fire_risk, e.ecology_chunk_count))
        .unwrap_or((0.0, 0.0, 0));
    *out = ClimateVisualAggregate {
        mean_rain: wx.rain01,
        mean_snow: wx.snow01,
        mean_fog_density: wx.fog,
        mean_wind_speed: wx.wind_speed01,
        mean_wind_mps: wx.wind_mps,
        mean_lightning_risk: wx.lightning,
        mean_biomass: bio,
        mean_fire_risk: frisk,
        weather_chunk_count: wx.tiles,
        ecology_chunk_count: ne,
    };
}

/// Clear-air reference distance (metres) before fog optical depth (LGT-04 skies/haze).
pub const VOLUMETRIC_BASE_VISIBILITY_M: f32 = 10_000.0;

/// LGT-04: fog density → unit transmittance + metre visibility for precip/haze presentation.
#[inline]
#[must_use]
pub fn volumetric_haze_from_fog(fog01: f32) -> (f32, f32) {
    let unit = super::participating_media_visibility(0.0, fog01, 0.0);
    (unit, VOLUMETRIC_BASE_VISIBILITY_M * unit)
}

pub(crate) fn sync_weather_precip_sample_from_climate_aggregate(
    climate: Res<ClimateVisualAggregate>,
    mut sample: ResMut<WeatherPrecipVisualSample>,
) {
    let n = climate.weather_chunk_count;
    if n == 0 {
        *sample = WeatherPrecipVisualSample::default();
        return;
    }
    let (visibility_factor, visibility_m) = volumetric_haze_from_fog(climate.mean_fog_density);
    *sample = WeatherPrecipVisualSample {
        rain: climate.mean_rain,
        snow: climate.mean_snow,
        fog: climate.mean_fog_density,
        chunk_count: n,
        visibility_factor,
        wind_mps: climate.mean_wind_mps,
        gust_mps: 0.0,
        cloud: 0.0,
        rain_mm_h: climate.mean_rain * crate::systems::weather::CHUNK_RAIN_FULL_SCALE_MM_H,
        visibility_m,
        local_ready: false,
    };
}

/// Chunk smoke GPU extract — **W7:** prefer [`AtmosphereFrame`] tiles; slab cutover optional;
/// ChunkSmokeField is fallback only when the frame is not built yet.
pub(crate) fn publish_sim_visual_extract(
    mut smoke_out: ResMut<SimChunkSmokeVisualExtract>,
    mut diag: ResMut<AtmosphereDiagnostics>,
    frame: Option<Res<AtmosphereFrame>>,
    ecs_retire: Option<Res<crate::substrate::EcsRetireState>>,
    substrate: Option<Res<crate::substrate::WorldSubstrateRegistry>>,
    smoke_q: Query<(&Chunk, &ChunkSmokeField)>,
) {
    smoke_out.instances.clear();

    let slab_smoke_extract = ecs_retire
        .as_ref()
        .is_some_and(|r| r.smoke_cutover_complete && !r.hybrid_smoke_authoritative)
        && substrate.is_some();

    if slab_smoke_extract {
        if let Some(reg) = substrate.as_ref() {
            for (key, state) in &reg.chunks.chunks {
                if !reg.chunks.is_resident(*key) {
                    continue;
                }
                let density = state
                    .contamination
                    .airborne
                    .first()
                    .copied()
                    .unwrap_or(0.0);
                if density <= SMOKE_EXTRACT_EPS {
                    continue;
                }
                let coord = IVec2::from(*key);
                smoke_out.instances.push(ChunkSmokeGpu {
                    chunk_xy: Vec4::new(coord.x as f32, coord.y as f32, 0.0, 0.0),
                    density_tox_vis: Vec4::new(
                        density,
                        state
                            .contamination
                            .airborne
                            .get(1)
                            .copied()
                            .unwrap_or(density * 0.2),
                        0.0,
                        0.0,
                    ),
                });
            }
        }
    } else if let Some(frame) = frame.as_deref().filter(|f| f.is_built()) {
        let origin = crate::systems::atmosphere::ATMO_GRID_ORIGIN;
        let size = frame.size();
        for (i, tile) in frame.tiles().iter().enumerate() {
            if tile.smoke <= SMOKE_EXTRACT_EPS && tile.toxicity <= SMOKE_EXTRACT_EPS {
                continue;
            }
            let x = (i as u32) % size.x;
            let y = (i as u32) / size.x;
            let chunk = IVec2::new(origin.x + x as i32, origin.y + y as i32);
            let fog = if tile.weather_valid {
                tile.weather.fog
            } else {
                0.0
            };
            let vis = (tile.smoke * 0.75 + fog * 0.35 + tile.toxicity * 0.25).clamp(0.0, 0.98);
            smoke_out.instances.push(ChunkSmokeGpu {
                chunk_xy: Vec4::new(chunk.x as f32, chunk.y as f32, 0.0, 0.0),
                density_tox_vis: Vec4::new(tile.smoke, tile.toxicity, vis, 0.0),
            });
        }
    } else {
        for (chunk, smoke) in &smoke_q {
            if smoke.density <= SMOKE_EXTRACT_EPS && smoke.toxicity <= SMOKE_EXTRACT_EPS {
                continue;
            }
            smoke_out.instances.push(ChunkSmokeGpu {
                chunk_xy: Vec4::new(
                    chunk.coord.x as f32,
                    chunk.coord.y as f32,
                    0.0,
                    0.0,
                ),
                density_tox_vis: Vec4::new(
                    smoke.density,
                    smoke.toxicity,
                    smoke.visibility_penalty,
                    0.0,
                ),
            });
        }
    }

    diag.visual_extract_runs = diag.visual_extract_runs.wrapping_add(1);
    diag.last_smoke_extract_count = smoke_out.instances.len();
}

pub fn visual_extract_systems(app: &mut App) {
    app.init_resource::<SimChunkSmokeVisualExtract>()
        .init_resource::<ClimateVisualAggregate>()
        .init_resource::<WeatherPrecipVisualSample>();
}

#[cfg(test)]
mod tests {
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    use crate::gui::InputBindings;
    use crate::systems::atmosphere::AtmosphereDiagnostics;
    use crate::systems::atmosphere::AtmospherePlugin;
    use crate::systems::sim_control::SimControlPlugin;

    #[test]
    fn visual_extract_runs_each_frame() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);
        app.init_resource::<InputBindings>();
        app.add_plugins(SimControlPlugin);
        app.add_plugins(AtmospherePlugin);

        app.update();
        let a = app.world().resource::<AtmosphereDiagnostics>().visual_extract_runs;
        app.update();
        let b = app.world().resource::<AtmosphereDiagnostics>().visual_extract_runs;
        assert!(b > a);
    }

    #[test]
    fn climate_aggregate_weather_channels_equal_frame_means() {
        use crate::render::ClimateVisualAggregate;
        use crate::systems::atmosphere::AtmosphereFrame;
        use crate::systems::weather::{WeatherField, WeatherTuning};
        use crate::terrain::world_scale_contract::{CHUNK_TILES_SIM, TILE_METRES_GAMEPLAY};

        let extent = Vec2::splat(8.0 * CHUNK_TILES_SIM as f32 * TILE_METRES_GAMEPLAY);
        let field = WeatherField::new(13, extent, &WeatherTuning::default(), None, Default::default(), 0);
        let mut stack = crate::substrate::atmosphere::AtmosphereClipmapStack::default();
        stack.bump_revision();
        let mut frame = AtmosphereFrame::default();
        frame.rebuild(Some(&field), &stack, 2_000_000, &WeatherTuning::default());
        let m = *frame.weather_means();
        assert!(m.tiles > 0);

        let mut app = App::new();
        app.insert_resource(frame)
            .init_resource::<ClimateVisualAggregate>()
            .add_systems(Update, super::publish_climate_visual_aggregate);
        app.update();
        let c = app.world().resource::<ClimateVisualAggregate>();
        assert_eq!(c.weather_chunk_count, m.tiles);
        assert_eq!(c.mean_rain, m.rain01);
        assert_eq!(c.mean_snow, m.snow01);
        assert_eq!(c.mean_fog_density, m.fog);
        assert_eq!(c.mean_wind_speed, m.wind_speed01);
        assert_eq!(c.mean_wind_mps, m.wind_mps);
        assert_eq!(c.mean_lightning_risk, m.lightning);
    }

    /// W2/W7 gate: the aggregate no longer scans `ChunkWeather` or `ChunkEcology`.
    #[test]
    fn climate_publisher_does_not_scan_chunk_weather_or_ecology() {
        let src = include_str!("visual_extract.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap();
        assert!(!prod.contains("Query<&ChunkWeather>"));
        assert!(!prod.contains("Query<&ChunkEcology>"));
        assert!(!prod.contains("Res<WeatherField>"));
    }

    #[test]
    fn lgt04_volumetric_haze_denser_fog_lowers_visibility() {
        let (clear_f, clear_m) = super::volumetric_haze_from_fog(0.0);
        let (fog_f, fog_m) = super::volumetric_haze_from_fog(0.8);
        assert!(clear_f > fog_f);
        assert!(clear_m > fog_m);
        assert!((clear_m - super::VOLUMETRIC_BASE_VISIBILITY_M).abs() < 1e-3);
    }
}
