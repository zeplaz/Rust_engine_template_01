//! Rail settings shared with the corridor tool (profile, grade limit) + junction registry.
//! Placement itself is the one corridor tool (`construction::roads::input`, PLAN-ROADS-v1 RD-6).

mod junction;
mod placement;
mod validation;

pub use junction::RailJunctionAuthority;
pub use placement::ActiveRailPlacement;
pub use validation::validate_rail_segment;
