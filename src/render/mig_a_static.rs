//! Runtime markers for Bevy 0.19 static-scene adoption (MIG-A2/A16).
//!
//! Migration plugin, audit JSON, and rollup witnesses live in [`crate::dev::mig_a_adoption`].

use bevy::camera::visibility::VisibilityRange;
use bevy::prelude::*;

/// Marks entities that are static for the session (iso tile bulk, module LOD0, fallback sprite).
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MigAStaticBulk;

/// Bundle for newly spawned static bulk draw entities (MIG-A2 + A16 marker).
///
/// MIG-A2 correction (2026-07-06): this bundle previously also inserted `NoCpuCulling`.
/// In Bevy 0.19 that component *excludes the entity from `check_visibility`* — a separate
/// system keeps `ViewVisibility` true (so extraction still runs), but the entity is never
/// added to any view's per-class `VisibleEntities` list, and CPU-queued renderers
/// (`queue_sprites`, mesh2d) queue exclusively from that list. Net effect: every tagged
/// Sprite/Mesh2d became permanently invisible (tactical terrain sprite, street furniture,
/// LOD impostors, procedural build modules). `NoCpuCulling` is only valid on GPU-culled
/// mesh paths; do not re-add it to this bundle.
#[must_use]
pub fn mig_a_static_bulk_bundle() -> MigAStaticBulk {
    MigAStaticBulk
}

/// Tactical-map static bulk — always visible; GPU range hints align with GU-REP LOD later (MIG-A14).
#[must_use]
pub fn mig_a_tactical_visibility_range() -> VisibilityRange {
    VisibilityRange {
        start_margin: 0.0..0.0,
        end_margin: 1.0e9..1.0e9,
        use_aabb: true,
    }
}

#[must_use]
pub fn mig_a1_static_transform_optimizations_enabled() -> bool {
    std::env::var("MIG_A1")
        .ok()
        .is_none_or(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
}

#[must_use]
pub fn mig_a10_spine_dispatch_authority() -> bool {
    std::env::var("MIG_A10")
        .ok()
        .is_none_or(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mig_a_static_bulk_bundle_components() {
        // Must stay a bare marker: adding `NoCpuCulling` here makes CPU-queued
        // renderables (sprites/mesh2d) permanently invisible under Bevy 0.19.
        let bulk = mig_a_static_bulk_bundle();
        let _: MigAStaticBulk = bulk;
    }
}
