use bevy::prelude::*;
use entities::entity::EntityInfo;
use linfa::Distance;

#[derive(Bundle, Debug, Clone)]

pub struct Tile {
    entityinfo: EntityInfo,
    //entity stuff?
    entities: Vec<Entity>,

    building_id: Option<EntityId>,
    vehicles_id: Vec<EntityId>,

    //neighbor stuff
    neighbor_ids: [Option<EntityId>; 8],
    neighbor_gradients: [Option<f32>; 8],

    //model tags
    pub safety_rating: f32,

    //spatial hash
    spatial_hash: SpatialHash,
}

impl Tile {
    fn notify_neighbors_height_change(&self, geo_region: &mut GeoRegion) {
        for (index, neighbor_id_option) in self.neighbor_ids.iter().enumerate() {
            if let Some(neighbor_id) = neighbor_id_option {
                if let Some(neighbor) = geo_region.get_tile_by_id_mut(*neighbor_id) {
                    let opposite_index = match (delta.0 as i32, delta.1 as i32) {
                        (-1, 0) => 0,
                        (-1, 1) => 1,
                        (0, 1) => 2,
                        (1, 1) => 3,
                        (1, 0) => 4,
                        (1, -1) => 5,
                        (0, -1) => 6,
                        (-1, -1) => 7,
                        _ => continue,
                    };
                    let new_gradient = calculate_gradient(
                        self,
                        self.entityinfo.position,
                        neighbor,
                        neighbor.entityinfo.position,
                    );
                    neighbor.neighbor_gradients[opposite_index] = Some(new_gradient);
                }
            }
        }
    }
}
impl Distance for Tile {}
fn calculate_gradient(
    tile1: &Tile,
    pos1: (usize, usize),
    tile2: &Tile,
    pos2: (usize, usize),
) -> f32 {
    let height_diff = tile2.elevation - tile1.elevation;
    let distance = (((pos2.0 as f32) - (pos1.0 as f32)).powi(2)
        + ((pos2.1 as f32) - (pos1.1 as f32)).powi(2))
    .sqrt();
    height_diff / distance
}
