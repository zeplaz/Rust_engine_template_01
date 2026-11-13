//! **CITY-P2-001** — block LOD impostor vs street-detail swap by [`WorldLodBand`].

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;

use super::block::BlockBook;
use super::block_frame::{block_bounds_from_tiles, BlockFrameBook};
use super::block_rollout::BlockRolloutStagingBook;
use super::block_street_visual::{BlockStreetFurniturePiece, BlockStreetFurnitureRoot};
use super::ids::BlockId;
use crate::gui::{WorldLodBand, WorldRepresentationFrame};

// AssetServer roots at `assets/` — paths must be relative to that root (no `assets/` prefix).
const C8_PILOT_IMPOSTOR_GLB: &str = "staging/city_c8_pilot_merge_run001/model.glb#Scene0";
const FALLBACK_IMPOSTOR_GLB: &str = "models/modules/prop_fence_production_run001/model.glb#Scene0";

#[derive(Resource, Debug, Default)]
pub struct BlockLodImpostorCatalog {
    pub scene: Option<Handle<bevy::world_serialization::WorldAsset>>,
    pub load_started: bool,
}

#[derive(Component, Debug, Clone)]
pub struct BlockLodImpostor {
    pub block_id: BlockId,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct BlockLodImpostorBook {
    pub spawned_blocks: HashMap<BlockId, Entity>,
    pub last_band: Option<WorldLodBand>,
    pub impostor_visible: bool,
    pub detail_visible: bool,
}

#[must_use]
fn use_impostor_for_band(band: WorldLodBand) -> bool {
    matches!(band, WorldLodBand::Strategic | WorldLodBand::Macro)
}

#[must_use]
fn block_center_world(block_id: &BlockId, blocks: &BlockBook, frames: &BlockFrameBook) -> Vec3 {
    const METRES_PER_TILE: f32 = 16.0;
    if let Some(frame) = frames.frames.get(block_id) {
        return Vec3::new(
            frame.anchor.x as f32 * METRES_PER_TILE + frame.extent.x as f32 * METRES_PER_TILE * 0.5,
            0.0,
            frame.anchor.y as f32 * METRES_PER_TILE + frame.extent.y as f32 * METRES_PER_TILE * 0.5,
        );
    }
    if let Some(block) = blocks.blocks.get(block_id) {
        if let Some((min, extent)) = block_bounds_from_tiles(&block.tiles) {
            return Vec3::new(
                min.x as f32 * METRES_PER_TILE + extent.x as f32 * METRES_PER_TILE * 0.5,
                0.0,
                min.y as f32 * METRES_PER_TILE + extent.y as f32 * METRES_PER_TILE * 0.5,
            );
        }
    }
    Vec3::ZERO
}

pub fn load_block_lod_impostor_scene(
    mut catalog: ResMut<BlockLodImpostorCatalog>,
    asset_server: Option<Res<AssetServer>>,
    world_assets: Option<Res<Assets<bevy::world_serialization::WorldAsset>>>,
) {
    if catalog.load_started {
        return;
    }
    // Headless / minimal apps (sim harness, tests) lack `AssetServer` / `WorldAsset`; loading would panic.
    let (Some(asset_server), Some(_)) = (asset_server, world_assets) else {
        return;
    };
    let path = if std::path::Path::new("assets/staging/city_c8_pilot_merge_run001/model.glb").is_file() {
        C8_PILOT_IMPOSTOR_GLB
    } else {
        FALLBACK_IMPOSTOR_GLB
    };
    catalog.scene = Some(asset_server.load(path));
    catalog.load_started = true;
}

pub fn spawn_block_lod_impostors_system(
    mut commands: Commands,
    staging: Res<BlockRolloutStagingBook>,
    blocks: Res<BlockBook>,
    frames: Res<BlockFrameBook>,
    catalog: Res<BlockLodImpostorCatalog>,
    mut book: ResMut<BlockLodImpostorBook>,
) {
    for block_id in &staging.committed {
        if book.spawned_blocks.contains_key(block_id) {
            continue;
        }
        let center = block_center_world(block_id, &blocks, &frames);
        let entity = if let Some(scene) = catalog.scene.as_ref() {
            commands
                .spawn((
                    BlockLodImpostor {
                        block_id: block_id.clone(),
                    },
                    Transform::from_translation(center),
                    GlobalTransform::default(),
                    Visibility::Hidden,
                    WorldAssetRoot(scene.clone()),
                    crate::render::mig_a_static_bulk_bundle(),
                ))
                .id()
        } else {
            commands
                .spawn((
                    BlockLodImpostor {
                        block_id: block_id.clone(),
                    },
                    Transform::from_translation(center),
                    GlobalTransform::default(),
                    Visibility::Hidden,
                    crate::render::mig_a_static_bulk_bundle(),
                ))
                .id()
        };
        book.spawned_blocks.insert(block_id.clone(), entity);
    }
}

pub fn sync_block_lod_impostor_visibility_system(
    lod: Option<Res<WorldRepresentationFrame>>,
    mut book: ResMut<BlockLodImpostorBook>,
    mut impostors: Query<&mut Visibility, With<BlockLodImpostor>>,
    mut details: Query<
        &mut Visibility,
        (
            Or<(With<BlockStreetFurnitureRoot>, With<BlockStreetFurniturePiece>)>,
            Without<BlockLodImpostor>,
        ),
    >,
) {
    let band = lod
        .as_ref()
        .map(|f| f.global_band())
        .unwrap_or(WorldLodBand::Operational);
    let impostor_on = use_impostor_for_band(band);
    book.last_band = Some(band);
    book.impostor_visible = impostor_on;
    book.detail_visible = !impostor_on;

    for mut vis in &mut impostors {
        *vis = if impostor_on {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut vis in &mut details {
        *vis = if impostor_on {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}



#[cfg(test)]
mod tests {
    

}
