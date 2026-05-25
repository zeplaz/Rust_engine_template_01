//! Zone paint commit → pending queue and strategic [`Zone`] spawn on confirm.

use bevy::prelude::*;

use crate::strategic::{BuildSiteTile, FootprintTiles, LayerType, SiteArchetype, Zone, ZoneKind};

use super::super::build_tool_authority::ZoneTool;
use super::super::pending_construction::{
    PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind,
};

#[must_use]
pub fn zone_tool_tag(zone: ZoneTool) -> &'static str {
    match zone {
        ZoneTool::ResidentialLow => "res_low",
        ZoneTool::ResidentialMedium => "res_med",
        ZoneTool::ResidentialHigh => "res_high",
        ZoneTool::Apartments => "apartments",
        ZoneTool::MixedUse => "mixed",
    }
}

fn zone_strategic_params(zone: ZoneTool) -> (ZoneKind, f32, f32) {
    match zone {
        ZoneTool::ResidentialLow => (ZoneKind::Supply, 2.5, 0.35),
        ZoneTool::ResidentialMedium => (ZoneKind::Supply, 3.5, 0.5),
        ZoneTool::ResidentialHigh => (ZoneKind::Supply, 4.0, 0.65),
        ZoneTool::Apartments => (ZoneKind::Control, 3.0, 0.55),
        ZoneTool::MixedUse => (ZoneKind::Control, 3.5, 0.45),
    }
}

/// Enqueue zone paint rows (not site blueprints).
pub fn commit_painted_zones_to_pending(
    zone: ZoneTool,
    painted: &[BuildSiteTile],
    pending: &mut PendingConstructionQueue,
) -> usize {
    let tag = zone_tool_tag(zone);
    let footprint = FootprintTiles {
        width: 1,
        depth: 1,
    };
    let mut n = 0usize;
    for &origin in painted {
        pending.push(PendingBuildBlueprint {
            kind: PendingEntryKind::ZonePaint(zone),
            label: format!("zone:{tag}:{},{}", origin.x, origin.z),
            archetype: SiteArchetype::CivilHousing,
            origin,
            footprint,
            layer: LayerType::Surface,
            rotation_quarter_turns: 0,
            mirror_x: false,
            approved: false,
            catalog_id: None,
        });
        n += 1;
    }
    n
}

/// Spawn strategic zone field at tile (confirm path).
pub fn spawn_zone_at_tile(commands: &mut Commands, zone: ZoneTool, tile: BuildSiteTile) -> Entity {
    let (kind, radius, intensity) = zone_strategic_params(zone);
    commands
        .spawn(Zone {
            kind,
            faction_slot: 0,
            center_tile: IVec2::new(tile.x as i32, tile.z as i32),
            radius_tiles: radius,
            intensity,
        })
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::BuildSiteTile;

    #[test]
    fn commit_painted_zones_queues_zone_kind() {
        let mut pending = PendingConstructionQueue::default();
        let tiles = [BuildSiteTile { x: 1, z: 2 }];
        let n = commit_painted_zones_to_pending(ZoneTool::ResidentialLow, &tiles, &mut pending);
        assert_eq!(n, 1);
        assert!(matches!(
            pending.entries[0].kind,
            PendingEntryKind::ZonePaint(ZoneTool::ResidentialLow)
        ));
        assert!(pending.entries[0].label.starts_with("zone:res_low:"));
    }
}
