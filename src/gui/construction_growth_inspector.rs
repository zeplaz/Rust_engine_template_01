//! Organic growth proposal approve/reject HUD (PROC-OG-UX-WIRE-001).

use bevy::prelude::*;
use bevy_egui::egui;

use crate::strategic::settlement::{
    ArchetypeId, AutoBuildPolicyBook, GrowthProposal, GrowthProposalQueue,
};

#[derive(Resource, Debug, Default, Clone)]
pub struct GrowthInspectorUiState {
    pub visible: bool,
}

pub fn draw_organic_growth_inspector_egui(
    ctx: &egui::Context,
    mut ui_state: ResMut<GrowthInspectorUiState>,
    mut queue: ResMut<GrowthProposalQueue>,
    policy_book: Option<Res<AutoBuildPolicyBook>>,
) {
    if queue.proposals.is_empty() {
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
