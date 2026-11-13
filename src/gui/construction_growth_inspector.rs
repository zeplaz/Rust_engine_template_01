//! Ecology program summary read by the info tabs. (The growth approve/reject inspector had no UI and
//! was removed in the dev-panel cull; growth proposals are approved through `strategic::settlement`.)

use bevy::prelude::*;

use crate::systems::ecology::LandscapeProgramOnChunk;

/// Ecology program summary for growth HUD hints (CDR-B-GROWTH-HUD-VEG-001).
#[derive(Resource, Debug, Default, Clone)]
pub struct EcologyGrowthHint {
    pub program_chunks: u32,
    pub unique_presets: u32,
    pub topology_kind_count: u32,
}

pub fn sync_ecology_growth_hint(
    programs: Query<&LandscapeProgramOnChunk>,
    mut hint: ResMut<EcologyGrowthHint>,
) {
    let mut presets = std::collections::BTreeSet::new();
    let mut kinds = std::collections::BTreeSet::new();
    let mut count = 0u32;
    for program in &programs {
        count = count.saturating_add(1);
        presets.insert(program.preset_id.clone());
        for kind in &program.evaluation.topology_kinds {
            kinds.insert(kind.clone());
        }
    }
    hint.program_chunks = count;
    hint.unique_presets = presets.len() as u32;
    hint.topology_kind_count = kinds.len() as u32;
}

#[cfg(test)]
mod growth_hud_tests {
}
