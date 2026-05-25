//! World placement for construction sites — grid + logistics need `Transform`.

use bevy::prelude::*;

use crate::strategic::{BuildSiteTile, PlannedSite};

/// Designer tile → world XZ (metres); Y flat until terrain conform lands on sites.
#[must_use]
pub fn site_world_position(origin: BuildSiteTile) -> Vec3 {
    const METRES_PER_TILE: f32 = 16.0;
    Vec3::new(
        origin.x as f32 * METRES_PER_TILE,
        0.0,
        origin.z as f32 * METRES_PER_TILE,
    )
}

#[derive(Component, Clone, Copy, Debug)]
pub struct SiteWorldTransformApplied;

pub fn ensure_site_world_transform_system(
    mut commands: Commands,
    q: Query<(Entity, &PlannedSite), Without<SiteWorldTransformApplied>>,
) {
    for (entity, planned) in &q {
        commands.entity(entity).insert((
            Transform::from_translation(site_world_position(planned.origin)),
            GlobalTransform::default(),
            SiteWorldTransformApplied,
        ));
    }
}
