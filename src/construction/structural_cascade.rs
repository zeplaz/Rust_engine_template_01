//! STRUCT-CASCADE — foundation loss → load-bearing mask collapse.

use bevy::prelude::*;

use super::structural_role::{SiteStructuralRoles, StructuralRole};
use crate::render::pixel_world::{DestructionMaskBook, MASK_DESTROYED};
use crate::strategic::SiteFootprint;

/// If any Foundation tile is destroyed, mark all load-bearing footprint tiles destroyed.
pub fn cascade_destroyed_load_bearing(
    mask: &mut DestructionMaskBook,
    footprint: &SiteFootprint,
    roles: &SiteStructuralRoles,
) -> u32 {
    if footprint.tiles.is_empty() || roles.roles.is_empty() {
        return 0;
    }
    let n = footprint.tiles.len().min(roles.roles.len());
    let foundation_hit = (0..n).any(|i| {
        roles.roles[i] == StructuralRole::Foundation
            && mask.cell(footprint.tiles[i].x, footprint.tiles[i].y) >= MASK_DESTROYED
    });
    if !foundation_hit {
        return 0;
    }
    let mut marked = 0u32;
    for i in 0..n {
        if !roles.roles[i].is_load_bearing() {
            continue;
        }
        let t = footprint.tiles[i];
        if mask.cell(t.x, t.y) < MASK_DESTROYED {
            mask.mark_tile(t.x, t.y, MASK_DESTROYED);
            marked = marked.saturating_add(1);
        }
    }
    marked
}

/// Presentation cascade — runs when mask + dam/site roles are present.
pub fn cascade_mask_from_roles_system(
    mut mask: Option<ResMut<DestructionMaskBook>>,
    q: Query<(&SiteFootprint, &SiteStructuralRoles)>,
) {
    let Some(mut mask) = mask else {
        return;
    };
    if mask.width == 0 {
        return;
    }
    for (fp, roles) in &q {
        cascade_destroyed_load_bearing(&mut mask, fp, roles);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::IVec2;

    #[test]
    fn struct_cascade_foundation_hit_marks_columns() {
        let mut mask = DestructionMaskBook::default();
        mask.ensure_size(16, 16);
        let fp = SiteFootprint {
            tiles: vec![
                IVec2::new(1, 1),
                IVec2::new(2, 1),
                IVec2::new(3, 1),
            ],
            layer: crate::strategic::LayerType::Surface,
        };
        let mut roles = SiteStructuralRoles::default();
        roles.ensure_dam_span(3);
        mask.mark_tile(1, 1, MASK_DESTROYED); // foundation end
        let n = cascade_destroyed_load_bearing(&mut mask, &fp, &roles);
        assert!(n >= 1);
        assert_eq!(mask.cell(2, 1), MASK_DESTROYED);
        assert_eq!(mask.cell(3, 1), MASK_DESTROYED);
    }
}
