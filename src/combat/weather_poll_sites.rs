//! Gathers chunk keys for [`WeatherPollSiteBook`] before the weather pass (SENSE-02).

use bevy::math::{IVec2, Vec2};
use bevy::prelude::*;

use crate::entities::production::power::capabilities::VariableRenewable;
use crate::systems::weather::{tile_to_weather_chunk, WeatherPollSiteBook};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

use super::drone_missions::MissionBook;
use super::formation::Formation;

/// Sole writer of [`WeatherPollSiteBook`] — no field sampling here.
pub fn gather_weather_poll_sites(
    formations: Query<&Formation>,
    mission: Res<MissionBook>,
    plants: Query<&Transform, With<VariableRenewable>>,
    mut sites: ResMut<WeatherPollSiteBook>,
) {
    sites.chunks.clear();

    for f in &formations {
        sites.chunks.insert(tile_to_weather_chunk(f.tile));
    }

    for ao in mission.drone_aos.values() {
        sites.chunks.insert(tile_to_weather_chunk(ao.centroid_tile()));
    }
    for z in mission.air_op_zones.values() {
        sites.chunks.insert(tile_to_weather_chunk(z.centroid_tile()));
    }

    for tr in &plants {
        let pos = Vec2::new(tr.translation.x, tr.translation.z);
        let tile = IVec2::new(
            (pos.x / TILE_METRES_GAMEPLAY).floor() as i32,
            (pos.y / TILE_METRES_GAMEPLAY).floor() as i32,
        );
        sites.chunks.insert(tile_to_weather_chunk(tile));
    }
}