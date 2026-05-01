//! Feature-gated bridge from materialized chunks to `bevy_ecs_tilemap` (U6 / U7 multi-layer).
//!
//! **Feature:** `bevy_tilemap_adapter` pulls `bevy_ecs_tilemap` with **default-features = false**.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::gui::editor::world_gen_ui::{PreviewMode, WorldGenUiState};
use crate::terrain::biome::TerrainClass;
use crate::terrain::generation::ChunkCellMatrix;
use crate::terrain::material::{MaterialId, MaterializedChunk, MaterializedResources};

/// Three tilemaps per chunk: terrain (z=0), overlay preview (z=10), resources (z=20).
#[derive(Component, Clone, Copy, Debug)]
pub struct ChunkTilemaps {
    pub terrain: Entity,
    pub overlay: Entity,
    pub resource: Entity,
}

#[derive(Resource, Clone, Debug)]
pub struct TilemapLayerVisibility {
    pub terrain: bool,
    pub overlay: bool,
    pub resources: bool,
}

impl Default for TilemapLayerVisibility {
    fn default() -> Self {
        Self {
            terrain: true,
            overlay: true,
            resources: true,
        }
    }
}

fn spawn_chunk_tilemaps(
    mut commands: Commands,
    q: Query<(Entity, &MaterializedChunk), Without<ChunkTilemaps>>,
) {
    for (parent, mat) in q.iter() {
        let map_size = TilemapSize {
            x: mat.size.x,
            y: mat.size.y,
        };

        let mut make_map = |cmds: &mut Commands, z: f32| -> Entity {
            let mut storage = TileStorage::empty(map_size);
            let tilemap = cmds.spawn_empty().id();
            for y in 0..map_size.y {
                for x in 0..map_size.x {
                    let pos = TilePos { x, y };
                    let tile = cmds
                        .spawn(TileBundle {
                            position: pos,
                            tilemap_id: TilemapId(tilemap),
                            ..Default::default()
                        })
                        .id();
                    storage.set(&pos, tile);
                }
            }
            cmds.entity(tilemap)
                .insert((storage, Transform::from_xyz(0.0, 0.0, z)));
            tilemap
        };

        let terrain_e = make_map(&mut commands, 0.0);
        let overlay_e = make_map(&mut commands, 10.0);
        let resource_e = make_map(&mut commands, 20.0);

        commands.entity(parent).insert(ChunkTilemaps {
            terrain: terrain_e,
            overlay: overlay_e,
            resource: resource_e,
        });
    }
}

fn sync_terrain_layer(
    mut commands: Commands,
    chunks: Query<
        (&MaterializedChunk, &ChunkTilemaps),
        Or<(Changed<MaterializedChunk>, Changed<ChunkTilemaps>)>,
    >,
    storages: Query<&TileStorage>,
) {
    for (mat, link) in chunks.iter() {
        sync_material_indices_inner(&mut commands, mat, link.terrain, &storages);
    }
}

fn sync_material_indices_inner(
    commands: &mut Commands,
    mat: &MaterializedChunk,
    tilemap: Entity,
    storages: &Query<&TileStorage>,
) {
    let Ok(storage) = storages.get(tilemap) else {
        return;
    };
    if storage.size.x != mat.size.x || storage.size.y != mat.size.y {
        return;
    }
    let expected = (mat.size.x * mat.size.y) as usize;
    if mat.materials.len() != expected {
        return;
    }
    for y in 0..mat.size.y {
        for x in 0..mat.size.x {
            let i = (y * mat.size.x + x) as usize;
            let MaterialId(id) = mat.materials[i];
            let pos = TilePos { x, y };
            let Some(tile_e) = storage.get(&pos) else {
                continue;
            };
            commands
                .entity(tile_e)
                .insert(TileTextureIndex(id as u32));
        }
    }
}

fn terrain_class_discriminant(c: TerrainClass) -> u32 {
    match c {
        TerrainClass::DeepWater => 0,
        TerrainClass::ShallowWater => 1,
        TerrainClass::Beach => 2,
        TerrainClass::Desert => 3,
        TerrainClass::Grassland => 4,
        TerrainClass::Forest => 5,
        TerrainClass::DenseForest => 6,
        TerrainClass::Mountain => 7,
        TerrainClass::SnowCappedMountain => 8,
        TerrainClass::Tundra => 9,
        TerrainClass::Swamp => 10,
        TerrainClass::Cliff => 11,
        TerrainClass::Concrete => 12,
        TerrainClass::Dirt => 13,
        TerrainClass::Snow => 14,
        TerrainClass::Stone => 15,
    }
}

fn overlay_index_for_cell(matrix: &ChunkCellMatrix, x: u32, y: u32, mode: PreviewMode) -> u32 {
    let i = matrix.idx(x, y);
    match mode {
        PreviewMode::None => 0,
        PreviewMode::Height => (matrix.elevation[i].clamp(0.0, 1.0) * 255.0) as u32,
        PreviewMode::Moisture => (matrix.moisture[i].clamp(0.0, 1.0) * 255.0) as u32,
        PreviewMode::Temperature => (matrix.temperature[i].clamp(0.0, 1.0) * 255.0) as u32,
        PreviewMode::Biome => terrain_class_discriminant(matrix.family[i]),
        PreviewMode::Regions => 0,
        PreviewMode::Tag(tag) => {
            if matrix.tags[i].contains(tag) {
                240
            } else {
                0
            }
        }
    }
}

fn sync_overlay_layer_changed(
    mut commands: Commands,
    chunks: Query<
        (&ChunkCellMatrix, &ChunkTilemaps),
        Or<(Changed<ChunkCellMatrix>, Changed<ChunkTilemaps>)>,
    >,
    ui: Res<WorldGenUiState>,
    storages: Query<&TileStorage>,
) {
    let mode = ui.preview_mode;
    for (matrix, maps) in chunks.iter() {
        apply_overlay_indices(&mut commands, matrix, maps.overlay, mode, &storages);
    }
}

fn sync_overlay_on_preview_change(
    mut commands: Commands,
    chunks: Query<(&ChunkCellMatrix, &ChunkTilemaps)>,
    ui: Res<WorldGenUiState>,
    storages: Query<&TileStorage>,
) {
    if !ui.is_changed() {
        return;
    }
    let mode = ui.preview_mode;
    for (matrix, maps) in chunks.iter() {
        apply_overlay_indices(&mut commands, matrix, maps.overlay, mode, &storages);
    }
}

fn apply_overlay_indices(
    commands: &mut Commands,
    matrix: &ChunkCellMatrix,
    tilemap: Entity,
    mode: PreviewMode,
    storages: &Query<&TileStorage>,
) {
    let Ok(storage) = storages.get(tilemap) else {
        return;
    };
    if storage.size.x != matrix.size.x || storage.size.y != matrix.size.y {
        return;
    }
    for y in 0..matrix.size.y {
        for x in 0..matrix.size.x {
            let idx = overlay_index_for_cell(matrix, x, y, mode);
            let pos = TilePos { x, y };
            let Some(tile_e) = storage.get(&pos) else {
                continue;
            };
            commands.entity(tile_e).insert(TileTextureIndex(idx));
        }
    }
}

fn sync_resource_layer(
    mut commands: Commands,
    chunks: Query<
        (&MaterializedResources, &ChunkTilemaps),
        Or<(Changed<MaterializedResources>, Changed<ChunkTilemaps>)>,
    >,
    storages: Query<&TileStorage>,
) {
    for (res, maps) in chunks.iter() {
        let Ok(storage) = storages.get(maps.resource) else {
            continue;
        };
        let size = storage.size;
        let expected = (size.x * size.y) as usize;
        if res.ids.len() != expected {
            continue;
        }
        for y in 0..size.y {
            for x in 0..size.x {
                let i = (y * size.x + x) as usize;
                let MaterialId(id) = res.ids[i];
                let pos = TilePos { x, y };
                let Some(tile_e) = storage.get(&pos) else {
                    continue;
                };
                commands
                    .entity(tile_e)
                    .insert(TileTextureIndex(id as u32));
            }
        }
    }
}

fn apply_tilemap_layer_visibility(
    vis: Res<TilemapLayerVisibility>,
    maps: Query<&ChunkTilemaps>,
    mut qv: Query<&mut Visibility>,
) {
    if !vis.is_changed() {
        return;
    }
    for m in maps.iter() {
        for (e, on) in [
            (m.terrain, vis.terrain),
            (m.overlay, vis.overlay),
            (m.resource, vis.resources),
        ] {
            if let Ok(mut v) = qv.get_mut(e) {
                *v = if on {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

pub struct TilemapAdapterPlugin;

impl Plugin for TilemapAdapterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .init_resource::<TilemapLayerVisibility>()
            .add_systems(
                Update,
                (
                    spawn_chunk_tilemaps,
                    sync_terrain_layer,
                    sync_overlay_layer_changed,
                    sync_overlay_on_preview_change,
                    sync_resource_layer,
                    apply_tilemap_layer_visibility,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::MinimalPlugins;

    fn setup_tilemaps_entity(mut commands: Commands) {
        let map_size = TilemapSize { x: 2, y: 1 };
        let mut tile_storage = TileStorage::empty(map_size);
        let tilemap_entity = commands.spawn_empty().id();

        for y in 0..map_size.y {
            for x in 0..map_size.x {
                let tile_pos = TilePos { x, y };
                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        ..Default::default()
                    })
                    .id();
                tile_storage.set(&tile_pos, tile_entity);
            }
        }
        commands
            .entity(tilemap_entity)
            .insert((tile_storage, Transform::from_xyz(0.0, 0.0, 0.0)));

        commands.spawn((
            ChunkCellMatrix::new(UVec2::new(2, 1)),
            MaterializedChunk {
                size: UVec2::new(2, 1),
                materials: vec![MaterialId(7), MaterialId(9)],
            },
            MaterializedResources {
                ids: vec![MaterialId(7), MaterialId(9)],
            },
        ));
    }

    #[test]
    fn tilemap_adapter_writes_terrain_layer() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(TilemapAdapterPlugin)
            .init_resource::<WorldGenUiState>()
            .add_systems(Startup, setup_tilemaps_entity);
        app.update();
        app.update();

        let world = app.world_mut();
        let chunk_entity = world
          .query_filtered::<Entity, (With<MaterializedChunk>, With<ChunkTilemaps>)>()
          .iter(world)
          .next()
          .expect("chunk with tilemaps");
        let maps = world.entity(chunk_entity).get::<ChunkTilemaps>().unwrap();
        let storage = world.entity(maps.terrain).get::<TileStorage>().unwrap();
        let e00 = storage.get(&TilePos { x: 0, y: 0 }).unwrap();
        let e10 = storage.get(&TilePos { x: 1, y: 0 }).unwrap();
        assert_eq!(
            *world.entity(e00).get::<TileTextureIndex>().unwrap(),
            TileTextureIndex(7)
        );
        assert_eq!(
            *world.entity(e10).get::<TileTextureIndex>().unwrap(),
            TileTextureIndex(9)
        );
    }

    #[test]
    fn multi_layer_spawn_three_tilemaps() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(TilemapAdapterPlugin)
            .init_resource::<WorldGenUiState>()
            .add_systems(Startup, setup_tilemaps_entity);
        app.update();
        app.update();

        let world = app.world_mut();
        let chunk_entity = world
            .query_filtered::<Entity, (With<MaterializedChunk>, With<ChunkTilemaps>)>()
            .iter(world)
            .next()
            .expect("chunk with tilemaps");
        let maps = world.entity(chunk_entity).get::<ChunkTilemaps>().unwrap();

        for (e, z) in [
            (maps.terrain, 0.0),
            (maps.overlay, 10.0),
            (maps.resource, 20.0),
        ] {
            let t = world.entity(e).get::<Transform>().unwrap();
            assert!((t.translation.z - z).abs() < 0.001);
        }
    }
}
