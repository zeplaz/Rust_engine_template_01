//! A long, expensive Dam site must receive its material bill through the same order-sim path as a
//! small factory site (regression: trucks reached the dam and never finished unloading).

use bevy::asset::AssetPlugin;
use bevy::prelude::*;

use crate::construction::{default_buildings_dir, load_building_definitions_from_dir, SiteStageTickPlugin};
use crate::economy::logistics::orders::{LogisticsOrderSiteId, LogisticsOrdersPlugin, LogisticsOrdersSnapshotRes};
use crate::strategic::{
    BuildSiteTile, CommitConstructionSiteEvent, ConstructionSite, FootprintTiles, LayerType, SiteArchetype,
    SiteConstructionBook, SiteIdIssuer, SiteResourceManifest, StrategicFieldsPlugin,
};
use crate::systems::sim_control::{SimControlState, SimTick, SimTimeMicros};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::systems::terrain::MaterialUnificationPlugin;

fn frame(app: &mut App) {
    app.world_mut().resource_mut::<SimTick>().0 += 1;
    app.world_mut().resource_mut::<SimTimeMicros>().0 += 1_000_000;
    app.update();
}

#[test]
fn dam_site_receives_its_material_bill() {
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
        .add_systems(Update, crate::economy::ensure_site_world_transform_system);
    let mut start = crate::ai::faction::FactionStartConfig::load_default().expect("shipped start config");
    start.factions.retain(|f| f.slot == 0);
    start.factions[0].hq = crate::ai::faction::start::HqPlacement::Tile { x: 0, z: 0 };
    app.insert_resource(start);
    app.add_plugins(crate::ai::faction::FactionStartPlugin);
    for _ in 0..3 {
        frame(&mut app);
    }
    let owner = app.world_mut().spawn_empty().id();
    app.world_mut().write_message(CommitConstructionSiteEvent {
        site_id: crate::strategic::SiteId::UNASSIGNED,
        owner,
        archetype: SiteArchetype::Dam,
        origin: BuildSiteTile { x: 14, z: 8 },
        footprint: FootprintTiles { width: 12, depth: 2 },
        layer: LayerType::Surface,
        catalog_id: Some("dam_embankment_v1".to_string()),
        placement: None,
    });
    for _ in 0..400 {
        frame(&mut app);
    }
    let w = app.world_mut();
    let mut q = w.query::<(&ConstructionSite, &SiteResourceManifest, &LogisticsOrderSiteId)>();
    let (site, manifest, sid) = q.iter(w).next().expect("dam site registered with logistics");
    assert!(manifest.concrete_required >= 5000.0, "dam carries a large bill: {manifest:?}");
    assert!(site.phase != crate::strategic::SiteConstructionPhase::Operational);
    let sid = sid.0;
    let manifest_ratio = manifest.min_delivered_ratio();
    let snap = &w.resource::<LogisticsOrdersSnapshotRes>().0;
    let delivered: u32 = snap.orders.iter().filter(|o| o.sink == sid).map(|o| o.delivered).sum();
    assert!(delivered > 0, "trucks finish unloading at the dam: {:?}", snap.orders);
    assert!(manifest_ratio > 0.0, "deliveries land in the manifest");
}

/// The dam finishes on the stock its scenario gives it: deliveries stop at the bill instead of
/// refilling the site's stock to the full bill after each build-in (regression: the standing
/// line kept refilling, the HQ ran dry and the dam stalled at ~77 %).
#[test]
fn dam_finishes_without_over_delivery() {
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
        .add_systems(Update, crate::economy::ensure_site_world_transform_system);
    let mut start = crate::ai::faction::FactionStartConfig::load_default().expect("shipped start config");
    start.factions.retain(|f| f.slot == 0);
    start.factions[0].hq = crate::ai::faction::start::HqPlacement::Tile { x: 0, z: 0 };
    // Exactly the dam's bill plus one truck load of slack per material.
    start.factions[0].stockpile = vec![
        crate::ai::faction::start::StockLine { cargo: "Concrete".into(), amount: 5600 + 40 },
        crate::ai::faction::start::StockLine { cargo: "Steel".into(), amount: 2240 + 40 },
        crate::ai::faction::start::StockLine { cargo: "Fuel".into(), amount: 1120 + 40 },
        crate::ai::faction::start::StockLine { cargo: "Machinery".into(), amount: 560 + 40 },
    ];
    app.insert_resource(start);
    app.add_plugins(crate::ai::faction::FactionStartPlugin);
    for _ in 0..3 {
        frame(&mut app);
    }
    let owner = app.world_mut().spawn_empty().id();
    app.world_mut().write_message(CommitConstructionSiteEvent {
        site_id: crate::strategic::SiteId::UNASSIGNED,
        owner,
        archetype: SiteArchetype::Dam,
        origin: BuildSiteTile { x: 14, z: 8 },
        footprint: FootprintTiles { width: 12, depth: 2 },
        layer: LayerType::Surface,
        catalog_id: Some("dam_embankment_v1".to_string()),
        placement: None,
    });
    let mut done = false;
    for _ in 0..20_000 {
        frame(&mut app);
        let w = app.world_mut();
        let mut q = w.query::<&ConstructionSite>();
        if q.iter(w).any(|s| s.phase == crate::strategic::SiteConstructionPhase::Operational) {
            done = true;
            break;
        }
    }
    let w = app.world_mut();
    let mut q = w.query::<(&ConstructionSite, &SiteResourceManifest)>();
    let (site, manifest) = q.iter(w).next().expect("dam site");
    assert!(done, "the dam finishes on its bill: phase {:?}, {manifest:?}", site.phase);
}
