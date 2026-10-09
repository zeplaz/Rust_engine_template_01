//! Logistics orders — standing supply lines, one-off multi-trip supply runs, draw demands
//! (PLAN-LOGISTICS-ORDERS-v1, `src/dev/plan_logistics_orders_v1.md`).
//!
//! Isolation: [`sim::LogisticsOrderSim`] is pure Rust (no Bevy), deterministic and headless-testable.
//! Writes go only through [`sim::LogisticsOrderCommand`]; time only through `tick`.

pub mod model;
pub mod sim;

#[cfg(test)]
mod tests;

pub use model::*;
pub use sim::{
    CommandOutcome, LogisticsOrderCommand, LogisticsOrderSim, OrderEvent, OrderEventKind,
    SimConfig, TravelModel,
};
