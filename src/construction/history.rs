//! Construction undo stack (Round 2 — Wave C).

use bevy::prelude::*;

use crate::economy::activation::BuildingDefinitionRef;
use crate::strategic::{
    align_corridor_book_with_transport_directory, BuildSiteTile, ChunkStrategicOverlay,
    CommitConstructionSiteEvent, ConstructionSite, CorridorConstructionBook, NetworkDirtyMask,
    PlannedSite, SiteConstructionBook, SiteConstructionPhase, SiteFootprint, SiteId, Zone,
};
use crate::systems::transport::bake_snapshot_from_ordered_tile_markers;
use crate::systems::transport::{
    hydrate_transport_from_snapshot, TransportEdgeDirectory, TransportFieldStore, TransportTopology,
};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::construction_pipeline::{
    ExecutedRoadNetwork, SimRailSegmentMarker, SimRoadSegmentMarker, ROAD_MARKER_Y_BIAS,
    ROAD_MARKER_Y_SCALE,
};

const MAX_UNDO: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructionActionKind {
    Road,
    Rail,
    Site,
    Zone,
}

#[derive(Clone, Debug)]
pub enum ConstructionAction {
    RoadSegment {
        tiles_added: Vec<BuildSiteTile>,
        marker_entities: Vec<Entity>,
    },
    Site {
        entity: Entity,
        site_id: SiteId,
    },
    Zones {
        entities: Vec<Entity>,
    },
    /// Sites removed by demolish — undo restores via [`CommitConstructionSiteEvent`].
    DemolishedSites(Vec<CommitConstructionSiteEvent>),
}

/// Sites committed this frame — resolved to [`ConstructionAction::Site`] after spawn.
#[derive(Clone, Debug)]
pub struct PendingSiteHistory {
    pub origin: BuildSiteTile,
}

#[derive(Resource, Default, Debug)]
pub struct ConstructionHistory {
    pub undo_stack: Vec<ConstructionAction>,
    pub redo_stack: Vec<ConstructionAction>,
    pub pending_sites: Vec<PendingSiteHistory>,
    pub last_action_kind: Option<ConstructionActionKind>,
}

impl ConstructionHistory {
    pub fn push(&mut self, action: ConstructionAction, kind: ConstructionActionKind) {
        self.redo_stack.clear();
        self.last_action_kind = Some(kind);
        self.undo_stack.push(action);
        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.remove(0);
        }
    }

    pub fn queue_site(&mut self, origin: BuildSiteTile) {
        self.pending_sites.push(PendingSiteHistory { origin });
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
}

pub fn finalize_site_history_records(
    mut history: ResMut<ConstructionHistory>,
    sites: Query<(Entity, &PlannedSite), Added<PlannedSite>>,
) {
    if history.pending_sites.is_empty() {
        return;
    }
    let pending: Vec<_> = history.pending_sites.drain(..).collect();
    for p in pending {
        for (entity, planned) in sites.iter() {
            if planned.origin == p.origin {
                history.push(
                    ConstructionAction::Site {
                        entity,
                        site_id: planned.site_id,
                    },
                    ConstructionActionKind::Site,
                );
                break;
            }
        }
    }
}

pub fn construction_undo_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut history: ResMut<ConstructionHistory>,
    mut site_events: MessageWriter<CommitConstructionSiteEvent>,
    mut commands: Commands,
    mut roads: ResMut<ExecutedRoadNetwork>,
    mut book: ResMut<SiteConstructionBook>,
    mut topology: ResMut<TransportTopology>,
    mut fields: ResMut<TransportFieldStore>,
    mut directory: ResMut<TransportEdgeDirectory>,
    mut corridor_book: ResMut<CorridorConstructionBook>,
    mut overlays: Query<&mut NetworkDirtyMask, With<ChunkStrategicOverlay>>,
    params: Res<WorldGenParams>,
    road_markers: Query<Entity, With<SimRoadSegmentMarker>>,
    rail_markers: Query<Entity, With<SimRailSegmentMarker>>,
    sites: Query<Entity, With<ConstructionSite>>,
    zones: Query<Entity, With<Zone>>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if !ctrl || !keyboard.just_pressed(KeyCode::KeyZ) {
        return;
    }
    let Some(action) = history.undo_stack.pop() else {
        return;
    };
    history.redo_stack.push(action.clone());
    match action {
        ConstructionAction::RoadSegment {
            tiles_added,
            marker_entities,
        } => {
            for entity in marker_entities {
                if road_markers.get(entity).is_ok() || rail_markers.get(entity).is_ok() {
                    commands.entity(entity).despawn();
                }
            }
            for tile in tiles_added {
                roads.tiles.retain(|t| *t != tile);
            }
            rebuild_road_transport(
                &mut roads,
                &params,
                &mut topology,
                &mut fields,
                &mut directory,
                &mut corridor_book,
                &mut overlays,
            );
        }
        ConstructionAction::Site { entity, site_id } => {
            if sites.get(entity).is_ok() {
                commands.entity(entity).despawn();
            }
            book.by_site.remove(&site_id);
        }
        ConstructionAction::Zones { entities } => {
            for entity in entities {
                if zones.get(entity).is_ok() {
                    commands.entity(entity).despawn();
                }
            }
        }
        ConstructionAction::DemolishedSites(events) => {
            for ev in &events {
                site_events.write(ev.clone());
            }
        }
    }
}

pub fn construction_redo_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut history: ResMut<ConstructionHistory>,
    mut commands: Commands,
    mut roads: ResMut<super::construction_pipeline::ExecutedRoadNetwork>,
    mut topology: ResMut<TransportTopology>,
    mut fields: ResMut<TransportFieldStore>,
    mut directory: ResMut<TransportEdgeDirectory>,
    mut corridor_book: ResMut<CorridorConstructionBook>,
    mut overlays: Query<&mut NetworkDirtyMask, With<ChunkStrategicOverlay>>,
    _site_events: MessageWriter<CommitConstructionSiteEvent>,
    params: Res<WorldGenParams>,
    road_markers: Query<Entity, With<super::construction_pipeline::SimRoadSegmentMarker>>,
    rail_markers: Query<Entity, With<super::construction_pipeline::SimRailSegmentMarker>>,
    sites: Query<(
        Entity,
        &ConstructionSite,
        &PlannedSite,
        &SiteFootprint,
        Option<&BuildingDefinitionRef>,
    )>,
    _zones: Query<Entity, With<Zone>>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if !ctrl || !keyboard.just_pressed(KeyCode::KeyY) {
        return;
    }
    let Some(action) = history.redo_stack.pop() else {
        return;
    };
    history.undo_stack.push(action.clone());
    match action {
        ConstructionAction::RoadSegment {
            tiles_added,
            marker_entities: _,
        } => {
            let _markers = super::construction_pipeline::replay_road_tiles_for_redo(
                &mut commands,
                &mut roads,
                &params,
                &mut topology,
                &mut fields,
                &mut directory,
                &mut corridor_book,
                &mut overlays,
                &tiles_added,
                super::construction_pipeline::SimRoadSegmentMarker,
            );
            let _ = road_markers;
            let _ = rail_markers;
        }
        ConstructionAction::DemolishedSites(events) => {
            for ev in events {
                super::demolish::execute_demolish_at_tile(
                    &mut commands,
                    ev.origin,
                    &sites,
                    None,
                );
            }
        }
        // Site/zone redo needs spawn replay (undo despawn only); demolish redo above.
        ConstructionAction::Site { .. } | ConstructionAction::Zones { .. } => {}
    }
}

fn rebuild_road_transport(
    roads: &mut ExecutedRoadNetwork,
    params: &WorldGenParams,
    topology: &mut TransportTopology,
    fields: &mut TransportFieldStore,
    directory: &mut TransportEdgeDirectory,
    corridor_book: &mut CorridorConstructionBook,
    overlays: &mut Query<&mut NetworkDirtyMask, With<ChunkStrategicOverlay>>,
) {
    let marker_tiles: Vec<(u32, u32)> = roads.tiles.iter().map(|t| (t.x, t.z)).collect();
    if marker_tiles.is_empty() {
        return;
    }
    let snap = bake_snapshot_from_ordered_tile_markers(
        &marker_tiles,
        |_x, _z| 0.5,
        ROAD_MARKER_Y_SCALE,
        ROAD_MARKER_Y_BIAS,
    );
    if hydrate_transport_from_snapshot(topology, fields, directory, &snap).is_ok() {
        align_corridor_book_with_transport_directory(directory, corridor_book);
        for mut mask in overlays.iter_mut() {
            mask.mask |= crate::strategic::NETWORK_DIRTY_FLOW
                | crate::strategic::NETWORK_DIRTY_CONNECTIVITY;
        }
    }
    let _ = params;
}

pub fn record_road_execution(
    history: &mut ConstructionHistory,
    tiles_added: Vec<BuildSiteTile>,
    marker_entities: Vec<Entity>,
) {
    if tiles_added.is_empty() && marker_entities.is_empty() {
        return;
    }
    history.push(
        ConstructionAction::RoadSegment {
            tiles_added,
            marker_entities,
        },
        ConstructionActionKind::Road,
    );
}

pub fn record_zone_spawns(history: &mut ConstructionHistory, entities: Vec<Entity>) {
    if entities.is_empty() {
        return;
    }
    history.push(
        ConstructionAction::Zones { entities },
        ConstructionActionKind::Zone,
    );
}

pub fn record_demolish_execution(
    history: &mut ConstructionHistory,
    events: Vec<CommitConstructionSiteEvent>,
) {
    if events.is_empty() {
        return;
    }
    history.push(
        ConstructionAction::DemolishedSites(events),
        ConstructionActionKind::Site,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::{BuildSiteTile, FootprintTiles, LayerType, SiteArchetype};

    #[test]
    fn road_redo_stack_records_and_replays_tiles() {
        let mut history = ConstructionHistory::default();
        let tiles = vec![
            BuildSiteTile { x: 1, z: 2 },
            BuildSiteTile { x: 2, z: 2 },
        ];
        record_road_execution(&mut history, tiles.clone(), vec![]);
        assert!(history.can_undo());
        let action = history.undo_stack.pop().unwrap();
        history.redo_stack.push(action.clone());
        match action {
            ConstructionAction::RoadSegment { tiles_added, .. } => {
                assert_eq!(tiles_added, tiles);
            }
            _ => panic!("expected road segment action"),
        }
    }

    #[test]
    fn demolish_undo_action_stores_restore_events() {
        let mut history = ConstructionHistory::default();
        let ev = CommitConstructionSiteEvent {
            site_id: SiteId(42),
            owner: Entity::PLACEHOLDER,
            archetype: SiteArchetype::MilitaryBase,
            origin: BuildSiteTile { x: 5, z: 7 },
            footprint: FootprintTiles {
                width: 2,
                depth: 2,
            },
            layer: LayerType::Surface,
            catalog_id: Some("test_factory".into()),
            placement: None,
        };
        record_demolish_execution(&mut history, vec![ev.clone()]);
        match &history.undo_stack[0] {
            ConstructionAction::DemolishedSites(events) => {
                assert_eq!(events.len(), 1);
                assert_eq!(events[0].origin, ev.origin);
            }
            _ => panic!("expected demolished sites action"),
        }
    }
}

/// Advance survey phases before strategic logistics takes UnderConstruction.
/// Superseded by [`advance_site_construction_tick_system`]; remove when CON-P2 schedule lands.
#[allow(dead_code)]
pub fn advance_early_construction_phases_system(
    time: Res<Time>,
    mut q: Query<(&mut ConstructionSite, &PlannedSite)>,
    mut book: ResMut<SiteConstructionBook>,
) {
    let dt = time.delta_secs();
    const STEP_SECS: f32 = 1.25;
    for (mut site, planned) in &mut q {
        let next = match site.phase {
            SiteConstructionPhase::Planned => Some(SiteConstructionPhase::Surveying),
            SiteConstructionPhase::Surveying => Some(SiteConstructionPhase::Clearing),
            SiteConstructionPhase::Clearing => Some(SiteConstructionPhase::Foundation),
            SiteConstructionPhase::Foundation => Some(SiteConstructionPhase::UnderConstruction),
            _ => None,
        };
        let Some(next_phase) = next else {
            continue;
        };
        if let Some(st) = book.by_site.get_mut(&planned.site_id) {
            st.progress = (st.progress + dt / STEP_SECS).clamp(0.0, 1.0);
            if st.progress >= 1.0 {
                site.phase = next_phase;
                st.phase = next_phase;
                st.progress = 0.0;
                site.operational_readiness = match next_phase {
                    SiteConstructionPhase::UnderConstruction => 0.05,
                    _ => 0.0,
                };
            }
        }
    }
}
