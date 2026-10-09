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

use crate::economy::resource_flow::ResourceFlowNode;
use crate::entities::production::core::LogisticsSiteRoot;
use crate::systems::sim_control::{SimControlSystemSet, SimTimeMicros};

use super::model::{CargoClassMask, OwnerId, SiteId};
use super::sim::{CommandOutcome, LogisticsOrderCommand, LogisticsOrderSim, SimConfig};
use super::snapshot::{build_snapshot, LogisticsOrdersSnapshot, SiteMeta, SiteRole};
use super::travel::SiteDistanceTravel;

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
#[derive(Resource, Debug)]
pub struct LogisticsOrderSimRes {
    pub sim: LogisticsOrderSim,
    /// Simulated microseconds per order-sim tick (1 s).
    pub step_micros: u64,
    /// Max ticks per frame (catch-up cap at high sim speed).
    pub max_steps_per_frame: u32,
    pub last_sim_micros: Option<u64>,
    /// Outcomes of the last applied requests (most recent last, capped).
    pub last_outcomes: Vec<CommandOutcome>,
}

impl Default for LogisticsOrderSimRes {
    fn default() -> Self {
        Self {
            sim: LogisticsOrderSim::new(SimConfig::default()),
            step_micros: 1_000_000,
            max_steps_per_frame: 30,
            last_sim_micros: None,
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
}

impl LogisticsSiteRegistry {
    #[must_use]
    pub fn site_of(&self, entity: Entity) -> Option<SiteId> {
        self.by_entity.get(&entity).copied()
    }
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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogisticsOrdersSet {
    TagSites,
    Step,
    Snapshot,
}

pub struct LogisticsOrdersPlugin;

impl Plugin for LogisticsOrdersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LogisticsOrderSimRes>()
            .init_resource::<LogisticsSiteRegistry>()
            .init_resource::<DepotFleetSeed>()
            .init_resource::<LogisticsOrdersSnapshotRes>()
            .add_message::<LogisticsOrderRequest>()
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
                    tag_logistics_hubs_as_order_sites,
                    tag_flow_facilities_as_order_sites,
                )
                    .in_set(LogisticsOrdersSet::TagSites),
            )
            .add_systems(
                Update,
                step_logistics_orders_system.in_set(LogisticsOrdersSet::Step),
            )
            .add_systems(
                Update,
                publish_logistics_orders_snapshot_system.in_set(LogisticsOrdersSet::Snapshot),
            );
    }
}

/// Logistics hubs (`LogisticsSiteRoot`) become player depots.
pub fn tag_logistics_hubs_as_order_sites(
    mut commands: Commands,
    hubs: Query<(Entity, Option<&Name>), (With<LogisticsSiteRoot>, Without<LogisticsOrderSite>)>,
) {
    for (entity, name) in &hubs {
        commands.entity(entity).insert(LogisticsOrderSite {
            name: name.map_or_else(|| format!("Depot {}", entity.index()), |n| n.to_string()),
            owner: OwnerId::PLAYER,
            role: SiteRole::Depot,
            position_m: None,
        });
    }
}

/// Activated industrial facilities become plant endpoints (sources/sinks for runs and lines).
pub fn tag_flow_facilities_as_order_sites(
    mut commands: Commands,
    nodes: Query<
        (Entity, &ResourceFlowNode),
        (Without<LogisticsOrderSite>, Without<LogisticsSiteRoot>),
    >,
) {
    for (entity, node) in &nodes {
        commands.entity(entity).insert(LogisticsOrderSite {
            name: format!("{} #{}", node.catalog_id, entity.index()),
            owner: OwnerId::PLAYER,
            role: SiteRole::Plant,
            position_m: None,
        });
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

/// Number of whole order-sim ticks elapsed between two sim-time readings.
#[must_use]
pub fn due_steps(last: Option<u64>, now: u64, step_micros: u64, cap: u32) -> (u32, u64) {
    let step = step_micros.max(1);
    let Some(last) = last else {
        return (0, now);
    };
    if now <= last {
        return (0, now.min(last));
    }
    let n = (now - last) / step;
    let steps = n.min(u64::from(cap)) as u32;
    // Drop the backlog beyond the cap rather than spiralling.
    let new_last = if n > u64::from(cap) { now } else { last + n * step };
    (steps, new_last)
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn step_logistics_orders_system(
    mut commands: Commands,
    mut res: ResMut<LogisticsOrderSimRes>,
    mut registry: ResMut<LogisticsSiteRegistry>,
    seed: Res<DepotFleetSeed>,
    sim_time: Option<Res<SimTimeMicros>>,
    mut requests: MessageReader<LogisticsOrderRequest>,
    new_sites: Query<
        (Entity, &LogisticsOrderSite, Option<&Transform>, Option<&GlobalTransform>),
        Without<LogisticsOrderSiteId>,
    >,
    known_sites: Query<
        (&LogisticsOrderSiteId, &LogisticsOrderSite, Option<&Transform>, Option<&GlobalTransform>),
        Or<(Changed<Transform>, Changed<LogisticsOrderSite>)>,
    >,
    mut removed: RemovedComponents<LogisticsOrderSiteId>,
) {
    let res = &mut *res;
    let registry = &mut *registry;

    for entity in removed.read() {
        if let Some(site) = registry.by_entity.remove(&entity) {
            registry.entity_of.remove(&site);
            registry.meta.remove(&site);
            registry.travel.positions.remove(&site);
            res.sim.apply(LogisticsOrderCommand::RemoveSite { site });
        }
    }

    for (entity, site, t, g) in &new_sites {
        let Some(xz) = site_xz(site, t, g) else {
            continue;
        };
        registry.next += 1;
        let id = SiteId(registry.next);
        registry.by_entity.insert(entity, id);
        registry.entity_of.insert(id, entity);
        registry.meta.insert(
            id,
            SiteMeta {
                name: site.name.clone(),
                owner: site.owner,
                role: site.role,
                world_xz: xz,
            },
        );
        registry.travel.positions.insert(id, xz);
        commands.entity(entity).insert(LogisticsOrderSiteId(id));
        res.sim.apply(LogisticsOrderCommand::RegisterSite {
            site: id,
            name: site.name.clone(),
        });
        if seed.enabled && site.role == SiteRole::Depot {
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

    for LogisticsOrderRequest(cmd) in requests.read() {
        let out = res.sim.apply(cmd.clone());
        res.last_outcomes.push(out);
    }
    if res.last_outcomes.len() > 32 {
        let drop = res.last_outcomes.len() - 32;
        res.last_outcomes.drain(0..drop);
    }

    let now = sim_time.map_or(0, |t| t.0);
    let (steps, new_last) = due_steps(
        res.last_sim_micros,
        now,
        res.step_micros,
        res.max_steps_per_frame,
    );
    res.last_sim_micros = Some(new_last);
    registry.travel.seconds_per_tick = res.step_micros as f32 / 1_000_000.0;
    for _ in 0..steps {
        res.sim.tick(&registry.travel);
    }
}

pub fn publish_logistics_orders_snapshot_system(
    res: Res<LogisticsOrderSimRes>,
    registry: Res<LogisticsSiteRegistry>,
    mut snap: ResMut<LogisticsOrdersSnapshotRes>,
) {
    if !res.is_changed() && !registry.is_changed() {
        return;
    }
    snap.0 = build_snapshot(&res.sim, &registry.meta, &registry.travel);
}
