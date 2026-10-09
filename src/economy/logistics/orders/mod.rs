//! Logistics orders — standing supply lines, one-off multi-trip supply runs, draw demands
//! (PLAN-LOGISTICS-ORDERS-v1, `src/dev/plan_logistics_orders_v1.md`).
//!
//! Isolation: [`sim::LogisticsOrderSim`] is pure Rust (no Bevy), deterministic and headless-testable.
//! Writes go only through [`sim::LogisticsOrderCommand`] (in-app: [`plugin::LogisticsOrderRequest`]);
//! UI, overlay and AI read only [`snapshot::LogisticsOrdersSnapshot`].

pub mod advisor;
pub mod construction_demand;
pub mod model;
pub mod persistence;
pub mod plugin;
pub mod sim;
pub mod snapshot;
pub mod travel;

#[cfg(test)]
mod tests;

pub use advisor::{best_source_for, suggest_lines_for_unserved, AdvisorGap, SupplyAdvice};
pub use model::*;
pub use persistence::{LogisticsOrdersSave, SavedSite};
pub use plugin::{
    capture_logistics_orders_save, install_logistics_orders_save, LogisticsOrdersHydrateState,
    DepotFleetSeed, LogisticsOrderRequest, LogisticsOrderSimRes, LogisticsOrderSite,
    LogisticsOrderSiteId, LogisticsOrdersPlugin, LogisticsOrdersSet, LogisticsOrdersSnapshotRes,
    LogisticsSiteRegistry,
};
pub use sim::{
    CommandOutcome, LogisticsOrderCommand, LogisticsOrderSim, OrderEvent, OrderEventKind,
    SimConfig, TravelModel,
};
pub use snapshot::{
    build_snapshot, DemandRow, EventRow, FleetRow, LogisticsOrdersSnapshot, OrderRow, SiteMeta,
    SiteRole, SiteRow, StockRow, VehicleRow,
};
pub use travel::SiteDistanceTravel;
