//! [`SimEffectEvent`] vocabulary — single enqueue surface for cause→effect dispatch.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::substrate::hydrology::HydrologyDirtyEvent;
use crate::terrain::ChunkCellKey;

/// Producer class for telemetry rows (`source` field).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
#[reflect(Serialize, Deserialize)]
pub enum SimEffectSource {
    Ecology,
    Lightning,
    GridOverload,
    Construction,
    ScenarioScript,
    SimEffectTest,
}

impl SimEffectSource {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ecology => "ecology",
            Self::Lightning => "lightning",
            Self::GridOverload => "grid_overload",
            Self::Construction => "construction",
            Self::ScenarioScript => "scenario_script",
            Self::SimEffectTest => "sim_effect_test",
        }
    }
}

/// Tick-drained sim effect — adapters map to domain buses (ember, hydro, …).
#[derive(Clone, Debug)]
pub struct SimEffectEvent {
    pub source: SimEffectSource,
    pub cause_id: String,
    pub parent_effect_id: Option<u64>,
    pub kind: SimEffectKind,
}

#[derive(Clone, Debug)]
pub enum SimEffectKind {
    /// Maps to one or more [`EmberSpotIgnitionEvent`] (fire waist).
    IgniteCells {
        cells: Vec<(ChunkCellKey, f32)>,
    },
    /// Lightning strike batch at chunk cells.
    LightningStrike {
        chunk: IVec2,
        cell_indices: Vec<u32>,
        spark: f32,
    },
    /// Construction / scenario hydro dirty — adapter pushes [`HydrologyDirtyEvent`].
    HydroDirty(HydrologyDirtyEvent),
    /// Landscape grammar disturbance (harvest or construction clear) — VEG-SIM-EFFECT-HOOK-001.
    LandscapeDisturbance {
        chunk: IVec2,
        harvest: bool,
    },
    /// Grid / structure catastrophe heat spots (ember waist).
    StructureHeat {
        chunk: IVec2,
        cells: Vec<(u32, f32)>,
    },
}

impl SimEffectKind {
    #[must_use]
    pub fn dedupe_tag(&self) -> u8 {
        match self {
            Self::IgniteCells { .. } => 1,
            Self::LightningStrike { .. } => 2,
            Self::HydroDirty { .. } => 3,
            Self::LandscapeDisturbance { .. } => 5,
            Self::StructureHeat { .. } => 4,
        }
    }
}
