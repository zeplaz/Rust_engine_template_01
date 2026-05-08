//! Gameplay-facing **macro** terrain labels derived from continuous fields — aligns with
//! [`voronoi_polygon_worlds_notes`](../../../prompts/guides/voronoi_polygon_worlds_notes.md.md)
//! (graph-first worlds, strategic regions, semantics before pure tile noise).
//!
//! Voronoi sites in [`super::world_generator_enhanced`] still control ECS grouping only; this layer
//! tags each tile with a coarse **strategic kind** for AI / logistics / future coarse simulation.

use bevy::prelude::Component;

/// Coarse strategic category for a single tile (Layer 0–style semantics, not biome materials).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum MacroStrategicKind {
    #[default]
    Mixed,
    /// High relief — barriers, weather shadow, mining / defense stubs.
    MountainBarrier,
    /// Lower wet corridor between high areas — natural logistics funnels.
    ValleyCorridor,
    /// Low, wet tiles — flood / ag / marshy dynamics.
    Floodplain,
    /// Elevated but sub-orographic — plateaus, steppe-like constraints.
    HighPlateau,
    /// Very low height — coasts, deltas, nearshore logistics.
    CoastalLowland,
    /// Hot or temperate dry interior — supply / water stress semantics.
    AridBasin,
}

/// Rule-based classification from normalized height / moisture / temperature.
#[inline]
pub fn classify_strategic_tile(
    height: f32,
    moisture: f32,
    temperature: f32,
    mountain_threshold: f32,
) -> MacroStrategicKind {
    let mountain_threshold = mountain_threshold.clamp(0.05, 0.95);
    if height >= mountain_threshold {
        return MacroStrategicKind::MountainBarrier;
    }
    if height < 0.18 {
        return MacroStrategicKind::CoastalLowland;
    }
    if moisture > 0.62 && height < 0.38 {
        return MacroStrategicKind::Floodplain;
    }
    if moisture < 0.32 && temperature > 0.52 && height > 0.22 {
        return MacroStrategicKind::AridBasin;
    }
    if height > 0.42 && height < mountain_threshold && temperature < 0.48 {
        return MacroStrategicKind::HighPlateau;
    }
    if height < 0.42 && moisture > 0.35 && moisture < 0.62 {
        return MacroStrategicKind::ValleyCorridor;
    }
    MacroStrategicKind::Mixed
}

/// Small moisture/temperature nudge from semantics so “structure first” can bias fields without replacing noise.
#[inline]
pub fn apply_strategic_field_nudge(
    kind: MacroStrategicKind,
    coupling: f32,
    moisture: &mut f32,
    temperature: &mut f32,
) {
    let k = coupling.clamp(0.0, 1.0);
    if k <= 0.0 {
        return;
    }
    match kind {
        MacroStrategicKind::MountainBarrier => {
            *temperature -= 0.07 * k;
            *moisture -= 0.05 * k;
        }
        MacroStrategicKind::ValleyCorridor => {
            *moisture += 0.05 * k;
            *temperature += 0.02 * k;
        }
        MacroStrategicKind::Floodplain => {
            *moisture += 0.08 * k;
            *temperature -= 0.02 * k;
        }
        MacroStrategicKind::HighPlateau => {
            *moisture -= 0.04 * k;
            *temperature -= 0.05 * k;
        }
        MacroStrategicKind::CoastalLowland => {
            *moisture += 0.06 * k;
        }
        MacroStrategicKind::AridBasin => {
            *moisture -= 0.08 * k;
            *temperature += 0.06 * k;
        }
        MacroStrategicKind::Mixed => {}
    }
    *moisture = moisture.clamp(0.0, 1.0);
    *temperature = temperature.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_height_is_mountain_barrier() {
        assert_eq!(
            classify_strategic_tile(0.85, 0.5, 0.5, 0.7),
            MacroStrategicKind::MountainBarrier
        );
    }

    #[test]
    fn low_wet_is_floodplain_or_coastal() {
        assert_eq!(
            classify_strategic_tile(0.1, 0.8, 0.5, 0.7),
            MacroStrategicKind::CoastalLowland
        );
        assert_eq!(
            classify_strategic_tile(0.3, 0.7, 0.5, 0.7),
            MacroStrategicKind::Floodplain
        );
    }
}
