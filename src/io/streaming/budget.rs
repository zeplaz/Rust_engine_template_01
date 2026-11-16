//! Per-frame streaming work budgets (PERF-PLAY-001 / industrial play spine).
//!
//! **IO-03** — Memory for object spawning (gea4v2 17.4.4 / 17.4.4.1 offline · 17.4.4.2 dynamic).
//! LEGACY: `memory_management` (gea_v1_e4 p412) · `world_streaming` (gea_v2_e4 p589).
//! Offline = fixed per-chunk spawn ceiling; dynamic = per-frame entity reserve cap that
//! clamps hydrate/reconstruct so RemoteAllocator fill stays ahead of hydrate work.
//!
//! **EXEC-03** (aig3 10.1.4–10.1.5 *Hierarchical / Priority Scheduling*):
//! [`allocate_nested_domain_budgets`] nests domain caps under a parent frame budget by priority.

use bevy::prelude::*;

/// Extra chunk radius beyond [`load`](super::residency::chunk_within_load_radius) before a chunk may drop
/// from the streaming interest set (IO-02 air lock; stops edge thrash).
pub const STREAMING_RESIDENCY_UNLOAD_HYSTERESIS_CHUNKS: i32 = 1;

/// Offline spawn ceiling: entity ids reserved per streamed chunk body (gea4v2 17.4.4.1).
pub const STREAMING_SPAWN_ENTITIES_PER_CHUNK_DEFAULT: usize = 1;

/// Dynamic spawn ceiling: max entity ids reserved per frame before hydrate/apply (gea4v2 17.4.4.2).
pub const STREAMING_SPAWN_ENTITIES_PER_FRAME_DEFAULT: usize = 32;

/// Frame budgets for the async chunk streaming spine. Full interest may be large; work is spread by priority.
#[derive(Resource, Clone, Debug)]
pub struct StreamingSpineBudget {
    pub max_hydrate_chunks_per_frame: usize,
    pub max_reconstruct_chunks_per_frame: usize,
    pub max_pending_chunks: usize,
    /// Offline — max entity ids planned per chunk body (IO-03 / gea4v2 17.4.4.1).
    pub max_spawn_entities_per_chunk: usize,
    /// Dynamic — max entity ids reserved in one frame (IO-03 / gea4v2 17.4.4.2).
    pub max_spawn_entities_per_frame: usize,
}

impl Default for StreamingSpineBudget {
    fn default() -> Self {
        Self {
            // Keep frame-time stable under active play input; users can raise via env when needed.
            max_hydrate_chunks_per_frame: hydrate_budget_from_env().unwrap_or(24),
            max_reconstruct_chunks_per_frame: reconstruct_budget_from_env().unwrap_or(4),
            max_pending_chunks: pending_chunks_budget_from_env().unwrap_or(1024),
            max_spawn_entities_per_chunk: spawn_per_chunk_from_env()
                .unwrap_or(STREAMING_SPAWN_ENTITIES_PER_CHUNK_DEFAULT),
            max_spawn_entities_per_frame: spawn_per_frame_from_env()
                .unwrap_or(STREAMING_SPAWN_ENTITIES_PER_FRAME_DEFAULT),
        }
    }
}

impl StreamingSpineBudget {
    /// Hydrate batch size after the IO-03 spawn gate (never exceeds entity frame ceiling).
    #[must_use]
    pub fn effective_hydrate_chunks_per_frame(&self) -> usize {
        clamp_hydrate_batch_by_spawn_budget(
            self.max_hydrate_chunks_per_frame,
            self.max_spawn_entities_per_chunk,
            self.max_spawn_entities_per_frame,
        )
    }

    /// Reconstruct batch size after the IO-03 spawn gate.
    #[must_use]
    pub fn effective_reconstruct_chunks_per_frame(&self) -> usize {
        clamp_hydrate_batch_by_spawn_budget(
            self.max_reconstruct_chunks_per_frame,
            self.max_spawn_entities_per_chunk,
            self.max_spawn_entities_per_frame,
        )
    }
}

/// How many entity ids to reserve for a pending chunk batch (offline × dynamic gate).
///
/// Returns 0 when any gate is closed so callers skip RemoteAllocator fill.
#[must_use]
pub fn spawn_reserve_for_batch(
    chunk_count: usize,
    max_per_chunk: usize,
    max_per_frame: usize,
) -> usize {
    if chunk_count == 0 || max_per_chunk == 0 || max_per_frame == 0 {
        return 0;
    }
    chunk_count
        .saturating_mul(max_per_chunk)
        .min(max_per_frame)
}

/// Clamp a hydrate/reconstruct chunk cap so planned spawns stay within the frame entity budget.
#[must_use]
pub fn clamp_hydrate_batch_by_spawn_budget(
    hydrate_cap: usize,
    max_per_chunk: usize,
    max_per_frame: usize,
) -> usize {
    if hydrate_cap == 0 || max_per_chunk == 0 || max_per_frame == 0 {
        return 0;
    }
    let spawn_limited = max_per_frame / max_per_chunk;
    hydrate_cap.min(spawn_limited)
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

fn spawn_per_chunk_from_env() -> Option<usize> {
    std::env::var("STREAMING_SPAWN_ENTITIES_PER_CHUNK")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n| n > 0)
}

fn spawn_per_frame_from_env() -> Option<usize> {
    std::env::var("STREAMING_SPAWN_ENTITIES_PER_FRAME")
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

/// EXEC-03: one domain under a hierarchical frame budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DomainBudgetAsk {
    pub domain: &'static str,
    /// Higher runs first when the parent frame budget is scarce.
    pub priority: u8,
    pub want: usize,
}

/// EXEC-03: nested allocation result (domain → granted units this frame).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DomainBudgetGrant {
    pub domain: &'static str,
    pub granted: usize,
}

/// EXEC-03: allocate a parent `frame_budget` across domains by priority (desc), then lexical domain.
/// Each ask is capped by `want`; leftovers cascade to lower-priority domains.
#[must_use]
pub fn allocate_nested_domain_budgets(
    frame_budget: usize,
    asks: &[DomainBudgetAsk],
) -> Vec<DomainBudgetGrant> {
    let mut order: Vec<usize> = (0..asks.len()).collect();
    order.sort_by(|&a, &b| {
        asks[b]
            .priority
            .cmp(&asks[a].priority)
            .then(asks[a].domain.cmp(asks[b].domain))
    });
    let mut remaining = frame_budget;
    let mut granted = vec![0usize; asks.len()];
    for i in order {
        let take = asks[i].want.min(remaining);
        granted[i] = take;
        remaining = remaining.saturating_sub(take);
    }
    asks.iter()
        .zip(granted)
        .map(|(a, g)| DomainBudgetGrant {
            domain: a.domain,
            granted: g,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_reserve_offline_times_chunks_then_dynamic_cap() {
        assert_eq!(spawn_reserve_for_batch(4, 2, 32), 8);
        assert_eq!(spawn_reserve_for_batch(20, 2, 10), 10);
        assert_eq!(spawn_reserve_for_batch(0, 2, 10), 0);
        assert_eq!(spawn_reserve_for_batch(4, 0, 10), 0);
        assert_eq!(spawn_reserve_for_batch(4, 2, 0), 0);
    }

    #[test]
    fn hydrate_batch_clamped_by_spawn_frame_budget() {
        assert_eq!(clamp_hydrate_batch_by_spawn_budget(24, 1, 32), 24);
        assert_eq!(clamp_hydrate_batch_by_spawn_budget(24, 1, 8), 8);
        assert_eq!(clamp_hydrate_batch_by_spawn_budget(24, 4, 8), 2);
        assert_eq!(clamp_hydrate_batch_by_spawn_budget(24, 0, 8), 0);
        assert_eq!(clamp_hydrate_batch_by_spawn_budget(0, 1, 8), 0);
    }

    #[test]
    fn effective_hydrate_and_reconstruct_respect_spawn_gates() {
        let budget = StreamingSpineBudget {
            max_hydrate_chunks_per_frame: 24,
            max_reconstruct_chunks_per_frame: 4,
            max_pending_chunks: 1024,
            max_spawn_entities_per_chunk: 2,
            max_spawn_entities_per_frame: 6,
        };
        assert_eq!(budget.effective_hydrate_chunks_per_frame(), 3);
        assert_eq!(budget.effective_reconstruct_chunks_per_frame(), 3);
    }

    #[test]
    fn default_spawn_budget_matches_io03_constants() {
        let budget = StreamingSpineBudget {
            max_hydrate_chunks_per_frame: 24,
            max_reconstruct_chunks_per_frame: 4,
            max_pending_chunks: 1024,
            max_spawn_entities_per_chunk: STREAMING_SPAWN_ENTITIES_PER_CHUNK_DEFAULT,
            max_spawn_entities_per_frame: STREAMING_SPAWN_ENTITIES_PER_FRAME_DEFAULT,
        };
        assert_eq!(
            budget.effective_hydrate_chunks_per_frame(),
            STREAMING_SPAWN_ENTITIES_PER_FRAME_DEFAULT.min(24)
        );
        assert_eq!(
            spawn_reserve_for_batch(
                4,
                budget.max_spawn_entities_per_chunk,
                budget.max_spawn_entities_per_frame
            ),
            4
        );
    }

    #[test]
    fn exec03_nested_domain_budgets_prefer_higher_priority() {
        let asks = [
            DomainBudgetAsk {
                domain: "hydrate",
                priority: 1,
                want: 20,
            },
            DomainBudgetAsk {
                domain: "reconstruct",
                priority: 5,
                want: 8,
            },
            DomainBudgetAsk {
                domain: "pending",
                priority: 3,
                want: 10,
            },
        ];
        let grants = allocate_nested_domain_budgets(12, &asks);
        assert_eq!(grants.len(), 3);
        let by: std::collections::HashMap<_, _> =
            grants.iter().map(|g| (g.domain, g.granted)).collect();
        assert_eq!(by["reconstruct"], 8);
        assert_eq!(by["pending"], 4);
        assert_eq!(by["hydrate"], 0);
        assert_eq!(
            grants.iter().map(|g| g.granted).sum::<usize>(),
            12
        );
    }
}
