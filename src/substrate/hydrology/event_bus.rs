//! Construction → hydrology dirty event bus (WSS-PLAN-003 / B-H2).
//!
//! Construction **enqueues** only; hydrology drain marks slab dirty — never writes `water_depth`.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::substrate::ChunkKey;

/// Event-driven deep-solve triggers (WSS-PLAN-003). Construction may only emit
/// [`HydrologyDirtyReason::ConstructionComplete`] and [`HydrologyDirtyReason::DamBreach`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HydrologyDirtyReason {
    None,
    DamBreach { structure_id: u64 },
    ConstructionComplete { structure_id: u64 },
    Explosive { cell_index: u32 },
    ScenarioScript { script_id: String },
    UpstreamOverflow,
    ErosionThreshold,
    ManualEditor,
}

impl HydrologyDirtyReason {
    #[must_use]
    pub fn dedupe_tag(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::DamBreach { .. } => 1,
            Self::ConstructionComplete { .. } => 2,
            Self::Explosive { .. } => 3,
            Self::ScenarioScript { .. } => 4,
            Self::UpstreamOverflow => 5,
            Self::ErosionThreshold => 6,
            Self::ManualEditor => 7,
        }
    }
}

/// Published by construction execute paths; drained by hydrology runtime (HY-005).
#[derive(Clone, Debug)]
pub struct HydrologyDirtyEvent {
    pub key: ChunkKey,
    pub reason: HydrologyDirtyReason,
    pub structure_id: u64,
    pub affected_cells: Vec<u32>,
}

/// B-H2 bridge telemetry — updated by `crate::construction::hydro_coupling` only.
#[derive(Resource, Clone, Debug, Default)]
pub struct HydrologyConstructionCouplingWitness {
    pub bridge_registered: bool,
    pub execute_emit_count: u32,
    pub preview_emit_count: u32,
}

#[must_use]
pub fn construction_hydro_coupling_witness_green(
    coupling: &HydrologyConstructionCouplingWitness,
    queue: &HydrologyEventQueue,
) -> bool {
    coupling.bridge_registered
        && coupling.execute_emit_count > 0
        && coupling.preview_emit_count == 0
        && queue.construction_events_drained > 0
}

#[derive(Resource, Debug, Default)]
pub struct HydrologyEventQueue {
    pub pending: Vec<HydrologyDirtyEvent>,
    pub drained_total: u64,
    pub last_drain_count: u32,
    pub construction_events_drained: u64,
    tick_dedupe: HashSet<(ChunkKey, u64, u8)>,
}

impl HydrologyEventQueue {
    pub fn clear_tick_dedupe(&mut self) {
        self.tick_dedupe.clear();
    }

    pub fn push(&mut self, event: HydrologyDirtyEvent) -> bool {
        let tag = event.reason.dedupe_tag();
        let dedupe = (event.key, event.structure_id, tag);
        if !self.tick_dedupe.insert(dedupe) {
            return false;
        }
        self.pending.push(event);
        true
    }
}
