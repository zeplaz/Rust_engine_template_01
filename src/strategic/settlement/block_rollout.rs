//! **CITY-G3** — BlockRecipe rollout: evaluate → growth lots · staging messages · edge/scatter visuals.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::actors::{BuildingUsage, GrowthActorLayer};
use super::block::BlockBook;
use super::block_archetype::{BlockArchetypeRegistry, BlockScore, noise_jitter_from_block_seed};
use super::block_frame::BlockFrameBook;
use super::block_recipe::{
    evaluate_block_recipe_for_archetype, BlockRecipeEvaluation, BlockRecipeRegistry,
};
use super::district::{DevelopmentPressureBook, DistrictBook, DistrictMetricsBook};
use super::growth::{GrowthProposal, GrowthProposalQueue};
use super::ids::{ArchetypeId, BlockId, TownId};
use super::market::MarketSaturationBook;
use super::seed_chain::{block_seed, town_seed, DEFAULT_TOWN_ID, DEFAULT_WORLD_SEED};
use super::town::TownBook;
use crate::construction::{ConstructionVisualRequests, FootprintTileColorKind, FootprintTileRequest};

pub const CITY_G3_ROLLOUT_LIVE_JSON: &str = "debug_runs/city_g3_rollout_live.json";

#[derive(Resource, Debug, Default, Clone)]
pub struct BlockRecipeEvaluationBook {
    pub by_block: HashMap<BlockId, BlockRecipeEvaluation>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct BlockRolloutGrowthBook {
    /// Lots released from recipe eval into the growth queue (one per assembled frame).
    pub lots_released: HashMap<BlockId, usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockRolloutPhase {
    Planned,
    Assembled,
    Committed,
}

#[derive(Clone, Debug)]
struct BlockRolloutState {
    block_id: BlockId,
    phase: BlockRolloutPhase,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct BlockRolloutStagingBook {
    active: Vec<BlockRolloutState>,
    pub committed: HashSet<BlockId>,
    pub planned_messages: u32,
    pub assembled_messages: u32,
    pub committed_messages: u32,
}

impl BlockRolloutStagingBook {
    fn tracks(&self, block_id: &BlockId) -> bool {
        self.committed.contains(block_id)
            || self.active.iter().any(|s| &s.block_id == block_id)
    }

    /// Blocks in **Assembled** staging (or already **Committed**) — OK for presentation spawn.
    pub fn blocks_ready_for_street_visual(&self) -> Vec<BlockId> {
        let mut out: HashSet<BlockId> = self.committed.clone();
        for state in &self.active {
            if state.phase == BlockRolloutPhase::Assembled {
                out.insert(state.block_id.clone());
            }
        }
        out.into_iter().collect()
    }
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct BlockPlanned {
    pub block_id: BlockId,
    pub lot_count: usize,
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct BlockAssembled {
    pub block_id: BlockId,
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct BlockCommitted {
    pub block_id: BlockId,
}

#[must_use]
fn default_town_id(towns: &TownBook) -> TownId {
    towns
        .towns
        .keys()
        .next()
        .cloned()
        .unwrap_or_else(|| TownId(DEFAULT_TOWN_ID.into()))
}

pub fn resolve_block_archetypes_system(
    mut blocks: ResMut<BlockBook>,
    districts: Res<DistrictBook>,
    metrics: Res<DistrictMetricsBook>,
    pressure: Res<DevelopmentPressureBook>,
    saturation: Res<MarketSaturationBook>,
    registry: Res<BlockArchetypeRegistry>,
    towns: Res<TownBook>,
) {
    let town_id = default_town_id(&towns);
    let ts = town_seed(DEFAULT_WORLD_SEED, &town_id);
    for block in blocks.blocks.values_mut() {
        if block.archetype.is_some() {
            continue;
        }
        let Some(district) = districts.districts.get(&block.district_id) else {
            continue;
        };
        let Some(metrics) = metrics.by_district.get(&block.district_id) else {
            continue;
        };
        let Some(pressure) = pressure.by_district.get(&block.district_id) else {
            continue;
        };
        let sat = saturation
            .by_district
            .get(&block.district_id)
            .map(|book| {
                book.by_usage
                    .values()
                    .copied()
                    .fold(0.0_f32, f32::max)
            })
            .unwrap_or(0.0);
        let bs = block_seed(ts, &block.id);
        let score = BlockScore::from_district_fields(
            metrics,
            pressure,
            sat,
            district.zoning_default,
            noise_jitter_from_block_seed(bs),
        );
        block.archetype = Some(registry.resolve(&score));
    }
}

pub fn evaluate_block_recipes_system(
    blocks: Res<BlockBook>,
    frames: Res<BlockFrameBook>,
    recipes: Res<BlockRecipeRegistry>,
    towns: Res<TownBook>,
    mut eval_book: ResMut<BlockRecipeEvaluationBook>,
) {
    let town_id = default_town_id(&towns);
    let ts = town_seed(DEFAULT_WORLD_SEED, &town_id);
    let mut next = HashMap::new();
    for (block_id, block) in &blocks.blocks {
        let Some(archetype) = block.archetype else {
            continue;
        };
        let Some(frame) = frames.frames.get(block_id) else {
            continue;
        };
        let bs = block_seed(ts, block_id);
        if let Some(eval) =
            evaluate_block_recipe_for_archetype(&recipes, archetype, frame, block, bs)
        {
            next.insert(block_id.clone(), eval);
        }
    }
    if eval_book.by_block != next {
        eval_book.by_block = next;
    }
}

fn lot_usage_for_archetype(name: &str) -> BuildingUsage {
    if name.contains("Warehouse") || name.contains("Industrial") {
        BuildingUsage::Logistics
    } else if name.contains("Shop") {
        BuildingUsage::Commercial
    } else {
        BuildingUsage::Residential
    }
}

fn growth_proposal_for_lot(
    block_id: &BlockId,
    block: &super::block::BlockRecord,
    lot: &super::block_recipe::BlockLotPlacement,
) -> GrowthProposal {
    GrowthProposal {
        district_id: block.district_id.clone(),
        block_id: Some(block_id.clone()),
        archetype_id: ArchetypeId(lot.building_archetype.clone()),
        usage: lot_usage_for_archetype(&lot.building_archetype),
        actor_layer: GrowthActorLayer::Growth,
        anchor_tile: lot.origin,
        priority: 0.5,
        seed: lot.lot_seed,
        reason_codes: Vec::new(),
        saturation_at_submit: 0.0,
    }
}

pub fn release_staged_block_lot_proposals_system(
    staging: Res<BlockRolloutStagingBook>,
    eval_book: Res<BlockRecipeEvaluationBook>,
    blocks: Res<BlockBook>,
    mut queue: ResMut<GrowthProposalQueue>,
    mut growth: ResMut<BlockRolloutGrowthBook>,
) {
    for state in &staging.active {
        if state.phase != BlockRolloutPhase::Assembled {
            continue;
        }
        let Some(eval) = eval_book.by_block.get(&state.block_id) else {
            continue;
        };
        if !eval.errors.is_empty() || eval.lots.is_empty() {
            continue;
        }
        let Some(block) = blocks.blocks.get(&state.block_id) else {
            continue;
        };
        if !block.site_ids.is_empty() {
            continue;
        }
        let released = growth
            .lots_released
            .get(&state.block_id)
            .copied()
            .unwrap_or(0);
        if released >= eval.lots.len() {
            continue;
        }
        let lot = &eval.lots[released];
        let proposal = growth_proposal_for_lot(&state.block_id, block, lot);
        if queue.proposals.iter().any(|p| {
            p.block_id == proposal.block_id && p.anchor_tile == proposal.anchor_tile
        }) {
            continue;
        }
        queue.enqueue(proposal);
        growth
            .lots_released
            .insert(state.block_id.clone(), released + 1);
    }
}

pub fn execute_staged_block_growth_system(
    staging: Res<BlockRolloutStagingBook>,
    mut queue: ResMut<GrowthProposalQueue>,
    mut pending: Option<ResMut<crate::construction::PendingConstructionQueue>>,
) {
    let Some(pending) = pending.as_mut() else {
        return;
    };
    for state in &staging.active {
        if state.phase != BlockRolloutPhase::Assembled {
            continue;
        }
        let Some(idx) = queue
            .proposals
            .iter()
            .position(|p| p.block_id.as_ref() == Some(&state.block_id))
        else {
            continue;
        };
        let _ = super::execute::approve_growth_proposal_into_pending(&mut queue, idx, pending);
    }
}

pub fn begin_block_rollout_staging_system(
    eval_book: Res<BlockRecipeEvaluationBook>,
    mut staging: ResMut<BlockRolloutStagingBook>,
    mut planned: MessageWriter<BlockPlanned>,
) {
    for (block_id, eval) in &eval_book.by_block {
        if !eval.errors.is_empty() || staging.tracks(block_id) {
            continue;
        }
        staging.active.push(BlockRolloutState {
            block_id: block_id.clone(),
            phase: BlockRolloutPhase::Planned,
        });
        planned.write(BlockPlanned {
            block_id: block_id.clone(),
            lot_count: eval.lots.len(),
        });
        staging.planned_messages += 1;
    }
}

pub fn advance_block_rollout_staging_system(
    eval_book: Res<BlockRecipeEvaluationBook>,
    growth: Res<BlockRolloutGrowthBook>,
    mut staging: ResMut<BlockRolloutStagingBook>,
    mut assembled: MessageWriter<BlockAssembled>,
    mut committed: MessageWriter<BlockCommitted>,
) {
    let drained: Vec<_> = staging.active.drain(..).collect();
    let mut next_active = Vec::new();
    for mut state in drained {
        match state.phase {
            BlockRolloutPhase::Planned => {
                assembled.write(BlockAssembled {
                    block_id: state.block_id.clone(),
                });
                staging.assembled_messages += 1;
                state.phase = BlockRolloutPhase::Assembled;
                next_active.push(state);
            }
            BlockRolloutPhase::Assembled => {
                let expected = eval_book
                    .by_block
                    .get(&state.block_id)
                    .map(|eval| eval.lots.len())
                    .unwrap_or(0);
                let released = growth
                    .lots_released
                    .get(&state.block_id)
                    .copied()
                    .unwrap_or(0);
                if released < expected {
                    next_active.push(state);
                    continue;
                }
                state.phase = BlockRolloutPhase::Committed;
                committed.write(BlockCommitted {
                    block_id: state.block_id.clone(),
                });
                staging.committed_messages += 1;
                staging.committed.insert(state.block_id);
            }
            BlockRolloutPhase::Committed => {}
        }
    }
    staging.active = next_active;
}

pub fn push_block_recipe_visual_requests_system(
    eval_book: Res<BlockRecipeEvaluationBook>,
    test_scene: Option<Res<crate::engine::test_harness::ActiveTestScene>>,
    mut requests: Option<ResMut<ConstructionVisualRequests>>,
) {
    if test_scene.is_some() {
        return;
    }
    let Some(requests) = requests.as_mut() else {
        return;
    };
    for eval in eval_book.by_block.values() {
        for edge in &eval.edges {
            for &tile in &edge.street_tiles {
                requests.footprint_tiles.push(FootprintTileRequest {
                    tile,
                    color_kind: FootprintTileColorKind::Valid,
                    weight: 0.35,
                });
            }
        }
        for scatter in &eval.scatters {
            for &tile in &scatter.tiles {
                requests.footprint_tiles.push(FootprintTileRequest {
                    tile,
                    color_kind: FootprintTileColorKind::Risky,
                    weight: 0.25,
                });
            }
        }
        for plaza in &eval.plazas {
            for &tile in &plaza.tiles {
                requests.footprint_tiles.push(FootprintTileRequest {
                    tile,
                    color_kind: FootprintTileColorKind::Valid,
                    weight: 0.2,
                });
            }
        }
    }
}

#[must_use]
pub fn block_rollout_fixture_witness_green() -> bool {
    use super::block_frame::{
        build_block_frame, fixture_block_record_with_tiles, fixture_transport_graph_for_block_frame,
        rebuild_block_frames,
    };

    let recipes = super::block_recipe::load_block_recipe_registry();
    if !recipes.load_errors.is_empty() || recipes.recipes.len() < 3 {
        return false;
    }

    let graph = fixture_transport_graph_for_block_frame();
    let mut block = fixture_block_record_with_tiles();
    block.site_ids.clear();
    let frame = build_block_frame(&block, Some(&graph)).expect("frame");
    let bs = block_seed(
        town_seed(DEFAULT_WORLD_SEED, &TownId(DEFAULT_TOWN_ID.into())),
        &block.id,
    );
    let eval = evaluate_block_recipe_for_archetype(
        &recipes,
        super::block_archetype::BlockArchetype::Industrial,
        &frame,
        &block,
        bs,
    )
    .expect("eval");
    if eval.errors.is_empty() && eval.lots.is_empty() {
        return false;
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(recipes)
        .insert_resource(load_block_archetype_registry())
        .insert_resource(BlockBook {
            blocks: HashMap::from([(block.id.clone(), block.clone())]),
            tile_to_block: HashMap::new(),
        })
        .insert_resource(rebuild_block_frames(
            &BlockBook {
                blocks: HashMap::from([(block.id.clone(), block)]),
                tile_to_block: HashMap::new(),
            },
            Some(&graph),
        ))
        .insert_resource(portland_fixture_town())
        .insert_resource(portland_fixture_district(&portland_fixture_town()))
        .init_resource::<DistrictMetricsBook>()
        .init_resource::<DevelopmentPressureBook>()
        .init_resource::<MarketSaturationBook>()
        .init_resource::<BlockRecipeEvaluationBook>()
        .init_resource::<BlockRolloutGrowthBook>()
        .init_resource::<BlockRolloutStagingBook>()
        .init_resource::<GrowthProposalQueue>()
        .init_resource::<crate::construction::PendingConstructionQueue>()
        .insert_resource(ConstructionVisualRequests::default())
        .add_message::<BlockPlanned>()
        .add_message::<BlockAssembled>()
        .add_message::<BlockCommitted>()
        .add_systems(
            Update,
            (
                evaluate_block_recipes_system,
                advance_block_rollout_staging_system,
                begin_block_rollout_staging_system,
                release_staged_block_lot_proposals_system,
                execute_staged_block_growth_system,
                push_block_recipe_visual_requests_system,
            )
                .chain(),
        );
    app.update();

    let eval_book = app.world().resource::<BlockRecipeEvaluationBook>();
    if eval_book.by_block.is_empty() {
        return false;
    }
    let reqs = app.world().resource::<ConstructionVisualRequests>();
    if reqs.footprint_tiles.is_empty() {
        return false;
    }

    for _ in 0..4 {
        app.update();
    }

    let queue = app.world().resource::<GrowthProposalQueue>();
    let pending = app.world().resource::<crate::construction::PendingConstructionQueue>();
    if !queue.proposals.is_empty() || pending.entries.len() != 2 {
        return false;
    }
    let growth = app.world().resource::<BlockRolloutGrowthBook>();
    if growth.lots_released.values().copied().sum::<usize>() != 2 {
        return false;
    }

    let staging = app.world().resource::<BlockRolloutStagingBook>();
    staging.planned_messages >= 1
        && staging.assembled_messages >= 1
        && staging.committed_messages >= 1
        && !staging.committed.is_empty()
}

fn load_block_archetype_registry() -> BlockArchetypeRegistry {
    super::block_archetype::load_block_archetype_registry()
}

fn portland_fixture_town() -> TownBook {
    super::town::portland_fixture_town()
}

fn portland_fixture_district(town: &TownBook) -> super::district::DistrictBook {
    super::district::portland_fixture_district(town)
}

#[must_use]
pub fn build_city_g3_rollout_witness_body() -> serde_json::Value {
    let recipes = super::block_recipe::load_block_recipe_registry();
    let plugin_ok = recipes.load_errors.is_empty() && recipes.recipes.len() >= 3;
    let fixture_ok = block_rollout_fixture_witness_green();
    let c3 = super::block_recipe::city_g1_c3_001_block_recipe_witness_green();
    let execute_ok = super::execute::growth_approve_execute_pipeline_witness_green();

    let green = plugin_ok && fixture_ok && c3 && execute_ok;

    serde_json::json!({
        "gate": "CITY-G3-ROLLOUT-001",
        "green": green,
        "plugin_registry_loaded": plugin_ok,
        "recipes_loaded": recipes.recipes.len(),
        "rollout_fixture_green": fixture_ok,
        "growth_execute_wired": execute_ok,
        "city_g1_c3_still_green": c3,
    })
}

#[must_use]
pub fn city_g3_rollout_witness_green() -> bool {
    build_city_g3_rollout_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn refresh_city_g3_rollout_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_city_g3_rollout_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "CITY-G3-ROLLOUT-001",
        "refresh_city_g3_rollout_witness",
        CITY_G3_ROLLOUT_LIVE_JSON,
        body,
    );
    write_debug_run_json(CITY_G3_ROLLOUT_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_g3_rollout_fixture_witness_green_lib() {
        assert!(block_rollout_fixture_witness_green());
    }

    #[test]
    fn city_g3_rollout_witness_green_lib() {
        assert!(city_g3_rollout_witness_green());
    }
}
