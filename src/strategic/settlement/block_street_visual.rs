//! **CITY-C6-BSN-001** — presentation-only street furniture via BSN `WorldAssetRoot` (MIG-A9 lane).

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;

use super::block_frame::BlockFrameBook;
use super::block_rollout::{BlockRecipeEvaluationBook, BlockRolloutStagingBook};
use super::ids::BlockId;

const METRES_PER_TILE: f32 = crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Promoted street-prop module jobs under `assets/models/modules/` (the `*_lod0_run001` twins were
/// retired to `assets/archive/kit_lod0_twins_retired_2026-09/`; there is no promoted lamp module yet,
/// so `lamp` / `lamp_row` recipe props spawn nothing until one is promoted).
const STREET_PROP_JOBS: [&str; 2] = ["prop_fence_production_run001", "prop_vent_production_run001"];

/// Recipe `edge()` / `scatter()` asset id → promoted module job id (presentation-only).
fn street_asset_to_job_id(asset: &str) -> Option<&'static str> {
    match asset {
        "hedge" | "fence" | "bench" => Some(STREET_PROP_JOBS[0]),
        "tree" | "scatter_tree" => Some(STREET_PROP_JOBS[1]),
        _ => None,
    }
}

#[must_use]
fn tile_world_translation(tile: IVec2) -> Vec3 {
    Vec3::new(
        tile.x as f32 * METRES_PER_TILE,
        0.0,
        tile.y as f32 * METRES_PER_TILE,
    )
}

#[must_use]
fn edge_piece_scale(street_side: super::block_frame::StreetSide) -> Vec3 {
    match street_side {
        super::block_frame::StreetSide::PosX | super::block_frame::StreetSide::NegX => {
            Vec3::new(1.0, 1.0, 1.35)
        }
        super::block_frame::StreetSide::PosZ | super::block_frame::StreetSide::NegZ => {
            Vec3::new(1.35, 1.0, 1.0)
        }
    }
}

/// Presentation-only — never feeds logistics / traffic sim.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PresentationOnlyStreetVisual;

#[derive(Component, Debug, Clone)]
pub struct BlockStreetFurnitureRoot {
    pub block_id: BlockId,
}

#[derive(Component, Debug, Clone)]
pub struct BlockStreetFurniturePiece {
    pub block_id: BlockId,
    pub asset: String,
    pub tile: IVec2,
}

#[derive(Resource, Debug, Default)]
pub struct BlockStreetFurnitureCatalog {
    pub scenes: HashMap<String, Handle<bevy::world_serialization::WorldAsset>>,
    pub load_started: bool,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct BlockStreetVisualBook {
    pub spawned_pieces: HashMap<BlockId, usize>,
    pub total_pieces: u32,
}

impl BlockStreetVisualBook {
    fn next_spawn_index(&self, block_id: &BlockId) -> usize {
        self.spawned_pieces.get(block_id).copied().unwrap_or(0)
    }
}

#[derive(Clone, Debug)]
struct StreetSpawnSpec {
    asset: String,
    tile: IVec2,
    stretch: bool,
}

#[must_use]
fn collect_spawn_specs(eval: &super::block_recipe::BlockRecipeEvaluation) -> Vec<StreetSpawnSpec> {
    let mut out = Vec::new();
    for edge in &eval.edges {
        if edge.sim_authority {
            continue;
        }
        for &tile in &edge.street_tiles {
            out.push(StreetSpawnSpec {
                asset: edge.asset.clone(),
                tile,
                stretch: true,
            });
        }
    }
    for scatter in &eval.scatters {
        if scatter.sim_authority {
            continue;
        }
        for &tile in &scatter.tiles {
            out.push(StreetSpawnSpec {
                asset: scatter.asset.clone(),
                tile,
                stretch: false,
            });
        }
    }
    out
}

fn assembled_blocks(staging: &BlockRolloutStagingBook) -> Vec<BlockId> {
    staging.blocks_ready_for_street_visual()
}

pub fn load_block_street_furniture_scenes(
    mut catalog: ResMut<BlockStreetFurnitureCatalog>,
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
    for job in STREET_PROP_JOBS {
        // AssetServer roots at `assets/` — do not prefix `assets/` again (else `assets/assets/...`).
        let label = format!("models/modules/{job}/model.glb#Scene0");
        catalog
            .scenes
            .insert(job.to_string(), asset_server.load(label));
    }
    catalog.load_started = true;
}

pub fn spawn_block_street_furniture_system(
    mut commands: Commands,
    staging: Res<BlockRolloutStagingBook>,
    eval_book: Res<BlockRecipeEvaluationBook>,
    frames: Res<BlockFrameBook>,
    catalog: Res<BlockStreetFurnitureCatalog>,
    mut visual_book: ResMut<BlockStreetVisualBook>,
) {
    let assembled = assembled_blocks(&staging);
    for block_id in assembled {
        let Some(eval) = eval_book.by_block.get(&block_id) else {
            continue;
        };
        if !eval.errors.is_empty() {
            continue;
        }
        let specs = collect_spawn_specs(eval);
        if specs.is_empty() {
            continue;
        }
        let idx = visual_book.next_spawn_index(&block_id);
        if idx >= specs.len() {
            continue;
        }
        let spec = &specs[idx];
        let Some(job_id) = street_asset_to_job_id(&spec.asset) else {
            visual_book
                .spawned_pieces
                .insert(block_id.clone(), idx + 1);
            continue;
        };
        let street_side = frames
            .frames
            .get(&block_id)
            .map(|f| f.street_side)
            .unwrap_or(super::block_frame::StreetSide::NegZ);
        let mut transform = Transform::from_translation(tile_world_translation(spec.tile));
        if spec.stretch {
            transform.scale = edge_piece_scale(street_side);
        }
        let root = commands
            .spawn((
                BlockStreetFurnitureRoot {
                    block_id: block_id.clone(),
                },
                PresentationOnlyStreetVisual,
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
            ))
            .id();
        let piece = BlockStreetFurniturePiece {
            block_id: block_id.clone(),
            asset: spec.asset.clone(),
            tile: spec.tile,
        };
        if let Some(scene) = catalog.scenes.get(job_id) {
            commands.entity(root).with_children(|parent| {
                parent.spawn((
                    WorldAssetRoot(scene.clone()),
                    transform,
                    crate::render::mig_a_static_bulk_bundle(),
                    piece,
                    PresentationOnlyStreetVisual,
                ));
            });
        } else {
            commands.entity(root).with_children(|parent| {
                parent.spawn((
                    transform,
                    crate::render::mig_a_static_bulk_bundle(),
                    piece,
                    PresentationOnlyStreetVisual,
                ));
            });
        }
        visual_book
            .spawned_pieces
            .insert(block_id.clone(), idx + 1);
        visual_book.total_pieces = visual_book.total_pieces.saturating_add(1);
    }
}



#[cfg(test)]
mod tests {
    

}
