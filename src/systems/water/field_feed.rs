//! WM-5 / WM-6 -- one read surface of "which chunks of the map-wide water field moved this frame".
//!
//! The field queues its moved chunks for exactly one consumer: [`publish_water_dirty_feed_system`]
//! drains that queue once per frame and publishes the list here (the substrate no longer keeps a
//! copy of the depth, so nobody else drains it). Every render consumer (GPU texture, PixelWorld
//! surface, minimap mask) reads the same list, each with its own cursor
//! ([`WaterFieldDirtyFeed::chunks_since`]); a consumer that missed a frame gets `None` and resyncs
//! fully.

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

/// Sole writer of [`WaterFieldDirtyFeed`] and sole consumer of the field's dirty-chunk queue. Runs
/// after the solver step. Drains the queue without marking the field changed.
pub fn publish_water_dirty_feed_system(
    mut field: ResMut<SurfaceWaterField>,
    mut feed: ResMut<WaterFieldDirtyFeed>,
) {
    let field = field.bypass_change_detection();
    if !field.wired {
        return;
    }
    if field.has_pending_dirty() {
        feed.chunks = field.take_dirty_chunks();
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

    /// The feed is the queue's only consumer: a frame's moved chunks are published once, then the
    /// queue is empty until the water moves again.
    #[test]
    fn the_feed_drains_the_field_queue_once() {
        let mut app = App::new();
        let mut f = SurfaceWaterField::default();
        f.allocate(0, 0, 64, 64, vec![0.0; 64 * 64], 0.0);
        app.insert_resource(f)
            .init_resource::<WaterFieldDirtyFeed>()
            .add_systems(Update, publish_water_dirty_feed_system);
        app.update();
        let feed = app.world().resource::<WaterFieldDirtyFeed>();
        assert_eq!((feed.seq, feed.chunks.len()), (1, 4), "allocation queues every chunk");
        assert!(!app.world().resource::<SurfaceWaterField>().has_pending_dirty(), "drained");
        app.update();
        let feed = app.world().resource::<WaterFieldDirtyFeed>();
        assert_eq!((feed.seq, feed.chunks.len()), (1, 0), "nothing moved: nothing published");
        app.world_mut().resource_mut::<SurfaceWaterField>().touch_world(3, 3);
        app.update();
        let feed = app.world().resource::<WaterFieldDirtyFeed>();
        assert_eq!((feed.seq, feed.chunks.as_slice()), (2, &[IVec2::new(0, 0)][..]));
    }
}
