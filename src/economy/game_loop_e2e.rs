//! Regression test for the core simulation loop:
//! place building → site publishes material demands → logistics delivers → site consumes and
//! advances stages (stalls while starved) → operational → industrial activation → plant output
//! becomes order stock → output feeds the construction of the next building.
//!
//! The faction's `AutoSupplyPolicy` (on by default headless) issues the supply lines.
//!
//! Real systems throughout: `commit_construction_site_system` (the construction funnel),
//! `StrategicFieldsPlugin` site sets, `SiteStageTickPlugin`, `LogisticsOrdersPlugin`,
//! `activate_industrial_facilities_system` and `ResourceFlowPlugin`.

use bevy::asset::AssetPlugin;
use bevy::prelude::*;

use crate::construction::{
    default_buildings_dir, load_building_definitions_from_dir, SiteStageProgress, SiteStageTickPlugin,
};
use crate::economy::activation::{
    activate_industrial_facilities_system, BuildingDefinitionRef, IndustrialFacilityActivated,
};
use crate::economy::logistics::orders::construction_demand::CONSTRUCTION_MATERIALS;
use crate::economy::logistics::orders::{
    CargoKind, ConsumerKind, CargoClassMask, LogisticsOrderCommand, LogisticsOrderRequest,
    LogisticsOrderSite, LogisticsOrderSiteId, LogisticsOrdersPlugin, LogisticsOrdersSnapshotRes,
    LogisticsSiteRegistry, OwnerId, SiteRole,
};
use crate::economy::facility_inputs::FacilityInputs;
use crate::economy::resource_flow::{ResourceFlowNode, ResourceFlowPlugin};
use crate::strategic::{
    BuildSiteTile, CommitConstructionSiteEvent, ConstructionSite, FootprintTiles, LayerType,
    SiteArchetype, SiteConstructionBook, SiteMaterialStarved, SiteConstructionPhase, SiteIdIssuer, SiteResourceManifest,
    StrategicFieldsPlugin,
};
use crate::systems::sim_control::{SimControlState, SimTick, SimTimeMicros};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::systems::terrain::MaterialUnificationPlugin;

fn loop_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_resource::<WorldGenParams>()
        .add_plugins(MaterialUnificationPlugin)
        .init_resource::<SimControlState>()
        .init_resource::<SimTick>()
        .init_resource::<SimTimeMicros>()
        .init_resource::<SiteConstructionBook>()
        .init_resource::<SiteIdIssuer>()
        .insert_resource(load_building_definitions_from_dir(default_buildings_dir()))
        .add_plugins(StrategicFieldsPlugin)
        .add_plugins(SiteStageTickPlugin)
        .add_plugins(LogisticsOrdersPlugin)
        .add_plugins(ResourceFlowPlugin)
        .add_systems(
            Update,
            (
                crate::economy::ensure_site_world_transform_system,
                activate_industrial_facilities_system
                    .after(crate::strategic::site_provisioning_system),
                crate::infrastructure::sync_utility_connection_power_system,
            ),
        );
    seed_town_and_grid(&mut app);
    app
}

/// A town near the build area (workforce) and a power graph feeding both build tiles.
fn seed_town_and_grid(app: &mut App) {
    use crate::infrastructure::utility::{UtilityGraph, UtilityGraphEdge, UtilityGraphNode};
    use crate::strategic::settlement::TownAccessPolicy;
    use crate::strategic::{TownBook, TownId, TownRecord};
    let id = TownId("e2e_town".into());
    app.world_mut().resource_mut::<TownBook>().towns.insert(
        id.clone(),
        TownRecord {
            id,
            name: "E2E".into(),
            region_name: "E2E".into(),
            center_tile: IVec2::new(5, 5),
            population: 2_000,
            jobs: 100,
            housing: 1_000,
            faction: 1,
            access: TownAccessPolicy::FreeLabor,
            required_clearance: 0,
        },
    );
    let keys = ["power_plant_main", "pwr_160_0", "pwr_0_160", "pwr_160_160"];
    let nodes = keys
        .iter()
        .enumerate()
        .map(|(i, k)| UtilityGraphNode {
            id: 1 + i as u64,
            key: (*k).into(),
            position: Vec2::new(i as f32, 0.0),
        })
        .collect();
    let power_edges = [2_u64, 3, 4]
        .iter()
        .map(|&to| UtilityGraphEdge {
            from: 1,
            to,
            link_id: 100 + to,
            throughput: 1.0,
        })
        .collect();
    app.insert_resource(UtilityGraph {
        nodes,
        power_edges,
        hydrated_from_snapshot: true,
    });
}

/// One sim second: tick counter and sim clock advance, then one frame.
fn frame(app: &mut App) {
    app.world_mut().resource_mut::<SimTick>().0 += 1;
    app.world_mut().resource_mut::<SimTimeMicros>().0 += 1_000_000;
    app.update();
}

fn frames(app: &mut App, n: u32) {
    for _ in 0..n {
        frame(app);
    }
}

fn place(app: &mut App, catalog_id: &str, x: u32, z: u32) {
    let owner = app.world_mut().spawn_empty().id();
    app.world_mut().write_message(CommitConstructionSiteEvent {
        site_id: crate::strategic::SiteId::UNASSIGNED,
        owner,
        archetype: SiteArchetype::Factory,
        origin: BuildSiteTile { x, z },
        footprint: FootprintTiles { width: 2, depth: 2 },
        layer: LayerType::Surface,
        catalog_id: Some(catalog_id.to_string()),
        placement: None,
    });
}

fn site_entity(app: &mut App, catalog_id: &str) -> Option<Entity> {
    let world = app.world_mut();
    world
        .query::<(Entity, &BuildingDefinitionRef)>()
        .iter(world)
        .find(|(_, d)| d.catalog_id == catalog_id)
        .map(|(e, _)| e)
}

fn phase(app: &App, e: Entity) -> SiteConstructionPhase {
    app.world().get::<ConstructionSite>(e).expect("site").phase
}

fn stock_of(app: &App, e: Entity, cargo: &str) -> u32 {
    let sid = app.world().get::<LogisticsOrderSiteId>(e).expect("registered site").0;
    let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
    snap.site(sid)
        .and_then(|s| s.stock.iter().find(|r| r.cargo == cargo))
        .map_or(0, |r| r.on_hand)
}

fn depot_stock(app: &mut App, depot: Entity, cargo: &str, n: u32) {
    let site = app.world().resource::<LogisticsSiteRegistry>().site_of(depot).expect("depot id");
    app.world_mut().write_message(LogisticsOrderRequest(LogisticsOrderCommand::SetStock {
        site,
        cargo: CargoKind::new(cargo),
        on_hand: n,
    }));
}

#[test]
fn building_logistics_construction_production_logistics_loop_closes() {
    let mut app = loop_app();
    let depot = app
        .world_mut()
        .spawn((
            LogisticsOrderSite {
                name: "Depot".into(),
                owner: OwnerId::PLAYER,
                role: SiteRole::Depot,
                position_m: None,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();
    frames(&mut app, 3);

    // 1. Player places a materials import terminal through the commit funnel.
    place(&mut app, "materials_import_terminal", 10, 0);
    frames(&mut app, 2);
    let terminal = site_entity(&mut app, "materials_import_terminal").expect("commit spawned a site");

    // 2. The site publishes one draw demand per material of its bill.
    frames(&mut app, 3);
    let manifest = app.world().get::<SiteResourceManifest>(terminal).expect("manifest").clone();
    assert!(manifest.concrete_required > 0.0, "committed site carries a material bill: {manifest:?}");
    {
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        let n = snap.demands.iter().filter(|d| d.consumer == ConsumerKind::Construction).count();
        assert_eq!(n, 4, "site publishes a construction demand per material: {:?}", snap.demands);
    }

    // 3. Starved: the depot is empty, so the site must not finish however long we wait.
    frames(&mut app, 80);
    assert_ne!(phase(&app, terminal), SiteConstructionPhase::Operational, "starved site must stall");
    assert_ne!(phase(&app, terminal), SiteConstructionPhase::Provisioning);
    assert!(app.world().get::<SiteStageProgress>(terminal).is_some());
    {
        let starved = app.world().get::<SiteMaterialStarved>(terminal);
        assert!(starved.is_some(), "starvation is visible on the site");
        assert_eq!(starved.unwrap().missing.len(), 4, "all four materials are missing");
    }

    // 4. Starting stockpile arrives at the depot; logistics delivers; the site builds out, using
    // material progressively (steel is consumed while the stage is still running).
    for tag in CONSTRUCTION_MATERIALS {
        depot_stock(&mut app, depot, tag, 500);
    }
    let mut saw_partial_consumption = false;
    let mut guard = 0;
    while phase(&app, terminal) != SiteConstructionPhase::Operational && guard < 600 {
        frame(&mut app);
        guard += 1;
        let m = app.world().get::<SiteResourceManifest>(terminal).unwrap();
        if phase(&app, terminal) == SiteConstructionPhase::UnderConstruction
            && m.consumed_steel > 0.0
            && m.consumed_steel < m.steel_required
        {
            saw_partial_consumption = true;
        }
    }
    assert_eq!(
        phase(&app, terminal),
        SiteConstructionPhase::Operational,
        "terminal finished after deliveries (frames={guard}, manifest={:?})",
        app.world().get::<SiteResourceManifest>(terminal)
    );
    assert!(saw_partial_consumption, "materials are consumed progressively, not at phase exit");
    assert!(app.world().get::<SiteMaterialStarved>(terminal).is_none());

    // 5. Operational: activation attaches the facility + flow node; demands retract.
    frames(&mut app, 4);
    assert!(app.world().get::<IndustrialFacilityActivated>(terminal).is_some());
    assert!(app.world().get::<ResourceFlowNode>(terminal).is_some(), "flow node registered");
    {
        let sid = app.world().get::<LogisticsOrderSiteId>(terminal).unwrap().0;
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        assert_eq!(snap.site(sid).unwrap().role, SiteRole::Plant, "finished building is a plant endpoint");
        assert!(
            snap.demands.iter().all(|d| d.site != sid || d.consumer != ConsumerKind::Construction),
            "construction demands retracted"
        );
    }
    // Labour and power come from the town and the grid, not from a buffer.
    let inputs = *app.world().get::<FacilityInputs>(terminal).expect("facility inputs");
    assert_eq!(inputs, FacilityInputs { workforce: 1.0, power: 1.0 }, "{inputs:?}");

    // 6. Production lands in order stock: the terminal makes the materials nothing else does.
    frames(&mut app, 30);
    assert!(stock_of(&app, terminal, "Steel") > 0, "plant output is visible to logistics as stock");

    // 7. The starting stockpile of steel/machinery/fuel is gone; the next building (a concrete
    // mixer) has to be built from the terminal's output.
    for tag in ["Steel", "Fuel", "Machinery"] {
        depot_stock(&mut app, depot, tag, 0);
    }
    frames(&mut app, 2);
    place(&mut app, "concrete_mixer_plant", 0, 10);
    frames(&mut app, 4);
    let mixer = site_entity(&mut app, "concrete_mixer_plant").expect("second site");
    let mut guard = 0;
    while phase(&app, mixer) != SiteConstructionPhase::Operational && guard < 1500 {
        frame(&mut app);
        guard += 1;
    }
    assert_eq!(
        phase(&app, mixer),
        SiteConstructionPhase::Operational,
        "mixer built from terminal output (frames={guard}, manifest={:?})",
        app.world().get::<SiteResourceManifest>(mixer)
    );
    let terminal_id = app.world().get::<LogisticsOrderSiteId>(terminal).unwrap().0;
    {
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        assert!(
            snap.orders.iter().any(|o| o.source == terminal_id && o.cargo == "Steel" && o.delivered > 0),
            "steel for the mixer was hauled from the terminal: {:?}",
            snap.orders
        );
    }

    // 8. The mixer's own inputs publish demands, the depot (starting stock) feeds them, and its
    // concrete output builds the next building: the loop closes.
    frames(&mut app, 4);
    {
        let sid = app.world().get::<LogisticsOrderSiteId>(mixer).unwrap().0;
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        assert!(
            snap.demands.iter().any(|d| d.site == sid && d.consumer == ConsumerKind::Plant),
            "operational plant publishes input demands: {:?}",
            snap.demands
        );
    }
    depot_stock(&mut app, depot, "Concrete", 0);
    // A bigger fleet: the mixer eats four hauled inputs per unit of concrete.
    let depot_site = app.world().resource::<LogisticsSiteRegistry>().site_of(depot).unwrap();
    for _ in 0..16 {
        app.world_mut().write_message(LogisticsOrderRequest(LogisticsOrderCommand::AddVehicle {
            owner: OwnerId::PLAYER,
            home: depot_site,
            class_mask: CargoClassMask::FREIGHT,
            capacity_units: 10,
            speed_mps: 12.0,
        }));
    }
    for tag in ["Cement", "Gravel"] {
        depot_stock(&mut app, depot, tag, 2000);
    }
    frames(&mut app, 2);
    place(&mut app, "concrete_cement_kiln", 10, 10);
    frames(&mut app, 4);
    let kiln = site_entity(&mut app, "concrete_cement_kiln").expect("third site");
    let mut guard = 0;
    while phase(&app, kiln) != SiteConstructionPhase::Operational && guard < 2500 {
        frame(&mut app);
        guard += 1;
    }
    assert_eq!(
        phase(&app, kiln),
        SiteConstructionPhase::Operational,
        "kiln built from plant concrete (frames={guard}, manifest={:?})",
        app.world().get::<SiteResourceManifest>(kiln)
    );
    let mixer_id = app.world().get::<LogisticsOrderSiteId>(mixer).unwrap().0;
    let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
    assert!(
        snap.orders.iter().any(|o| o.source == mixer_id && o.cargo == "Concrete" && o.delivered > 0),
        "concrete for the kiln was hauled from the mixer: {:?}",
        snap.orders
    );
    assert!(app.world().get::<SiteMaterialStarved>(kiln).is_none(), "no stall flag once built");
}
