//! **FEAT-WSS-HYDRO-READ-001** — player-facing hydrology strings (design pass v1).

use bevy::prelude::*;

use super::HydrologyRuntimeWitness;

/// Surface read for hover / status strip (no engine jargon).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HydroPlayerBand {
    Ocean,
    River,
    Lake,
    Flood,
    DryRiverbed,
    None,
}

impl HydroPlayerBand {
    #[must_use]
    pub const fn status_line(self) -> &'static str {
        match self {
            Self::Ocean => "Open water",
            Self::River => "River — flowing",
            Self::Lake => "Standing water",
            Self::Flood => "Flood — spreading",
            Self::DryRiverbed => "Dry riverbed",
            Self::None => "",
        }
    }

    #[must_use]
    pub const fn tooltip_detail(self) -> Option<(&'static str, Option<&'static str>)> {
        match self {
            Self::River => Some(("Flow follows terrain.", None)),
            Self::Lake => Some(("Calm surface.", None)),
            Self::Ocean => Some((
                "Deep water — strategic view simplifies detail.",
                None,
            )),
            Self::Flood => Some(("Water level rising.", Some("Not a contamination plume."))),
            _ => None,
        }
    }
}

/// Classify slab witness counts into a player band (coarse; tile-local refine later).
#[must_use]
pub fn hydro_player_band_from_witness(witness: &HydrologyRuntimeWitness) -> HydroPlayerBand {
    if witness.ocean_tile_count > 0 && witness.river_channel_cells == 0 {
        return HydroPlayerBand::Ocean;
    }
    if witness.river_channel_cells > 0 {
        return HydroPlayerBand::River;
    }
    if witness.waterborne_contamination_max > 0.15 {
        return HydroPlayerBand::Flood;
    }
    HydroPlayerBand::None
}

/// F3 diagnostics row (dev / expanded diagnostics only).
#[must_use]
pub fn hydro_f3_diagnostics_line(witness: &HydrologyRuntimeWitness) -> String {
    format!(
        "WSS hydro: ocean={} rivers={} slab={}",
        witness.ocean_tile_count,
        witness.river_channel_cells,
        witness.hydrology_hydrated
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feat_wss_hydro_read_001_strings_match_design_pass() {
        assert_eq!(HydroPlayerBand::Ocean.status_line(), "Open water");
        assert_eq!(HydroPlayerBand::River.status_line(), "River — flowing");
        let w = HydrologyRuntimeWitness {
            ocean_tile_count: 4,
            river_channel_cells: 0,
            hydrology_hydrated: true,
            ..Default::default()
        };
        assert_eq!(hydro_player_band_from_witness(&w), HydroPlayerBand::Ocean);
    }
}
