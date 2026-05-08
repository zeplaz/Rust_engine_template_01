//! Ordering for per-chunk **environment** systems: LOD refresh → weather → fire → ecology.
//! Configured from [`crate::engine::EnginePlugin`](crate::engine::EnginePlugin).

use bevy::prelude::*;

use crate::systems::sim_control::SimControlSystemSet;

/// Fixed ordering after [`SimControlSystemSet::AdvanceSimTick`] for chunk fields that feed each other.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChunkEnvironmentSet {
    /// Refresh sim intensity tier from prior frame’s weather + fire (adaptive tick weight).
    Lod,
    Weather,
    Fire,
    Ecology,
}

/// Call from the root engine plugin after `SimControlPlugin`.
pub fn configure_chunk_environment_sets(app: &mut App) {
    app.configure_sets(
        Update,
        (
            ChunkEnvironmentSet::Lod.after(SimControlSystemSet::AdvanceSimTick),
            ChunkEnvironmentSet::Weather.after(ChunkEnvironmentSet::Lod),
            ChunkEnvironmentSet::Fire.after(ChunkEnvironmentSet::Weather),
            ChunkEnvironmentSet::Ecology.after(ChunkEnvironmentSet::Fire),
        ),
    );
}
