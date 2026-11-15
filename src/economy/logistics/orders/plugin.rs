//! Bevy adapter for the order sim (plan §2, LO-3).
//!
//! Sole writers:
//! - [`step_logistics_orders_system`] → [`LogisticsOrderSimRes`], [`LogisticsSiteRegistry`]
//! - [`publish_logistics_orders_snapshot_system`] → [`LogisticsOrdersSnapshotRes`]
//!
//! Player UI, AI and fronts write only [`LogisticsOrderRequest`] messages and read only the
//! snapshot. Any system can make an entity a logistics endpoint by inserting
//! [`LogisticsOrderSite`]; the adapter assigns a [`SiteId`] ([`LogisticsOrderSiteId`]).

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;

use crate::economy::resource_flow::{resource_type_from_tag, ResourceFlowNode};
use crate::entities::production::core::{
    LogisticsSiteRoot, ManufacturingNode, ManufacturingOutputBuffers,
};
use crate::entities::types::p_enumz::ResourceType;
use crate::systems::sim_control::{SimControlSystemSet, SimStepClock};

use super::model::{CargoClassMask, CargoKind, OwnerId, SiteId};
use super::sim::{CommandOutcome, LogisticsOrderCommand, LogisticsOrderSim, SimConfig};
use super::snapshot::{LogisticsOrdersSnapshot, SiteMeta, SiteRole};
use super::persistence::{
    read_logistics_orders_overlay, saved_site_matches, write_logistics_orders_overlay,
    LogisticsOrdersSave, SavedSite, LOGISTICS_ORDERS_SAVE_SCHEMA,
};
use super::travel::SiteDistanceTravel;

/// Aggregate-tier order-sim step cadence (period 2; EXEC-01 phase 0 — even ticks vs pressure odd).
pub const LOGISTICS_ORDER_AGGREGATE_PERIOD: u64 = 2;
pub const LOGISTICS_ORDER_AGGREGATE_PHASE: u64 = 0;

/// Marks an entity as an order endpoint. Insert from any lane (fronts, construction, depots).
///
/// Position: `position_m` when set, else the entity's `Transform` translation XZ, which must be
/// in **metres** (16 m per tile, as `economy::site_placement::site_world_position` writes).
#[derive(Component, Clone, Debug, PartialEq, Default)]
pub struct LogisticsOrderSite {
    pub name: String,
    pub owner: OwnerId,
    pub role: SiteRole,
    /// Explicit world XZ in metres (for endpoints without a metre-space `Transform`).
    pub position_m: Option<[f32; 2]>,
}

/// Assigned by the adapter once the site is registered with the sim.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogisticsOrderSiteId(pub SiteId);

/// The only write path into the order sim (player UI, AI, fronts).
#[derive(Message, Clone, Debug, PartialEq)]
pub struct LogisticsOrderRequest(pub LogisticsOrderCommand);

/// Owner of the sim state. Read it only for tests/diagnostics — UI reads the snapshot.
///
/// One order-sim tick per [`SimStepClock`] step (one simulated second).
#[derive(Resource, Debug)]
pub struct LogisticsOrderSimRes {
    pub sim: LogisticsOrderSim,
    /// Outcomes of the last applied requests (most recent last, capped).
    pub last_outcomes: Vec<CommandOutcome>,
}

impl Default for LogisticsOrderSimRes {
    fn default() -> Self {
        Self {
            sim: LogisticsOrderSim::new(SimConfig::default()),
            last_outcomes: Vec::new(),
        }
    }
}

/// Entity ↔ site map plus placement for travel and snapshot.
#[derive(Resource, Debug, Default)]
pub struct LogisticsSiteRegistry {
    pub by_entity: HashMap<Entity, SiteId>,
    pub entity_of: BTreeMap<SiteId, Entity>,
    pub meta: BTreeMap<SiteId, SiteMeta>,
    pub travel: SiteDistanceTravel,
    next: u32,
    /// Saved sites awaiting their endpoint entity after a load (rebind by name/owner/role/tile).
    pub pending_rebind: Vec<SavedSite>,
    /// Frames left before unclaimed saved sites are removed from the sim.
    pub rebind_frames_left: u32,
    /// Bumped on every registry mutation (snapshot rebuild fingerprint).
    pub revision: u64,
}

/// Frames a loaded save waits for its endpoints to re-register (~10 s at 60 fps).
pub const REBIND_GRACE_FRAMES: u32 = 600;

impl LogisticsSiteRegistry {
    #[must_use]
    pub fn site_of(&self, entity: Entity) -> Option<SiteId> {
        self.by_entity.get(&entity).copied()
    }
}

/// Faction slot that owns the entity this is attached to (a construction-site owner, a hub).
/// Endpoints created for an entity take their logistics owner from it; no slot means the player.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactionSlot(pub u8);

/// Stock a depot starts with when it first registers: `(cargo, units)` in the cargo's own unit
/// (tonnes for bulk, m3 for liquids). Data-defined by the faction start config.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct DepotStartingStock(pub Vec<(String, u32)>);

/// Fleet a depot starts with instead of the global [`DepotFleetSeed`]:
/// `(class_mask, capacity_units, speed_mps, count)`.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct DepotStartingFleet(pub Vec<(CargoClassMask, u32, f32, u32)>);

fn owner_of(slots: &Query<&FactionSlot>, entity: Entity) -> OwnerId {
    slots.get(entity).map_or(OwnerId::PLAYER, |s| OwnerId(s.0))
}

/// Starting vehicles granted to each newly registered depot.
///
/// Interim until vehicle production feeds the pool (plan LO-5); `AddVehicle` requests work any time.
#[derive(Resource, Clone, Debug)]
pub struct DepotFleetSeed {
    pub enabled: bool,
    /// `(class_mask, capacity_units, speed_mps, count)`.
    pub vehicles: Vec<(CargoClassMask, u32, f32, u32)>,
}

impl Default for DepotFleetSeed {
    fn default() -> Self {
        Self {
            enabled: true,
            vehicles: vec![
                // 10 t box trucks at ~43 km/h.
                (CargoClassMask::FREIGHT, 10, 12.0, 4),
                // 20 m³ tanker at ~40 km/h.
                (CargoClassMask::TANKER, 20, 11.0, 1),
                // 12-seat troop transport at ~54 km/h.
                (CargoClassMask::TROOP, 12, 15.0, 1),
            ],
        }
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct LogisticsOrdersSnapshotRes(pub LogisticsOrdersSnapshot);

/// Last order-sim on-hand written from a plant's [`ResourceFlowNode`] buffer, per `(site, tag)`.
///
/// Sole writer: [`step_logistics_orders_system`] (via [`bridge_plant_stock`]).
#[derive(Resource, Debug, Default, Clone)]
pub struct PlantStockMirror(pub BTreeMap<(SiteId, CargoKind), u32>);

/// Mirror-delta reconcile for one plant buffer (plan LO-5b).
///
/// The node buffer is the physical warehouse. Order-sim changes since the last mirror
/// (deliveries add, pickups subtract) are applied to it as a delta; the sim then re-reads
/// the whole units on hand. First contact pushes any pre-existing sim stock into the node.
/// Returns `(node_amount, sim_on_hand, mirror)`.
#[must_use]
pub fn reconcile_plant_stock(node_amount: f32, sim_on_hand: u32, last_mirror: Option<u32>) -> (f32, u32, u32) {
    let delta = i64::from(sim_on_hand) - i64::from(last_mirror.unwrap_or(0));
    let node = (f64::from(node_amount.max(0.0)) + delta as f64).max(0.0) as f32;
    let whole = node.floor().min(u32::MAX as f32) as u32;
    (node, whole, whole)
}

/// Where a cargo tag physically lives on a [`ResourceFlowNode`]: typed resources (`Concrete`,
/// `Steel`, `Fuel`, ...) in `inventory`, designer tags (`Cement`, `Gravel`, ...) in `buffer_by_tag`.
#[derive(Clone, PartialEq, Eq, Hash)]
enum NodeSlot {
    Tag(String),
    Typed(ResourceType),
}

impl NodeSlot {
    fn of(tag: &str) -> Self {
        resource_type_from_tag(tag).map_or_else(|| Self::Tag(tag.to_string()), Self::Typed)
    }

    fn amount(&self, node: &ResourceFlowNode) -> f32 {
        match self {
            Self::Tag(t) => node.buffer_by_tag.get(t).copied().unwrap_or(0.0),
            Self::Typed(rt) => node.inventory.get(rt).copied().unwrap_or(0.0),
        }
    }

    fn set(&self, node: &mut ResourceFlowNode, value: f32) {
        match self {
            Self::Tag(t) => {
                node.buffer_by_tag.insert(t.clone(), value);
            }
            Self::Typed(rt) => {
                node.inventory.insert(*rt, value);
            }
        }
    }
}

/// Reconcile every plant site's buffers with the order sim (sole caller: step system).
///
/// Tags that alias one typed resource (`Fuel` / `Diesel`) share a slot; the first tag the node
/// itself names (production, consumption, buffer) is the one the order sim sees.
pub fn bridge_plant_stock(
    sim: &mut LogisticsOrderSim,
    registry: &LogisticsSiteRegistry,
    mirror: &mut PlantStockMirror,
    nodes: &mut Query<&mut ResourceFlowNode>,
) {
    for (&site, meta) in &registry.meta {
        if meta.role != SiteRole::Plant {
            continue;
        }
        let Some(&entity) = registry.entity_of.get(&site) else {
            continue;
        };
        let Ok(mut node) = nodes.get_mut(entity) else {
            continue;
        };
        let mut buffer_tags: Vec<String> = node.buffer_by_tag.keys().cloned().collect();
        buffer_tags.sort();
        let candidates: Vec<String> = node
            .production
            .iter()
            .chain(node.consumption.iter())
            .map(|r| r.tag.clone())
            .chain(buffer_tags)
            .chain(
                sim.stock
                    .entries
                    .keys()
                    .filter(|(s, _)| *s == site)
                    .map(|(_, c)| c.0.clone()),
            )
            .collect();
        let mut slots: Vec<(String, NodeSlot)> = Vec::new();
        for tag in candidates {
            let slot = NodeSlot::of(&tag);
            if !slots.iter().any(|(_, s)| *s == slot) {
                slots.push((tag, slot));
            }
        }
        for (tag, slot) in slots {
            let cargo = CargoKind::new(tag);
            let key = (site, cargo.clone());
            let node_amount = slot.amount(&node);
            let sim_on_hand = sim.stock.get(site, &cargo).on_hand;
            let last = mirror.0.get(&key).copied();
            if last == Some(sim_on_hand) && node_amount.floor() as u32 == sim_on_hand {
                continue;
            }
            let (node_new, sim_new, mirror_new) = reconcile_plant_stock(node_amount, sim_on_hand, last);
            if (node_new - node_amount).abs() > f32::EPSILON {
                slot.set(&mut node, node_new);
            }
            if sim_new != sim_on_hand {
                sim.apply(LogisticsOrderCommand::SetStock {
                    site,
                    cargo,
                    on_hand: sim_new,
                });
            }
            mirror.0.insert(key, mirror_new);
        }
    }
}

/// P2 — push [`ManufacturingOutputBuffers`] into order-sim plant stock (discrete UAS / munitions).
pub fn bridge_mfg_output_stock(
    sim: &mut LogisticsOrderSim,
    registry: &LogisticsSiteRegistry,
    mirror: &mut PlantStockMirror,
    buffers: &mut Query<&mut ManufacturingOutputBuffers>,
) {
    for (&site, meta) in &registry.meta {
        if meta.role != SiteRole::Plant {
            continue;
        }
        let Some(&entity) = registry.entity_of.get(&site) else {
            continue;
        };
        let Ok(mut buf) = buffers.get_mut(entity) else {
            continue;
        };
        let mut tags: Vec<String> = buf.amounts.keys().cloned().collect();
        for (s, c) in sim.stock.entries.keys() {
            if *s == site {
                tags.push(c.0.clone());
            }
        }
        tags.sort();
        tags.dedup();
        for tag in tags {
            if crate::entities::production::core::is_deployable_staging_buffer_tag(&tag) {
                continue; // deployables stay on ledger haul
            }
            let cargo = CargoKind::new(tag.clone());
            let key = (site, cargo.clone());
            let node_amount = buf.get(&tag);
            let sim_on_hand = sim.stock.get(site, &cargo).on_hand;
            let last = mirror.0.get(&key).copied();
            if last == Some(sim_on_hand) && node_amount.floor() as u32 == sim_on_hand {
                continue;
            }
            let (node_new, sim_new, mirror_new) = reconcile_plant_stock(node_amount, sim_on_hand, last);
            if (node_new - node_amount).abs() > f32::EPSILON {
                let cur = buf.get(&tag);
                if node_new > cur {
                    buf.credit(&tag, node_new - cur);
                } else if node_new < cur {
                    buf.debit(&tag, cur - node_new);
                }
            }
            if sim_new != sim_on_hand {
                sim.apply(LogisticsOrderCommand::SetStock {
                    site,
                    cargo,
                    on_hand: sim_new,
                });
            }
            mirror.0.insert(key, mirror_new);
        }
    }
}

/// Snapshot the live order state for a save.
#[must_use]
pub fn capture_logistics_orders_save(
    res: &LogisticsOrderSimRes,
    registry: &LogisticsSiteRegistry,
) -> LogisticsOrdersSave {
    let mut sites: Vec<SavedSite> = registry
        .meta
        .iter()
        .map(|(&id, meta)| SavedSite::from_meta(id, meta))
        .collect();
    // Sites still waiting for a rebind keep their saved identity.
    sites.extend(registry.pending_rebind.iter().cloned());
    sites.sort_by_key(|s| s.id);
    LogisticsOrdersSave {
        schema_version: LOGISTICS_ORDERS_SAVE_SCHEMA,
        sim: res.sim.clone(),
        sites,
    }
}

/// Replace live order state with a save. Endpoint entities must re-register afterwards
/// (the hydrate system strips their [`LogisticsOrderSiteId`]); they rebind to saved ids.
pub fn install_logistics_orders_save(
    res: &mut LogisticsOrderSimRes,
    registry: &mut LogisticsSiteRegistry,
    mirror: &mut PlantStockMirror,
    save: LogisticsOrdersSave,
) {
    res.sim = save.sim;
    res.sim.migrate_legacy_cargo_tags();
    registry.by_entity.clear();
    registry.entity_of.clear();
    registry.meta.clear();
    registry.travel.positions.clear();
    let max_saved = save.sites.iter().map(|s| s.id.0).max().unwrap_or(0);
    let max_sim = res.sim.sites.keys().map(|s| s.0).max().unwrap_or(0);
    registry.next = max_saved.max(max_sim);
    registry.pending_rebind = save.sites;
    registry.rebind_frames_left = REBIND_GRACE_FRAMES;
    registry.revision += 1;
    mirror.0.clear();
}

/// Tracks the once-per-bundle-dir hydrate, mirroring the transport/utility overlays.
#[derive(Resource, Debug, Default)]
pub struct LogisticsOrdersHydrateState {
    pub last_bundle_dir: Option<std::path::PathBuf>,
    pub hydrate_attempted: bool,
    pub hydrated: bool,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogisticsOrdersSet {
    TagSites,
    Step,
    Snapshot,
}

pub struct LogisticsOrdersPlugin;

impl Plugin for LogisticsOrdersPlugin {
    fn build(&self, app: &mut App) {
        crate::systems::sim_control::ensure_sim_step_clock(app);
        app.init_resource::<LogisticsOrderSimRes>()
            .init_resource::<LogisticsSiteRegistry>()
            .init_resource::<DepotFleetSeed>()
            .init_resource::<LogisticsOrdersSnapshotRes>()
            .init_resource::<PlantStockMirror>()
            .init_resource::<LogisticsOrdersHydrateState>()
            .init_resource::<super::cargo_registry::CargoKindRegistry>()
            .init_resource::<super::construction_demand::ConstructionDemandBook>()
            .init_resource::<super::plant_demand::PlantDemandBook>()
            .init_resource::<super::plant_demand::DepotStockDemandBook>()
            .init_resource::<super::auto_supply::AutoSupplyPolicy>()
            .init_resource::<super::fleet_growth::FleetGrowthPolicy>()
            .init_resource::<super::weather_haul::HaulWeatherDump>()
            .init_resource::<crate::systems::navigation::LogisticsCostForecastBook>()
            .add_message::<LogisticsOrderRequest>()
            .add_systems(Startup, super::cargo_registry::load_cargo_kind_registry_on_startup)
            .configure_sets(
                Update,
                (
                    LogisticsOrdersSet::TagSites,
                    LogisticsOrdersSet::Step
                        .after(LogisticsOrdersSet::TagSites)
                        .after(SimControlSystemSet::AdvanceSimTick),
                    LogisticsOrdersSet::Snapshot.after(LogisticsOrdersSet::Step),
                ),
            )
            .add_systems(
                Update,
                (
                    try_hydrate_logistics_orders_overlay_on_bundle_dir,
                    (
                        tag_logistics_hubs_as_order_sites,
                        tag_flow_facilities_as_order_sites,
                        tag_mfg_facilities_as_order_sites,
                        super::construction_demand::tag_construction_sites_as_order_sites,
                    ),
                )
                    .chain()
                    .in_set(LogisticsOrdersSet::TagSites),
            )
            .add_systems(
                Update,
                write_logistics_orders_overlay_on_save_flush
                    .after(LogisticsOrdersSet::Step)
                    .before(crate::io::save::flush_dirty_chunk_save_queue),
            )
            .add_systems(
                Update,
                (
                    crate::systems::navigation::sync_logistics_cost_forecast
                        .after(crate::systems::chunk_environment_set::ChunkEnvironmentSet::Weather),
                    super::weather_haul::sync_haul_weather_mul_system
                        .after(crate::systems::navigation::sync_logistics_cost_forecast),
                    super::haul_amplify::sync_haul_amplify_mul_system,
                    super::vehicle_fire::apply_order_sim_vehicle_fire_system
                        .after(super::weather_haul::sync_haul_weather_mul_system),
                    step_logistics_orders_system
                        .after(super::weather_haul::sync_haul_weather_mul_system)
                        .after(super::haul_amplify::sync_haul_amplify_mul_system)
                        .after(super::vehicle_fire::apply_order_sim_vehicle_fire_system),
                )
                    .in_set(LogisticsOrdersSet::Step),
            )
            .add_systems(
                Update,
                (
                    publish_logistics_orders_snapshot_system,
                    super::construction_demand::sync_construction_site_demands_system,
                    super::plant_demand::sync_plant_input_demands_system,
                    super::plant_demand::sync_depot_stock_demands_system,
                    super::auto_supply::unattended_player_auto_supply_system,
                    super::auto_supply::auto_supply_unserved_system,
                    super::fleet_growth::grow_depot_fleets_system,
                )
                    .chain()
                    .in_set(LogisticsOrdersSet::Snapshot),
            );
    }
}

/// Logistics hubs (`LogisticsSiteRoot`) become player depots.
pub fn tag_logistics_hubs_as_order_sites(
    mut commands: Commands,
    hubs: Query<
        (Entity, Option<&Name>, Option<&FactionSlot>),
        (With<LogisticsSiteRoot>, Without<LogisticsOrderSite>),
    >,
) {
    for (entity, name, slot) in &hubs {
        commands.entity(entity).insert(LogisticsOrderSite {
            name: name.map_or_else(|| format!("Depot {}", entity.index()), |n| n.to_string()),
            owner: slot.map_or(OwnerId::PLAYER, |s| OwnerId(s.0)),
            role: SiteRole::Depot,
            position_m: None,
        });
    }
}

/// Activated industrial facilities become plant endpoints (sources/sinks for runs and lines).
///
/// A finished building was a `Construction` endpoint while it was being built; it keeps its site
/// id and becomes the plant in place, so orders and stock identity survive the hand-over.
pub fn tag_flow_facilities_as_order_sites(
    mut commands: Commands,
    nodes: Query<
        (
            Entity,
            &ResourceFlowNode,
            Option<&mut LogisticsOrderSite>,
            Option<&crate::strategic::ConstructionSite>,
        ),
        Without<LogisticsSiteRoot>,
    >,
    slots: Query<&FactionSlot>,
) {
    for (entity, node, existing, built) in nodes {
        let name = format!("{} #{}", node.catalog_id, entity.index());
        match existing {
            None => {
                commands.entity(entity).insert(LogisticsOrderSite {
                    name,
                    owner: built.map_or(OwnerId::PLAYER, |b| owner_of(&slots, b.owner)),
                    role: SiteRole::Plant,
                    position_m: None,
                });
            }
            Some(mut site) if site.role == SiteRole::Construction => {
                site.role = SiteRole::Plant;
                site.name = name;
            }
            Some(_) => {}
        }
    }
}

/// Discrete MFG plants (UAS) without ResourceFlowNode still become Plant order sites.
pub fn tag_mfg_facilities_as_order_sites(
    mut commands: Commands,
    nodes: Query<
        (
            Entity,
            &ManufacturingNode,
            Option<&mut LogisticsOrderSite>,
            Option<&crate::strategic::ConstructionSite>,
        ),
        (
            With<ManufacturingOutputBuffers>,
            Without<ResourceFlowNode>,
            Without<LogisticsSiteRoot>,
        ),
    >,
    slots: Query<&FactionSlot>,
) {
    for (entity, node, existing, built) in nodes {
        let name = format!("{} #{}", node.blueprint_id, entity.index());
        match existing {
            None => {
                commands.entity(entity).insert(LogisticsOrderSite {
                    name,
                    owner: built.map_or(OwnerId::PLAYER, |b| owner_of(&slots, b.owner)),
                    role: SiteRole::Plant,
                    position_m: None,
                });
            }
            Some(mut site) if site.role == SiteRole::Construction => {
                site.role = SiteRole::Plant;
                site.name = name;
            }
            Some(_) => {}
        }
    }
}

fn site_xz(
    site: &LogisticsOrderSite,
    transform: Option<&Transform>,
    global: Option<&GlobalTransform>,
) -> Option<[f32; 2]> {
    if let Some(p) = site.position_m {
        return Some(p);
    }
    let p = transform
        .map(|t| t.translation)
        .or_else(|| global.map(GlobalTransform::translation))?;
    Some([p.x, p.z])
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn step_logistics_orders_system(
    mut commands: Commands,
    mut res: ResMut<LogisticsOrderSimRes>,
    mut registry: ResMut<LogisticsSiteRegistry>,
    seed: Res<DepotFleetSeed>,
    clock: Res<SimStepClock>,
    attention: Option<Res<crate::systems::sim_attention::SimAttentionFrame>>,
    mut requests: MessageReader<LogisticsOrderRequest>,
    new_sites: Query<
        (
            Entity,
            &LogisticsOrderSite,
            Option<&Transform>,
            Option<&GlobalTransform>,
            Option<&DepotStartingStock>,
            Option<&DepotStartingFleet>,
        ),
        Without<LogisticsOrderSiteId>,
    >,
    known_sites: Query<
        (&LogisticsOrderSiteId, &LogisticsOrderSite, Option<&Transform>, Option<&GlobalTransform>),
        Or<(Changed<Transform>, Changed<LogisticsOrderSite>)>,
    >,
    mut removed: RemovedComponents<LogisticsOrderSiteId>,
    mut mirror: ResMut<PlantStockMirror>,
    mut flow_nodes: Query<&mut ResourceFlowNode>,
    mut mfg_buffers: Query<&mut ManufacturingOutputBuffers>,
) {
    let res = &mut *res;
    let registry = &mut *registry;

    for entity in removed.read() {
        if let Some(site) = registry.by_entity.remove(&entity) {
            registry.revision += 1;
            registry.entity_of.remove(&site);
            registry.meta.remove(&site);
            registry.travel.positions.remove(&site);
            res.sim.apply(LogisticsOrderCommand::RemoveSite { site });
            mirror.0.retain(|(s, _), _| *s != site);
        }
    }

    for (entity, site, t, g, start_stock, start_fleet) in &new_sites {
        let Some(xz) = site_xz(site, t, g) else {
            continue;
        };
        let meta = SiteMeta {
            name: site.name.clone(),
            owner: site.owner,
            role: site.role,
            world_xz: xz,
        };
        let rebound = registry
            .pending_rebind
            .iter()
            .position(|saved| saved_site_matches(saved, &meta))
            .map(|i| registry.pending_rebind.swap_remove(i).id);
        let id = rebound.unwrap_or_else(|| {
            registry.next += 1;
            SiteId(registry.next)
        });
        registry.revision += 1;
        registry.by_entity.insert(entity, id);
        registry.entity_of.insert(id, entity);
        registry.meta.insert(id, meta);
        registry.travel.positions.insert(id, xz);
        commands.entity(entity).insert(LogisticsOrderSiteId(id));
        res.sim.apply(LogisticsOrderCommand::RegisterSite {
            site: id,
            name: site.name.clone(),
        });
        // A rebound site already has its saved stock and fleet.
        if rebound.is_none() {
            for (cargo, units) in start_stock.map_or(&[][..], |s| s.0.as_slice()) {
                res.sim.apply(LogisticsOrderCommand::AddStock {
                    site: id,
                    cargo: CargoKind::new(cargo.clone()),
                    amount: *units,
                });
            }
        }
        if rebound.is_none() && site.role == SiteRole::Depot && start_fleet.is_some() {
            for &(class_mask, capacity_units, speed_mps, count) in &start_fleet.unwrap().0 {
                for _ in 0..count {
                    res.sim.apply(LogisticsOrderCommand::AddVehicle {
                        owner: site.owner,
                        home: id,
                        class_mask,
                        capacity_units,
                        speed_mps,
                    });
                }
            }
        } else if rebound.is_none() && seed.enabled && site.role == SiteRole::Depot {
            for &(class_mask, capacity_units, speed_mps, count) in &seed.vehicles {
                for _ in 0..count {
                    res.sim.apply(LogisticsOrderCommand::AddVehicle {
                        owner: site.owner,
                        home: id,
                        class_mask,
                        capacity_units,
                        speed_mps,
                    });
                }
            }
        }
    }

    for (sid, site, t, g) in &known_sites {
        registry.revision += 1;
        if let Some(meta) = registry.meta.get_mut(&sid.0) {
            meta.name = site.name.clone();
            meta.owner = site.owner;
            meta.role = site.role;
            if let Some(xz) = site_xz(site, t, g) {
                meta.world_xz = xz;
                registry.travel.positions.insert(sid.0, xz);
            }
        }
    }

    if !registry.pending_rebind.is_empty() {
        registry.rebind_frames_left = registry.rebind_frames_left.saturating_sub(1);
        if registry.rebind_frames_left == 0 {
            for saved in std::mem::take(&mut registry.pending_rebind) {
                res.sim.apply(LogisticsOrderCommand::RemoveSite { site: saved.id });
            }
        }
    }

    for LogisticsOrderRequest(cmd) in requests.read() {
        let out = res.sim.apply(cmd.clone());
        res.last_outcomes.push(out);
    }
    if res.last_outcomes.len() > 32 {
        let drop = res.last_outcomes.len() - 32;
        res.last_outcomes.drain(0..drop);
    }

    bridge_plant_stock(&mut res.sim, registry, &mut mirror, &mut flow_nodes);
    bridge_mfg_output_stock(&mut res.sim, registry, &mut mirror, &mut mfg_buffers);

    registry.travel.seconds_per_tick = clock.step_secs();
    // FIDELITY-1: player theater Full → every due step; Aggregate → every 2nd; no theater → Aggregate.
    let tier = attention.as_ref().map_or(
        crate::systems::sim_attention::AttentionTier::Full,
        |a| {
            if a.player_theater_active {
                crate::systems::sim_attention::AttentionTier::Full
            } else if a.foci.is_empty() {
                crate::systems::sim_attention::AttentionTier::Aggregate
            } else {
                crate::systems::sim_attention::AttentionTier::Aggregate
            }
        },
    );
    let scale = tier.work_scale();
    for i in 0..clock.due {
        if scale <= 0.0 {
            break;
        }
        if scale < 1.0
            && !crate::systems::heavy_periodic::periodic_step_fires(
                clock.steps + u64::from(i),
                LOGISTICS_ORDER_AGGREGATE_PERIOD,
                LOGISTICS_ORDER_AGGREGATE_PHASE,
            )
        {
            continue;
        }
        res.sim.tick(&registry.travel);
    }
}

/// Rebuilds only when the sim ticked, a command applied, or the registry changed.
pub fn publish_logistics_orders_snapshot_system(
    res: Res<LogisticsOrderSimRes>,
    registry: Res<LogisticsSiteRegistry>,
    forecast: Option<Res<crate::systems::navigation::LogisticsCostForecastBook>>,
    mut snap: ResMut<LogisticsOrdersSnapshotRes>,
    mut last: Local<Option<(u64, u64, u64, u64)>>,
) {
    let forecast_rev = forecast.as_ref().map(|f| f.revision).unwrap_or(0);
    let fingerprint = (res.sim.tick, res.sim.command_seq, registry.revision, forecast_rev);
    if *last == Some(fingerprint) {
        return;
    }
    *last = Some(fingerprint);
    snap.0 = super::snapshot::build_snapshot_with_forecast(
        &res.sim,
        &registry.meta,
        &registry.travel,
        forecast.as_deref(),
    );
}

/// Write `overlays/logistics_orders.ron` when a save flush is armed (before the manifest flush).
pub fn write_logistics_orders_overlay_on_save_flush(
    flush: Option<Res<crate::io::save::SaveFlushRequested>>,
    settings: Option<Res<crate::io::save::WorldSaveBundleSettings>>,
    res: Res<LogisticsOrderSimRes>,
    registry: Res<LogisticsSiteRegistry>,
) {
    let (Some(flush), Some(settings)) = (flush, settings) else {
        return;
    };
    if !flush.0 {
        return;
    }
    let save = capture_logistics_orders_save(&res, &registry);
    if let Err(err) = write_logistics_orders_overlay(&settings.bundle_dir, &save) {
        bevy::log::warn!("logistics orders overlay write failed: {err}");
    }
}

/// Hydrate order state once per bundle dir when the bundle carries the orders overlay.
#[allow(clippy::too_many_arguments)]
pub fn try_hydrate_logistics_orders_overlay_on_bundle_dir(
    mut commands: Commands,
    settings: Option<Res<crate::io::save::WorldSaveBundleSettings>>,
    mut state: ResMut<LogisticsOrdersHydrateState>,
    mut res: ResMut<LogisticsOrderSimRes>,
    mut registry: ResMut<LogisticsSiteRegistry>,
    mut mirror: ResMut<PlantStockMirror>,
    bound: Query<Entity, With<LogisticsOrderSiteId>>,
) {
    let Some(settings) = settings else {
        return;
    };
    if state.last_bundle_dir.as_deref() != Some(settings.bundle_dir.as_path()) {
        state.last_bundle_dir = Some(settings.bundle_dir.clone());
        state.hydrate_attempted = false;
        state.hydrated = false;
    }
    if state.hydrate_attempted {
        return;
    }
    state.hydrate_attempted = true;
    let save = match read_logistics_orders_overlay(&settings.bundle_dir) {
        Ok(Some(save)) => save,
        Ok(None) => return,
        Err(err) => {
            bevy::log::warn!("logistics orders overlay unreadable: {err}");
            return;
        }
    };
    install_logistics_orders_save(&mut res, &mut registry, &mut mirror, save);
    for entity in &bound {
        commands.entity(entity).remove::<LogisticsOrderSiteId>();
    }
    state.hydrated = true;
}
