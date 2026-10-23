//! Regression test for the core simulation loop:
//! place building → site publishes material demands → logistics delivers → site consumes and
//! advances stages (stalls while starved) → operational → industrial activation → plant output
//! becomes order stock → output feeds the construction of the next building.
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
    suggest_lines_for_unserved, CargoKind, ConsumerKind, LogisticsOrderCommand, LogisticsOrderRequest,
    LogisticsOrderSite, LogisticsOrderSiteId, LogisticsOrdersPlugin, LogisticsOrdersSnapshotRes,
    LogisticsSiteRegistry, OwnerId, SiteRole,
};
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
            ),
        );
    app
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

/// Same as pressing "Auto-supply" in the logistics panel.
fn auto_supply(app: &mut App) {
    let snap = app.world().resource::<LogisticsOrdersSnapshotRes>().0.clone();
    for cmd in suggest_lines_for_unserved(&snap, OwnerId::PLAYER).commands {
        app.world_mut().write_message(LogisticsOrderRequest(cmd));
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

    // 1. Player places a concrete mixer through the commit funnel.
    place(&mut app, "concrete_mixer_plant", 10, 0);
    frames(&mut app, 2);
    let mixer = site_entity(&mut app, "concrete_mixer_plant").expect("commit spawned a site");

    // 2. The site publishes one draw demand per material of its bill.
    frames(&mut app, 3);
    let manifest = app.world().get::<SiteResourceManifest>(mixer).expect("manifest").clone();
    assert!(manifest.concrete_required > 0.0, "committed site carries a material bill: {manifest:?}");
    {
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        let n = snap.demands.iter().filter(|d| d.consumer == ConsumerKind::Construction).count();
        assert_eq!(n, 4, "site publishes a construction demand per material: {:?}", snap.demands);
    }

    // 3. Starved: the depot is empty, so the site must not finish however long we wait.
    frames(&mut app, 40);
    auto_supply(&mut app);
    frames(&mut app, 40);
    assert_ne!(phase(&app, mixer), SiteConstructionPhase::Operational, "starved site must stall");
    assert_ne!(phase(&app, mixer), SiteConstructionPhase::Provisioning);
    assert!(app.world().get::<SiteStageProgress>(mixer).is_some());
    assert!(
        app.world().get::<SiteMaterialStarved>(mixer).is_some(),
        "starvation is visible on the site"
    );

    // 4. Fill the depot; logistics delivers; the site builds out.
    for tag in CONSTRUCTION_MATERIALS {
        depot_stock(&mut app, depot, tag, 500);
    }
    let mut guard = 0;
    while phase(&app, mixer) != SiteConstructionPhase::Operational && guard < 600 {
        if guard % 5 == 0 {
            auto_supply(&mut app);
        }
        frame(&mut app);
        guard += 1;
    }
    assert_eq!(
        phase(&app, mixer),
        SiteConstructionPhase::Operational,
        "site finished after deliveries (frames={guard}, manifest={:?})",
        app.world().get::<SiteResourceManifest>(mixer)
    );

    // 5. Operational: activation attaches the facility + flow node; demands retract, stock consumed.
    frames(&mut app, 4);
    assert!(app.world().get::<IndustrialFacilityActivated>(mixer).is_some());
    assert!(app.world().get::<ResourceFlowNode>(mixer).is_some(), "flow node registered");
    for tag in ["Steel", "Fuel", "Machinery"] {
        assert_eq!(stock_of(&app, mixer, tag), 0, "{tag} consumed into the building");
    }
    // The mixer already produces Concrete, so its stock is output, not the delivered bill.
    assert!(
        (stock_of(&app, mixer, "Concrete") as f32) < manifest.concrete_required / 2.0,
        "delivered concrete consumed into the building"
    );
    {
        let sid = app.world().get::<LogisticsOrderSiteId>(mixer).unwrap().0;
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        assert_eq!(snap.site(sid).unwrap().role, SiteRole::Plant, "finished building is a plant endpoint");
        assert!(
            snap.demands.iter().all(|d| d.site != sid || d.consumer != ConsumerKind::Construction),
            "construction demands retracted"
        );
    }

    // 6a. Production lands in order stock.
    frames(&mut app, 30);
    let made = stock_of(&app, mixer, "Concrete");
    assert!(made > 0, "plant output is visible to logistics as stock");

    // 6b. Inputs publish draw demands.
    {
        let sid = app.world().get::<LogisticsOrderSiteId>(mixer).unwrap().0;
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        assert!(
            snap.demands.iter().any(|d| d.site == sid && d.consumer != ConsumerKind::Construction),
            "operational plant publishes input demands: {:?}",
            snap.demands
        );
    }

    // 7. Close the loop: the next building's concrete can only come from the plant.
    depot_stock(&mut app, depot, "Concrete", 0);
    // The depot also stocks the mixer's inputs, which the plant's own demands pull in.
    for tag in ["Cement", "Water", "Gravel", "Chemicals"] {
        depot_stock(&mut app, depot, tag, 2000);
    }
    frames(&mut app, 2);
    place(&mut app, "concrete_cement_kiln", 0, 10);
    frames(&mut app, 4);
    let kiln = site_entity(&mut app, "concrete_cement_kiln").expect("second site");
    let mut guard = 0;
    while phase(&app, kiln) != SiteConstructionPhase::Operational && guard < 2000 {
        if guard % 5 == 0 {
            auto_supply(&mut app);
        }
        frame(&mut app);
        guard += 1;
    }
    assert_eq!(
        phase(&app, kiln),
        SiteConstructionPhase::Operational,
        "second building built from plant concrete (frames={guard}, manifest={:?})",
        app.world().get::<SiteResourceManifest>(kiln)
    );
    let plant_id = app.world().get::<LogisticsOrderSiteId>(mixer).unwrap().0;
    let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
    assert!(
        snap.orders.iter().any(|o| o.source == plant_id && o.cargo == "Concrete" && o.delivered > 0),
        "concrete for the second building was hauled from the plant: {:?}",
        snap.orders
    );
    assert!(app.world().get::<SiteMaterialStarved>(kiln).is_none(), "no stall flag once built");
}
