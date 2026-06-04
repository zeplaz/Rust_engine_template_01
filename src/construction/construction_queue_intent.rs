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
    /// **BQ-128-APPLY-002** — import hydrated Wave S presets into pending queue.
    ImportWaveSPresets {
        mode: super::blueprint_preset::BlueprintImportQueueMode,
        /// Required when `mode == Replace` and queue already has rows.
        replace_confirmed: bool,
    },
}

/// Panel UI state for Wave S import mode (**BQ-128-APPLY-002**).
#[derive(Resource, Clone, Debug, Default)]
pub struct ConstructionBlueprintImportUi {
    pub mode: super::blueprint_preset::BlueprintImportQueueMode,
    pub replace_confirm: bool,
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
            ConstructionQueueIntent::ImportWaveSPresets {
                mode,
                replace_confirmed,
            } => {
                let Some(collection) = imported
                    .as_ref()
                    .and_then(|w| w.collection.as_ref())
                else {
                    continue;
                };
                if collection.presets.is_empty() {
                    continue;
                }
                if matches!(
                    mode,
                    super::blueprint_preset::BlueprintImportQueueMode::Replace
                ) && !pending.entries.is_empty()
                    && !replace_confirmed
                {
                    continue;
                }
                let _ = super::blueprint_preset::import_preset_collection_into_pending_queue(
                    &mut pending,
                    collection,
                    *mode,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::blueprint_preset::{
        blueprint_preset_entry_from_pending, BlueprintImportQueueMode, BlueprintPresetCollectionR8,
    };
    use crate::construction::build_state::BuildGhostState;
    use crate::construction::build_strip::BuildStripState;
    use crate::construction::build_tool_authority::ActiveBuildTool;
    use crate::io::save::WaveSImportedBlueprints;
    use crate::strategic::{BuildSiteTile, FootprintTiles, SiteArchetype};

    #[test]
    fn apply_imported_preset_updates_ghost_not_queue() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ConstructionQueueIntent>()
            .init_resource::<PendingConstructionQueue>()
            .init_resource::<BuildGhostState>()
            .init_resource::<ActiveBuildTool>()
            .init_resource::<BuildStripState>()
            .insert_resource(WaveSImportedBlueprints {
                collection: Some(BlueprintPresetCollectionR8 {
                    schema_version: 1,
                    presets: vec![blueprint_preset_entry_from_pending(
                        "depot_a",
                        SiteArchetype::RailDepot,
                        BuildSiteTile { x: 9, z: 11 },
                        FootprintTiles {
                            width: 2,
                            depth: 2,
                        },
                        "Surface",
                        0,
                        false,
                    )],
                }),
            })
            .add_systems(Update, apply_construction_queue_intents);

        let queue_len_before = app
            .world()
            .resource::<PendingConstructionQueue>()
            .entries
            .len();
        app.world_mut()
            .write_message(ConstructionQueueIntent::ApplyImportedPreset { preset_index: 0 });
        app.update();

        let queue = app.world().resource::<PendingConstructionQueue>();
        assert_eq!(queue.entries.len(), queue_len_before);
        let ghost = app.world().resource::<BuildGhostState>();
        assert_eq!(ghost.origin, Some(BuildSiteTile { x: 9, z: 11 }));
        assert_eq!(ghost.footprint.width, 2);
    }

    #[test]
    fn import_wave_s_presets_append_without_replace_confirm() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ConstructionQueueIntent>()
            .init_resource::<PendingConstructionQueue>()
            .init_resource::<BuildGhostState>()
            .init_resource::<ActiveBuildTool>()
            .init_resource::<BuildStripState>()
            .insert_resource(WaveSImportedBlueprints {
                collection: Some(BlueprintPresetCollectionR8 {
                    schema_version: 1,
                    presets: vec![blueprint_preset_entry_from_pending(
                        "imported",
                        SiteArchetype::RailDepot,
                        BuildSiteTile { x: 1, z: 2 },
                        FootprintTiles {
                            width: 2,
                            depth: 2,
                        },
                        "Surface",
                        0,
                        false,
                    )],
                }),
            })
            .add_systems(Update, apply_construction_queue_intents);

        {
            let mut queue = app.world_mut().resource_mut::<PendingConstructionQueue>();
            queue.push(crate::construction::pending_construction::PendingBuildBlueprint {
                kind: crate::construction::pending_construction::PendingEntryKind::BuildSite,
                label: "keep".into(),
                archetype: SiteArchetype::Factory,
                origin: BuildSiteTile { x: 0, z: 0 },
                footprint: FootprintTiles {
                    width: 1,
                    depth: 1,
                },
                layer: crate::strategic::LayerType::Surface,
                rotation_quarter_turns: 0,
                mirror_x: false,
                approved: false,
                catalog_id: None,
            });
        }
        app.world_mut().write_message(ConstructionQueueIntent::ImportWaveSPresets {
            mode: BlueprintImportQueueMode::Append,
            replace_confirmed: false,
        });
        app.update();

        let queue = app.world().resource::<PendingConstructionQueue>();
        assert_eq!(queue.entries.len(), 2);
        assert_eq!(queue.entries[0].label, "keep");
        assert_eq!(queue.entries[1].label, "imported");
    }

    #[test]
    fn import_wave_s_presets_replace_requires_confirm_when_queue_nonempty() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ConstructionQueueIntent>()
            .init_resource::<PendingConstructionQueue>()
            .init_resource::<BuildGhostState>()
            .init_resource::<ActiveBuildTool>()
            .init_resource::<BuildStripState>()
            .insert_resource(WaveSImportedBlueprints {
                collection: Some(BlueprintPresetCollectionR8 {
                    schema_version: 1,
                    presets: vec![blueprint_preset_entry_from_pending(
                        "imported",
                        SiteArchetype::RailDepot,
                        BuildSiteTile { x: 1, z: 2 },
                        FootprintTiles {
                            width: 2,
                            depth: 2,
                        },
                        "Surface",
                        0,
                        false,
                    )],
                }),
            })
            .add_systems(Update, apply_construction_queue_intents);

        {
            let mut queue = app.world_mut().resource_mut::<PendingConstructionQueue>();
            queue.push(crate::construction::pending_construction::PendingBuildBlueprint {
                kind: crate::construction::pending_construction::PendingEntryKind::BuildSite,
                label: "old".into(),
                archetype: SiteArchetype::Factory,
                origin: BuildSiteTile { x: 0, z: 0 },
                footprint: FootprintTiles {
                    width: 1,
                    depth: 1,
                },
                layer: crate::strategic::LayerType::Surface,
                rotation_quarter_turns: 0,
                mirror_x: false,
                approved: false,
                catalog_id: None,
            });
        }
        app.world_mut().write_message(ConstructionQueueIntent::ImportWaveSPresets {
            mode: BlueprintImportQueueMode::Replace,
            replace_confirmed: false,
        });
        app.update();
        assert_eq!(
            app.world()
                .resource::<PendingConstructionQueue>()
                .entries
                .len(),
            1
        );
        assert_eq!(
            app.world()
                .resource::<PendingConstructionQueue>()
                .entries[0]
                .label,
            "old"
        );

        app.world_mut().write_message(ConstructionQueueIntent::ImportWaveSPresets {
            mode: BlueprintImportQueueMode::Replace,
            replace_confirmed: true,
        });
        app.update();
        let queue = app.world().resource::<PendingConstructionQueue>();
        assert_eq!(queue.entries.len(), 1);
        assert_eq!(queue.entries[0].label, "imported");
    }
}
