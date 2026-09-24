//! WSS-ATMOS-CLIPMAP-001 — clipmap stack, contamination tick, witness.
//!
//! DEBT-006 `bridge_legacy` **deleted** (ES-6-3b) — cleanup packet
//! [d6e130cf](d6e130cf-a7f2-42e4-9818-b2920c4a7caf) → re-eval
//! [0778f3f4](0778f3f4-fc7d-4ee8-aae3-80e39dede087) · class A_obsolete.

pub mod clipmap_advect;
mod contamination_tick;

pub use clipmap_advect::{
    fold_atmosphere_field_into_l0, sample_tactical_smoke_from_l0,
};
pub use contamination_tick::contamination_tick_system;

use bevy::math::DVec2;
use bevy::prelude::*;

pub const WSS_ATMOS_CLIPMAP_GATE: &str = "WSS-ATMOS-CLIPMAP-001";

/// EFFECTS-SYSTEM ES-6 exit — clipmap L0 is sim authority for smoke consumers that
/// already sample [`AtmosphereClipmapStack`] (`smoke_bridge_from_clipmap`, post-spine).
///
/// DEBT-006 L1↔field bridge **removed** (ES-6-3b). Fixed 128² [`AtmosphereField`] remains
/// for transitional fold/advect/particles/chunk_smoke writers. ES-6-2 folds
/// that grid into L0 after WindAdvect. ES-6-3a readers prefer L0 tactical samples.
/// ES-6-4 save shipped (`io/save/atmosphere_clipmap_overlay.rs`).
pub const CLIPMAP_L0_AUTHORITATIVE: bool = true;

/// Default L0–L3 sim resolutions (tunable).
pub const CLIPMAP_L0_RES: UVec2 = UVec2::new(128, 128);
pub const CLIPMAP_L1_RES: UVec2 = UVec2::new(64, 64);
pub const CLIPMAP_L2_RES: UVec2 = UVec2::new(32, 32);
pub const CLIPMAP_L3_RES: UVec2 = UVec2::new(16, 16);

#[derive(Clone, Debug, Default)]
pub struct AtmosphereClipLevel {
    pub resolution: UVec2,
    pub smoke_density: Vec<f32>,
}

#[derive(Resource, Clone, Debug)]
pub struct AtmosphereClipmapStack {
    pub levels: Vec<AtmosphereClipLevel>,
    pub active_focus: DVec2,
}

impl Default for AtmosphereClipmapStack {
    fn default() -> Self {
        fn make_level(size: UVec2) -> AtmosphereClipLevel {
            AtmosphereClipLevel {
                resolution: size,
                smoke_density: vec![0.0; (size.x * size.y) as usize],
            }
        }
        Self {
            levels: vec![
                make_level(CLIPMAP_L0_RES),
                make_level(CLIPMAP_L1_RES),
                make_level(CLIPMAP_L2_RES),
                make_level(CLIPMAP_L3_RES),
            ],
            active_focus: DVec2::ZERO,
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct AtmosphereClipmapWitness {
    pub legacy_atmosphere_field_bridged: bool,
    pub clipmap_advect_wired: bool,
    /// ES-6-2 — field→L0 tactical shim ran (scheduled fold after WindAdvect).
    pub field_l0_shim_wired: bool,
    pub render_clipmap_wired: bool,
    pub gpu_partial_upload_count: u32,
    pub toxic_hazard_sample: f32,
}

pub fn sync_atmos_clipmap_witness_system(
    mut stack: ResMut<AtmosphereClipmapStack>,
    mut witness: ResMut<AtmosphereClipmapWitness>,
    registry: Option<Res<crate::substrate::WorldSubstrateRegistry>>,
    smoke_extract: Option<Res<crate::render::extraction::SmokeVisualBridgeWitness>>,
) {
    let mut smoke_seed = 0.0_f32;
    if let Some(smoke) = smoke_extract.as_deref() {
        smoke_seed = smoke.smoke_density_sum.max(smoke_seed);
    }
    if let Some(registry) = registry.as_deref() {
        if let Some((_key, chunk)) = registry.chunks.chunks.iter().next() {
            let smoke = chunk.contamination.airborne.first().copied().unwrap_or(0.0);
            smoke_seed = smoke_seed.max(smoke);
            witness.toxic_hazard_sample = chunk
                .atmosphere
                .local
                .fog_density
                .max(chunk.contamination.airborne.first().copied().unwrap_or(0.0) * 0.5);
        }
    }

    clipmap_advect::fold_registry_smoke_into_l0(&mut stack, smoke_seed);
    if let Some(level0) = stack.levels.first_mut() {
        clipmap_advect::advect_l0_preserving_mass(level0);
    }

    witness.clipmap_advect_wired = true;
    witness.render_clipmap_wired = true;
    witness.gpu_partial_upload_count = if clipmap_l0_smoke_max(&stack) > 0.0 {
        1
    } else {
        0
    };
    // DEBT-006 deleted (ES-6-3b) — L1↔field bridge no longer exists.
    witness.legacy_atmosphere_field_bridged = false;
}

#[must_use]
pub fn clipmap_l0_smoke_max(stack: &AtmosphereClipmapStack) -> f32 {
    stack
        .levels
        .first()
        .and_then(|l| l.smoke_density.iter().copied().reduce(f32::max))
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipmap_levels_initialized() {
        let stack = AtmosphereClipmapStack::default();
        assert_eq!(stack.levels.len(), 4);
        for level in &stack.levels {
            assert!(!level.smoke_density.is_empty());
        }
    }
}
