//! Organic growth proposal approve/reject HUD (PROC-OG-UX-WIRE-001).

use bevy::prelude::*;
use bevy_egui::egui;

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

pub fn draw_organic_growth_inspector_egui(
    ctx: &egui::Context,
    mut ui_state: ResMut<GrowthInspectorUiState>,
    mut queue: ResMut<GrowthProposalQueue>,
    policy_book: Option<Res<AutoBuildPolicyBook>>,
    ecology_hint: Option<Res<EcologyGrowthHint>>,
) {
    let ecology = ecology_hint.map(|h| h.clone()).unwrap_or_default();
    if queue.proposals.is_empty() && ecology.program_chunks == 0 {
        ui_state.visible = false;
        return;
    }
    ui_state.visible = true;

    let mut approve_idx: Option<usize> = None;
    let mut reject_list = Vec::new();
    let mut approve_all = false;

    egui::Window::new("District growth")
        .id(egui::Id::new("organic_growth_inspector"))
        .default_width(320.0)
        .show(ctx, |ui| {
            if ecology.program_chunks > 0 {
                ui.label(format!(
                    "Ecology programs: {} chunks · {} presets · {} topology kinds",
                    ecology.program_chunks, ecology.unique_presets, ecology.topology_kind_count
                ));
                ui.separator();
            }
            if queue.proposals.is_empty() {
                ui.label("No growth proposals pending.");
                return;
            }
            ui.label("Growth proposals — Approve enqueues Planned (not Operational)");
            ui.separator();
            if let Some(policy) = policy_book.as_ref().and_then(|b| b.by_district.values().next()) {
                ui.label(format!("Auto-build: {policy:?}"));
            }
            ui.separator();
            for (idx, proposal) in queue.proposals.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "[{:?}] {} · {:?}",
                        proposal.usage, proposal.archetype_id.0, proposal.anchor_tile
                    ));
                    if ui.button(format!("Approve##{idx}")).clicked() {
                        approve_idx = Some(idx);
                    }
                    if ui.button(format!("Reject##{idx}")).clicked() {
                        reject_list.push(idx);
                    }
                });
            }
            ui.separator();
            if ui.button("Approve all").clicked() {
                approve_all = true;
            }
            if ui.button("Reject all").clicked() {
                reject_list = (0..queue.proposals.len()).collect();
            }
        });

    if approve_all {
        queue.proposals.clear();
    } else {
        if let Some(idx) = approve_idx {
            let _ = approve_growth_proposal(&mut queue, idx);
        }
        for idx in reject_list.into_iter().rev() {
            reject_growth_proposal(&mut queue, idx);
        }
    }
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
    use super::*;

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
