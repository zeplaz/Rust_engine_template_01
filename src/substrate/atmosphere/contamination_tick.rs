//! AC-003 — slab contamination → L0 seed (W7: folded into FieldFill).
//!
//! Production L0 seed is [`crate::systems::atmosphere::atmosphere_field_seed_slab_smoke`].
//! This system remains scheduled as a no-op so SubstratePlugin wiring stays stable.

use bevy::prelude::*;

use crate::substrate::registry::WorldSubstrateRegistry;

use super::{AtmosphereClipmapStack, AtmosphereClipmapStats};

/// W7 — retired L0 writer; seed lives in `AtmospherePipelineSet::FieldFill`.
pub fn contamination_tick_system(
    _stack: ResMut<AtmosphereClipmapStack>,
    _witness: ResMut<AtmosphereClipmapStats>,
    _registry: Option<Res<WorldSubstrateRegistry>>,
) {
}
