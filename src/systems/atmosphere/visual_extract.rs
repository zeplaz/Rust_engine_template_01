//! Publishes sim state into [`crate::render::sim_visual_extract`] resources (`base_gui_next.md` Stage 2).

use bevy::prelude::*;

use crate::render::{ChunkSmokeGpu, ClimateVisualAggregate, SimChunkSmokeVisualExtract};
use crate::systems::ecology::ChunkEcology;
use crate::systems::fire::ChunkSmokeField;
use crate::systems::weather::{ChunkWeather, WeatherPrecipVisualSample};
use crate::terrain::generation::Chunk;

use super::diagnostics::AtmosphereDiagnostics;

const SMOKE_EXTRACT_EPS: f32 = 1e-4;

/// Single scan of chunk weather + ecology → [`ClimateVisualAggregate`] for GPU / precip overlay (no duplicate queries).
pub(crate) fn publish_climate_visual_aggregate(
    wx: Query<&ChunkWeather>,
    eco: Query<&ChunkEcology>,
    mut out: ResMut<ClimateVisualAggregate>,
) {
    let mut nw = 0u32;
    let mut r = 0f32;
    let mut s = 0f32;
    let mut fg = 0f32;
    let mut wind = 0f32;
    let mut li = 0f32;
    for w in &wx {
        nw += 1;
        r += w.rain_intensity;
        s += w.snow_depth;
        fg += w.fog_density;
        wind += w.wind_speed;
        li += w.lightning_risk;
    }

    let mut ne = 0u32;
    let mut bio = 0f32;
    let mut frisk = 0f32;
    for e in &eco {
        ne += 1;
        bio += e.biomass;
        frisk += e.fire_risk;
    }

    let nf_w = nw.max(1) as f32;
    let nf_e = ne.max(1) as f32;
    *out = ClimateVisualAggregate {
        mean_rain: r / nf_w,
        mean_snow: s / nf_w,
        mean_fog_density: fg / nf_w,
        mean_wind_speed: wind / nf_w,
        mean_lightning_risk: li / nf_w,
        mean_biomass: bio / nf_e,
        mean_fire_risk: frisk / nf_e,
        weather_chunk_count: nw,
        ecology_chunk_count: ne,
    };
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
    *sample = WeatherPrecipVisualSample {
        rain: climate.mean_rain,
        snow: climate.mean_snow,
        fog: climate.mean_fog_density,
        chunk_count: n,
    };
}

/// Chunk smoke only — fire rows come from [`crate::render::extraction::FireVisualFramePlugin`]
/// ([`crate::render::FireVisualFrame`]) so we do not scan [`FireEmitter`] twice.
pub(crate) fn publish_sim_visual_extract(
    mut smoke_out: ResMut<SimChunkSmokeVisualExtract>,
    mut diag: ResMut<AtmosphereDiagnostics>,
    smoke_q: Query<(&Chunk, &ChunkSmokeField)>,
) {
    smoke_out.instances.clear();

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
}
