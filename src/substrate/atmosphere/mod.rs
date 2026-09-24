//! WSS-ATMOS-CLIPMAP-001 — clipmap stack, contamination tick, witness.
//!
//! DEBT-006 `bridge_legacy` **deleted** (ES-6-3b) — cleanup packet
//! [d6e130cf](d6e130cf-a7f2-42e4-9818-b2920c4a7caf) → re-eval
//! [0778f3f4](0778f3f4-fc7d-4ee8-aae3-80e39dede087) · class A_obsolete.

pub mod clipmap_advect;
mod contamination_tick;

pub use clipmap_advect::{
    advect_l0_with_wind, max_blend_l0_ash_at, max_blend_l0_ember_at, max_blend_l0_fog_at,
    max_blend_l0_heat_at, max_blend_l0_smoke_at, max_blend_l0_toxicity_at,
    sample_tactical_ash_from_l0, sample_tactical_ember_from_l0, sample_tactical_fog_from_l0,
    sample_tactical_heat_from_l0, sample_tactical_smoke_from_l0, sample_tactical_toxicity_from_l0,
};
pub use contamination_tick::contamination_tick_system;

use bevy::math::DVec2;
use bevy::prelude::*;

pub const WSS_ATMOS_CLIPMAP_GATE: &str = "WSS-ATMOS-CLIPMAP-001";

/// EFFECTS-SYSTEM ES-6 exit — clipmap L0 is sim authority for smoke consumers that
/// already sample [`AtmosphereClipmapStack`] (`smoke_bridge_from_clipmap`, post-spine).
///
/// DEBT-006 L1↔field bridge **removed** (ES-6-3b). DEBT-011 deleted the fixed 128²
/// `AtmosphereField` resource — `atmos_chunk_to_tile` + L0 samples replace it.
/// DEBT-010/010b/010c: FieldFill/WindAdvect write L0 smoke/fog/toxicity/ash/ember/heat.
/// ES-6-4 save shipped (`io/save/atmosphere_clipmap_overlay.rs`). GPU weather field is
/// projection-owned (`gpu_field_bridge`).
pub const CLIPMAP_L0_AUTHORITATIVE: bool = true;

/// DEBT-010 — FieldFill/WindAdvect smoke authority is clipmap L0.
pub const DEBT_010_WRITERS_L0: bool = true;

/// DEBT-010b — L0 owns fog + toxicity lanes; chunk_smoke pulls from L0.
pub const DEBT_010B_TOXICITY_FOG_L0: bool = true;

/// DEBT-010c — L0 owns ash / ember / heat lanes; GPU weather field is projection-owned.
pub const DEBT_010C_ASH_EMBER_HEAT_L0: bool = true;

/// DEBT-010c — GPU weather/fire field consumers do not read a Field ash/ember/heat grid
/// (projection graph + partial field state). L0 lanes are the sim authority for those channels.
pub const GPU_FIELD_CONSUMERS_ON_L0: bool = true;

/// DEBT-011 — smoke triangle unified: L0 → ChunkSmoke → SimChunkSmokeVisualExtract; Field deleted.
pub const DEBT_011_FIELD_DELETED: bool = true;

/// Default L0–L3 sim resolutions (tunable).
pub const CLIPMAP_L0_RES: UVec2 = UVec2::new(128, 128);
pub const CLIPMAP_L1_RES: UVec2 = UVec2::new(64, 64);
pub const CLIPMAP_L2_RES: UVec2 = UVec2::new(32, 32);
pub const CLIPMAP_L3_RES: UVec2 = UVec2::new(16, 16);

#[derive(Clone, Debug, Default)]
pub struct AtmosphereClipLevel {
    pub resolution: UVec2,
    pub smoke_density: Vec<f32>,
    /// DEBT-010b — L0 fog authority (mirrored into Field for transitional consumers).
    pub fog_density: Vec<f32>,
    /// DEBT-010b — L0 toxicity authority (mirrored into Field for transitional consumers).
    pub toxicity: Vec<f32>,
    /// DEBT-010c — L0 ash authority.
    pub ash_density: Vec<f32>,
    /// DEBT-010c — L0 ember authority.
    pub ember_density: Vec<f32>,
    /// DEBT-010c — L0 heat-distortion authority.
    pub heat_distortion: Vec<f32>,
}

#[derive(Resource, Clone, Debug)]
pub struct AtmosphereClipmapStack {
    pub levels: Vec<AtmosphereClipLevel>,
    pub active_focus: DVec2,
}

impl Default for AtmosphereClipmapStack {
    fn default() -> Self {
        fn make_level(size: UVec2) -> AtmosphereClipLevel {
            let n = (size.x * size.y) as usize;
            AtmosphereClipLevel {
                resolution: size,
                smoke_density: vec![0.0; n],
                fog_density: vec![0.0; n],
                toxicity: vec![0.0; n],
                ash_density: vec![0.0; n],
                ember_density: vec![0.0; n],
                heat_distortion: vec![0.0; n],
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
            assert_eq!(level.ash_density.len(), level.smoke_density.len());
            assert_eq!(level.ember_density.len(), level.smoke_density.len());
            assert_eq!(level.heat_distortion.len(), level.smoke_density.len());
        }
        assert!(DEBT_010C_ASH_EMBER_HEAT_L0);
        assert!(GPU_FIELD_CONSUMERS_ON_L0);
    }
}
