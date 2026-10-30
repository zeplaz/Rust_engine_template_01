//! Headless scenario: one AI faction plays through the player's command APIs.
//!
//! Real systems throughout — the construction commit funnel (`commit_construction_site_system`),
//! site stage tick, logistics order sim, industrial activation and resource flow. The test only
//! plays the role of the world around the AI: the overlay it stands on, the stock its depot is
//! imported with, and the pressure book the front lane would publish.

use bevy::asset::AssetPlugin;
use bevy::prelude::*;

use super::*;
use crate::combat::balance::UnitKind;
use crate::combat::balance::FrontBalance;
use crate::combat::formation::Posture;
use crate::combat::front::{FrontCell as FrontTile, FrontSegment};
use crate::combat::orders::{execute_deployment_orders_system, DeploymentOrderQueue, DeploymentOutcomes};
use crate::combat::theater::TheaterBook;
use crate::combat::pressure::{PressureDrivers, PressureKind, PressurePoint, PressurePointBook};
use crate::construction::{default_buildings_dir, load_building_definitions_from_dir, SiteStageTickPlugin};
use crate::economy::activation::{activate_industrial_facilities_system, IndustrialFacilityActivated};
use crate::economy::logistics::orders::construction_demand::CONSTRUCTION_MATERIALS;
use crate::economy::logistics::orders::{
    CargoKind, LogisticsOrderCommand, LogisticsOrderSiteId, LogisticsOrdersPlugin, LogisticsOrdersSnapshotRes, SiteId,
};
use crate::economy::resource_flow::{ResourceFlowNode, ResourceFlowPlugin};
use crate::strategic::{SiteConstructionBook, SiteIdIssuer, StrategicFieldsPlugin};
use crate::systems::terrain::MaterialUnificationPlugin;

const SLOT: u8 = 1;
const INTERVAL: u64 = 10;
const FRAMES: u32 = 1200;

fn scenario_app(seed: u64) -> App {
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
        .insert_resource(AiSeed(seed))
        .init_resource::<DeploymentOrderQueue>()
        .init_resource::<PressurePointBook>()
        .init_resource::<DeploymentOutcomes>()
        .insert_resource(theater_book())
        .add_plugins(StrategicFieldsPlugin)
        .add_plugins(SiteStageTickPlugin)
        .add_plugins(LogisticsOrdersPlugin)
        .add_plugins(ResourceFlowPlugin)
        .add_plugins(FactionAiPlugin)
        .add_systems(
            Update,
            (
                crate::economy::ensure_site_world_transform_system,
                execute_deployment_orders_system,
                activate_industrial_facilities_system.after(crate::strategic::site_provisioning_system),
            ),
        );
    app
}

/// One theater along x = 28 where slot 1 is the defender.
fn theater_book() -> TheaterBook {
    let cells = (0..10)
        .map(|y| FrontTile { tile: IVec2::new(28, y), owner: SLOT, facing: 0, contested: false, margin: 0.5 })
        .collect();
    let seg = FrontSegment { pair: (0, 1), cells, polylines: vec![], contested_cells: 0, length_m: 0.0 };
    let mut book = TheaterBook::default();
    book.update(&[seg], &FrontBalance::default().theater, 25.0, 1);
    book
}

/// The territory the AI stands on: slot 1 holds a 32x32 chunk with decent logistics.
fn territory() -> ChunkStrategicOverlay {
    let mut ov = ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(32, 32));
    for i in 0..ov.len_cells() {
        ov.faction_control[i][SLOT as usize] = 0.9;
        ov.logistics_strength[i][SLOT as usize] = 0.6;
    }
    ov
}

fn frame(app: &mut App) {
    app.world_mut().resource_mut::<SimTick>().0 += 1;
    app.world_mut().resource_mut::<SimTimeMicros>().0 += 1_000_000;
    app.update();
}

/// What the world around the AI does: import stock into its depot once the depot exists.
fn import_stock_once(app: &mut App, done: &mut bool) {
    if *done {
        return;
    }
    let depot = {
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        snap.sites_of(OwnerId(SLOT), SiteRole::Depot).map(|s| s.id).next()
    };
    let Some(site) = depot else { return };
    for tag in CONSTRUCTION_MATERIALS.iter().chain(["Cement", "Water", "Gravel", "Chemicals"].iter()) {
        app.world_mut().write_message(LogisticsOrderRequest(LogisticsOrderCommand::SetStock {
            site,
            cargo: CargoKind::new(*tag),
            on_hand: 2000,
        }));
    }
    *done = true;
}

struct Outcome {
    /// Every decision record of the run (the log itself is a ring buffer).
    history: Vec<FactionAiDecisionRecord>,
    /// Every order the order sim ever held: `(id, owner, sink, cargo, max delivered)` (finished orders leave the snapshot).
    orders_seen: std::collections::BTreeMap<u32, (OwnerId, SiteId, String, u32)>,
    app: App,
    faction: Entity,
    formations: Vec<Entity>,
    theater: crate::combat::theater::TheaterId,
}

fn run(seed: u64, frames: u32) -> Outcome {
    let mut app = scenario_app(seed);
    app.world_mut().spawn(territory());
    let mut c = FactionAiController::new(SLOT, FactionDoctrine::default());
    c.decision_interval_ticks = INTERVAL;
    c.build_cooldown_ticks = 40;
    let faction = app.world_mut().spawn(c).id();

    // Two spare formations behind the line and a threatened point the front lane would have scored.
    let formations: Vec<Entity> = [(20, 20), (22, 20)]
        .into_iter()
        .map(|(x, y)| app.world_mut().spawn(Formation::new(SLOT, UnitKind::Infantry, IVec2::new(x, y))).id())
        .collect();
    let theater = app.world().resource::<TheaterBook>().theaters[0].id;
    let drivers = PressureDrivers { threat: 1.0, contest: 0.3, supply: 0.2, ..PressureDrivers::default() };
    app.world_mut().resource_mut::<PressurePointBook>().points = vec![PressurePoint {
        theater,
        side: SLOT,
        kind: PressureKind::Threatened,
        tile: IVec2::new(28, 4),
        score: drivers.total(),
        drivers,
    }];

    let mut imported = false;
    let mut history: Vec<FactionAiDecisionRecord> = Vec::new();
    let mut seen = 0u64;
    let mut orders_seen = std::collections::BTreeMap::new();
    for _ in 0..frames {
        import_stock_once(&mut app, &mut imported);
        frame(&mut app);
        let log = app.world().resource::<FactionAiDecisionLog>();
        let fresh = (log.total_decisions - seen) as usize;
        history.extend(log.records[log.records.len() - fresh..].iter().cloned());
        seen = log.total_decisions;
        for o in &app.world().resource::<LogisticsOrdersSnapshotRes>().0.orders {
            let e = orders_seen.entry(o.id.0).or_insert((o.owner, o.sink, o.cargo.clone(), 0));
            e.3 = e.3.max(o.delivered);
        }
    }
    Outcome { history, orders_seen, app, faction, formations, theater }
}

fn site_of(out: &mut Outcome) -> Option<(Entity, String)> {
    let world = out.app.world_mut();
    let faction = out.faction;
    world
        .query::<(Entity, &ConstructionSite, &BuildingDefinitionRef)>()
        .iter(world)
        .find(|(_, s, _)| s.owner == faction)
        .map(|(e, _, d)| (e, d.catalog_id.clone()))
}

#[test]
fn ai_faction_builds_supplies_produces_and_deploys() {
    let mut out = run(7, FRAMES);

    // 1. It placed a building through the commit funnel.
    let (site, catalog_id) = site_of(&mut out).expect("the AI committed a construction site");
    let log = out.app.world().resource::<FactionAiDecisionLog>();
    let build = out
        .history
        .iter()
        .flat_map(|r| r.orders.iter())
        .find(|(o, _)| matches!(o, AiOrder::Build { .. }))
        .expect("a Build order was logged");
    assert_eq!(build.1, OrderOutcome::Submitted { api: API_CONSTRUCTION_COMMIT });
    assert!(matches!(&build.0, AiOrder::Build { catalog_id: id, .. } if *id == catalog_id));
    assert!(
        out.history
            .iter()
            .flat_map(|r| r.orders.iter())
            .any(|(o, _)| matches!(o, AiOrder::FoundDepot { .. })),
        "it registered its first depot"
    );

    // 2. The site is the faction's own logistics endpoint and logistics supplied it from its depot.
    let endpoint = out.app.world().get::<LogisticsOrderSite>(site).expect("site is an order endpoint");
    assert_eq!(endpoint.owner, OwnerId(SLOT), "the AI's site is on the AI's books, not the player's");
    let sid = out.app.world().get::<LogisticsOrderSiteId>(site).expect("registered").0;
    {
        let lines: Vec<_> = out.orders_seen.values().filter(|(_, sink, _, _)| *sink == sid).collect();
        assert!(!lines.is_empty(), "the faction supplied its site: {:?}", out.orders_seen);
        assert!(lines.iter().all(|(owner, ..)| *owner == OwnerId(SLOT)), "on the AI's books: {lines:?}");
        assert!(lines.iter().any(|(.., delivered)| *delivered > 0), "and the deliveries arrived: {lines:?}");
        assert!(
            out.orders_seen.values().all(|(owner, ..)| *owner == OwnerId(SLOT)),
            "every order in this world is the AI's"
        );
    }

    // 3. The building went up and produces.
    let phase = out.app.world().get::<ConstructionSite>(site).expect("site").phase;
    assert_eq!(phase, SiteConstructionPhase::Operational, "site finished (log: {:?})", log.latest_for(SLOT));
    assert!(out.app.world().get::<IndustrialFacilityActivated>(site).is_some());
    assert!(out.app.world().get::<ResourceFlowNode>(site).is_some(), "flow node registered");
    {
        let reg = out.app.world().resource::<BuildingDefinitionRegistry>();
        let produces = reg.get(&catalog_id).expect("catalog row").produces.clone();
        let snap = &out.app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        let stock: u32 = snap
            .site(sid)
            .map(|s| {
                s.stock
                    .iter()
                    .filter(|r| produces.iter().any(|p| p.eq_ignore_ascii_case(&r.cargo)))
                    .map(|r| r.on_hand)
                    .sum()
            })
            .unwrap_or(0);
        assert!(stock > 0, "{catalog_id} produced {produces:?} into order stock");
    }

    // 4. It sent its formations toward the pressure point through the player's order queue, once.
    let deploys: Vec<_> = out
        .history
        .iter()
        .flat_map(|r| r.orders.iter())
        .filter(|(o, _)| matches!(o, AiOrder::Deploy { .. }))
        .collect();
    assert!(!deploys.is_empty(), "the AI issued a deployment");
    assert!(
        deploys.iter().all(|(_, o)| matches!(o, OrderOutcome::Issued { api: API_DEPLOYMENT_QUEUE, .. })),
        "{deploys:?}"
    );
    assert!(deploys.len() <= out.formations.len(), "no re-issue spam: {} deploys", deploys.len());
    for e in &out.formations {
        let f = out.app.world().get::<Formation>(*e).expect("formation");
        assert_eq!(f.faction_slot, SLOT);
        assert_eq!(f.target, Some(IVec2::new(28, 4)), "the executor accepted the order: {f:?}");
        assert_eq!(f.posture, Posture::Move);
    }
    let _ = out.theater;

    // 5. Every decision is explained.
    let rec = out
        .history
        .iter()
        .find(|r| r.orders.iter().any(|(o, _)| matches!(o, AiOrder::Build { .. })))
        .expect("the build decision");
    let lines = rec.explain().join("\n");
    assert!(lines.contains("[economy]") && lines.contains("build "), "{lines}");
    let trace = rec.traces.iter().find(|t| t.brain == "economy" && t.order.is_some()).expect("economy trace");
    assert!(trace.top_terms(3).len() == 3 && trace.terms.iter().any(|t| t.label == "build cost"));
    let mil = out
        .history
        .iter()
        .flat_map(|r| r.traces.iter())
        .find(|t| t.brain == "military")
        .expect("military trace");
    assert!(mil.terms.iter().any(|t| t.label == "threat" && t.value() > 0.0));
}

#[test]
fn decisions_are_throttled_to_the_interval() {
    let out = run(7, 100);
    // Ticks 1..=100, one decision per 10 ticks.
    let n = out.history.len();
    assert!((9..=11).contains(&n), "{n} decisions in 100 ticks");
}

fn digest(out: &Outcome) -> Vec<String> {
    let mut lines: Vec<String> = out
        .history
        .iter()
        .flat_map(|r| {
            r.orders
                .iter()
                .map(move |(o, outcome)| format!("t{} s{} {} => {}", r.sim_tick, r.slot, o.describe(), outcome.describe()))
        })
        .collect();
    lines.extend(out.formations.iter().map(|e| {
        let f = out.app.world().get::<Formation>(*e).expect("formation");
        format!("formation {:?} {:?} -> {:?}", f.tile, f.posture, f.target)
    }));
    lines
}

#[test]
fn same_seed_same_orders() {
    let a = digest(&run(42, 600));
    let b = digest(&run(42, 600));
    assert!(!a.is_empty());
    assert_eq!(a, b);
}

#[test]
fn seed_only_affects_tie_breaks_not_the_plan() {
    // Different seeds may place the building on a different tile but never change what is built.
    let kinds = |seed| {
        digest(&run(seed, 300))
            .into_iter()
            .filter(|l| l.contains("build "))
            .map(|l| l.split(" at ").next().unwrap_or("").to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(kinds(1), kinds(2));
}
