// Structure entities module
pub mod components;

/// Legacy ECS road/rail placeholders — not in default builds (see **INFRA-E0-003**).
#[cfg(feature = "legacy_transport_ecs_stubs")]
pub mod legacy_transport_stubs;

// Public exports
pub use components::*;