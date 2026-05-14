//! Incremental chunk save queue — dirty chunks only, not full-world serialize.

use bevy::math::IVec2;
use bevy::prelude::*;

/// Explicit save flush request (editor, dev tools, autosave hooks).
#[derive(Clone, Copy, Debug, Default, Message)]
pub struct RequestWorldSaveFlush;

/// Chunks pending serialization for the next save flush.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct DirtyChunkSaveQueue {
    pub dirty_chunks: Vec<IVec2>,
}

impl DirtyChunkSaveQueue {
    pub fn enqueue(&mut self, chunk: IVec2) {
        if self.dirty_chunks.contains(&chunk) {
            return;
        }
        self.dirty_chunks.push(chunk);
    }

    pub fn enqueue_many<I: IntoIterator<Item = IVec2>>(&mut self, chunks: I) {
        for chunk in chunks {
            self.enqueue(chunk);
        }
    }

    pub fn drain(&mut self) -> Vec<IVec2> {
        std::mem::take(&mut self.dirty_chunks)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dirty_chunks.is_empty()
    }
}

pub fn enqueue_dirty_chunks_from_preview(
    preview: Option<Res<crate::terrain::material::WorldPreviewState>>,
    mut queue: ResMut<DirtyChunkSaveQueue>,
) {
    let Some(preview) = preview else {
        return;
    };
    queue.enqueue_many(preview.dirty_queue.iter().copied());
}

pub fn enqueue_dirty_chunks_from_environment_hooks(
    hooks: Res<crate::systems::chunk_environment_persist::ChunkEnvironmentPersistHooks>,
    mut queue: ResMut<DirtyChunkSaveQueue>,
    chunks: Query<(&crate::terrain::generation::Chunk, &crate::systems::chunk_environment_persist::ChunkEnvironmentDirty)>,
) {
    if hooks.fire_field_dirty_events == 0 {
        return;
    }
    for (chunk, dirty) in &chunks {
        if dirty.fire_field {
            queue.enqueue(chunk.coord);
        }
    }
}

pub fn arm_save_flush_from_requests(
    mut requests: MessageReader<RequestWorldSaveFlush>,
    mut flush: ResMut<crate::io::save::pipeline::SaveFlushRequested>,
) {
    if requests.read().next().is_some() {
        flush.0 = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_queue_dedupes_chunk_coords() {
        let mut queue = DirtyChunkSaveQueue::default();
        queue.enqueue(IVec2::new(1, 2));
        queue.enqueue(IVec2::new(1, 2));
        assert_eq!(queue.dirty_chunks, vec![IVec2::new(1, 2)]);
    }
}
