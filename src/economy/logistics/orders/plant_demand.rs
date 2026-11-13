//! Operational plants as draw-demand consumers (closes the loop: finished building -> its
//! inputs become demands the supply advisor can serve).
//!
//! The plant's physical consumption stays in `ResourceFlowNode` (the buffer is mirrored into order
//! stock by `bridge_plant_stock`). Each demand publishes the plant's nominal draw of that input
//! (`rate_per_tick` at the order sim's one-tick-per-simulated-second convention) so lines are
//! sized for steady flow and an empty buffer reads Starving; the order sim does not draw it a
//! second time (`ConsumerKind::drawn_by_order_sim`).

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::economy::resource_flow::ResourceFlowNode;

use super::construction_demand::ConstructionDemandBook;
use super::model::{CargoKind, ConsumerKind, Priority, SiteId};
use super::plugin::{LogisticsOrderRequest, LogisticsOrderSimRes, LogisticsOrderSite, LogisticsOrderSiteId};
use super::sim::LogisticsOrderCommand;
use super::snapshot::SiteRole;

/// Units of each input a plant asks logistics to keep on hand.
pub const PLANT_INPUT_BUFFER_UNITS: u32 = crate::economy::resource_flow::INPUT_BUFFER_TARGET as u32;
/// Reorder at a quarter of the buffer.
pub const PLANT_INPUT_REORDER_UNITS: u32 = 12;

/// Consumption tags that are not hauled cargo.
#[must_use]
pub fn is_cargo_tag(tag: &str) -> bool {
    !crate::economy::facility_inputs::is_supply_input_tag(tag)
}

/// What this system last published per plant (`(input tag, draw per minute)`), so it only sends
/// changes.
#[derive(Resource, Debug, Default)]
pub struct PlantDemandBook {
    pub published: BTreeMap<SiteId, Vec<(String, u32)>>,
}

/// Nominal draw per hauled input of a flow node, in units per simulated minute. Inputs listed
/// twice add up.
#[must_use]
pub fn plant_input_draw_per_min(node: &ResourceFlowNode, ticks_per_minute: u32) -> Vec<(String, u32)> {
    let mut by_tag: BTreeMap<String, f32> = BTreeMap::new();
    for r in node.consumption.iter().filter(|r| is_cargo_tag(&r.tag)) {
        *by_tag.entry(r.tag.clone()).or_default() += r.rate_per_tick.max(0.0);
    }
    by_tag
        .into_iter()
        .map(|(tag, per_tick)| (tag, (per_tick * ticks_per_minute as f32).ceil() as u32))
        .collect()
}

/// Publish one demand per hauled input of every plant endpoint.
pub fn sync_plant_input_demands_system(
    mut book: ResMut<PlantDemandBook>,
    construction: Res<ConstructionDemandBook>,
    sim: Option<Res<LogisticsOrderSimRes>>,
    plants: Query<(&LogisticsOrderSiteId, &LogisticsOrderSite, &ResourceFlowNode)>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
) {
    let ticks_per_minute = sim.as_ref().map_or(60, |s| s.sim.config.ticks_per_minute.max(1));
    let mut live: Vec<SiteId> = Vec::new();
    for (sid, site, node) in &plants {
        // Still being built: the construction demands own this site's demand slots.
        if site.role != SiteRole::Plant || construction.published.contains_key(&sid.0) {
            continue;
        }
        live.push(sid.0);
        let inputs = plant_input_draw_per_min(node, ticks_per_minute);
        if book.published.get(&sid.0) == Some(&inputs) {
            continue;
        }
        for (tag, draw_rate_per_min) in &inputs {
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::PublishDemand {
                site: sid.0,
                cargo: CargoKind::new(tag.clone()),
                consumer: ConsumerKind::Plant,
                buffer_target: PLANT_INPUT_BUFFER_UNITS,
                reorder_point: PLANT_INPUT_REORDER_UNITS,
                draw_rate_per_min: *draw_rate_per_min,
                priority: Priority::Normal,
            }));
        }
        book.published.insert(sid.0, inputs);
    }
    book.published.retain(|id, _| live.contains(id));
}

/// Stock level a faction depot asks for of each cargo its plants make.
pub const DEPOT_STOCKPILE_TARGET: u32 = 2000;
/// The depot reorders from its plants below this.
pub const DEPOT_STOCKPILE_REORDER: u32 = 1500;

/// Depot stockpile demands this system published, `(depot, cargo)`.
#[derive(Resource, Debug, Default)]
pub struct DepotStockDemandBook {
    pub published: std::collections::BTreeSet<(SiteId, String)>,
}

/// A plant's output goes to its owner's nearest depot: that depot publishes a stockpile demand
/// (`ConsumerKind::Depot`, no draw) for every hauled cargo the plant makes, so the auto-supply /
/// AI lines carry product out of the plant buffer. Without a sink a plant fills its
/// `OUTPUT_BUFFER_CAP` and stands idle for good. Cargo the plant also consumes is skipped (no
/// plant <-> depot loop).
pub fn sync_depot_stock_demands_system(
    snapshot: Res<super::plugin::LogisticsOrdersSnapshotRes>,
    construction: Res<ConstructionDemandBook>,
    mut book: ResMut<DepotStockDemandBook>,
    plants: Query<(&LogisticsOrderSiteId, &LogisticsOrderSite, &ResourceFlowNode)>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
) {
    let snap = &snapshot.0;
    let mut want: std::collections::BTreeSet<(SiteId, String)> = std::collections::BTreeSet::new();
    for (sid, site, node) in &plants {
        if site.role != SiteRole::Plant || construction.published.contains_key(&sid.0) {
            continue;
        }
        let Some(plant) = snap.site(sid.0) else { continue };
        let depot = snap
            .sites_of(plant.owner, SiteRole::Depot)
            .map(|d| {
                let (dx, dz) = (d.world_xz[0] - plant.world_xz[0], d.world_xz[1] - plant.world_xz[1]);
                (dx * dx + dz * dz, d.id)
            })
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
            .map(|(_, id)| id);
        let Some(depot) = depot else { continue };
        for r in node.production.iter().filter(|r| is_cargo_tag(&r.tag)) {
            if node.consumption.iter().any(|c| c.tag == r.tag) {
                continue;
            }
            want.insert((depot, r.tag.clone()));
        }
    }
    for (depot, tag) in want.difference(&book.published) {
        requests.write(LogisticsOrderRequest(LogisticsOrderCommand::PublishDemand {
            site: *depot,
            cargo: CargoKind::new(tag.clone()),
            consumer: ConsumerKind::Depot,
            buffer_target: DEPOT_STOCKPILE_TARGET,
            reorder_point: DEPOT_STOCKPILE_REORDER,
            draw_rate_per_min: 0,
            priority: Priority::Low,
        }));
    }
    for (depot, tag) in book.published.difference(&want) {
        if snap.site(*depot).is_some() {
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::RetractDemand {
                site: *depot,
                cargo: CargoKind::new(tag.clone()),
            }));
        }
    }
    book.published = want;
}
