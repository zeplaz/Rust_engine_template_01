//! # PHYS-04 / PHYS-11 residual
//!
//! CQ-55 removed the unused coherent pair pipeline (`CoherentPairCache`, broad/mid/narrow/warm).
//! Nothing created collision bodies in play, and the "spatial hash" was O(n²). Spacing belongs
//! to CQ-53 steering; hard non-overlap for thousands of movers needs a designed uniform grid later.
//!
//! PHYS-04 material filter / solid-contact gates live on
//! [`crate::terrain::material::MaterialRegistry`] (`probe_hits_solid`, `materials_collide`).

use bevy::prelude::*;

/// Minimal collision plugin — registers no pair-cache resources (CQ-55).
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, _app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collision_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(CollisionPlugin);
    }
}
