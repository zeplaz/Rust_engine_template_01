//! Feature-gated bridge from materialized chunks to `bevy_ecs_tilemap` (U6 / U7 multi-layer).
//!
//! **Feature:** `bevy_tilemap_adapter` pulls `bevy_ecs_tilemap` with **default-features = false**.

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::gui::editor::world_gen_ui::{PreviewMode, WorldGenUiState};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::family::TerrainFamilyId;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{Chunk, ChunkCellMatrix, ChunkDerivedMetrics};
use crate::terrain::material::{
    MaterialId, MaterialRegistry, MaterializedChunk, MaterializedResources, TagRegistry, TagSet,
};
use crate::terrain::mobility::{evaluate_tile, MobilityProfile, MobilityProfileRegistry, MovementHint};
use crate::terrain::{ChunkCellKey, DynamicTerrainOverlay};

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

fn terrain_family_overlay_index(id: TerrainFamilyId) -> u32 {
    id.0 as u32
}

fn movement_hint_tile_index(hint: &MovementHint) -> u32 {
    if hint.blocked {
        255
    } else {
        let c = ((hint.cost_mul.clamp(1.0, 5.0) - 1.0) / 4.0 * 200.0
            + hint.stuck_risk.clamp(0.0, 1.0) * 54.0)
            .min(254.0);
        c as u32
    }
}

fn overlay_index_for_cell(
    matrix: &ChunkCellMatrix,
    x: u32,
    y: u32,
    mode: PreviewMode,
    tag_pool: &TagSet,
    derived_slope: Option<f32>,
    mobility_hint: Option<&MovementHint>,
) -> u32 {
    let i = matrix.idx(x, y);
    match mode {
        PreviewMode::None => 0,
        PreviewMode::Height => (matrix.elevation[i].clamp(0.0, 1.0) * 255.0) as u32,
        PreviewMode::Moisture => (matrix.moisture[i].clamp(0.0, 1.0) * 255.0) as u32,
        PreviewMode::Temperature => (matrix.temperature[i].clamp(0.0, 1.0) * 255.0) as u32,
        PreviewMode::Biome => terrain_family_overlay_index(matrix.family[i]),
        PreviewMode::Regions => 0,
        PreviewMode::Tag => {
            if matrix.tags[i].intersects(tag_pool) {
                240
            } else {
                0
            }
        }
        PreviewMode::DerivedSlope => {
            let s = derived_slope.unwrap_or(0.0);
            (s.clamp(0.0, 1.0) * 255.0) as u32
        }
        PreviewMode::Mobility => mobility_hint.map(movement_hint_tile_index).unwrap_or(0),
    }
}

fn mobility_overlay_context<'a>(
    handles: Option<&'a TerrainRegistriesHandles>,
    tag_assets: Option<&'a Assets<TagRegistry>>,
    mobility_assets: Option<&'a Assets<MobilityProfileRegistry>>,
    material_assets: Option<&'a Assets<MaterialRegistry>>,
    ui: &WorldGenUiState,
) -> Option<(
    &'a TagRegistry,
    &'a MobilityProfile,
    Option<&'a MaterialRegistry>,
)> {
    if ui.preview_mode != PreviewMode::Mobility {
        return None;
    }
    let handles = handles?;
    let tag_reg = tag_assets?.get(&handles.tag_registry)?;
    let mob_reg = mobility_assets?.get(&handles.mobility_profiles)?;
    if mob_reg.profiles.is_empty() {
        return None;
    }
    let idx = ui
        .mobility_profile_index
        .min(mob_reg.profiles.len() - 1);
    let mat_reg = material_assets.and_then(|a| a.get(&handles.material_registry));
    Some((tag_reg, &mob_reg.profiles[idx], mat_reg))
}

fn sync_overlay_layer_changed(
    mut commands: Commands,
    chunks: Query<
        (
            &Chunk,
            &ChunkCellMatrix,
            Option<&ChunkDerivedMetrics>,
            &MaterializedChunk,
            &ChunkTilemaps,
        ),
        Or<(
            Changed<ChunkCellMatrix>,
            Changed<ChunkTilemaps>,
            Changed<ChunkDerivedMetrics>,
            Changed<MaterializedChunk>,
        )>,
    >,
    ui: Res<WorldGenUiState>,
    params: Res<WorldGenParams>,
    storages: Query<&TileStorage>,
    overlay: Res<DynamicTerrainOverlay>,
    handles: Option<Res<TerrainRegistriesHandles>>,
    tag_assets: Option<Res<Assets<TagRegistry>>>,
    mobility_assets: Option<Res<Assets<MobilityProfileRegistry>>>,
    material_assets: Option<Res<Assets<MaterialRegistry>>>,
) {
    let mode = ui.preview_mode;
    let pool = params.tag_pool;
    let mob_ctx = mobility_overlay_context(
        handles.as_ref().map(|r| &**r),
        tag_assets.as_ref().map(|r| &**r),
        mobility_assets.as_ref().map(|r| &**r),
        material_assets.as_ref().map(|r| &**r),
        &ui,
    );
    for (chunk, matrix, derived, mat_chunk, maps) in chunks.iter() {
        apply_overlay_indices(
            &mut commands,
            chunk,
            &overlay,
            matrix,
            derived,
            mat_chunk,
            maps.overlay,
            mode,
            &pool,
            &storages,
            mob_ctx,
        );
    }
}

fn sync_overlay_on_preview_change(
    mut commands: Commands,
    chunks: Query<(
        &Chunk,
        &ChunkCellMatrix,
        Option<&ChunkDerivedMetrics>,
        &MaterializedChunk,
        &ChunkTilemaps,
    )>,
    ui: Res<WorldGenUiState>,
    params: Res<WorldGenParams>,
    storages: Query<&TileStorage>,
    overlay: Res<DynamicTerrainOverlay>,
    handles: Option<Res<TerrainRegistriesHandles>>,
    tag_assets: Option<Res<Assets<TagRegistry>>>,
    mobility_assets: Option<Res<Assets<MobilityProfileRegistry>>>,
    material_assets: Option<Res<Assets<MaterialRegistry>>>,
) {
    if !ui.is_changed() && !params.is_changed() {
        return;
    }
    let mode = ui.preview_mode;
    let pool = params.tag_pool;
    let mob_ctx = mobility_overlay_context(
        handles.as_ref().map(|r| &**r),
        tag_assets.as_ref().map(|r| &**r),
        mobility_assets.as_ref().map(|r| &**r),
        material_assets.as_ref().map(|r| &**r),
        &ui,
    );
    for (chunk, matrix, derived, mat_chunk, maps) in chunks.iter() {
        apply_overlay_indices(
            &mut commands,
            chunk,
            &overlay,
            matrix,
            derived,
            mat_chunk,
            maps.overlay,
            mode,
            &pool,
            &storages,
            mob_ctx,
        );
    }
}

fn apply_overlay_indices(
    commands: &mut Commands,
    chunk: &Chunk,
    overlay: &DynamicTerrainOverlay,
    matrix: &ChunkCellMatrix,
    derived: Option<&ChunkDerivedMetrics>,
    mat_chunk: &MaterializedChunk,
    tilemap: Entity,
    mode: PreviewMode,
    tag_pool: &TagSet,
    storages: &Query<&TileStorage>,
    mobility_ctx: Option<(
        &TagRegistry,
        &MobilityProfile,
        Option<&MaterialRegistry>,
    )>,
) {
    let Ok(storage) = storages.get(tilemap) else {
        return;
    };
    if storage.size.x != matrix.size.x || storage.size.y != matrix.size.y {
        return;
    }
    for y in 0..matrix.size.y {
        for x in 0..matrix.size.x {
            let cell_i = matrix.idx(x, y);
            let slope_cell = derived.and_then(|d| d.slope_grade.get(cell_i).copied());
            let mobility_hint = mobility_ctx.map(|(tr, pr, mreg)| {
                let mut scale = mreg
                    .and_then(|reg| {
                        mat_chunk.materials.get(cell_i).and_then(|mid| {
                            reg.materials
                                .get(mid.0 as usize)
                                .and_then(|d| d.sim_f32("traction_mod"))
                        })
                    })
                    .unwrap_or(1.0);
                let key = ChunkCellKey::new(chunk.coord, cell_i as u32);
                let mud_boost = overlay
                    .mud
                    .get(&key)
                    .copied()
                    .filter(|&m| m > 1e-6)
                    .map(|mud| 1.0 + mud * 0.25)
                    .unwrap_or(1.0);
                scale *= mud_boost;
                evaluate_tile(
                    pr,
                    &matrix.tags[cell_i],
                    slope_cell.unwrap_or(0.0),
                    1.0,
                    tr,
                    scale,
                )
            });
            let idx = overlay_index_for_cell(
                matrix,
                x,
                y,
                mode,
                tag_pool,
                slope_cell,
                mobility_hint.as_ref(),
            );
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
        app.init_resource::<DynamicTerrainOverlay>()
            .add_plugins(TilemapPlugin)
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
    use bevy::prelude::{IVec2, MinimalPlugins, UVec2};

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
            Chunk {
                coord: IVec2::ZERO,
            },
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
            .init_resource::<WorldGenParams>()
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
            .init_resource::<WorldGenParams>()
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
