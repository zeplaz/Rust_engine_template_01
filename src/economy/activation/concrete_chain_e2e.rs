//! Portland concrete chain E2E witness (IND-E01): mine → kiln → mixer operational in sim.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::construction::procedural::{ProceduralBuildingRequest, StylePackId};
use crate::construction::queue_commit_construction_site;
use crate::economy::activation::{BuildingDefinitionRef, IndustrialFacilityActivated};
use crate::economy::resource_flow::{ResourceFlowNode, ResourceFlowRegistry, ResourceFlowSimWitness};
use crate::strategic::{
    BuildSiteTile, CommitConstructionSiteEvent, ConstructionSite, FootprintTiles, LayerType,
    PlannedSite, ProceduralBuildingSpec, SiteArchetype, SiteConstructionBook,
    SiteConstructionPhase, SiteFootprint, SiteId, SiteOperationalStats,
};

const PORTLAND_FOOTPRINT: FootprintTiles = FootprintTiles {
    width: 3,
    depth: 2,
};

/// Victorian rowhouse production pilot footprint (matches `victorian_4x3_s42_a7cb`).
pub const ROWHOUSE_VICTORIAN_PRODUCTION_FOOTPRINT: FootprintTiles = FootprintTiles {
    width: 4,
    depth: 3,
};

pub const ROWHOUSE_VICTORIAN_DEMO_SITE_ID: u64 = 42;

pub const CONCRETE_PORTLAND_CHAIN: &str = "concrete_portland";

pub const CONCRETE_PORTLAND_STEPS: &[&str] = &[
    "concrete_aggregate_mine",
    "concrete_cement_kiln",
    "concrete_mixer_plant",
];

/// Live counters for `debug_runs/industrial_activation_live.json` (`concrete_chain_e2e` block).
#[derive(Resource, Debug, Default, Clone)]
pub struct ConcreteChainE2eWitness {
    pub operational_mine: u32,
    pub operational_kiln: u32,
    pub operational_mixer: u32,
    pub activated_mine: u32,
    pub activated_kiln: u32,
    pub activated_mixer: u32,
    pub flow_edges: u32,
    pub production_ticks: u32,
    /// IND-E02: sites entered via [`CommitConstructionSiteEvent`] (construction spine), not direct spawn.
    pub placed_via_construction: bool,
    pub sites_committed: u32,
}

impl ConcreteChainE2eWitness {
    #[must_use]
    pub fn chain_operational(&self) -> bool {
        self.operational_mine >= 1
            && self.operational_kiln >= 1
            && self.operational_mixer >= 1
            && self.activated_mine >= 1
            && self.activated_kiln >= 1
            && self.activated_mixer >= 1
    }

    /// IND-E01 exit: full chain activated, linked, and at least one propagation tick.
    #[must_use]
    pub fn production_green(&self) -> bool {
        self.chain_operational() && self.flow_edges >= 2 && self.production_ticks >= 1
    }

    /// IND-E02 exit: same chain metrics after construction commit path (in-play).
    #[must_use]
    pub fn in_play_green(&self) -> bool {
        self.production_green() && self.placed_via_construction && self.sites_committed >= 3
    }
}

fn count_for_catalog(
    sites: &Query<(&ConstructionSite, &BuildingDefinitionRef)>,
    activated: &Query<&BuildingDefinitionRef, With<IndustrialFacilityActivated>>,
    catalog_id: &str,
    operational_only: bool,
) -> u32 {
    if operational_only {
        sites
            .iter()
            .filter(|(site, def_ref)| {
                site.phase == SiteConstructionPhase::Operational
                    && def_ref.catalog_id == catalog_id
            })
            .count() as u32
    } else {
        activated
            .iter()
            .filter(|def_ref| def_ref.catalog_id == catalog_id)
            .count() as u32
    }
}

fn count_portland_flow_edges(
    flow: &ResourceFlowRegistry,
    nodes: &Query<(Entity, &ResourceFlowNode)>,
) -> u32 {
    let catalog_by_entity: HashMap<Entity, &str> = nodes
        .iter()
        .map(|(e, node)| (e, node.catalog_id.as_str()))
        .collect();
    flow.edges
        .iter()
        .filter(|edge| {
            let Some(from_id) = catalog_by_entity.get(&edge.from) else {
                return false;
            };
            let Some(to_id) = catalog_by_entity.get(&edge.to) else {
                return false;
            };
            matches!(
                (from_id.as_ref(), to_id.as_ref()),
                ("concrete_aggregate_mine", "concrete_cement_kiln")
                    | ("concrete_cement_kiln", "concrete_mixer_plant")
            )
        })
        .count() as u32
}

pub fn refresh_concrete_chain_e2e_witness_system(
    mut witness: ResMut<ConcreteChainE2eWitness>,
    flow: Res<ResourceFlowRegistry>,
    flow_witness: Res<ResourceFlowSimWitness>,
    sites: Query<(&ConstructionSite, &BuildingDefinitionRef)>,
    activated: Query<&BuildingDefinitionRef, With<IndustrialFacilityActivated>>,
    nodes: Query<(Entity, &ResourceFlowNode)>,
) {
    witness.operational_mine =
        count_for_catalog(&sites, &activated, CONCRETE_PORTLAND_STEPS[0], true);
    witness.operational_kiln =
        count_for_catalog(&sites, &activated, CONCRETE_PORTLAND_STEPS[1], true);
    witness.operational_mixer =
        count_for_catalog(&sites, &activated, CONCRETE_PORTLAND_STEPS[2], true);
    witness.activated_mine =
        count_for_catalog(&sites, &activated, CONCRETE_PORTLAND_STEPS[0], false);
    witness.activated_kiln =
        count_for_catalog(&sites, &activated, CONCRETE_PORTLAND_STEPS[1], false);
    witness.activated_mixer =
        count_for_catalog(&sites, &activated, CONCRETE_PORTLAND_STEPS[2], false);
    witness.flow_edges = count_portland_flow_edges(&flow, &nodes);
    witness.production_ticks = flow_witness.ticks_propagated;
}

/// IND-E02: enqueue mine → kiln → mixer through the construction commit funnel.
pub fn commit_concrete_portland_chain_in_play(
    writer: &mut MessageWriter<CommitConstructionSiteEvent>,
    witness: &mut ConcreteChainE2eWitness,
    owner: Entity,
    origin: BuildSiteTile,
) {
    let offsets = [(0i32, 0i32), (4, 0), (8, 0)];
    for (catalog_id, (dx, dz)) in CONCRETE_PORTLAND_STEPS.iter().zip(offsets.iter()) {
        let tile = BuildSiteTile {
            x: origin.x.saturating_add(*dx as u32),
            z: origin.z.saturating_add(*dz as u32),
        };
        queue_commit_construction_site(
            writer,
            owner,
            SiteArchetype::Factory,
            tile,
            PORTLAND_FOOTPRINT,
            LayerType::Surface,
            Some((*catalog_id).to_string()),
            None,
        );
    }
    witness.placed_via_construction = true;
    witness.sites_committed = CONCRETE_PORTLAND_STEPS.len() as u32;
}

/// **ENG-PT-4-001** — operational rowhouse with production atlas stamp wiring (DefaultIndustrial play).
pub fn spawn_rowhouse_victorian_production_demo(
    commands: &mut Commands,
    origin: BuildSiteTile,
) -> Entity {
    let footprint = ROWHOUSE_VICTORIAN_PRODUCTION_FOOTPRINT;
    let ox = origin.x as i32;
    let oz = origin.z as i32;
    let mut tiles = Vec::with_capacity((footprint.width * footprint.depth) as usize);
    for dz in 0..footprint.depth {
        for dx in 0..footprint.width {
            tiles.push(IVec2::new(ox + dx as i32, oz + dz as i32));
        }
    }
    let spec = ProceduralBuildingRequest {
        archetype_id: "rect_perimeter".into(),
        width: footprint.width,
        depth: footprint.depth,
        floors: 2,
        style: StylePackId("style_victorian".into()),
        seed: ROWHOUSE_VICTORIAN_DEMO_SITE_ID,
        arch_dna_preset_id: None,
    };
    commands
        .spawn((
            ConstructionSite {
                site_id: ROWHOUSE_VICTORIAN_DEMO_SITE_ID,
                owner: Entity::PLACEHOLDER,
                archetype: SiteArchetype::CivilHousing,
                phase: SiteConstructionPhase::Operational,
                operational_readiness: 1.0,
            },
            PlannedSite {
                site_id: SiteId(ROWHOUSE_VICTORIAN_DEMO_SITE_ID),
                origin,
                footprint,
                archetype: SiteArchetype::CivilHousing,
                layer: LayerType::Surface,
                catalog_id: Some(crate::gui::map_tile_atlas_stamp::ROWHOUSE_VICTORIAN_TILE_ID.into()),
                placement: None,
            },
            ProceduralBuildingSpec(spec),
            SiteFootprint {
                tiles,
                layer: LayerType::Surface,
            },
            BuildingDefinitionRef {
                catalog_id: crate::gui::map_tile_atlas_stamp::ROWHOUSE_VICTORIAN_TILE_ID.into(),
            },
            Transform::from_translation(crate::economy::site_placement::site_world_position(
                origin,
            )),
            GlobalTransform::default(),
        ))
        .id()
}

/// Opt-in proof fast-path (`RUST_ENGINE_CONSTRUCTION_INSTANT=1`); default play uses staged tick (CON-P2-001).
#[inline]
fn construction_instant_operational_enabled() -> bool {
    std::env::var("RUST_ENGINE_CONSTRUCTION_INSTANT")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Proof fast-path after commit: move Portland steps to **Operational** with provisioning stats satisfied.
pub fn fast_forward_portland_chain_sites_to_operational(
    mut sites: Query<(
        &BuildingDefinitionRef,
        &mut ConstructionSite,
        &mut SiteOperationalStats,
        &PlannedSite,
    )>,
    mut book: Option<ResMut<SiteConstructionBook>>,
) {
    if !construction_instant_operational_enabled() {
        return;
    }
    for (def_ref, mut site, mut stats, planned) in &mut sites {
        if !CONCRETE_PORTLAND_STEPS
            .iter()
            .any(|id| *id == def_ref.catalog_id.as_str())
        {
            continue;
        }
        site.phase = SiteConstructionPhase::Operational;
        site.operational_readiness = 1.0;
        stats.power_ratio = 1.0;
        stats.supply_ratio = 1.0;
        stats.workforce_ratio = 1.0;
        stats.integrity = 1.0;
        if let Some(st) = book
            .as_deref_mut()
            .and_then(|b| b.by_site.get_mut(&planned.site_id))
        {
            st.phase = SiteConstructionPhase::Operational;
            st.progress = 1.0;
        }
    }
}

/// Spawn three operational Portland chain sites (construction spine satisfied via `PlannedSite`).
pub fn spawn_concrete_portland_chain_operational(
    commands: &mut Commands,
    origin: BuildSiteTile,
) -> [Entity; 3] {
    let offsets = [(0i32, 0i32), (4, 0), (8, 0)];
    let site_ids = [901u64, 902, 903];
    let mut entities = [Entity::PLACEHOLDER; 3];
    for (i, (catalog_id, (dx, dz))) in CONCRETE_PORTLAND_STEPS
        .iter()
        .zip(offsets.iter())
        .enumerate()
    {
        let tile = BuildSiteTile {
            x: origin.x.saturating_add(*dx as u32),
            z: origin.z.saturating_add(*dz as u32),
        };
        entities[i] = commands
            .spawn((
                ConstructionSite {
                    site_id: site_ids[i],
                    owner: Entity::PLACEHOLDER,
                    archetype: SiteArchetype::Factory,
                    phase: SiteConstructionPhase::Operational,
                    operational_readiness: 1.0,
                },
                PlannedSite {
                    site_id: SiteId(site_ids[i]),
                    origin: tile,
                    footprint: FootprintTiles {
                        width: 3,
                        depth: 2,
                    },
                    archetype: SiteArchetype::Factory,
                    layer: LayerType::Surface,
                    catalog_id: Some((*catalog_id).into()),
                    placement: None,
                },
                BuildingDefinitionRef {
                    catalog_id: (*catalog_id).into(),
                },
                Transform::from_translation(crate::economy::site_placement::site_world_position(
                    tile,
                )),
                GlobalTransform::default(),
            ))
            .id();
    }
    entities
}

/// Opt-in debug seed when `RUST_ENGINE_STAGE7_PLAY_SEED` is set.
/// **DefaultIndustrial** uses [`seed_ind_e02_default_play_once`] instead (PLAY-TRUTH-001-TAIL).
/// **DEHACK-ENV-002:** `RUST_ENGINE_S7P_STEWARD` sunset — use scenario / play plugin only.
#[derive(Resource, Debug, Default)]
pub struct Stage7PlayChainSeedState {
    pub seeded: bool,
}

#[inline]
fn stage7_play_seed_enabled() -> bool {
    fn env_on(key: &str) -> bool {
        std::env::var(key)
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    }
    env_on("RUST_ENGINE_STAGE7_PLAY_SEED")
}

pub fn seed_stage7_play_concrete_chain_once(
    base: Res<State<crate::engine::states::BaseState>>,
    scenario: Option<Res<crate::engine::ActivePlayScenario>>,
    mut seed: ResMut<Stage7PlayChainSeedState>,
    mut witness: ResMut<ConcreteChainE2eWitness>,
    mut commands: Commands,
    def_ref: Query<&BuildingDefinitionRef>,
) {
    if seed.seeded || !matches!(base.get(), crate::engine::states::BaseState::Simulation) {
        return;
    }
    if scenario.is_some_and(|s| s.is_default_industrial()) {
        return;
    }
    if !stage7_play_seed_enabled() {
        return;
    }
    let already = def_ref.iter().any(|d| d.catalog_id == CONCRETE_PORTLAND_STEPS[2]);
    if already {
        seed.seeded = true;
        return;
    }
    spawn_concrete_portland_chain_operational(&mut commands, BuildSiteTile { x: 48, z: 48 });
    witness.placed_via_construction = false;
    witness.sites_committed = 0;
    seed.seeded = true;
}

pub fn reset_stage7_play_chain_seed_on_enter_simulation(
    mut seed: ResMut<Stage7PlayChainSeedState>,
) {
    seed.seeded = false;
}

/// **IND-E02-DEFAULT-PLAY-001** — default Simulation path: Portland chain via construction commit (no seed env).
#[derive(Resource, Debug, Default)]
pub struct IndE02DefaultPlaySeedState {
    pub enqueued: bool,
}

pub fn reset_ind_e02_default_play_seed_on_enter_simulation(
    mut seed: ResMut<IndE02DefaultPlaySeedState>,
) {
    *seed = IndE02DefaultPlaySeedState::default();
}

pub fn seed_ind_e02_default_play_once(
    base: Res<State<crate::engine::states::BaseState>>,
    scenario: Option<Res<crate::engine::ActivePlayScenario>>,
    test_scene: Option<Res<crate::engine::ActiveTestScene>>,
    mut seed: ResMut<IndE02DefaultPlaySeedState>,
    mut witness: ResMut<ConcreteChainE2eWitness>,
    sites: Query<&BuildingDefinitionRef, With<ConstructionSite>>,
    mut writer: MessageWriter<CommitConstructionSiteEvent>,
) {
    if !matches!(base.get(), crate::engine::states::BaseState::Simulation) {
        return;
    }
    if test_scene.is_some() {
        return;
    }
    let default_industrial = scenario
        .as_ref()
        .map(|s| s.is_default_industrial())
        .unwrap_or(true);
    if scenario.is_some_and(|s| !s.is_default_industrial()) {
        return;
    }
    if witness.in_play_green() {
        return;
    }
    if witness.production_green() && !witness.placed_via_construction && !default_industrial {
        return;
    }
    if seed.enqueued {
        return;
    }
    let has_portland = sites.iter().any(|d| {
        CONCRETE_PORTLAND_STEPS
            .iter()
            .any(|id| *id == d.catalog_id.as_str())
    });
    if has_portland && witness.placed_via_construction {
        return;
    }
    if has_portland && !default_industrial {
        return;
    }
    let portland_origin = crate::engine::DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN;
    commit_concrete_portland_chain_in_play(
        &mut writer,
        witness.as_mut(),
        Entity::PLACEHOLDER,
        portland_origin,
    );
    seed.enqueued = true;
}

/// **ENG-PT-4-001** — one-shot rowhouse production demo site for map iso stamp.
#[derive(Resource, Debug, Default)]
pub struct RowhouseVictorianDemoSeedState {
    pub spawned: bool,
}

pub fn reset_rowhouse_victorian_demo_seed_on_enter_simulation(
    mut seed: ResMut<RowhouseVictorianDemoSeedState>,
) {
    *seed = RowhouseVictorianDemoSeedState::default();
}

/// After Portland chain, spawn operational rowhouse where production atlas stamps on the map.
pub fn seed_rowhouse_victorian_production_demo_once(
    base: Res<State<crate::engine::states::BaseState>>,
    test_scene: Option<Res<crate::engine::ActiveTestScene>>,
    mut seed: ResMut<RowhouseVictorianDemoSeedState>,
    mut commands: Commands,
    sites: Query<&ConstructionSite>,
) {
    if !matches!(base.get(), crate::engine::states::BaseState::Simulation) {
        return;
    }
    if test_scene.is_some() {
        return;
    }
    if seed.spawned {
        return;
    }
    if sites
        .iter()
        .any(|s| s.site_id == ROWHOUSE_VICTORIAN_DEMO_SITE_ID)
    {
        seed.spawned = true;
        return;
    }
    let origin = BuildSiteTile {
        x: crate::engine::DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN
            .x
            .saturating_add(12),
        z: crate::engine::DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN
            .z
            .saturating_add(6),
    };
    spawn_rowhouse_victorian_production_demo(&mut commands, origin);
    seed.spawned = true;
}

/// **IND-E03-CODER-A** — one-shot grid overload cluster for live proof depth.
/// Plan: `src/dev/industrial_grid_overload_impl_plan_v1.md` (PLAN-IND-E03-001).
#[derive(Resource, Debug, Default)]
pub struct IndE03GridOverloadSeedState {
    pub seeded: bool,
    pub cluster_spawned: bool,
}

#[inline]
fn ind_e03_grid_seed_enabled(launch: Option<&crate::engine::launch_args::EngineLaunchArgs>) -> bool {
    fn env_on(key: &str) -> bool {
        std::env::var(key)
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    }
    if env_on("RUST_ENGINE_IND_E03_SEED") || env_on("RUST_ENGINE_STAGE7_PLAY_SEED") {
        return true;
    }
    launch.is_some_and(|l| l.full_capture_active())
}

/// Spawns transformer host + high-load members (mirrors `bridge` overload integration test).
pub fn spawn_ind_e03_grid_overload_cluster(commands: &mut Commands, origin: BuildSiteTile) {
    use crate::entities::production::power::{
        ElectricalComponent, ElectricalGrid, TransformerComponent,
    };
    use crate::entities::structure::components::Building;
    use crate::entities::types::s_flagz::BuildingType;

    let base = crate::economy::site_placement::site_world_position(origin);
    commands.spawn((
        Transform::from_translation(base),
        GlobalTransform::default(),
        TransformerComponent {
            input_voltage: 138_000.0,
            output_voltage: 13_800.0,
        },
        ElectricalGrid::default(),
        ElectricalComponent {
            base_load: 0.1,
            current_load: 0.1,
            max_transfer: 2.0,
            capacity: 2.0,
        },
        Building {
            building_type: BuildingType::Generic,
        },
    ));
    for i in 0..4 {
        commands.spawn((
            Transform::from_translation(base + Vec3::new(i as f32 * 8.0, 0.0, 0.0)),
            GlobalTransform::default(),
            Building {
                building_type: BuildingType::Generic,
            },
            ElectricalComponent {
                base_load: 2.0,
                current_load: 2.0,
                max_transfer: 2.0,
                capacity: 0.0,
            },
        ));
    }
}

pub fn reset_ind_e03_grid_overload_seed_on_enter_simulation(
    mut seed: ResMut<IndE03GridOverloadSeedState>,
) {
    *seed = IndE03GridOverloadSeedState::default();
}

pub fn seed_ind_e03_grid_overload_witness_once(
    launch: Option<Res<crate::engine::launch_args::EngineLaunchArgs>>,
    base: Res<State<crate::engine::states::BaseState>>,
    chain: Res<ConcreteChainE2eWitness>,
    mut seed: ResMut<IndE03GridOverloadSeedState>,
    mut commands: Commands,
    flow: Option<Res<ResourceFlowSimWitness>>,
) {
    if !matches!(base.get(), crate::engine::states::BaseState::Simulation) {
        return;
    }
    if seed.seeded || !ind_e03_grid_seed_enabled(launch.as_deref()) {
        return;
    }
    if !chain.production_green() {
        return;
    }
    if !seed.cluster_spawned {
        spawn_ind_e03_grid_overload_cluster(
            &mut commands,
            BuildSiteTile { x: 50, z: 50 },
        );
        seed.cluster_spawned = true;
        info!(
            target: "economy::activation::ind_e03",
            "IND-E03: spawned grid overload cluster for witness depth"
        );
        return;
    }
    if flow.as_ref().is_some_and(|f| f.overload_events_total > 0) {
        seed.seeded = true;
        info!(
            target: "economy::activation::ind_e03",
            "IND-E03: grid overload witness depth green (overload_events_total={})",
            flow.as_ref().map(|f| f.overload_events_total).unwrap_or(0)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::{default_buildings_dir, load_building_definitions_from_dir};
    use crate::economy::activation::activate_industrial_facilities_system;
    use crate::economy::resource_flow::{
        link_supply_chain_edges_system, register_resource_flow_nodes_system,
    };
    use crate::economy::ResourceFlowPlugin;
    use crate::entities::production::concrete::{
        AggregateMineRuntime, CementKilnRuntime, ConcreteMixerRuntime,
    };
    use crate::systems::sim_control::SimControlState;

    fn assemble_chain_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(load_building_definitions_from_dir(default_buildings_dir()));
        app.insert_resource(SimControlState {
            paused: false,
            steps_remaining: 0,
            speed: 1.0,
        });
        app.init_resource::<ConcreteChainE2eWitness>();
        app.add_plugins(ResourceFlowPlugin);
        app.add_systems(
            Update,
            (
                activate_industrial_facilities_system,
                register_resource_flow_nodes_system,
                link_supply_chain_edges_system,
                refresh_concrete_chain_e2e_witness_system,
            )
                .chain(),
        );
        app
    }

    #[test]
    fn concrete_portland_chain_e2e_operational_production_tick() {
        let mut app = assemble_chain_app();
        spawn_concrete_portland_chain_operational(
            &mut app.world_mut().commands(),
            BuildSiteTile { x: 16, z: 16 },
        );
        for _ in 0..24 {
            app.update();
        }

        let world = app.world_mut();
        let mut mine_q = world.query_filtered::<Entity, With<AggregateMineRuntime>>();
        let mine = mine_q
            .iter(world)
            .next()
            .expect("mine runtime");
        let mut kiln_q = world.query_filtered::<Entity, With<CementKilnRuntime>>();
        let kiln = kiln_q
            .iter(world)
            .next()
            .expect("kiln runtime");
        let mut mixer_q = world.query_filtered::<Entity, With<ConcreteMixerRuntime>>();
        let mixer = mixer_q
            .iter(world)
            .next()
            .expect("mixer runtime");
        assert_ne!(mine, kiln);
        assert_ne!(kiln, mixer);

        let flow = world.resource::<ResourceFlowRegistry>();
        assert!(
            flow.edges.len() >= 2,
            "expected mine→kiln and kiln→mixer edges, got {}",
            flow.edges.len()
        );

        let w = world.resource::<ConcreteChainE2eWitness>();
        assert!(w.chain_operational(), "witness: {w:?}");
        assert!(
            w.production_green(),
            "expected production tick + flow edges, witness: {w:?}"
        );
        assert!(!w.placed_via_construction);
    }

    #[test]
    fn concrete_portland_chain_in_play_commit_and_production() {
        use bevy::ecs::system::RunSystemOnce;

        let prior_instant = std::env::var("RUST_ENGINE_CONSTRUCTION_INSTANT").ok();
        std::env::set_var("RUST_ENGINE_CONSTRUCTION_INSTANT", "1");

        let mut app = assemble_chain_app();
        app.init_resource::<SiteConstructionBook>()
            .init_resource::<crate::strategic::SiteIdIssuer>()
            .add_message::<CommitConstructionSiteEvent>()
            .add_systems(
                Update,
                (
                    crate::strategic::commit_construction_site_system,
                    fast_forward_portland_chain_sites_to_operational,
                )
                    .chain()
                    .before(activate_industrial_facilities_system),
            );

        let owner = app.world_mut().spawn_empty().id();
        app.world_mut()
            .run_system_once(
                move |mut writer: MessageWriter<CommitConstructionSiteEvent>,
                      mut witness: ResMut<ConcreteChainE2eWitness>| {
                    commit_concrete_portland_chain_in_play(
                        &mut writer,
                        witness.as_mut(),
                        owner,
                        BuildSiteTile { x: 24, z: 24 },
                    );
                },
            )
            .expect("commit portland chain");

        for _ in 0..32 {
            app.update();
        }

        let w = app.world().resource::<ConcreteChainE2eWitness>();
        assert!(w.placed_via_construction);
        assert!(w.in_play_green(), "witness: {w:?}");

        match prior_instant {
            Some(v) => std::env::set_var("RUST_ENGINE_CONSTRUCTION_INSTANT", v),
            None => {
                let _ = std::env::remove_var("RUST_ENGINE_CONSTRUCTION_INSTANT");
            }
        }
    }
}
