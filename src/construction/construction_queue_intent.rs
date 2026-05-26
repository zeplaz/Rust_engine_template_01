//! Construction queue UI intents — panel emits; queue mutation stays in Update consumers.

use bevy::prelude::*;

use crate::strategic::SiteArchetype;

use super::build_state::BuildPlacementPreview;
use super::path_feedback::ConstructionPathFeedback;
use super::pending_construction::PendingConstructionQueue;

#[derive(Message, Clone, Debug)]
pub enum ConstructionQueueIntent {
    ApproveAll,
    ApproveFactories,
    ClearUnapproved,
    ClearAll,
    SetApproved {
        index: usize,
        approved: bool,
    },
    Remove {
        index: usize,
    },
    /// **BQ-128-APPLY-001** — apply imported Wave S preset to ghost (no queue / no commit).
    ApplyImportedPreset {
        preset_index: usize,
    },
}

#[derive(Clone, Debug, Default)]
pub struct ConstructionQueuePanelEntryView {
    pub label: String,
    pub origin_x: u32,
    pub origin_z: u32,
    pub rotation_quarter_turns: u8,
    pub mirror_x: bool,
    pub approved: bool,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct ConstructionQueuePanelView {
    pub ghost_valid: bool,
    pub commit_allowed: bool,
    pub terrain_score: f32,
    pub logistics_score: f32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    /// Path tool (road/rail): last snap hint from cursor placement.
    pub path_snap_hint: Option<String>,
    /// Path tool: validation `required_actions` for blocked segments.
    pub path_required_actions: Vec<String>,
    pub pending_count: usize,
    pub total_count: usize,
    pub entries: Vec<ConstructionQueuePanelEntryView>,
}

pub fn sync_construction_queue_panel_view(
    pending: Res<PendingConstructionQueue>,
    preview: Res<BuildPlacementPreview>,
    path_feedback: Res<ConstructionPathFeedback>,
    mut view: ResMut<ConstructionQueuePanelView>,
) {
    view.ghost_valid = preview.report.valid;
    view.commit_allowed = preview.report.allows_commit;
    view.terrain_score = preview.report.terrain_score;
    view.logistics_score = preview.report.logistics_score;
    view.errors = preview.report.errors.clone();
    view.warnings = preview.report.warnings.clone();
    view.path_snap_hint = path_feedback.snap_hint.clone();
    view.path_required_actions = path_feedback.required_actions.clone();
    view.pending_count = pending.pending_count();
    view.total_count = pending.entries.len();
    view.entries = pending
        .entries
        .iter()
        .map(|entry| ConstructionQueuePanelEntryView {
            label: entry.label.clone(),
            origin_x: entry.origin.x,
            origin_z: entry.origin.z,
            rotation_quarter_turns: entry.rotation_quarter_turns,
            mirror_x: entry.mirror_x,
            approved: entry.approved,
        })
        .collect();
}

pub fn apply_construction_queue_intents(
    mut pending: ResMut<PendingConstructionQueue>,
    mut ghost: ResMut<super::build_state::BuildGhostState>,
    mut tool: ResMut<super::build_tool_authority::ActiveBuildTool>,
    mut strip: ResMut<super::BuildStripState>,
    imported: Option<Res<crate::io::save::WaveSImportedBlueprints>>,
    mut intents: MessageReader<ConstructionQueueIntent>,
) {
    for intent in intents.read() {
        match intent {
            ConstructionQueueIntent::ApproveAll => pending.approve_all(),
            ConstructionQueueIntent::ApproveFactories => {
                pending.approve_matching_archetype(SiteArchetype::Factory);
            }
            ConstructionQueueIntent::ClearUnapproved => pending.clear_unapproved(),
            ConstructionQueueIntent::ClearAll => pending.clear(),
            ConstructionQueueIntent::SetApproved { index, approved } => {
                if let Some(entry) = pending.entries.get_mut(*index) {
                    entry.approved = *approved;
                }
            }
            ConstructionQueueIntent::Remove { index } => pending.remove_at(*index),
            ConstructionQueueIntent::ApplyImportedPreset { preset_index } => {
                let Some(collection) = imported
                    .as_ref()
                    .and_then(|w| w.collection.as_ref())
                else {
                    continue;
                };
                let Some(entry) = collection.presets.get(*preset_index) else {
                    continue;
                };
                super::blueprint_preset::apply_blueprint_preset_to_build_ghost(
                    entry,
                    &mut ghost,
                    &mut tool,
                    &mut strip,
                );
            }
        }
    }
}
