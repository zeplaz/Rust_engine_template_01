//! Parametric placement payloads — strategic DTO (raster lives in `crate::construction`).

use bevy::prelude::IVec2;

use crate::strategic::build_order::BuildSiteTile;

/// Committed parametric placement snapshot (carried on [`super::events::CommitConstructionSiteEvent`]).
#[derive(Clone, Debug, PartialEq)]
pub struct CommittedPlacementSnapshot {
    pub origin: BuildSiteTile,
    pub scale_factor: f32,
    pub effective_scale: f32,
    pub rotation_quarter_turns: u8,
    pub mirror_x: bool,
    pub weights: Vec<(IVec2, f32)>,
}

impl CommittedPlacementSnapshot {
    #[must_use]
    pub fn occupied_mass(&self) -> f32 {
        self.weights.iter().map(|(_, w)| *w).sum()
    }
}

#[cfg(test)]
mod tests {
    

}
