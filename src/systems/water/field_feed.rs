//! WM-5 -- one read surface of "which chunks of the map-wide water field moved this frame".
//!
//! The field queues its moved chunks for one consumer (`take_dirty_chunks`, the substrate
//! projection). Render consumers (GPU texture, PixelWorld surface, minimap mask) must not steal that
//! queue, so [`publish_water_dirty_feed_system`] *peeks* it once per frame, before the projection
//! drains it, and publishes the list here. Every consumer reads the same list, each with its own
//! cursor ([`WaterFieldDirtyFeed::chunks_since`]); a consumer that missed a frame gets `None` and
//! resyncs fully.

use bevy::prelude::*;

use super::surface_water::SurfaceWaterField;

/// Chunk list of the latest field movement (sim chunk coordinates, `CHUNK_TILES_SIM` tiles).
#[derive(Resource, Debug, Default, Clone)]
pub struct WaterFieldDirtyFeed {
    /// Bumps once per frame in which some chunk moved.
    pub seq: u64,
    /// Chunks that moved in frame `seq` (cleared on frames with no movement).
    pub chunks: Vec<IVec2>,
}

impl WaterFieldDirtyFeed {
    /// Chunks to apply for a consumer that last saw `last_seen`. `Some(&[])` = nothing new,
    /// `None` = the consumer skipped at least one published frame and must resync everything.
    #[must_use]
    pub fn chunks_since(&self, last_seen: u64) -> Option<&[IVec2]> {
        if self.seq == last_seen {
            Some(&[])
        } else if self.seq == last_seen.wrapping_add(1) {
            Some(&self.chunks)
        } else {
            None
        }
    }
}

/// Non-consuming view of the field's pending dirty chunks (separate impl: the queue and its mask
/// stay owned by the field; this only flushes tile marks into the chunk list).
impl SurfaceWaterField {
    /// Pending dirty chunks, left queued for [`SurfaceWaterField::take_dirty_chunks`].
    pub fn peek_dirty_chunks(&mut self) -> &[IVec2] {
        let mut tiles = std::mem::take(&mut self.tiles);
        tiles.flush_dirty_into(&mut self.dirty_chunks);
        self.tiles = tiles;
        &self.dirty_chunks
    }
}

/// Sole writer of [`WaterFieldDirtyFeed`]. Runs after the solver step and before the substrate
/// projection drains the queue. Reads the field without marking it changed.
pub fn publish_water_dirty_feed_system(
    mut field: ResMut<SurfaceWaterField>,
    mut feed: ResMut<WaterFieldDirtyFeed>,
) {
    let field = field.bypass_change_detection();
    if !field.wired {
        return;
    }
    if field.has_pending_dirty() {
        let peeked = field.peek_dirty_chunks();
        feed.chunks.clear();
        feed.chunks.extend_from_slice(peeked);
        feed.seq = feed.seq.wrapping_add(1);
    } else if !feed.chunks.is_empty() {
        feed.chunks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feed_cursor_detects_missed_frames() {
        let feed = WaterFieldDirtyFeed { seq: 5, chunks: vec![IVec2::new(1, 2)] };
        assert_eq!(feed.chunks_since(5), Some(&[][..]));
        assert_eq!(feed.chunks_since(4), Some(&[IVec2::new(1, 2)][..]));
        assert!(feed.chunks_since(2).is_none(), "skipped frames need a full resync");
    }

    #[test]
    fn peek_leaves_the_queue_for_the_projection() {
        let mut f = SurfaceWaterField::default();
        f.allocate(0, 0, 64, 64, vec![0.0; 64 * 64], 0.0);
        let peeked = f.peek_dirty_chunks().to_vec();
        assert!(!peeked.is_empty(), "allocation queues every chunk");
        assert_eq!(f.take_dirty_chunks().len(), peeked.len(), "the owner still drains the same chunks");
    }
}
