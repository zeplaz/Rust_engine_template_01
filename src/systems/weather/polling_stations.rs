//! SENSE-02 — per-chunk weather polling stations (sole field sampler for gameplay readers).
//!
//! [`WeatherPollSiteBook`] lists chunk keys for this tick (written by [`crate::combat::gather_weather_poll_sites`]).
//! [`sync_weather_polling_stations`] is the only system that calls [`WeatherField::sample`] for those sites.
//! Consumers read [`WeatherPollingStations`] — they must not sample the field or build a private cache.

use std::collections::{HashMap, HashSet};

use bevy::math::{IVec2, UVec2, Vec2};
use bevy::prelude::*;

use crate::systems::sim_control::SimTimeMicros;
use crate::terrain::generation::chunk_world_center;
use crate::terrain::world_scale_contract::{CHUNK_TILES_SIM, TILE_METRES_GAMEPLAY};

use super::air_ops::tile_to_smoke_chunk;
use super::field::{WeatherField, WeatherSample};

/// Chunks that need a weather station sample this sim tick (gathered before the weather pass).
#[derive(Resource, Debug, Clone, Default)]
pub struct WeatherPollSiteBook {
    pub chunks: HashSet<IVec2>,
}

/// Cached per-chunk samples for the current tick (shape mirrors centre fields on [`super::AirOpsWeatherStatus`]).
#[derive(Resource, Debug, Clone)]
pub struct WeatherPollingStations {
    pub ready: bool,
    pub centre: WeatherSample,
    pub by_chunk: HashMap<IVec2, WeatherSample>,
    /// Unique chunk keys materialised in [`Self::by_chunk`] this tick.
    pub chunks_polled: u64,
    /// Count of [`WeatherField::sample`] calls issued while building this frame (evidence / tests).
    pub field_samples: u64,
}

impl Default for WeatherPollingStations {
    fn default() -> Self {
        Self {
            ready: false,
            centre: WeatherSample::default(),
            by_chunk: HashMap::new(),
            chunks_polled: 0,
            field_samples: 0,
        }
    }
}

/// Map a gameplay tile to the weather polling chunk (same grid as smoke / transport chunking).
#[inline]
#[must_use]
pub fn tile_to_weather_chunk(tile: IVec2) -> IVec2 {
    tile_to_smoke_chunk(tile, UVec2::splat(CHUNK_TILES_SIM))
}

#[inline]
fn chunk_centre_pos_m(chunk: IVec2) -> Vec2 {
    chunk_world_center(chunk, UVec2::splat(CHUNK_TILES_SIM)) * TILE_METRES_GAMEPLAY
}

impl WeatherPollingStations {
    /// Weather at a gameplay tile via its chunk station (chunk-centre sample).
    #[must_use]
    pub fn sample_at_tile(&self, tile: IVec2) -> WeatherSample {
        if !self.ready {
            return WeatherSample::default();
        }
        let chunk = tile_to_weather_chunk(tile);
        self.by_chunk.get(&chunk).copied().unwrap_or(self.centre)
    }

    /// Weather at a world position in metres (snaps to the owning chunk station).
    #[must_use]
    pub fn sample_at_pos_m(&self, pos_m: Vec2) -> WeatherSample {
        let tile = IVec2::new(
            (pos_m.x / TILE_METRES_GAMEPLAY).floor() as i32,
            (pos_m.y / TILE_METRES_GAMEPLAY).floor() as i32,
        );
        self.sample_at_tile(tile)
    }
}

/// Sole writer of [`WeatherPollingStations`] from [`WeatherField`] + [`WeatherPollSiteBook`].
pub fn sync_weather_polling_stations(
    field: Option<Res<WeatherField>>,
    sim_time: Option<Res<SimTimeMicros>>,
    sites: Res<WeatherPollSiteBook>,
    mut stations: ResMut<WeatherPollingStations>,
) {
    stations.by_chunk.clear();
    stations.chunks_polled = 0;
    stations.field_samples = 0;

    let Some(field) = field.filter(|f| f.is_ready()) else {
        stations.ready = false;
        stations.centre = WeatherSample::default();
        return;
    };

    let t = sim_time
        .as_deref()
        .copied()
        .unwrap_or(SimTimeMicros(field.prev_micros));

    let centre_m = field.extent_m * 0.5;
    stations.centre = field.sample(centre_m, t);
    stations.field_samples = 1;
    stations.ready = true;

    let centre_tile = IVec2::new(
        (centre_m.x / TILE_METRES_GAMEPLAY).floor() as i32,
        (centre_m.y / TILE_METRES_GAMEPLAY).floor() as i32,
    );
    let centre_chunk = tile_to_weather_chunk(centre_tile);

    let mut unique: HashSet<IVec2> = sites.chunks.iter().copied().collect();
    unique.insert(centre_chunk);

    for chunk in unique {
        let sample = if chunk == centre_chunk {
            stations.centre
        } else {
            stations.field_samples += 1;
            field.sample(chunk_centre_pos_m(chunk), t)
        };
        stations.by_chunk.insert(chunk, sample);
        stations.chunks_polled += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{App, MinimalPlugins, Update};

    use crate::systems::sim_control::SimTimeMicros;
    use crate::systems::weather::tuning::WeatherTuning;

    fn ready_field() -> WeatherField {
        let tuning = WeatherTuning::default();
        WeatherField::new(
            42,
            Vec2::new(512.0 * TILE_METRES_GAMEPLAY, 512.0 * TILE_METRES_GAMEPLAY),
            &tuning,
            None,
            Default::default(),
            0,
        )
    }

    #[test]
    fn sync_dedupes_chunks_and_reuses_centre() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<WeatherPollSiteBook>();
        app.init_resource::<WeatherPollingStations>();
        app.insert_resource(ready_field());
        app.insert_resource(SimTimeMicros(0));

        {
            let mut sites = app.world_mut().resource_mut::<WeatherPollSiteBook>();
            let c = tile_to_weather_chunk(IVec2::new(3, 7));
            sites.chunks.insert(c);
            sites.chunks.insert(c);
            sites.chunks.insert(tile_to_weather_chunk(IVec2::new(4, 7)));
        }

        app.add_systems(Update, sync_weather_polling_stations);
        app.update();

        let stations = app.world().resource::<WeatherPollingStations>();
        assert!(stations.ready);
        assert!(stations.field_samples <= stations.chunks_polled);
        assert!(stations.chunks_polled >= 2);
    }

    #[test]
    fn sample_at_tile_uses_chunk_station() {
        let mut stations = WeatherPollingStations::default();
        stations.ready = true;
        let chunk = IVec2::new(0, 0);
        let mut sample = WeatherSample::default();
        sample.wind_mps = Vec2::new(7.0, 0.0);
        stations.by_chunk.insert(chunk, sample);

        let read = stations.sample_at_tile(IVec2::new(1, 2));
        assert!((read.wind_mps.x - 7.0).abs() < 1e-4);
    }
}