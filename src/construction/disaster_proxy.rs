//! DISASTER-PROXY — cheap hydro/fire → integrity / demolish (construction funnel).

use bevy::prelude::*;

use super::demolish::{
    destructible_stage_from_integrity, execute_demolish_at_tile, DestructibleDamage,
    DestructibleDamageStage, DESTRUCTIBLE_COLLAPSE_INTEGRITY,
};
use super::structural_role::{SiteStructuralRoles, StructuralRole};
use crate::economy::activation::BuildingDefinitionRef;
use crate::render::pixel_world::{DestructionMaskBook, MASK_DESTROYED};
use crate::strategic::{
    BuildSiteTile, ConstructionSite, PlannedSite, SiteFootprint, TileOccupationBook,
    VoxelOccupationBook, VoxelOccupationConfig,
};
use crate::systems::water::SurfaceWaterField;

/// Depth (m) over a Foundation tile that starts flood integrity loss.
pub const FLOOD_FOUNDATION_DEPTH_M: f32 = 1.0;

/// Queue of site origins to demolish after disaster proxies trip Collapsed.
#[derive(Resource, Debug, Default)]
pub struct StructuralCollapseQueue {
    pub tiles: Vec<BuildSiteTile>,
    pub collapsed: u64,
}

/// Flood over Foundation tiles → escalate [`DestructibleDamage`] + mask; terminal → queue.
pub fn flood_foundation_integrity_proxy_system(
    field: Option<Res<SurfaceWaterField>>,
    mut mask: Option<ResMut<DestructionMaskBook>>,
    mut queue: ResMut<StructuralCollapseQueue>,
    mut sites: Query<(
        &SiteFootprint,
        &PlannedSite,
        &SiteStructuralRoles,
        Option<&mut DestructibleDamage>,
    )>,
) {
    let Some(field) = field else {
        return;
    };
    if !field.wired {
        return;
    }
    for (fp, planned, roles, damage) in &mut sites {
        let n = fp.tiles.len().min(roles.roles.len());
        if n == 0 {
            continue;
        }
        let mut max_depth = 0.0f32;
        let mut foundation_wet = false;
        for i in 0..n {
            if roles.roles[i] != StructuralRole::Foundation {
                continue;
            }
            let t = fp.tiles[i];
            let Some(d) = field.depth_at_world(t.x, t.y) else {
                continue;
            };
            if d >= FLOOD_FOUNDATION_DEPTH_M {
                foundation_wet = true;
                max_depth = max_depth.max(d);
                if let Some(mask) = mask.as_mut() {
                    mask.mark_tile(t.x, t.y, MASK_DESTROYED);
                }
            }
        }
        if !foundation_wet {
            continue;
        }
        // Integrity falls as depth exceeds gate (2m ≈ total loss).
        let integrity = (1.0 - ((max_depth - FLOOD_FOUNDATION_DEPTH_M) / 2.0).clamp(0.0, 1.0))
            .clamp(0.0, 1.0);
        let stage = destructible_stage_from_integrity(integrity, DESTRUCTIBLE_COLLAPSE_INTEGRITY);
        if let Some(mut dmg) = damage {
            dmg.stage = dmg.stage.escalate(stage);
            if dmg.stage.is_terminal() {
                queue.tiles.push(planned.origin);
            }
        } else if stage == DestructibleDamageStage::Collapsed
            || integrity <= DESTRUCTIBLE_COLLAPSE_INTEGRITY
        {
            queue.tiles.push(planned.origin);
        }
    }
}

/// Drain [`StructuralCollapseQueue`] through [`execute_demolish_at_tile`].
pub fn apply_structural_collapse_demolish(
    mut commands: Commands,
    mut queue: ResMut<StructuralCollapseQueue>,
    mut occupation: Option<ResMut<TileOccupationBook>>,
    mut voxel_occupation: Option<ResMut<VoxelOccupationBook>>,
    voxel_cfg: Option<Res<VoxelOccupationConfig>>,
    mut destruction_mask: Option<ResMut<DestructionMaskBook>>,
    mut debris_frame: Option<ResMut<crate::render::WorldFireParticleFrame>>,
    sites: Query<(
        Entity,
        &ConstructionSite,
        &PlannedSite,
        &SiteFootprint,
        Option<&BuildingDefinitionRef>,
    )>,
) {
    if queue.tiles.is_empty() {
        return;
    }
    let commit_auth = voxel_cfg
        .as_ref()
        .is_some_and(|c| c.commit_authority_enabled);
    let tiles = std::mem::take(&mut queue.tiles);
    let mut seen = std::collections::HashSet::new();
    for tile in tiles {
        let key = (tile.x, tile.z);
        if !seen.insert(key) {
            continue;
        }
        let (n, _) = execute_demolish_at_tile(
            &mut commands,
            tile,
            &sites,
            occupation.as_deref_mut(),
            voxel_occupation.as_deref_mut(),
            commit_auth,
            destruction_mask.as_deref_mut(),
            debris_frame.as_deref_mut(),
        );
        if n > 0 {
            queue.collapsed = queue.collapsed.saturating_add(u64::from(n));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disaster_proxy_fire_path_uses_demolish_funnel() {
        let fire = include_str!("../systems/fire/entity_exposure.rs");
        let prod = fire.split_once("#[cfg(test)]").map_or(fire, |(p, _)| p);
        assert!(prod.contains("execute_demolish_at_tile"));
        assert!(prod.contains("FireCollapseQueue"));
    }

    #[test]
    fn disaster_proxy_flood_gate_constant_positive() {
        assert!(FLOOD_FOUNDATION_DEPTH_M > 0.0);
    }
}
