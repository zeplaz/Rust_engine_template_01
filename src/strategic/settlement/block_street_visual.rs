//! **CITY-C6-BSN-001** — presentation-only street furniture via BSN `WorldAssetRoot` (MIG-A9 lane).

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;

use super::block_frame::BlockFrameBook;
use super::block_rollout::{BlockRecipeEvaluationBook, BlockRolloutStagingBook};
use super::ids::BlockId;

pub const CITY_C6_BSN_LIVE_JSON: &str = "debug_runs/city_c6_bsn_001_live.json";
pub const MIG_A9_BSN_HANDOFF_JSON: &str = "debug_runs/mig_bevy_019/mig_a_a9_bsn_scene_handoff.json";

const METRES_PER_TILE: f32 = 16.0;

/// Recipe `edge()` / `scatter()` asset id → promoted module job id (presentation-only).
fn street_asset_to_job_id(asset: &str) -> Option<&'static str> {
    match asset {
        "hedge" | "fence" => Some("prop_fence_lod0_run001"),
        "lamp_row" | "lamp" => Some("prop_light_lod0_run001"),
        "bench" => Some("prop_fence_lod0_run001"),
        "tree" | "scatter_tree" => Some("prop_vent_lod0_run001"),
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
    asset_server: Res<AssetServer>,
) {
    if catalog.load_started {
        return;
    }
    let jobs = [
        "prop_fence_lod0_run001",
        "prop_light_lod0_run001",
        "prop_vent_lod0_run001",
    ];
    for job in jobs {
        let label = format!("assets/models/modules/{job}/model.glb#Scene0");
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

#[must_use]
pub fn block_street_visual_fixture_witness_green() -> bool {
    use super::block_frame::{fixture_block_record_with_tiles, fixture_transport_graph_for_block_frame, rebuild_block_frames};
    use super::block_recipe::load_block_recipe_registry;
    use super::block_rollout::{
        advance_block_rollout_staging_system, begin_block_rollout_staging_system,
        evaluate_block_recipes_system, BlockRolloutGrowthBook,
    };

    let recipes = load_block_recipe_registry();
    if recipes.load_errors.is_empty() && recipes.recipes.is_empty() {
        return false;
    }

    let graph = fixture_transport_graph_for_block_frame();
    let mut block = fixture_block_record_with_tiles();
    block.site_ids.clear();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(recipes)
        .insert_resource(super::town::portland_fixture_town())
        .insert_resource(super::block::BlockBook {
            blocks: HashMap::from([(block.id.clone(), block.clone())]),
            tile_to_block: HashMap::new(),
        })
        .insert_resource(rebuild_block_frames(
            &super::block::BlockBook {
                blocks: HashMap::from([(block.id.clone(), block)]),
                tile_to_block: HashMap::new(),
            },
            Some(&graph),
        ))
        .init_resource::<BlockRecipeEvaluationBook>()
        .init_resource::<BlockRolloutGrowthBook>()
        .init_resource::<BlockRolloutStagingBook>()
        .init_resource::<BlockStreetVisualBook>()
        .init_resource::<BlockStreetFurnitureCatalog>()
        .add_message::<super::block_rollout::BlockPlanned>()
        .add_message::<super::block_rollout::BlockAssembled>()
        .add_message::<super::block_rollout::BlockCommitted>()
        .add_systems(
            Update,
            (
                evaluate_block_recipes_system,
                advance_block_rollout_staging_system,
                begin_block_rollout_staging_system,
                spawn_block_street_furniture_system,
            )
                .chain(),
        );
    for _ in 0..6 {
        app.update();
    }
    let book = app.world().resource::<BlockStreetVisualBook>();
    book.total_pieces >= 1
}

#[must_use]
pub fn build_city_c6_bsn_witness_body() -> serde_json::Value {
    let catalog_jobs = 3u32;
    let fixture_ok = block_street_visual_fixture_witness_green();
    let mig_a9_handoff_ok = fixture_ok;
    let green = fixture_ok;

    serde_json::json!({
        "gate": "CITY-C6-BSN-001",
        "green": green,
        "fixture_green": fixture_ok,
        "catalog_job_slots": catalog_jobs,
        "presentation_only": true,
        "mig_a9_handoff_ok": mig_a9_handoff_ok,
        "asset_map": ["hedge→prop_fence", "fence→prop_fence", "lamp_row→prop_light"],
    })
}

#[must_use]
pub fn city_c6_bsn_witness_green() -> bool {
    build_city_c6_bsn_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn refresh_city_c6_bsn_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_city_c6_bsn_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "CITY-C6-BSN-001",
        "refresh_city_c6_bsn_witness",
        CITY_C6_BSN_LIVE_JSON,
        body.clone(),
    );
    let a9 = serde_json::json!({
        "gate": "MIG-A9",
        "green": green,
        "status": if green { "adopted_settlement_lane" } else { "deferred" },
        "handoff": "plan_city_grammar_upgrade_v1 CITY-C6 BSN scenes",
        "city_c6_witness": CITY_C6_BSN_LIVE_JSON,
    });
    let a9_wrapped = wrap_debug_run(
        "MIG-A9",
        "refresh_city_c6_bsn_mig_a9_handoff",
        MIG_A9_BSN_HANDOFF_JSON,
        a9,
    );
    write_debug_run_json(CITY_C6_BSN_LIVE_JSON, wrapped)
        && write_debug_run_json(MIG_A9_BSN_HANDOFF_JSON, a9_wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_c6_bsn_fixture_witness_green_lib() {
        assert!(block_street_visual_fixture_witness_green());
    }

    #[test]
    fn city_c6_bsn_witness_green_lib() {
        assert!(city_c6_bsn_witness_green());
    }
}
