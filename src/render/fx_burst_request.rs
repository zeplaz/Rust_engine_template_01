//! Sim → render **burst** requests (Hanabi / instanced mesh bridge, `base_gui_next.md`).
//! Simulation emits these; a render-side bridge consumes them (stub until GPU path lands).

use bevy::prelude::*;

use crate::systems::FireEmitter;
use crate::terrain::generation::Chunk;

#[derive(Message, Clone, Copy, Debug)]
pub struct FxParticleBurstRequest {
    pub chunk_ix: i32,
    pub chunk_iy: i32,
    pub intensity: f32,
}

/// Hot [`FireEmitter`] rows enqueue burst hints for the GPU particle bridge (coalescing TBD).
pub(crate) fn enqueue_fx_bursts_from_hot_emitters(
    q: Query<(&Chunk, &FireEmitter)>,
    mut w: MessageWriter<FxParticleBurstRequest>,
) {
    const THRESH: f32 = 0.9;
    for (chunk, em) in &q {
        if em.intensity >= THRESH {
            w.write(FxParticleBurstRequest {
                chunk_ix: chunk.coord.x,
                chunk_iy: chunk.coord.y,
                intensity: em.intensity,
            });
        }
    }
}
