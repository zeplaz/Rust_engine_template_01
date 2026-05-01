//! Chunk grid index for streaming / materialization — see material unification U5.

use bevy::prelude::{Component, IVec2};

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Chunk {
    pub coord: IVec2,
}
