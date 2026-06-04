//! Per-frame streaming work budgets (PERF-PLAY-001 / industrial play spine).

use bevy::prelude::*;

/// Frame budgets for the async chunk streaming spine. Full interest may be large; work is spread by priority.
#[derive(Resource, Clone, Debug)]
pub struct StreamingSpineBudget {
    pub max_hydrate_chunks_per_frame: usize,
    pub max_reconstruct_chunks_per_frame: usize,
    pub max_pending_chunks: usize,
}

impl Default for StreamingSpineBudget {
    fn default() -> Self {
        Self {
            // Keep frame-time stable under active play input; users can raise via env when needed.
            max_hydrate_chunks_per_frame: hydrate_budget_from_env().unwrap_or(24),
            max_reconstruct_chunks_per_frame: reconstruct_budget_from_env().unwrap_or(4),
            max_pending_chunks: pending_chunks_budget_from_env().unwrap_or(1024),
        }
    }
}

fn hydrate_budget_from_env() -> Option<usize> {
    std::env::var("STREAMING_HYDRATE_BUDGET")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n| n > 0)
}

fn reconstruct_budget_from_env() -> Option<usize> {
    std::env::var("STREAMING_RECONSTRUCT_BUDGET")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n| n > 0)
}

fn pending_chunks_budget_from_env() -> Option<usize> {
    std::env::var("MAX_STREAMING_PENDING_CHUNKS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n| n > 0)
}

#[must_use]
pub fn stream_sync_hydrate_enabled() -> bool {
    std::env::var("STREAMING_SYNC_HYDRATE")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}
