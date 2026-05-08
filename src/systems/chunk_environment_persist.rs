//! Persistence / scheduler **stubs** for chunk environment fields (fire overlay, ecology).
//! Wire real IO from `chunk_scheduler_runbook_v1.md` when chunk save streams land.

use bevy::prelude::*;

/// Marks a chunk whose environment sim may need serialization (CPU fields only).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ChunkEnvironmentDirty {
    pub fire_field: bool,
}

#[derive(Resource, Debug, Default)]
pub struct ChunkEnvironmentPersistHooks {
    /// Incremented when a chunk reports its fire field materially changed (stub metric).
    pub fire_field_dirty_events: u64,
}

impl ChunkEnvironmentPersistHooks {
    pub fn notify_fire_field_dirty(&mut self, _entity: Entity) {
        self.fire_field_dirty_events = self.fire_field_dirty_events.wrapping_add(1);
    }
}

pub struct ChunkEnvironmentPersistPlugin;

impl Plugin for ChunkEnvironmentPersistPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkEnvironmentPersistHooks>();
    }
}
