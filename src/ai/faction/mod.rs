//! **Faction strategic AI** (`PLAN-AI-FLEET-v1`) — perceive → decide → plan → act → log.
//!
//! - Perception is a pure fold over [`ChunkStrategicOverlay`] ([`perception::perceive_faction`]).
//! - Goal choice is plain-weight scoring with named terms ([`decide::choose_goal`]).
//! - Three brains turn the goal into orders, each through the API the player uses:
//!   [`economy`] places buildings via `CommitConstructionSiteEvent`, [`logistics`] keeps sites and
//!   fronts supplied via `LogisticsOrderRequest`, [`military`] sends formations to pressure points
//!   via `DeploymentOrderQueue::enqueue`. The AI never writes overlay fields, graphs, `Transform`,
//!   or occupation.
//! - Every decision lands in [`FactionAiDecisionLog`] with its goal scores, each brain's top scoring
//!   terms, and the API outcome of each order.
//! - Cadence is the sim tick (`SimTick`, `SimControlState::should_tick`) — no camera, no LOD gate —
//!   and each faction decides once per `decision_interval_ticks`, staggered by slot.
//! - Randomness is [`rng::DetRng`] seeded from session seed + slot + tick (FAI-2).

pub mod decide;
pub mod economy;
pub mod log;
pub mod logistics;
pub mod military;
pub mod orders;
pub mod perception;
pub mod rng;
pub mod start;

#[cfg(test)]
mod scenario;

use std::collections::HashSet;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::combat::formation::Formation;
use crate::combat::orders::{DeploymentKind, DeploymentOrder, DeploymentOrderQueue};
use crate::combat::pressure::PressurePointBook;
use crate::combat::theater::TheaterId;
use crate::construction::BuildingDefinitionRegistry;
use crate::economy::activation::BuildingDefinitionRef;
use crate::economy::logistics::orders::construction_demand::site_accepts_materials;
use crate::economy::logistics::orders::{
    AutoSupplyPolicy, FactionSlot, LogisticsOrderRequest, LogisticsOrderSite, LogisticsOrdersSnapshot, LogisticsOrdersSnapshotRes, OwnerId,
    SiteRole,
};
use crate::strategic::{
    evaluate_site_placement_at_world_tile, BuildSiteTile, ChunkStrategicOverlay,
    CommitConstructionSiteEvent, ConstructionSite, FootprintTiles, LayerType, SiteConstructionPhase, SiteFootprint,
    SiteId, StrategicCommandQueue, StrategicFieldPipeline, StrategicRasterConfig,
};
use crate::systems::sim_control::{
    SimControlState, SimControlSystemSet, SimStepStamp, SimTick, SimTimeMicros,
};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

pub use decide::{choose_goal, FactionDoctrine, GoalChoice, GoalScore, ScoreTerm, StrategicGoal};
pub use log::{DecisionTrace, FactionAiDecisionLog, FactionAiDecisionRecord, PerceptionDigest};
pub use orders::{
    plan_orders, AiOrder, BuildReason, DeployKind, FormationRef, OrderOutcome, MAX_ORDERS_PER_DECISION,
};
pub use perception::{perceive_faction, FactionPerception, FrontCell};
pub use rng::{AiSeed, DetRng};
pub use start::{FactionHq, FactionStartConfig, FactionStartPlugin};

/// Default sim ticks between two decisions of one faction.
pub const DEFAULT_DECISION_INTERVAL_TICKS: u64 = 60;

/// Interim front/logistics surface for apps without the real order APIs.
pub const API_STRATEGIC_COMMAND_QUEUE: &str = "StrategicCommandQueue";
/// The front's order queue (`DeploymentOrderQueue::enqueue`).
pub const API_DEPLOYMENT_QUEUE: &str = "DeploymentOrderQueue";
/// The logistics order message (`LogisticsOrderRequest`).
pub const API_LOGISTICS_ORDERS: &str = "LogisticsOrderRequest";
/// The construction commit funnel (`CommitConstructionSiteEvent`).
pub const API_CONSTRUCTION_COMMIT: &str = "CommitConstructionSiteEvent";
/// A logistics endpoint registered by marker component, like a front supply point.
pub const API_LOGISTICS_SITE: &str = "LogisticsOrderSite";

/// World metres per tile for logistics endpoint placement (`economy::site_placement`).
const METRES_PER_TILE: f32 = crate::economy::logistics::orders::snapshot::METRES_PER_TILE;

/// Ordering label for partners that consume AI orders the same frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FactionAiSet {
    Tick,
}

/// One AI-controlled faction. Lives on the faction kernel entity (`strategic::Faction`) or any
/// entity standing in for the faction slot; that entity is the `owner` of the construction sites the
/// faction commits. Sole writer: [`faction_ai_tick_system`].
#[derive(Component, Clone, Debug)]
pub struct FactionAiController {
    /// Overlay faction slot (`faction_control[slot]`). The player keeps slot 0 without a controller.
    pub slot: u8,
    pub doctrine: FactionDoctrine,
    pub decision_interval_ticks: u64,
    pub next_decision_tick: u64,
    pub goal: Option<StrategicGoal>,
    pub enabled: bool,
    /// Construction sites the faction may have open at once.
    pub max_active_sites: u32,
    /// Sim ticks between two placements.
    pub build_cooldown_ticks: u64,
    pub next_build_tick: u64,
    /// The faction's first depot has been registered (or found to exist).
    pub depot_founded: bool,
    /// Logistics orders issued recently, so a demand is not ordered twice before the snapshot updates.
    pub memory: logistics::LogisticsMemory,
    pub military: military::MilitaryParams,
}

impl FactionAiController {
    #[must_use]
    pub fn new(slot: u8, doctrine: FactionDoctrine) -> Self {
        Self {
            slot,
            doctrine,
            decision_interval_ticks: DEFAULT_DECISION_INTERVAL_TICKS,
            next_decision_tick: 0,
            goal: None,
            enabled: true,
            max_active_sites: economy::DEFAULT_MAX_ACTIVE_SITES,
            build_cooldown_ticks: economy::DEFAULT_BUILD_COOLDOWN_TICKS,
            next_build_tick: 0,
            depot_founded: false,
            memory: logistics::LogisticsMemory::default(),
            military: military::MilitaryParams::default(),
        }
    }
}

/// Which AI factions the session runs. Empty by default (no AI); a session setup system inserts the
/// slots and [`seed_faction_ai_controllers_system`] spawns one controller entity per missing slot.
#[derive(Resource, Clone, Debug, Default)]
pub struct FactionAiRoster {
    pub slots: Vec<(u8, FactionDoctrine)>,
}

/// Spawn a controller for every roster slot that has none, staggering first decisions by slot so
/// factions do not all think in the same frame.
pub fn seed_faction_ai_controllers_system(
    mut commands: Commands,
    roster: Option<Res<FactionAiRoster>>,
    existing: Query<&FactionAiController>,
) {
    let Some(roster) = roster else { return };
    for (slot, doctrine) in &roster.slots {
        if existing.iter().any(|c| c.slot == *slot) {
            continue;
        }
        let mut c = FactionAiController::new(*slot, *doctrine);
        c.next_decision_tick = u64::from(*slot) * 13 % c.decision_interval_ticks.max(1);
        commands.spawn((Name::new(format!("AI faction {slot}")), c));
    }
}

/// Every player-facing enqueue surface the AI may use. Each is optional so the AI degrades to
/// `Deferred` instead of panicking when an API is not in the app.
#[derive(SystemParam)]
pub struct AiOrderSinks<'w> {
    pub command_queue: Option<ResMut<'w, StrategicCommandQueue>>,
    pub deployments: Option<ResMut<'w, DeploymentOrderQueue>>,
    pub logistics: Option<ResMut<'w, Messages<LogisticsOrderRequest>>>,
    pub commits: Option<ResMut<'w, Messages<CommitConstructionSiteEvent>>>,
}

impl AiOrderSinks<'_> {
    /// Send one interim order through the strategic command queue (used only when the order's own
    /// API is not in the app).
    pub fn issue(&mut self, slot: u8, order: &AiOrder, stamp: SimStepStamp) -> OrderOutcome {
        let Some(queue) = self.command_queue.as_deref_mut() else {
            return OrderOutcome::Deferred {
                reason: "no order API in this app",
            };
        };
        let summary = format!(
            "[ai faction {slot}] {}: {}",
            order.corridor_mission(),
            order.describe()
        );
        let msg = queue.enqueue_strategic(stamp, summary);
        OrderOutcome::Fallback {
            api: API_STRATEGIC_COMMAND_QUEUE,
            id: msg.command_id,
        }
    }

    /// Does the real API behind a legacy interim order exist in this app? If so a brain owns it.
    fn real_api_for(&self, order: &AiOrder) -> bool {
        match order {
            AiOrder::Reinforce { .. } | AiOrder::Attack { .. } => self.deployments.is_some(),
            AiOrder::SupplyLine { .. } => self.logistics.is_some(),
            AiOrder::Develop { .. } => self.commits.is_some(),
            _ => true,
        }
    }
}

/// Read-only world the brains look at.
#[derive(SystemParam)]
pub struct AiWorld<'w, 's> {
    pub overlays: Query<'w, 's, &'static ChunkStrategicOverlay>,
    pub pressure: Option<Res<'w, PressurePointBook>>,
    /// Executor feedback: refused orders are not re-issued.
    pub outcomes: Option<Res<'w, crate::combat::DeploymentOutcomes>>,
    pub formations: Query<'w, 's, (Entity, &'static Formation)>,
    pub snapshot: Option<Res<'w, LogisticsOrdersSnapshotRes>>,
    pub auto_supply: Option<Res<'w, AutoSupplyPolicy>>,
    pub catalog: Option<Res<'w, BuildingDefinitionRegistry>>,
    pub sites: Query<'w, 's, (&'static ConstructionSite, Option<&'static BuildingDefinitionRef>)>,
    pub footprints: Query<'w, 's, &'static SiteFootprint>,
    pub raster: Option<Res<'w, StrategicRasterConfig>>,
    pub ai_seed: Option<Res<'w, AiSeed>>,
    pub world_gen: Option<Res<'w, WorldGenParams>>,
    pub hqs: Query<'w, 's, &'static FactionHq>,
    pub start: Option<Res<'w, FactionStartConfig>>,
}

/// Per-decision scratch: what a decision has ordered and why.
#[derive(Default)]
struct Decision {
    orders: Vec<(AiOrder, OrderOutcome)>,
    traces: Vec<DecisionTrace>,
}

impl Decision {
    fn push(
        &mut self,
        brain: &'static str,
        subject: String,
        total: f32,
        terms: Vec<ScoreTerm>,
        order: Option<(AiOrder, OrderOutcome)>,
    ) {
        let idx = order.map(|o| {
            self.orders.push(o);
            self.orders.len() - 1
        });
        self.traces.push(DecisionTrace { brain, subject, total, terms, order: idx });
    }
}

fn overlay_cell(ov: &ChunkStrategicOverlay, tile: IVec2) -> Option<usize> {
    let (sx, sy) = (ov.size.x as i32, ov.size.y as i32);
    if sx <= 0 || sy <= 0 {
        return None;
    }
    let lx = tile.x - ov.chunk_coord.x.checked_mul(sx)?;
    let ly = tile.y - ov.chunk_coord.y.checked_mul(sy)?;
    if lx < 0 || ly < 0 || lx >= sx || ly >= sy {
        return None;
    }
    let i = ly as usize * sx as usize + lx as usize;
    (i < ov.len_cells()).then_some(i)
}

/// `(own control, strongest enemy control, own threat)` at a tile, when an overlay covers it.
fn cell_stance(overlays: &Query<&ChunkStrategicOverlay>, tile: IVec2, slot: u8) -> Option<(f32, f32, f32)> {
    let s = slot as usize;
    overlays.iter().find_map(|ov| {
        let i = overlay_cell(ov, tile)?;
        let control = ov.faction_control.get(i)?;
        let own = *control.get(s)?;
        let enemy = control.iter().enumerate().filter(|(o, _)| *o != s).fold(0.0_f32, |m, (_, v)| m.max(*v));
        Some((own, enemy, ov.threat.get(i).map_or(0.0, |t| t[s])))
    })
}

fn footprint_tiles(origin: IVec2, fp: (u32, u32), margin: i32) -> impl Iterator<Item = (i32, i32)> {
    let (w, d) = (fp.0 as i32, fp.1 as i32);
    (-margin..d + margin).flat_map(move |dy| (-margin..w + margin).map(move |dx| (origin.x + dx, origin.y + dy)))
}

fn run_logistics_brain(
    c: &mut FactionAiController,
    goal: StrategicGoal,
    now: u64,
    snap: &LogisticsOrdersSnapshot,
    auto_supply: Option<&AutoSupplyPolicy>,
    sinks: &mut AiOrderSinks,
    out: &mut Decision,
) {
    let Some(messages) = sinks.logistics.as_deref_mut() else { return };
    c.memory.prune(now);
    let owner = OwnerId(c.slot);
    let ttl = c.decision_interval_ticks.max(1).saturating_mul(3);
    // Standing lines are the auto-supply pass's job when it is on for this faction.
    let lines = !auto_supply.is_some_and(|p| p.enabled(owner));
    for scored in logistics::plan_logistics(snap, owner, &c.doctrine, goal, &c.memory, now, lines) {
        messages.write(LogisticsOrderRequest(scored.action.to_command(owner)));
        let order = match &scored.action {
            logistics::LogisticsAction::Line { source, sink, cargo, priority } => {
                c.memory.note(*sink, cargo, false, now + ttl);
                AiOrder::StandingLine {
                    source: source.0,
                    sink: sink.0,
                    cargo: cargo.clone(),
                    priority: priority.label(),
                }
            }
            logistics::LogisticsAction::Run { source, sink, cargo, quantity, priority } => {
                c.memory.note(*sink, cargo, true, now + ttl);
                AiOrder::SupplyRun {
                    source: source.0,
                    sink: sink.0,
                    cargo: cargo.clone(),
                    quantity: *quantity,
                    priority: priority.label(),
                }
            }
        };
        out.push(
            "logistics",
            scored.subject,
            scored.total,
            scored.terms,
            Some((order, OrderOutcome::Submitted { api: API_LOGISTICS_ORDERS })),
        );
    }
}

/// A formation whose order was refused sits out new orders this long (sim seconds).
const REJECT_WINDOW_S: f32 = 60.0;

fn run_military_brain(
    c: &FactionAiController,
    goal: StrategicGoal,
    perception: &FactionPerception,
    snap: Option<&LogisticsOrdersSnapshot>,
    world: &AiWorld,
    now_micros: u64,
    sinks: &mut AiOrderSinks,
    out: &mut Decision,
) {
    let (Some(queue), Some(book)) = (sinks.deployments.as_deref_mut(), world.pressure.as_deref()) else {
        return;
    };
    if !book.points.iter().any(|p| p.side == c.slot) {
        return;
    }
    let mut formations: Vec<military::FormationView> = world
        .formations
        .iter()
        .filter(|(_, f)| f.faction_slot == c.slot)
        .map(|(e, f)| military::FormationView::of(e, f))
        .collect();
    formations.retain(|f| world.outcomes.as_deref().is_none_or(|o| o.recently_rejected(f.entity, now_micros, REJECT_WINDOW_S).is_none()));
    if formations.is_empty() {
        return;
    }
    formations.sort_by_key(|f| f.entity.to_bits());
    let fill = snap
        .and_then(|s| logistics::front_supply_fill(s, OwnerId(c.slot)))
        .unwrap_or(perception.mean_logistics_held);
    for d in military::plan_deployments(c.slot, goal, &c.doctrine, &book.points, &formations, fill, &c.military) {
        let kind = match d.kind {
            DeployKind::Reinforce => DeploymentKind::Reinforce { theater: TheaterId(d.theater), tile: d.tile },
            DeployKind::Attack => DeploymentKind::Attack(d.tile),
            DeployKind::Fortify => DeploymentKind::Fortify(TheaterId(d.theater)),
        };
        let outcome = match queue.enqueue(DeploymentOrder { formation: d.formation, kind, issued_by: c.slot }) {
            Ok(id) => OrderOutcome::Issued { api: API_DEPLOYMENT_QUEUE, id },
            Err(e) => OrderOutcome::Refused { reason: format!("{e:?}") },
        };
        let order = AiOrder::Deploy {
            formation: FormationRef(d.formation),
            kind: d.kind,
            theater: d.theater,
            tile: d.tile,
        };
        out.push("military", d.subject, d.total, d.terms, Some((order, outcome)));
    }
}

#[allow(clippy::too_many_arguments)]
fn run_economy_brain(
    entity: Entity,
    c: &mut FactionAiController,
    goal: StrategicGoal,
    perception: &FactionPerception,
    now: u64,
    session: u64,
    snap: &LogisticsOrdersSnapshot,
    rows: &[economy::CatalogRow],
    world: &AiWorld,
    sinks: &mut AiOrderSinks,
    commands: &mut Commands,
    reserved: &mut Vec<(i32, i32)>,
    out: &mut Decision,
) {
    let owner = OwnerId(c.slot);
    // The faction needs somewhere for supplies to come from before it can build anything.
    if !c.depot_founded {
        if snap.sites_of(owner, SiteRole::Depot).next().is_some() || world.hqs.iter().any(|h| h.slot == c.slot) {
            // The faction's HQ (start config) is its first depot; nothing to found.
            c.depot_founded = true;
        } else if world.start.as_deref().is_some_and(|cfg| cfg.factions.iter().any(|f| f.slot == c.slot)) {
            // The start config gives this faction an HQ; it appears once the faction holds ground.
            return;
        } else if let Some(tile) = perception.rear_anchor {
            let centre = (tile.as_vec2() + Vec2::splat(0.5)) * METRES_PER_TILE;
            commands.spawn((
                Name::new(format!("AI depot {}", c.slot)),
                Transform::from_xyz(centre.x, 0.0, centre.y),
                LogisticsOrderSite {
                    name: format!("AI depot {}", c.slot),
                    owner,
                    role: SiteRole::Depot,
                    position_m: Some([centre.x, centre.y]),
                },
            ));
            c.depot_founded = true;
            out.push(
                "economy",
                "found depot".to_string(),
                1.0,
                vec![ScoreTerm { label: "no depot", input: 1.0, weight: 1.0 }],
                Some((AiOrder::FoundDepot { tile }, OrderOutcome::Submitted { api: API_LOGISTICS_SITE })),
            );
            return;
        }
    }
    if now < c.next_build_tick {
        return;
    }
    let mut owned: Vec<economy::OwnedBuilding> = Vec::new();
    let mut active = 0u32;
    for (site, def) in world.sites.iter().filter(|(s, _)| s.owner == entity) {
        let operational = site.phase == SiteConstructionPhase::Operational;
        if site_accepts_materials(site.phase) {
            active += 1;
        }
        if let Some(def) = def {
            owned.push(economy::OwnedBuilding { catalog_id: def.catalog_id.clone(), operational });
        }
    }
    if active >= c.max_active_sites {
        return;
    }
    let view = economy::EconomyView { owner, snapshot: snap, catalog: rows, owned: &owned };
    let wants = economy::rank_builds(&view, &c.doctrine, goal);
    let Some(want) = wants.into_iter().next() else { return };
    if want.total < economy::BUILD_MIN_SCORE {
        out.push("economy", format!("{} — below threshold", want.subject), want.total, want.terms, None);
        return;
    }
    let Some(anchor) = economy::depot_anchor(snap, owner).or(perception.rear_anchor) else {
        out.push("economy", format!("{} — no anchor", want.subject), want.total, want.terms, None);
        return;
    };

    let mut occupied: HashSet<(i32, i32)> = reserved.iter().copied().collect();
    for fp in world.footprints.iter() {
        occupied.extend(fp.tiles.iter().map(|t| (t.x, t.y)));
    }
    // Depots and other endpoints are not construction sites but still stand on their tile.
    occupied.extend(snap.sites.iter().map(|s| (s.tile[0], s.tile[1])));
    let slot = c.slot;
    let fp = want.footprint;
    let mut rng = DetRng::for_decision(session, slot, now, rng::SALT_ECONOMY);
    let chosen = economy::choose_origin(anchor, &mut rng, |origin| {
        if footprint_tiles(origin, fp, 1).any(|t| occupied.contains(&t)) {
            return None;
        }
        let centre = origin + IVec2::new(fp.0 as i32 / 2, fp.1 as i32 / 2);
        let mut bonus = 0.0;
        if let Some((own, enemy, threat)) = cell_stance(&world.overlays, centre, slot) {
            if threat > 0.6 || enemy > own + 0.1 {
                return None;
            }
            bonus = 0.5 * own - 0.5 * threat;
        }
        let report = evaluate_site_placement_at_world_tile(
            BuildSiteTile { x: origin.x as u32, z: origin.y as u32 },
            FootprintTiles { width: fp.0, depth: fp.1 },
            world.raster.as_deref(),
            &world.overlays,
        );
        report
            .allows_commit
            .then(|| report.terrain_score + report.logistics_score + report.strategic_score + bonus)
    });
    let Some((origin, placement_score)) = chosen else {
        out.push("economy", format!("{} — no valid placement", want.subject), want.total, want.terms, None);
        return;
    };
    let Some(commits) = sinks.commits.as_deref_mut() else { return };
    commits.write(CommitConstructionSiteEvent {
        site_id: SiteId::UNASSIGNED,
        owner: entity,
        archetype: want.archetype,
        origin: BuildSiteTile { x: origin.x as u32, z: origin.y as u32 },
        footprint: FootprintTiles { width: fp.0, depth: fp.1 },
        layer: LayerType::Surface,
        catalog_id: Some(want.catalog_id.clone()),
        placement: None,
    });
    reserved.extend(footprint_tiles(origin, fp, 0));
    c.next_build_tick = now.saturating_add(c.build_cooldown_ticks);
    let mut terms = want.terms;
    terms.push(ScoreTerm { label: "placement", input: placement_score, weight: 0.1 });
    let total = terms.iter().map(ScoreTerm::value).sum();
    out.push(
        "economy",
        want.subject,
        total,
        terms,
        Some((
            AiOrder::Build { catalog_id: want.catalog_id, origin, footprint: fp, reason: want.reason },
            OrderOutcome::Submitted { api: API_CONSTRUCTION_COMMIT },
        )),
    );
}

/// Perceive, decide, plan, and act for every faction whose interval elapsed.
#[allow(clippy::too_many_arguments)]
pub fn faction_ai_tick_system(
    mut commands: Commands,
    tick: Option<Res<SimTick>>,
    time: Option<Res<SimTimeMicros>>,
    ctrl: Option<Res<SimControlState>>,
    mut controllers: Query<(Entity, &mut FactionAiController)>,
    mut log: ResMut<FactionAiDecisionLog>,
    mut sinks: AiOrderSinks,
    world: AiWorld,
) {
    if ctrl.as_deref().is_some_and(|c| !c.should_tick()) {
        return;
    }
    let now = tick.map_or(0, |t| t.0);
    let stamp = SimStepStamp::new(now, time.map_or(0, |t| t.0));

    let mut due: Vec<(Entity, Mut<FactionAiController>)> = controllers
        .iter_mut()
        .filter(|(_, c)| c.enabled && now >= c.next_decision_tick)
        .collect();
    if due.is_empty() {
        return;
    }
    due.sort_by_key(|(_, c)| c.slot);

    let session = rng::session_seed(world.ai_seed.as_deref(), world.world_gen.as_deref());
    let catalog_rows = world.catalog.as_deref().map(economy::catalog_rows).unwrap_or_default();
    let snap: Option<&LogisticsOrdersSnapshot> = world.snapshot.as_deref().map(|s| &s.0);
    let mut reserved: Vec<(i32, i32)> = Vec::new();

    for (entity, mut c) in due {
        let slot = c.slot;
        let perception = perceive_faction(slot, world.overlays.iter());
        let previous = c.goal;
        let choice = choose_goal(&perception, &c.doctrine, previous);
        let goal = choice.chosen;
        let mut out = Decision::default();

        // Interim orders only where the order's own API is not in the app.
        for order in plan_orders(goal, &perception) {
            if sinks.real_api_for(&order) {
                continue;
            }
            let outcome = sinks.issue(slot, &order, stamp);
            out.orders.push((order, outcome));
        }

        if let Some(snap) = snap {
            run_logistics_brain(&mut c, goal, now, snap, world.auto_supply.as_deref(), &mut sinks, &mut out);
            if sinks.commits.is_some() && !catalog_rows.is_empty() {
                run_economy_brain(
                    entity, &mut c, goal, &perception, now, session, snap, &catalog_rows, &world, &mut sinks,
                    &mut commands, &mut reserved, &mut out,
                );
            }
        }
        run_military_brain(&c, goal, &perception, snap, &world, stamp.sim_time_micros, &mut sinks, &mut out);

        c.goal = Some(goal);
        c.next_decision_tick = now.saturating_add(c.decision_interval_ticks.max(1));
        log.push(FactionAiDecisionRecord {
            sim_tick: now,
            slot,
            goal,
            previous_goal: previous,
            choice,
            digest: PerceptionDigest {
                cells_held: perception.cells_held,
                front_cells: perception.front.len() as u32,
                mean_front_threat: perception.mean_front_threat(),
                mean_front_advantage: perception.mean_front_advantage(),
                front_supply_shortfall: perception.front_supply_shortfall(),
                mean_logistics_held: perception.mean_logistics_held,
            },
            orders: out.orders,
            traces: out.traces,
        });
    }
}

/// The faction's controller entity is the `owner` of the sites it commits; its [`FactionSlot`] is how
/// the logistics lane maps those sites onto the faction's own order books (no after-the-fact patching).
pub fn ensure_controller_faction_slot_system(
    mut commands: Commands,
    controllers: Query<(Entity, &FactionAiController), Without<FactionSlot>>,
) {
    for (entity, c) in &controllers {
        commands.entity(entity).insert(FactionSlot(c.slot));
    }
}

pub struct FactionAiPlugin;

impl Plugin for FactionAiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FactionAiDecisionLog>()
            .init_resource::<AiSeed>()
            .init_resource::<FactionAiRoster>()
            .configure_sets(
                Update,
                FactionAiSet::Tick
                    .after(SimControlSystemSet::AdvanceSimTick)
                    .after(StrategicFieldPipeline::ZoneAndReadModel),
            )
            .add_systems(
                Update,
                (
                    seed_faction_ai_controllers_system,
                    ensure_controller_faction_slot_system,
                    faction_ai_tick_system,
                )
                    .chain()
                    .in_set(FactionAiSet::Tick),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::perception::test_support::two_faction_strip;
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SimTick>()
            .init_resource::<SimTimeMicros>()
            .init_resource::<SimControlState>()
            .init_resource::<StrategicCommandQueue>()
            .add_plugins(FactionAiPlugin);
        app.world_mut().spawn(two_faction_strip());
        app
    }

    #[test]
    fn fai_tick_decides_on_interval_and_queues_orders() {
        let mut app = app();
        let mut c = FactionAiController::new(1, FactionDoctrine::default());
        c.decision_interval_ticks = 10;
        app.world_mut().spawn(c);
        for t in 0..25u64 {
            app.world_mut().resource_mut::<SimTick>().0 = t;
            app.update();
        }
        let log = app.world().resource::<FactionAiDecisionLog>();
        assert_eq!(log.total_decisions, 3, "ticks 0, 10, 20");
        let rec = log.latest_for(1).expect("record");
        assert!(!rec.explain().is_empty());
        let reached = rec.orders.iter().filter(|(_, o)| o.reached_api()).count();
        assert_eq!(reached, rec.orders.len());
        let queue = app.world().resource::<StrategicCommandQueue>();
        assert!(queue.pending_count() > 0 || !queue.delivered.is_empty() || rec.orders.is_empty());
    }

    #[test]
    fn fai_paused_sim_does_not_decide() {
        let mut app = app();
        app.world_mut().resource_mut::<SimControlState>().paused = true;
        app.world_mut().spawn(FactionAiController::new(1, FactionDoctrine::default()));
        app.update();
        assert_eq!(app.world().resource::<FactionAiDecisionLog>().total_decisions, 0);
    }

    #[test]
    fn fai_missing_api_defers_instead_of_panicking() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(FactionAiPlugin);
        let mut ov = two_faction_strip();
        ov.threat[1][1] = 0.9;
        app.world_mut().spawn(ov);
        app.world_mut().spawn(FactionAiController::new(1, FactionDoctrine::default()));
        app.update();
        let log = app.world().resource::<FactionAiDecisionLog>();
        let rec = log.latest_for(1).expect("record");
        assert!(rec
            .orders
            .iter()
            .all(|(_, o)| matches!(o, OrderOutcome::Deferred { .. })));
    }

    #[test]
    fn fai_roster_spawns_one_staggered_controller_per_slot() {
        let mut app = app();
        app.insert_resource(FactionAiRoster {
            slots: vec![(1, FactionDoctrine::default()), (2, FactionDoctrine::default())],
        });
        app.update();
        app.update();
        let mut cs: Vec<(u8, u64)> = app
            .world_mut()
            .query::<&FactionAiController>()
            .iter(app.world())
            .map(|c| (c.slot, c.decision_interval_ticks))
            .collect();
        cs.sort_unstable();
        assert_eq!(cs, vec![(1, 60), (2, 60)]);
        let starts: HashSet<u64> = app
            .world_mut()
            .query::<&FactionAiController>()
            .iter(app.world())
            .map(|c| c.next_decision_tick)
            .collect();
        assert!(starts.len() <= 2);
    }
}
