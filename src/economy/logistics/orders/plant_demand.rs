//! Operational plants as draw-demand consumers (closes the loop: finished building -> its
//! inputs become demands the supply advisor can serve).
//!
//! The plant's physical consumption stays in `ResourceFlowNode` (the buffer is mirrored into order
//! stock by `bridge_plant_stock`), so these demands only describe the target level
//! (`draw_rate_per_min` 0): health goes Low when the buffer drops under the reorder point and
//! the advisor / panel create lines to refill it.

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::economy::resource_flow::ResourceFlowNode;

use super::construction_demand::ConstructionDemandBook;
use super::model::{CargoKind, ConsumerKind, Priority, SiteId};
use super::plugin::{LogisticsOrderRequest, LogisticsOrderSite, LogisticsOrderSiteId};
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

/// What this system last published per plant, so it only sends changes.
#[derive(Resource, Debug, Default)]
pub struct PlantDemandBook {
    pub published: BTreeMap<SiteId, Vec<String>>,
}

/// Publish one demand per hauled input of every plant endpoint.
pub fn sync_plant_input_demands_system(
    mut book: ResMut<PlantDemandBook>,
    construction: Res<ConstructionDemandBook>,
    plants: Query<(&LogisticsOrderSiteId, &LogisticsOrderSite, &ResourceFlowNode)>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
) {
    let mut live: Vec<SiteId> = Vec::new();
    for (sid, site, node) in &plants {
        // Still being built: the construction demands own this site's demand slots.
        if site.role != SiteRole::Plant || construction.published.contains_key(&sid.0) {
            continue;
        }
        live.push(sid.0);
        let mut tags: Vec<String> = node
            .consumption
            .iter()
            .map(|r| r.tag.clone())
            .filter(|t| is_cargo_tag(t))
            .collect();
        tags.sort();
        tags.dedup();
        if book.published.get(&sid.0) == Some(&tags) {
            continue;
        }
        for tag in &tags {
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::PublishDemand {
                site: sid.0,
                cargo: CargoKind::new(tag.clone()),
                consumer: ConsumerKind::Plant,
                buffer_target: PLANT_INPUT_BUFFER_UNITS,
                reorder_point: PLANT_INPUT_REORDER_UNITS,
                draw_rate_per_min: 0,
                priority: Priority::Normal,
            }));
        }
        book.published.insert(sid.0, tags);
    }
    book.published.retain(|id, _| live.contains(id));
}
