//! Cyclic Granfina Dashboard Tool
//!
//! This tool provides a cyclic dashboard for tracking blockers with atomic operations,
//! hash locks, and API-only access. It integrates with the existing witness integrity system
//! and ignores DCC status in the UI bar as requested.

pub mod dev;

pub use dev::cyclic_granfina_dashboard::{
    BlockStatus, CyclicGranfinaDashboard, GranfinaDashboardConfig, PriorityLevel,
};
