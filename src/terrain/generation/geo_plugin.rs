use crate::terrain::tiles::Tile;
use crate::terrain::world::GeoRegion;
use bevy::prelude::*;

#[derive(Component)]
pub struct TileComponent(Tile);

#[derive(Component)]
pub struct GeoRegionComponent(GeoRegion);

pub struct GeoPlugin;

impl Plugin for GeoPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_systems((update_tiles,update_geo_regions,)
    }
}

fn update_tiles(
    mut query: Query<&mut TileComponent>,
    // Add any other queries or resources you need
) {
    for mut tile_component in query.iter_mut() {
        // Update the tile as needed
    }
}

fn update_geo_regions(
    mut query: Query<&mut GeoRegionComponent>,
    // Add any other queries or resources you need
) {
    for mut geo_region_component in query.iter_mut() {
        // Update the geo region as needed
    }
}

fn update_voronoi_regions(
    mut query: Query<&mut GeoRegionComponent>,
)
