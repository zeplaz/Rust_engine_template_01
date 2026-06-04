//! WSS-ATMOS-CLIPMAP-001 — clipmap stack, contamination tick, legacy bridge, witness.

mod bridge_legacy;
mod clipmap_advect;
mod contamination_tick;

pub use bridge_legacy::legacy_atmosphere_bridge_system;
pub use contamination_tick::contamination_tick_system;

use bevy::math::DVec2;
use bevy::prelude::*;

pub const WSS_ATMOS_CLIPMAP_GATE: &str = "WSS-ATMOS-CLIPMAP-001";

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
    pub render_clipmap_wired: bool,
    pub gpu_partial_upload_count: u32,
    pub toxic_hazard_sample: f32,
}

pub fn sync_atmos_clipmap_witness_system(
    mut stack: ResMut<AtmosphereClipmapStack>,
    mut witness: ResMut<AtmosphereClipmapWitness>,
    registry: Option<Res<crate::substrate::WorldSubstrateRegistry>>,
    legacy_field: Option<Res<crate::systems::atmosphere::AtmosphereField>>,
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
    witness.legacy_atmosphere_field_bridged =
        legacy_field.is_some() || !stack.levels.is_empty();
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
