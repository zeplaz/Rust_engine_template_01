//! Organic growth proposal approve/reject HUD (PROC-OG-UX-WIRE-001).

use bevy::prelude::*;

use crate::strategic::settlement::{
    ArchetypeId, AutoBuildPolicyBook, GrowthProposal, GrowthProposalQueue,
};
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

#[derive(Resource, Debug, Default, Clone)]
pub struct GrowthInspectorUiState {
    pub visible: bool,
}


#[must_use]
pub fn growth_inspector_wired_witness_green() -> bool {
    use crate::strategic::settlement::{BuildingUsage, GrowthActorLayer};

    let mut queue = GrowthProposalQueue::default();
    queue.enqueue(GrowthProposal {
        district_id: crate::strategic::DistrictId("d".into()),
        block_id: None,
        archetype_id: ArchetypeId("shop".into()),
        usage: BuildingUsage::Commercial,
        actor_layer: GrowthActorLayer::Growth,
        anchor_tile: IVec2::new(2, 2),
        priority: 0.5,
        seed: 1,
        reason_codes: Vec::new(),
        saturation_at_submit: 0.0,
    });
    approve_growth_proposal(&mut queue, 0).is_some() && queue.proposals.is_empty()
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

pub fn approve_growth_proposal(
    queue: &mut GrowthProposalQueue,
    index: usize,
) -> Option<GrowthProposal> {
    if index < queue.proposals.len() {
        Some(queue.proposals.remove(index))
    } else {
        None
    }
}

pub fn reject_growth_proposal(queue: &mut GrowthProposalQueue, index: usize) {
    if index < queue.proposals.len() {
        queue.proposals.remove(index);
    }
}
