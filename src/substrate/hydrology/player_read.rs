//! **FEAT-WSS-HYDRO-READ-001** — player-facing hydrology strings (design pass v1).

use bevy::prelude::*;

use super::HydrologyRuntimeStats;

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

/// F3 diagnostics row (dev / expanded diagnostics only).
#[must_use]
pub fn hydro_f3_diagnostics_line(witness: &HydrologyRuntimeStats) -> String {
    format!(
        "WSS hydro: ocean={} rivers={} slab={}",
        witness.ocean_tile_count,
        witness.river_channel_cells,
        witness.hydrology_hydrated
    )
}

#[cfg(test)]
mod tests {
    

}
