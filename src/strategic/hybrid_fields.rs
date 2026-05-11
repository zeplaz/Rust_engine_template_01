//! Hybrid sim **statistical layer** — macro fields + regional overlays (probability shaping, not agent truth).
//!
//! Principle: stats **bias** outcomes; they do not dictate individual agent actions.
//! [`crate::strategic::sim::StrategicSimulationPlugin`]: **PreUpdate** runs [`region_stats_spatial_smoothing_system`];
//! **Update** runs behavioral intent via [`crate::strategic::sim::HybridSimPipeline`]; **PostUpdate** resolves + feedback
//! in [`super::hybrid_brain`].

use bevy::prelude::*;
use std::collections::HashMap;

/// Inertia blend: `prev * inertia + input * (1 - inertia)`.
/// `inertia` ∈ [0, 1]; higher inertia ⇒ slower movement toward `input`.
#[inline]
pub fn smooth(prev: f32, input: f32, inertia: f32) -> f32 {
    let i = inertia.clamp(0.0, 1.0);
    prev.mul_add(i, input * (1.0 - i))
}

/// Global continuous fields — **bias** global event distributions and agent perception, not discrete “system toggles”.
///
/// **Authority (runbook §1.1):** gameplay agents and UI **must not** assign these fields directly. Mutations belong
/// in the resolution/feedback path ([`crate::strategic::hybrid_brain::hybrid_resolve_and_feedback_system`]) and
/// intentional design-time defaults — not in preview or “direct agent writes”.
#[derive(Resource, Clone, Debug)]
pub struct WorldFields {
    pub economic_pressure: f32,
    pub instability_index: f32,
    pub war_tension: f32,
    pub resource_scarcity: f32,
    pub public_sentiment: f32,
    pub technological_entropy: f32,
}

impl Default for WorldFields {
    fn default() -> Self {
        Self {
            economic_pressure: 0.35,
            instability_index: 0.2,
            war_tension: 0.15,
            resource_scarcity: 0.25,
            public_sentiment: 0.5,
            technological_entropy: 0.3,
        }
    }
}

/// Regional statistical overlay — a region is **cohherent or incoherent** in aggregate, not a crisp owned tile set.
#[derive(Clone, Copy, Debug, Default)]
pub struct RegionStats {
    pub stability: f32,
    pub corruption: f32,
    pub militarization: f32,
    pub wealth_density: f32,
    pub control_fragmentation: f32,
}

/// Sparse store keyed by macro region id (aligned with Voronoi / `TileRegionIndex` semantics).
#[derive(Resource, Clone, Debug, Default)]
pub struct RegionalStatsOverlay {
    pub by_region_id: HashMap<u32, RegionStats>,
}

/// Target regional aggregates derived from [`WorldFields`] (neighbor graph can refine later; v1 uses global bias).
#[inline]
pub fn regional_target_from_world(w: &WorldFields) -> RegionStats {
    RegionStats {
        stability: w.public_sentiment * 0.7 + (1.0 - w.instability_index) * 0.3,
        corruption: w.instability_index * 0.4 + w.resource_scarcity * 0.2,
        militarization: w.war_tension * 0.6 + w.instability_index * 0.2,
        wealth_density: w.economic_pressure * 0.5 + (1.0 - w.resource_scarcity) * 0.3,
        control_fragmentation: w.instability_index * 0.35 + (1.0 - w.public_sentiment) * 0.25,
    }
}

/// Pull each stored region toward [`regional_target_from_world`] (statistical smoothing, §1.2 runbook).
pub fn region_stats_spatial_smoothing_system(
    world: Res<WorldFields>,
    mut overlay: ResMut<RegionalStatsOverlay>,
) {
    let inertia = 0.94_f32;
    for stats in overlay.by_region_id.values_mut() {
        let t = regional_target_from_world(&world);
        stats.stability = smooth(stats.stability, t.stability, inertia);
        stats.corruption = smooth(stats.corruption, t.corruption, inertia);
        stats.militarization = smooth(stats.militarization, t.militarization, inertia);
        stats.wealth_density = smooth(stats.wealth_density, t.wealth_density, inertia);
        stats.control_fragmentation =
            smooth(stats.control_fragmentation, t.control_fragmentation, inertia);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooth_midpoint_at_half_inertia() {
        assert!((smooth(0.0, 1.0, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn smooth_returns_prev_at_full_inertia() {
        assert!((smooth(0.3, 0.9, 1.0) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn smooth_returns_input_at_zero_inertia() {
        assert!((smooth(0.3, 0.9, 0.0) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn regional_target_tracks_sentiment_and_tension() {
        let w = WorldFields {
            public_sentiment: 0.9,
            war_tension: 0.8,
            instability_index: 0.1,
            economic_pressure: 0.5,
            resource_scarcity: 0.2,
            technological_entropy: 0.3,
        };
        let t = regional_target_from_world(&w);
        assert!(t.stability > 0.8);
        assert!(t.militarization >= 0.5);
    }
}
