use crate::terrain::tiles::Tile;
use crate::terrain::world::GeoRegion;
use bevy::prelude::*;

#[derive(Component)]
pub struct GeoRegionComponent(pub GeoRegion);

pub struct GeoPlugin;

impl Plugin for GeoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_tiles, update_geo_regions));
    }
}

fn update_tiles(mut _query: Query<&mut Tile>) {}

fn update_geo_regions(mut _query: Query<&mut GeoRegionComponent>) {}
