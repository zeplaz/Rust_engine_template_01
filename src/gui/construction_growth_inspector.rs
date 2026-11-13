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

#[must_use]
pub fn growth_hud_ecology_hint_wired_witness_green() -> bool {
    use crate::systems::ecology::{
        evaluate_landscape_program, load_landscape_grammar_catalog, ChunkEcology,
        LG1_PILOT_CHUNK, LG1_PILOT_PRESET_ID, VegetationField,
    };
    use crate::systems::weather::ChunkWeather;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<EcologyGrowthHint>()
        .add_systems(Update, sync_ecology_growth_hint);
    let catalog = load_landscape_grammar_catalog();
    let Some(preset) = catalog.presets.get(LG1_PILOT_PRESET_ID) else {
        return false;
    };
    let eval = evaluate_landscape_program(
        preset,
        LG1_PILOT_CHUNK,
        &ChunkEcology::default(),
        &VegetationField::default(),
        &ChunkWeather::default(),
    );
    app.world_mut().spawn((
        crate::systems::ecology::LandscapeProgramOnChunk {
            preset_id: preset.preset_id.clone(),
            evaluation: eval,
        },
    ));
    app.update();
    let hint = app.world().resource::<EcologyGrowthHint>();
    hint.program_chunks >= 1 && hint.topology_kind_count >= 3
}

#[cfg(test)]
mod growth_hud_tests {
    #[test]
    fn growth_hud_ecology_hint_wired_witness_green_lib() {
        assert!(super::growth_hud_ecology_hint_wired_witness_green());
    }
}
