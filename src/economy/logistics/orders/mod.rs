//! Logistics orders — standing supply lines, one-off multi-trip supply runs, draw demands
//! (PLAN-LOGISTICS-ORDERS-v1, `src/dev/plan_logistics_orders_v1.md`).
//!
//! Isolation: [`sim::LogisticsOrderSim`] is pure Rust (no Bevy), deterministic and headless-testable.
//! Writes go only through [`sim::LogisticsOrderCommand`] (in-app: [`plugin::LogisticsOrderRequest`]);
//! UI, overlay and AI read only [`snapshot::LogisticsOrdersSnapshot`].

pub mod advisor;
pub mod demand_advert;
pub mod auto_supply;
pub mod cargo_registry;
pub mod construction_demand;
pub mod fleet_growth;
pub mod model;
pub mod persistence;
pub mod plant_demand;
pub mod plugin;
pub mod sim;
pub mod snapshot;
pub mod source_pick;
pub mod travel;
pub mod haul_amplify;
pub mod vehicle_fire;
pub mod weather_haul;

#[cfg(test)]
mod tests;

pub use haul_amplify::{
    haul_capacity_mul_by_owner, unit_kind_haul_capacity_mul, sync_haul_amplify_mul_system,
};
pub use weather_haul::{weather_haul_speed_mul, HaulWeatherDump};

pub use auto_supply::{auto_supply_unserved_system, AutoSupplyPolicy};
pub use cargo_registry::{
    cargo_tag_known, load_cargo_kind_registry, CargoKindRegistry, RegisteredCargoKind, CARGO_KINDS_RON,
};
pub use fleet_growth::{
    grow_depot_fleets_system, recipe_costs_in_registry, saturated_recipe, FleetGrowthPolicy,
    VehicleRecipe,
};
pub use advisor::{best_source_for, suggest_lines_for_unserved, AdvisorGap, SupplyAdvice};
pub use demand_advert::{
    cmp_demands_by_advert, demand_advert_score, demand_advert_score_from_snapshot,
    travel_ticks_to_best_source, DemandAdvertTuning,
};
pub use model::*;
pub use persistence::{LogisticsOrdersSave, SavedSite};
pub use plugin::{
    capture_logistics_orders_save, install_logistics_orders_save, LogisticsOrdersHydrateState,
    DepotFleetSeed, DepotStartingFleet, DepotStartingStock, FactionSlot, LogisticsOrderRequest, LogisticsOrderSimRes, LogisticsOrderSite,
    LogisticsOrderSiteId, LogisticsOrdersPlugin, LogisticsOrdersSet, LogisticsOrdersSnapshotRes,
    LogisticsSiteRegistry,
};
pub use sim::{
    CommandOutcome, LogisticsOrderCommand, LogisticsOrderSim, OrderEvent, OrderEventKind,
    SimConfig, TravelModel,
};
pub use snapshot::{
    build_snapshot, quantize_telemetry_rate_f32, quantize_telemetry_unit_f32, DemandRow, EventRow,
    FleetRow, LogisticsOrdersSnapshot, OrderRow, SiteMeta, SiteRole, SiteRow, StockRow,
    VehicleRow, TELEMETRY_FLOAT_QUANT_SCALE,
};
pub use travel::SiteDistanceTravel;
