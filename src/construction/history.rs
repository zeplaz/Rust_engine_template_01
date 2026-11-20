//! Construction undo stack (Round 2 — Wave C).

use bevy::prelude::*;

use crate::economy::activation::BuildingDefinitionRef;
use crate::strategic::{
    BuildSiteTile, ChunkStrategicOverlay,
    CommitConstructionSiteEvent, ConstructionSite, CorridorConstructionBook, NetworkDirtyMask,
    PlannedSite, SiteConstructionBook, SiteConstructionPhase, SiteFootprint, SiteId, Zone,
};
use crate::systems::transport::{TransportEdgeDirectory, TransportFieldStore, TransportTopology};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::construction_pipeline::{
    rebuild_corridor_transport, SimRailSegmentMarker, SimRoadSegmentMarker,
};
use super::corridor_network::{CorridorKind, CorridorNetwork};

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
    /// One committed road/rail line: undo removes exactly this network segment.
    RoadSegment {
        segment_id: u32,
        kind: CorridorKind,
        head: BuildSiteTile,
        tail: BuildSiteTile,
        tiles_added: Vec<BuildSiteTile>,
        marker_entities: Vec<Entity>,
        /// Lines placed as one path share a group (0 = alone); undo / redo take the group.
        group: u64,
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

    /// Pops the top action and every action under it in the same path group.
    pub fn pop_step(stack: &mut Vec<ConstructionAction>) -> Vec<ConstructionAction> {
        let Some(first) = stack.pop() else {
            return Vec::new();
        };
        let group = first.group();
        let mut out = vec![first];
        while group != 0 && stack.last().is_some_and(|a| a.group() == group) {
            out.extend(stack.pop());
        }
        out
    }
}

impl ConstructionAction {
    /// Path group (0 = not grouped).
    #[must_use]
    pub fn group(&self) -> u64 {
        match self {
            ConstructionAction::RoadSegment { group, .. } => *group,
            _ => 0,
        }
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
    mut roads: ResMut<CorridorNetwork>,
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
    let step = ConstructionHistory::pop_step(&mut history.undo_stack);
    for action in step {
        history.redo_stack.push(action.clone());
        match action {
            ConstructionAction::RoadSegment {
                segment_id,
                marker_entities,
                ..
            } => {
                for entity in marker_entities {
                    if road_markers.get(entity).is_ok() || rail_markers.get(entity).is_ok() {
                        commands.entity(entity).despawn();
                    }
                }
                roads.remove_segment(segment_id);
                rebuild_corridor_transport(
                    &roads,
                    &mut topology,
                    &mut fields,
                    &mut directory,
                    &mut corridor_book,
                    &mut overlays,
                );
                let _ = &params;
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
}

pub fn construction_redo_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut history: ResMut<ConstructionHistory>,
    mut commands: Commands,
    mut roads: ResMut<CorridorNetwork>,
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
    let step = ConstructionHistory::pop_step(&mut history.redo_stack);
    for action in step {
        history.undo_stack.push(action.clone());
        match action {
            ConstructionAction::RoadSegment {
                kind,
                head,
                tail,
                tiles_added,
                group,
                ..
            } => {
                let rail = kind.is_rail();
                let replayed = if rail {
                    super::construction_pipeline::replay_corridor_segment_for_redo(
                        &mut commands, &mut roads, &mut topology, &mut fields, &mut directory,
                        &mut corridor_book, &mut overlays, kind.clone(), head, tail, SimRailSegmentMarker,
                    )
                } else {
                    super::construction_pipeline::replay_corridor_segment_for_redo(
                        &mut commands, &mut roads, &mut topology, &mut fields, &mut directory,
                        &mut corridor_book, &mut overlays, kind.clone(), head, tail, SimRoadSegmentMarker,
                    )
                };
                // The replayed line has a new id: the undo entry just pushed must point at it.
                if let (Some((id, markers)), Some(top)) = (replayed, history.undo_stack.last_mut()) {
                    *top = ConstructionAction::RoadSegment {
                        segment_id: id,
                        kind,
                        head,
                        tail,
                        tiles_added,
                        marker_entities: markers,
                        group,
                    };
                }
                let _ = (&road_markers, &rail_markers, &params);
            }
            ConstructionAction::DemolishedSites(events) => {
                for ev in events {
                    super::demolish::execute_demolish_at_tile(
                        &mut commands,
                        ev.origin,
                        &sites,
                        None,
                        None,
                        false,
                        None,
                        // Redo of a player demolish.
                        super::destruction::DestroyCause::PlayerDemolish,
                    );
                }
            }
            // Site/zone redo needs spawn replay (undo despawn only); demolish redo above.
            ConstructionAction::Site { .. } | ConstructionAction::Zones { .. } => {}
        }
    }
}

pub fn record_road_execution(
    history: &mut ConstructionHistory,
    segment_id: u32,
    kind: CorridorKind,
    (head, tail): (BuildSiteTile, BuildSiteTile),
    tiles_added: Vec<BuildSiteTile>,
    marker_entities: Vec<Entity>,
    edge_id: Option<crate::systems::transport::TransportEdgeId>,
    construction_record: Option<crate::systems::transport::TransportConstructionRecord>,
    group: u64,
) {
    if let (Some(eid), Some(rec)) = (edge_id, construction_record.as_ref()) {
        bevy::log::debug!(
            target: "construction::history",
            edge = eid.0,
            phase = %rec.phase,
            progress = rec.progress,
            "recorded corridor execute"
        );
    }
    let action_kind = if kind.is_rail() { ConstructionActionKind::Rail } else { ConstructionActionKind::Road };
    history.push(
        ConstructionAction::RoadSegment {
            segment_id,
            kind,
            head,
            tail,
            tiles_added,
            marker_entities,
            group,
        },
        action_kind,
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
        let tiles = vec![BuildSiteTile { x: 1, z: 2 }, BuildSiteTile { x: 2, z: 2 }];
        let kind = CorridorKind::Road { tier: crate::infrastructure::RoadTier::Gravel, lanes: 2 };
        record_road_execution(&mut history, 7, kind, (tiles[0], tiles[1]), tiles.clone(), vec![], None, None, 0);
        assert!(history.can_undo());
        let action = history.undo_stack.pop().unwrap();
        history.redo_stack.push(action.clone());
        match action {
            ConstructionAction::RoadSegment { tiles_added, segment_id, .. } => {
                assert_eq!(tiles_added, tiles);
                assert_eq!(segment_id, 7, "undo removes exactly the committed segment");
            }
            _ => panic!("expected road segment action"),
        }
    }

    #[test]
    fn a_placed_path_undoes_as_one_step() {
        let mut history = ConstructionHistory::default();
        let t = |x| BuildSiteTile { x, z: 3 };
        let kind = CorridorKind::Road { tier: crate::infrastructure::RoadTier::Gravel, lanes: 2 };
        record_road_execution(&mut history, 1, kind.clone(), (t(0), t(4)), vec![], vec![], None, None, 0);
        for (i, id) in [2_u32, 3, 4].into_iter().enumerate() {
            let x = 4 + i as u32 * 4;
            record_road_execution(&mut history, id, kind.clone(), (t(x), t(x + 4)), vec![], vec![], None, None, 9);
        }
        let step = ConstructionHistory::pop_step(&mut history.undo_stack);
        assert_eq!(step.len(), 3, "the three stretches of path 9 go together");
        assert_eq!(history.undo_stack.len(), 1, "the earlier lone line stays");
        assert_eq!(ConstructionHistory::pop_step(&mut history.undo_stack).len(), 1);
    }

    #[test]
    fn demolish_undo_action_stores_restore_events() {
        let mut history = ConstructionHistory::default();
        let ev = CommitConstructionSiteEvent {
            site_id: SiteId(42),
            owner: Entity::PLACEHOLDER,
            archetype: SiteArchetype::MilitaryBase,
            origin: BuildSiteTile { x: 5, z: 7 },
            footprint: FootprintTiles { width: 2, depth: 2 },
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
