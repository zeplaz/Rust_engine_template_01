//! **CITY-P2-001** — block LOD impostor vs street-detail swap by [`WorldLodBand`].

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;

use super::block::BlockBook;
use super::block_frame::{block_bounds_from_tiles, BlockFrameBook};
use super::block_rollout::BlockRolloutStagingBook;
use super::block_street_visual::{BlockStreetFurniturePiece, BlockStreetFurnitureRoot};
use super::ids::BlockId;
use crate::gui::{WorldLodBand, WorldLodBands, WorldRepresentationFrame};

pub const CITY_P2_LIVE_JSON: &str = "debug_runs/city_p2_001_live.json";

const C8_PILOT_IMPOSTOR_GLB: &str = "assets/staging/city_c8_pilot_merge_run001/model.glb#Scene0";
const FALLBACK_IMPOSTOR_GLB: &str = "assets/models/modules/prop_fence_lod0_run001/model.glb#Scene0";

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
    asset_server: Res<AssetServer>,
) {
    if catalog.load_started {
        return;
    }
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

#[must_use]
pub fn block_lod_impostor_fixture_witness_green() -> bool {
    use super::block_frame::{
        fixture_block_record_with_tiles, fixture_transport_graph_for_block_frame, rebuild_block_frames,
    };
    use super::block_recipe::load_block_recipe_registry;
    use super::block_rollout::{
        advance_block_rollout_staging_system, begin_block_rollout_staging_system,
        evaluate_block_recipes_system, execute_staged_block_growth_system,
        release_staged_block_lot_proposals_system, BlockRolloutGrowthBook,
    };
    use super::district::{portland_fixture_district, DevelopmentPressureBook, DistrictMetricsBook};
    use super::growth::GrowthProposalQueue;
    use super::market::MarketSaturationBook;

    let recipes = load_block_recipe_registry();
    if !recipes.load_errors.is_empty() || recipes.recipes.is_empty() {
        return false;
    }

    let graph = fixture_transport_graph_for_block_frame();
    let mut block = fixture_block_record_with_tiles();
    block.site_ids.clear();
    let town = super::town::portland_fixture_town();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(recipes)
        .insert_resource(super::block_archetype::load_block_archetype_registry())
        .insert_resource(town.clone())
        .insert_resource(portland_fixture_district(&town))
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
        .init_resource::<DistrictMetricsBook>()
        .init_resource::<DevelopmentPressureBook>()
        .init_resource::<MarketSaturationBook>()
        .init_resource::<super::block_rollout::BlockRecipeEvaluationBook>()
        .init_resource::<BlockRolloutGrowthBook>()
        .init_resource::<BlockRolloutStagingBook>()
        .init_resource::<GrowthProposalQueue>()
        .init_resource::<crate::construction::PendingConstructionQueue>()
        .init_resource::<super::block_street_visual::BlockStreetVisualBook>()
        .init_resource::<super::block_street_visual::BlockStreetFurnitureCatalog>()
        .init_resource::<BlockLodImpostorCatalog>()
        .init_resource::<BlockLodImpostorBook>()
        .insert_resource(WorldRepresentationFrame {
            bands: WorldLodBands {
                global: WorldLodBand::Operational,
            },
            ..Default::default()
        })
        .add_message::<super::block_rollout::BlockPlanned>()
        .add_message::<super::block_rollout::BlockAssembled>()
        .add_message::<super::block_rollout::BlockCommitted>()
        .add_systems(
            Update,
            (
                evaluate_block_recipes_system,
                advance_block_rollout_staging_system,
                begin_block_rollout_staging_system,
                release_staged_block_lot_proposals_system,
                execute_staged_block_growth_system,
                super::block_street_visual::spawn_block_street_furniture_system,
                spawn_block_lod_impostors_system,
                sync_block_lod_impostor_visibility_system,
            )
                .chain(),
        );
    app.update();
    {
        let eval = app.world().resource::<super::block_rollout::BlockRecipeEvaluationBook>();
        if eval.by_block.is_empty() {
            return false;
        }
    }
    for _ in 0..12 {
        app.update();
    }
    {
        let staging = app.world().resource::<BlockRolloutStagingBook>();
        if staging.committed.is_empty() {
            return false;
        }
    }
    {
        let book = app.world().resource::<BlockLodImpostorBook>();
        if book.spawned_blocks.is_empty() || !book.detail_visible {
            return false;
        }
    }
    {
        let mut frame = app.world_mut().resource_mut::<WorldRepresentationFrame>();
        frame.bands.global = WorldLodBand::Strategic;
    }
    app.update();
    let book = app.world().resource::<BlockLodImpostorBook>();
    book.impostor_visible && !book.detail_visible
}

#[must_use]
pub fn build_city_p2_witness_body() -> serde_json::Value {
    let fixture_ok = block_lod_impostor_fixture_witness_green();
    serde_json::json!({
        "gate": "CITY-P2-001",
        "green": fixture_ok,
        "fixture_green": fixture_ok,
        "impostor_source": C8_PILOT_IMPOSTOR_GLB,
        "fallback_source": FALLBACK_IMPOSTOR_GLB,
        "lod_bands_impostor": ["Strategic", "Macro"],
    })
}

#[must_use]
pub fn city_p2_witness_green() -> bool {
    build_city_p2_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn refresh_city_p2_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_city_p2_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "CITY-P2-001",
        "refresh_city_p2_witness",
        CITY_P2_LIVE_JSON,
        body,
    );
    write_debug_run_json(CITY_P2_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_p2_fixture_witness_green_lib() {
        assert!(block_lod_impostor_fixture_witness_green());
    }

    #[test]
    fn city_p2_witness_green_lib() {
        assert!(city_p2_witness_green());
    }
}
