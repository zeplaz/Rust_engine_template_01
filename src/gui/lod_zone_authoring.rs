//! Tier-2 operational LOD zones from gameplay sources (settlements, missions, hubs, transport junctions).
//! Writes [`LodZoneRegistry`] only; [`WorldLodPolicyEngine`] reads the registry each frame.

use std::collections::HashMap;

use bevy::diagnostic::FrameCount;
use bevy::math::UVec2;
use bevy::prelude::*;

use crate::entities::production::core::LogisticsSiteRoot;
use crate::gui::representation_policy::LodZoneClass;
use crate::gui::world_representation::{
    LodZoneId, LodZoneRegistry, LodZoneSource, OperationalLodZone, WorldLodBand,
};
use crate::scenario::objectives::{ObjectiveTargetRef, ScenarioObjectiveMarker};
use crate::strategic::SettlementSite;
use crate::systems::transport::TransportEdgeDirectory;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

const MAX_ZONES: usize = 256;

#[must_use]
pub const fn zone_class_for_source(source: LodZoneSource) -> LodZoneClass {
    match source {
        LodZoneSource::JumpPoint => LodZoneClass::JumpPoint,
        LodZoneSource::Settlement | LodZoneSource::LogisticsHub => LodZoneClass::Mission,
        LodZoneSource::CombatFront => LodZoneClass::Combat,
        LodZoneSource::MissionArea => LodZoneClass::Mission,
        LodZoneSource::PlayerFocus => LodZoneClass::Camera,
    }
}

#[inline]
fn chunk_tile_extent(chunk_tiles: UVec2) -> f32 {
    chunk_tiles.x.max(chunk_tiles.y).max(1) as f32
}

#[must_use]
pub fn world_center_from_chunk(anchor: IVec2, chunk_tiles: UVec2) -> Vec3 {
    let cw = chunk_tiles.x.max(1) as f32;
    let ch = chunk_tiles.y.max(1) as f32;
    Vec3::new(
        (anchor.x as f32 + 0.5) * cw,
        (anchor.y as f32 + 0.5) * ch,
        0.0,
    )
}

#[must_use]
pub fn world_center_from_tile(tile: IVec2, chunk_tiles: UVec2) -> Vec3 {
    world_center_from_chunk(tile, chunk_tiles)
}

#[must_use]
pub fn zone_radius_world(chunk_tiles: UVec2, radius_chunks: f32) -> f32 {
    radius_chunks.max(0.25) * chunk_tile_extent(chunk_tiles)
}

#[must_use]
pub fn settlement_lod_band(population: u32) -> WorldLodBand {
    if population >= 50_000 {
        WorldLodBand::Operational
    } else if population >= 5_000 {
        WorldLodBand::Operational
    } else {
        WorldLodBand::Strategic
    }
}

#[must_use]
pub fn settlement_priority(population: u32) -> f32 {
    (population as f32 / 10_000.0).clamp(0.25, 0.9)
}

#[must_use]
pub fn mission_lod_band(kind: crate::scenario::objectives::ScenarioObjectiveKindV1) -> WorldLodBand {
    use crate::scenario::objectives::ScenarioObjectiveKindV1;
    match kind {
        ScenarioObjectiveKindV1::CaptureRegion | ScenarioObjectiveKindV1::DestroyInfrastructure => {
            WorldLodBand::LocalTactical
        }
        ScenarioObjectiveKindV1::MaintainSupply => WorldLodBand::Operational,
    }
}

#[must_use]
pub fn mission_priority(kind: crate::scenario::objectives::ScenarioObjectiveKindV1) -> f32 {
    use crate::scenario::objectives::ScenarioObjectiveKindV1;
    match kind {
        ScenarioObjectiveKindV1::CaptureRegion => 0.9,
        ScenarioObjectiveKindV1::DestroyInfrastructure => 0.85,
        ScenarioObjectiveKindV1::MaintainSupply => 0.7,
    }
}

#[must_use]
pub fn objective_target_center(
    target: &ObjectiveTargetRef,
    chunk_tiles: UVec2,
) -> Option<Vec3> {
    match target {
        ObjectiveTargetRef::Chunk(coord) => Some(world_center_from_chunk(*coord, chunk_tiles)),
        ObjectiveTargetRef::Tile(coord) => Some(world_center_from_tile(*coord, chunk_tiles)),
        ObjectiveTargetRef::Region(_) | ObjectiveTargetRef::Corridor(_) | ObjectiveTargetRef::Site(_) => {
            None
        }
    }
}

#[must_use]
pub fn collect_settlement_zones(
    settlements: &[SettlementSite],
    chunk_tiles: UVec2,
    next_id: &mut LodZoneId,
) -> Vec<OperationalLodZone> {
    let mut out = Vec::new();
    let radius = zone_radius_world(chunk_tiles, 1.5);
    for site in settlements {
        let zone_id = *next_id;
        *next_id = next_id.saturating_add(1);
        out.push(OperationalLodZone {
            zone_id,
            class: zone_class_for_source(LodZoneSource::Settlement),
            center: world_center_from_chunk(site.anchor_chunk, chunk_tiles),
            radius,
            band: settlement_lod_band(site.population),
            priority: settlement_priority(site.population),
            source: LodZoneSource::Settlement,
        });
    }
    out
}

#[must_use]
pub fn collect_mission_zones(
    objectives: &[ScenarioObjectiveMarker],
    chunk_tiles: UVec2,
    next_id: &mut LodZoneId,
) -> Vec<OperationalLodZone> {
    let mut out = Vec::new();
    let radius = zone_radius_world(chunk_tiles, 2.0);
    for objective in objectives {
        let Some(target) = objective.target.as_ref() else {
            continue;
        };
        let Some(center) = objective_target_center(target, chunk_tiles) else {
            continue;
        };
        let zone_id = *next_id;
        *next_id = next_id.saturating_add(1);
        out.push(OperationalLodZone {
            zone_id,
            class: zone_class_for_source(LodZoneSource::MissionArea),
            center,
            radius,
            band: mission_lod_band(objective.kind),
            priority: mission_priority(objective.kind),
            source: LodZoneSource::MissionArea,
        });
    }
    out
}

#[must_use]
pub fn collect_logistics_hub_zones(
    hubs: &[Vec3],
    chunk_tiles: UVec2,
    next_id: &mut LodZoneId,
) -> Vec<OperationalLodZone> {
    let mut out = Vec::new();
    let radius = zone_radius_world(chunk_tiles, 1.0);
    for center in hubs {
        let zone_id = *next_id;
        *next_id = next_id.saturating_add(1);
        out.push(OperationalLodZone {
            zone_id,
            class: zone_class_for_source(LodZoneSource::LogisticsHub),
            center: *center,
            radius,
            band: WorldLodBand::Operational,
            priority: 0.75,
            source: LodZoneSource::LogisticsHub,
        });
    }
    out
}

#[must_use]
pub fn collect_transport_jump_zones(
    directory: &TransportEdgeDirectory,
    chunk_tiles: UVec2,
    next_id: &mut LodZoneId,
) -> Vec<OperationalLodZone> {
    let mut junctions: HashMap<String, (Vec3, u32)> = HashMap::new();
    for meta in directory.by_edge.values() {
        if meta.control_points.is_empty() {
            continue;
        }
        let head = meta.control_points[0];
        let tail = *meta.control_points.last().expect("non-empty");
        for (key, point) in [(&meta.head_key, head), (&meta.tail_key, tail)] {
            if key.is_empty() {
                continue;
            }
            let entry = junctions
                .entry(key.clone())
                .or_insert((Vec3::from_array(point), 0));
            entry.0 = Vec3::from_array(point);
            entry.1 = entry.1.saturating_add(1);
        }
    }

    let mut out = Vec::new();
    let radius = zone_radius_world(chunk_tiles, 0.75);
    for (key, (center, degree)) in junctions {
        if degree < 2 {
            continue;
        }
        let zone_id = *next_id;
        *next_id = next_id.saturating_add(1);
        let priority = (0.55 + (degree as f32 - 2.0) * 0.1).clamp(0.55, 0.85);
        let _ = key;
        out.push(OperationalLodZone {
            zone_id,
            class: zone_class_for_source(LodZoneSource::JumpPoint),
            center,
            radius,
            band: WorldLodBand::Operational,
            priority,
            source: LodZoneSource::JumpPoint,
        });
    }
    out
}

#[must_use]
pub fn build_operational_lod_zones(
    settlements: &[SettlementSite],
    objectives: &[ScenarioObjectiveMarker],
    hubs: &[Vec3],
    directory: &TransportEdgeDirectory,
    chunk_tiles: UVec2,
) -> Vec<OperationalLodZone> {
    let mut next_id = 1;
    let mut zones = Vec::new();
    zones.extend(collect_settlement_zones(settlements, chunk_tiles, &mut next_id));
    zones.extend(collect_mission_zones(objectives, chunk_tiles, &mut next_id));
    zones.extend(collect_logistics_hub_zones(hubs, chunk_tiles, &mut next_id));
    zones.extend(collect_transport_jump_zones(directory, chunk_tiles, &mut next_id));
    zones.truncate(MAX_ZONES);
    zones
}

#[inline]
fn tiles_per_chunk(chunks: &Query<(&Chunk, &ChunkCellMatrix)>) -> UVec2 {
    chunks
        .iter()
        .next()
        .map(|(_, m)| m.size)
        .unwrap_or(UVec2::new(32, 32))
}

pub fn refresh_lod_zone_registry(
    frame: Res<FrameCount>,
    mut last_refresh_frame: Local<u32>,
    chunks: Query<(&Chunk, &ChunkCellMatrix)>,
    settlements: Query<&SettlementSite>,
    objectives: Query<&ScenarioObjectiveMarker>,
    hubs: Query<&Transform, With<LogisticsSiteRoot>>,
    directory: Res<TransportEdgeDirectory>,
    mut registry: ResMut<LodZoneRegistry>,
) {
    const REFRESH_INTERVAL_FRAMES: u32 = 30;
    if *last_refresh_frame != 0
        && frame.0.saturating_sub(*last_refresh_frame) < REFRESH_INTERVAL_FRAMES
    {
        return;
    }
    *last_refresh_frame = frame.0 as u32;
    let chunk_tiles = tiles_per_chunk(&chunks);
    let settlement_rows: Vec<SettlementSite> = settlements.iter().cloned().collect();
    let objective_rows: Vec<ScenarioObjectiveMarker> = objectives.iter().cloned().collect();
    let hub_rows: Vec<Vec3> = hubs
        .iter()
        .map(|transform| {
            let t = transform.translation;
            Vec3::new(t.x, t.y, 0.0)
        })
        .collect();
    registry.zones = build_operational_lod_zones(
        &settlement_rows,
        &objective_rows,
        &hub_rows,
        &directory,
        chunk_tiles,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::objectives::ScenarioObjectiveKindV1;

    #[test]
    fn settlement_zone_uses_chunk_center_world_space() {
        let chunk_tiles = UVec2::new(32, 32);
        let mut next_id = 1;
        let zones = collect_settlement_zones(
            &[SettlementSite::new(12_000, IVec2::new(2, 3), 0.5)],
            chunk_tiles,
            &mut next_id,
        );
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].source, LodZoneSource::Settlement);
        assert_eq!(zones[0].center, world_center_from_chunk(IVec2::new(2, 3), chunk_tiles));
        assert_eq!(zones[0].band, WorldLodBand::Operational);
    }

    #[test]
    fn mission_chunk_target_emits_mission_area_zone() {
        let chunk_tiles = UVec2::new(16, 16);
        let mut next_id = 1;
        let zones = collect_mission_zones(
            &[ScenarioObjectiveMarker {
                objective_id: "cap_pass".to_string(),
                kind: ScenarioObjectiveKindV1::CaptureRegion,
                label: "Cap".to_string(),
                target: Some(ObjectiveTargetRef::Chunk(IVec2::new(1, 1))),
                owning_faction: None,
                opposing_faction: None,
                tags: Vec::new(),
            }],
            chunk_tiles,
            &mut next_id,
        );
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].source, LodZoneSource::MissionArea);
        assert_eq!(zones[0].band, WorldLodBand::LocalTactical);
    }

    #[test]
    fn transport_junction_degree_two_emits_jump_point() {
        use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId, TransportEdgeMeta};

        let mut directory = TransportEdgeDirectory::default();
        directory.by_edge.insert(
            TransportEdgeId(1),
            TransportEdgeMeta {
                head_key: "hub_a".to_string(),
                tail_key: "hub_b".to_string(),
                control_points: vec![[0.0, 0.0, 0.0], [64.0, 0.0, 64.0]],
                ..Default::default()
            },
        );
        directory.by_edge.insert(
            TransportEdgeId(2),
            TransportEdgeMeta {
                head_key: "hub_a".to_string(),
                tail_key: "hub_c".to_string(),
                control_points: vec![[0.0, 0.0, 0.0], [32.0, 0.0, 32.0]],
                ..Default::default()
            },
        );
        let mut next_id = 1;
        let zones = collect_transport_jump_zones(&directory, UVec2::new(32, 32), &mut next_id);
        assert!(zones.iter().any(|z| z.source == LodZoneSource::JumpPoint));
    }
}
