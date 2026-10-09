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

use super::model::{CargoClassMask, CargoKind, OwnerId, SiteId};
use super::sim::{CommandOutcome, LogisticsOrderCommand, LogisticsOrderSim, SimConfig};
use super::snapshot::{build_snapshot, LogisticsOrdersSnapshot, SiteMeta, SiteRole};
use super::persistence::{
    read_logistics_orders_overlay, saved_site_matches, write_logistics_orders_overlay,
    LogisticsOrdersSave, SavedSite, LOGISTICS_ORDERS_SAVE_SCHEMA,
};
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
    /// Saved sites awaiting their endpoint entity after a load (rebind by name/owner/role/tile).
    pub pending_rebind: Vec<SavedSite>,
    /// Frames left before unclaimed saved sites are removed from the sim.
    pub rebind_frames_left: u32,
}

/// Frames a loaded save waits for its endpoints to re-register (~10 s at 60 fps).
pub const REBIND_GRACE_FRAMES: u32 = 600;

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

/// Reconcile every plant site's buffers with the order sim (sole caller: step system).
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
        let mut tags: Vec<String> = node.buffer_by_tag.keys().cloned().collect();
        tags.extend(
            sim.stock
                .entries
                .keys()
                .filter(|(s, _)| *s == site)
                .map(|(_, c)| c.0.clone()),
        );
        tags.sort();
        tags.dedup();
        for tag in tags {
            let cargo = CargoKind::new(tag.clone());
            let key = (site, cargo.clone());
            let node_amount = node.buffer_by_tag.get(&tag).copied().unwrap_or(0.0);
            let sim_on_hand = sim.stock.get(site, &cargo).on_hand;
            let last = mirror.0.get(&key).copied();
            if last == Some(sim_on_hand) && node_amount.floor() as u32 == sim_on_hand {
                continue;
            }
            let (node_new, sim_new, mirror_new) = reconcile_plant_stock(node_amount, sim_on_hand, last);
            if (node_new - node_amount).abs() > f32::EPSILON {
                node.buffer_by_tag.insert(tag.clone(), node_new);
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
    res.last_sim_micros = None;
    registry.by_entity.clear();
    registry.entity_of.clear();
    registry.meta.clear();
    registry.travel.positions.clear();
    let max_saved = save.sites.iter().map(|s| s.id.0).max().unwrap_or(0);
    let max_sim = res.sim.sites.keys().map(|s| s.0).max().unwrap_or(0);
    registry.next = max_saved.max(max_sim);
    registry.pending_rebind = save.sites;
    registry.rebind_frames_left = REBIND_GRACE_FRAMES;
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
        app.init_resource::<LogisticsOrderSimRes>()
            .init_resource::<LogisticsSiteRegistry>()
            .init_resource::<DepotFleetSeed>()
            .init_resource::<LogisticsOrdersSnapshotRes>()
            .init_resource::<PlantStockMirror>()
            .init_resource::<LogisticsOrdersHydrateState>()
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
                    try_hydrate_logistics_orders_overlay_on_bundle_dir,
                    (
                        tag_logistics_hubs_as_order_sites,
                        tag_flow_facilities_as_order_sites,
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
    mut mirror: ResMut<PlantStockMirror>,
    mut flow_nodes: Query<&mut ResourceFlowNode>,
) {
    let res = &mut *res;
    let registry = &mut *registry;

    for entity in removed.read() {
        if let Some(site) = registry.by_entity.remove(&entity) {
            registry.entity_of.remove(&site);
            registry.meta.remove(&site);
            registry.travel.positions.remove(&site);
            res.sim.apply(LogisticsOrderCommand::RemoveSite { site });
            mirror.0.retain(|(s, _), _| *s != site);
        }
    }

    for (entity, site, t, g) in &new_sites {
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
        registry.by_entity.insert(entity, id);
        registry.entity_of.insert(id, entity);
        registry.meta.insert(id, meta);
        registry.travel.positions.insert(id, xz);
        commands.entity(entity).insert(LogisticsOrderSiteId(id));
        res.sim.apply(LogisticsOrderCommand::RegisterSite {
            site: id,
            name: site.name.clone(),
        });
        // A rebound depot already has its saved fleet.
        if rebound.is_none() && seed.enabled && site.role == SiteRole::Depot {
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
