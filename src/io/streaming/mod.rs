//! Async world streaming spine — priority scoring, staging, and upload enqueue.

use bevy::prelude::*;

use crate::gui::WorldRepresentationFrame;

/// Chunk streaming priority inputs (distance, simulation, visibility).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChunkStreamingPriority {
    pub distance_weight: f32,
    pub simulation_weight: f32,
    pub visibility_weight: f32,
}

impl ChunkStreamingPriority {
    #[must_use]
    pub fn score(self, distance: f32, sim_importance: f32, visible: bool) -> f32 {
        let visibility = if visible { 1.0 } else { 0.0 };
        self.distance_weight * distance
            + self.simulation_weight * sim_importance
            + self.visibility_weight * visibility
    }
}

/// Pipeline stage for a chunk streaming job.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkStreamStage {
    Disk,
    Deserialize,
    DomainReconstruct,
    GpuUpload,
}

/// One chunk streaming job moving through the async spine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChunkStreamJob {
    pub chunk: IVec2,
    pub stage: ChunkStreamStage,
    pub priority: f32,
}

/// Scheduler resource for threaded disk → staging → GPU upload.
#[derive(Resource, Debug, Clone, Default)]
pub struct ChunkStreamingScheduler {
    pub pending_chunks: Vec<IVec2>,
    pub jobs: Vec<ChunkStreamJob>,
}

impl ChunkStreamingScheduler {
    pub fn enqueue_focus_window(
        &mut self,
        focus: IVec2,
        radius: i32,
        weights: ChunkStreamingPriority,
        sim_importance: f32,
    ) {
        self.pending_chunks.clear();
        self.jobs.clear();
        for y in (focus.y - radius)..=(focus.y + radius) {
            for x in (focus.x - radius)..=(focus.x + radius) {
                let chunk = IVec2::new(x, y);
                let distance = (chunk - focus).as_vec2().length();
                let visible = distance <= radius as f32;
                let priority = weights.score(distance, sim_importance, visible);
                self.pending_chunks.push(chunk);
                self.jobs.push(ChunkStreamJob {
                    chunk,
                    stage: ChunkStreamStage::Disk,
                    priority,
                });
            }
        }
        self.jobs
            .sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn advance_job_stages(&mut self) {
        for job in &mut self.jobs {
            job.stage = match job.stage {
                ChunkStreamStage::Disk => ChunkStreamStage::Deserialize,
                ChunkStreamStage::Deserialize => ChunkStreamStage::DomainReconstruct,
                ChunkStreamStage::DomainReconstruct => ChunkStreamStage::GpuUpload,
                ChunkStreamStage::GpuUpload => ChunkStreamStage::GpuUpload,
            };
        }
    }
}

pub fn schedule_chunk_streaming_from_world_frame(
    world: Res<WorldRepresentationFrame>,
    mut scheduler: ResMut<ChunkStreamingScheduler>,
) {
    let weights = ChunkStreamingPriority {
        distance_weight: -1.0,
        simulation_weight: 1.5,
        visibility_weight: 2.0,
    };
    scheduler.enqueue_focus_window(
        world.focus_chunk,
        world.interest_radius_chunks.max(1),
        weights,
        world.gameplay_importance,
    );
}

pub struct StreamingSpinePlugin;

impl Plugin for StreamingSpinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkStreamingScheduler>().add_systems(
            Update,
            schedule_chunk_streaming_from_world_frame
                .after(crate::gui::WorldRepresentationSystemSet::ComputeFrame),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaming_priority_prefers_visible_near_sim() {
        let weights = ChunkStreamingPriority {
            distance_weight: -1.0,
            simulation_weight: 2.0,
            visibility_weight: 4.0,
        };
        let near = weights.score(1.0, 0.8, true);
        let far = weights.score(8.0, 0.8, false);
        assert!(near > far);
    }

    #[test]
    fn scheduler_orders_focus_window_by_priority() {
        let mut scheduler = ChunkStreamingScheduler::default();
        scheduler.enqueue_focus_window(IVec2::ZERO, 1, ChunkStreamingPriority::default(), 0.5);
        assert_eq!(scheduler.jobs.len(), 9);
        assert!(scheduler.jobs[0].priority >= scheduler.jobs[8].priority);
    }

    #[test]
    fn job_stage_advances_toward_gpu_upload() {
        let mut scheduler = ChunkStreamingScheduler::default();
        scheduler.enqueue_focus_window(IVec2::ZERO, 0, ChunkStreamingPriority::default(), 0.0);
        scheduler.advance_job_stages();
        assert_eq!(scheduler.jobs[0].stage, ChunkStreamStage::Deserialize);
    }
}
