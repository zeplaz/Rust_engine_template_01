//! Books, issuers, and construction phase enums (P2-A).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::strategic::construction_book::CorridorConstructionPhase;

/// Stable id for a construction site row in [`SiteConstructionBook`] and matching ECS entities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SiteId(pub u64);

impl SiteId {
    /// Sentinel: [`commit_construction_site_system`](super::systems::commit_construction_site_system) allocates via [`SiteIdIssuer`].
    pub const UNASSIGNED: SiteId = SiteId(0);
}

/// Monotonic issuer (game session). Persisted saves may remap ids; book is source after load.
#[derive(Resource, Debug, Default)]
pub struct SiteIdIssuer(pub u64);

impl SiteIdIssuer {
    #[inline]
    pub fn next(&mut self) -> SiteId {
        self.0 = self.0.wrapping_add(1);
        if self.0 == 0 {
            self.0 = 1;
        }
        SiteId(self.0)
    }
}

/// Runbook §10 / territorial IX — full site lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SiteConstructionPhase {
    #[default]
    Planned,
    Surveying,
    Clearing,
    Foundation,
    UnderConstruction,
    Provisioning,
    Operational,
    Damaged,
    Offline,
    Abandoned,
}

#[inline]
pub fn site_phase_from_corridor_coarse(phase: CorridorConstructionPhase) -> SiteConstructionPhase {
    match phase {
        CorridorConstructionPhase::Planned => SiteConstructionPhase::Planned,
        CorridorConstructionPhase::InProgress => SiteConstructionPhase::UnderConstruction,
        CorridorConstructionPhase::Completed => SiteConstructionPhase::Operational,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SiteConstructionStatus {
    pub phase: SiteConstructionPhase,
    /// Phased work (e.g. UnderConstruction, Provisioning); clamp 0..=1.
    pub progress: f32,
}

impl Default for SiteConstructionStatus {
    fn default() -> Self {
        Self {
            phase: SiteConstructionPhase::Operational,
            progress: 1.0,
        }
    }
}

impl SiteConstructionStatus {
    #[inline]
    pub fn operational_factor(&self) -> f32 {
        match self.phase {
            SiteConstructionPhase::Operational => 1.0,
            SiteConstructionPhase::Provisioning => self.progress.clamp(0.0, 1.0),
            SiteConstructionPhase::Damaged => 0.35 * self.progress.clamp(0.0, 1.0),
            SiteConstructionPhase::Offline | SiteConstructionPhase::Abandoned => 0.0,
            _ => 0.0,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct SiteConstructionBook {
    pub by_site: HashMap<SiteId, SiteConstructionStatus>,
}

impl SiteConstructionBook {
    #[inline]
    pub fn operational_factor(&self, id: SiteId) -> f32 {
        self.by_site
            .get(&id)
            .map(SiteConstructionStatus::operational_factor)
            .unwrap_or(1.0)
    }
}

/// Axis-aligned footprint in tile space (grid convention same as [`crate::strategic::build_order::BuildSiteTile`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FootprintTiles {
    pub width: u32,
    pub depth: u32,
}
