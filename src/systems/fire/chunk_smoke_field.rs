//! Strategic **smoke / visibility** scalars per chunk (`base_fire_sim.md` §5).
//!
//! **W7:** [`ChunkSmokeField`] is a **read-only projection** of [`AtmosphereFrame`] tracers
//! (sole writer: [`project_chunk_smoke_from_atmosphere_frame`]). Fire emit and L0 blend pulls
//! retired — L0 already receives fire smoke via FieldFill (surface fire + overlay).

use bevy::prelude::*;

use crate::systems::atmosphere::AtmosphereFrame;
use crate::systems::sim_control::SimControlState;
use crate::terrain::generation::Chunk;

#[derive(Component, Clone, Copy, Debug)]
pub struct ChunkSmokeField {
    pub density: f32,
    pub toxicity: f32,
    pub visibility_penalty: f32,
}

impl Default for ChunkSmokeField {
    fn default() -> Self {
        Self {
            density: 0.0,
            toxicity: 0.0,
            visibility_penalty: 0.0,
        }
    }
}

pub(crate) fn spawn_chunk_smoke_field_on_new_chunk(
    mut commands: Commands,
    q: Query<Entity, (Added<Chunk>, Without<ChunkSmokeField>)>,
) {
    for e in &q {
        commands.entity(e).insert(ChunkSmokeField::default());
    }
}

/// Sole ChunkSmoke writer (W7): copy frame tracers onto chunk components for sense / logistics.
pub fn project_chunk_smoke_from_atmosphere_frame(
    ctrl: Res<SimControlState>,
    frame: Res<AtmosphereFrame>,
    mut q: Query<(&Chunk, &mut ChunkSmokeField)>,
) {
    if !ctrl.should_tick() {
        return;
    }
    if !frame.is_built() {
        return;
    }
    for (chunk, mut smoke) in &mut q {
        let Some(tile) = frame.tile_at_chunk(chunk.coord) else {
            continue;
        };
        smoke.density = tile.smoke.clamp(0.0, 1.0);
        smoke.toxicity = tile.toxicity.clamp(0.0, 1.0);
        let fog = if tile.weather_valid {
            tile.weather.fog
        } else {
            0.0
        };
        smoke.visibility_penalty =
            (smoke.density * 0.75 + fog * 0.35 + smoke.toxicity * 0.25).clamp(0.0, 0.98);
    }
}

/// W7 — single projection writer (tick + L0-pull retired).
pub const CHUNK_SMOKE_WRITERS: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::atmosphere::AtmosphereFrame;

    #[test]
    fn projection_copies_frame_tracers() {
        let mut frame = AtmosphereFrame::default();
        frame.rebuild_for_tests_with_smoke(0, 0.55);
        assert!(frame.is_built());
        let tile = frame.tiles().first().expect("tile");
        let mut smoke = ChunkSmokeField::default();
        smoke.density = tile.smoke.clamp(0.0, 1.0);
        smoke.toxicity = tile.toxicity.clamp(0.0, 1.0);
        smoke.visibility_penalty =
            (smoke.density * 0.75 + smoke.toxicity * 0.25).clamp(0.0, 0.98);
        assert!(smoke.density > 0.4);
        assert_eq!(CHUNK_SMOKE_WRITERS, 1);
    }
}
