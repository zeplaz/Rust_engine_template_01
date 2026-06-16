//! Publishes sim state into [`crate::render::sim_visual_extract`] resources (`base_gui_next.md` Stage 2).

use bevy::prelude::*;

use crate::render::{ChunkSmokeGpu, ClimateVisualAggregate, SimChunkSmokeVisualExtract};
use crate::systems::ecology::{ChunkEcology, LandscapeProgramOnChunk};
use crate::systems::fire::ChunkSmokeField;
use crate::systems::weather::{ChunkWeather, WeatherPrecipVisualSample};
use crate::terrain::generation::Chunk;

use super::diagnostics::AtmosphereDiagnostics;

const SMOKE_EXTRACT_EPS: f32 = 1e-4;

/// Single scan of chunk weather + ecology → [`ClimateVisualAggregate`] for GPU / precip overlay (no duplicate queries).
pub(crate) fn publish_climate_visual_aggregate(
    wx: Query<&ChunkWeather>,
    eco: Query<&ChunkEcology>,
    programs: Query<&LandscapeProgramOnChunk>,
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

    let np = programs.iter().count() as u32;
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
        ecology_chunk_count: if np > 0 { np } else { ne },
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

/// Chunk smoke only — fire truth rows live in [`crate::render::extraction::FireVisualFrame`]; GPU field reads
/// [`crate::render::extraction::RenderProjectionGraph`] after [`crate::render::extraction::run_render_projection_graph`].
pub(crate) fn publish_sim_visual_extract(
    mut smoke_out: ResMut<SimChunkSmokeVisualExtract>,
    mut diag: ResMut<AtmosphereDiagnostics>,
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
}
