//! **BQ-C4-SCALE-001** — bake metres → grid placement → iso draw scale authority audit.

use super::module_contract::{FLOOR_HEIGHT_M, GRID_UNIT_M};
use crate::construction::iso_draw_scale::DEFAULT_ISO_DRAW_SCALE_MULTIPLIER;

/// **K01 authority:** collision / assembly use [`super::procedural_module_local_translation`];
/// [`ConstructionIsoDrawScale`] applies only at PG-2 spawn (visual readability — BUILD-READ-WORLD-002).
pub const SCALE_AUTHORITY_DECISION: &str = "visual_only_iso_draw";

#[derive(Debug, Clone, PartialEq)]
pub struct ScaleChainLink {
    pub stage: &'static str,
    pub authority: &'static str,
    pub constant: &'static str,
    pub value: f32,
}

#[must_use]
pub fn scale_chain_links() -> Vec<ScaleChainLink> {
    vec![
        ScaleChainLink {
            stage: "bake_contract",
            authority: "module_contract_v1",
            constant: "GRID_UNIT_M",
            value: GRID_UNIT_M,
        },
        ScaleChainLink {
            stage: "bake_contract",
            authority: "module_contract_v1",
            constant: "FLOOR_HEIGHT_M",
            value: FLOOR_HEIGHT_M,
        },
        ScaleChainLink {
            stage: "placement",
            authority: "procedural_module_local_translation",
            constant: "floor_y = floor * FLOOR_HEIGHT_M",
            value: FLOOR_HEIGHT_M,
        },
        ScaleChainLink {
            stage: "placement",
            authority: "procedural_module_local_translation",
            constant: "xz = grid * GRID_UNIT_M",
            value: GRID_UNIT_M,
        },
        ScaleChainLink {
            stage: "spawn_visual",
            authority: "ConstructionIsoDrawScale",
            constant: "iso_draw_scale_multiplier",
            value: DEFAULT_ISO_DRAW_SCALE_MULTIPLIER,
        },
    ]
}



#[cfg(test)]
mod tests {
    


}
