//! Chunk-level **fire light emission** metadata — simulation-owned, not render (`base_fire2_smoke.md` §9).
//!
//! Render consumes this via [`crate::render::extraction::FireVisualFramePlugin`] →
//! [`crate::render::extraction::FireVisualFrame`] → [`crate::render::light::RequestLocalLight`] → pooled [`PointLight`]s.

use bevy::prelude::*;

use super::ChunkSurfaceFire;
use crate::systems::ecology::ChunkEcology;
use crate::systems::sim_control::SimControlState;
use crate::terrain::generation::Chunk;

const HEAT_MIN: f32 = 0.08;

/// Per-entity **simulation** parameters for how much light fire should contribute this tick.
///
/// No [`PointLight`] here — extraction maps this into [`crate::render::ActiveLights`] through
/// [`crate::render::RequestLocalLight`].
#[derive(Component, Clone, Copy, Debug)]
pub struct FireLightEmission {
    pub radius: f32,
    pub base_intensity: f32,
    pub current_intensity: f32,
    pub flicker_strength: f32,
    pub flicker_phase: f32,
    /// Hint for extraction / budgeting (e.g. heat-linked); not authoritative sim state.
    pub extract_priority: f32,
}

pub(crate) fn maintain_fire_light_emission_from_surface_fire(
    ctrl: Res<SimControlState>,
    mut commands: Commands,
    q: Query<(
        Entity,
        &Chunk,
        &ChunkSurfaceFire,
        Option<&ChunkEcology>,
        Option<&FireLightEmission>,
    )>,
) {
    if !ctrl.should_tick() {
        return;
    }
    for (e, chunk, fire, eco, existing) in &q {
        if fire.heat < HEAT_MIN {
            if existing.is_some() {
                commands.entity(e).remove::<FireLightEmission>();
            }
            continue;
        }
        let bio = eco.map(|e| e.biomass).unwrap_or(0.35);
        let base_intensity = (fire.heat * (0.6 + bio * 0.9)).clamp(0.0, 2.5);
        let radius = 120.0 + fire.heat * 180.0;
        let flicker_phase =
            (chunk.coord.x as f32 * 0.37 + chunk.coord.y as f32 * 0.91).fract() * std::f32::consts::TAU;
        let flicker_strength = 0.12;
        let extract_priority = 0.5 + fire.heat * 2.0;

        commands.entity(e).insert(FireLightEmission {
            radius,
            base_intensity,
            // `update_fire_light_emission_flicker` runs later the same tick and applies phase.
            current_intensity: base_intensity,
            flicker_strength,
            flicker_phase,
            extract_priority,
        });
    }
}

pub(crate) fn update_fire_light_emission_flicker(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    mut q: Query<&mut FireLightEmission>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let t = time.elapsed_secs();
    for mut em in &mut q {
        let flicker = (t * 13.7 + em.flicker_phase).sin();
        em.current_intensity = (em.base_intensity * (1.0 + flicker * em.flicker_strength)).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emission_fields_sane() {
        let e = FireLightEmission {
            radius: 100.0,
            base_intensity: 1.0,
            current_intensity: 1.0,
            flicker_strength: 0.1,
            flicker_phase: 0.0,
            extract_priority: 1.0,
        };
        assert!(e.radius > 0.0 && e.base_intensity > 0.0);
    }
}
