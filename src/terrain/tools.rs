use lyon::path::{Path, PathBuilder};
use bevy::math::Vec2;
use world::World;
use tiles::Tile;
use super idgen::EntityID;


fn get_agent_owned_boundary(agent_id: EntityId, world: &World) -> Path {
    let agent_tiles = world.get_agent_owned_tiles(agent_id);
    let outline_points = get_agent_owned_outline_points(agent_tiles);

    let mut builder = Path::builder();
    if let Some(first_point) = outline_points.get(0) {
        builder.move_to((*first_point).into());
    }

    for point in outline_points.iter().skip(1) {
        builder.line_to((*point).into());
    }

    builder.close().build()
}

fn get_agent_owned_outline_points(agent_tiles: &[Tile]) -> Vec<Vec2> {
    // This function should return a sorted list of points
    // representing the outline of the agent's owned area.
    // You will need to implement this based on your game data structures.
}
