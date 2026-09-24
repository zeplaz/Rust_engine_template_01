//! Async district throughput solve scaffold (LOG-D-04).
//!
//! Pure district-slice results may be posted from a worker later; **only** the main-thread
//! apply system mutates [`ThroughputSolverState`]. Live path: enqueue → staging → promote →
//! apply next frame (no off-thread field mutation).

use bevy::prelude::*;
use std::collections::VecDeque;

use super::types::{
    DistrictSolveScope, LogisticsThroughputRuntimeWitness, ThroughputSolverState,
};

/// Posted solve result — applied on the main thread (never mutated off-thread).
#[derive(Clone, Debug)]
pub struct DistrictSolveResult {
    pub district_id: u32,
    pub edge_load: Vec<(usize, f32)>,
}

/// LOG-D-04: queue + apply boundary (no off-thread field mutation).
#[derive(Resource, Default)]
pub struct AsyncDistrictSolveQueue {
    /// Jobs posted this frame; promoted to [`Self::pending`] after apply.
    pub staging: VecDeque<DistrictSolveResult>,
    /// Ready for main-thread apply (prior-frame staging, or direct [`Self::post`] inject).
    pub pending: VecDeque<DistrictSolveResult>,
    pub applied_total: u64,
    pub posted_total: u64,
}

impl AsyncDistrictSolveQueue {
    /// Immediate inject (tests / harness) — available to apply this frame.
    pub fn post(&mut self, result: DistrictSolveResult) {
        self.pending.push_back(result);
        self.posted_total = self.posted_total.saturating_add(1);
    }

    /// Live enqueue — applies **next** frame after [`promote_async_district_staging_system`].
    pub fn post_deferred(&mut self, result: DistrictSolveResult) {
        self.staging.push_back(result);
        self.posted_total = self.posted_total.saturating_add(1);
    }

    pub fn promote_staging(&mut self) {
        while let Some(r) = self.staging.pop_front() {
            self.pending.push_back(r);
        }
    }

    pub fn drain_apply<F>(&mut self, mut apply: F)
    where
        F: FnMut(&DistrictSolveResult),
    {
        while let Some(r) = self.pending.pop_front() {
            apply(&r);
            self.applied_total = self.applied_total.saturating_add(1);
        }
    }
}

/// Pure district-slice pack — no ECS mutation (safe for off-thread workers later).
#[must_use]
pub fn compute_district_solve_result(
    district_id: u32,
    edge_indices: &[usize],
    load: &[f32],
) -> DistrictSolveResult {
    let edge_load = edge_indices
        .iter()
        .filter_map(|&idx| load.get(idx).copied().map(|l| (idx, l)))
        .collect();
    DistrictSolveResult {
        district_id,
        edge_load,
    }
}

/// After district scope + greedy solve: snapshot active-edge loads into deferred jobs.
pub fn enqueue_async_district_solve_system(
    scope: Res<DistrictSolveScope>,
    solver: Res<ThroughputSolverState>,
    mut queue: ResMut<AsyncDistrictSolveQueue>,
) {
    let scoped = scope.active_district_count > 0
        || scope.empty_districts_skipped > 0
        || !scope.active_edge_indices.is_empty();
    if !scoped {
        return;
    }
    // One in-flight job — scaffold cadence; real async would shard by district_id.
    if !queue.staging.is_empty() || !queue.pending.is_empty() {
        return;
    }
    let result = compute_district_solve_result(
        scope.active_district_count.max(1),
        &scope.active_edge_indices,
        &solver.load,
    );
    queue.post_deferred(result);
}

/// Main-thread apply only — merges posted loads into SoA solver state.
pub fn apply_async_district_solve_results_system(
    mut queue: ResMut<AsyncDistrictSolveQueue>,
    mut solver: ResMut<ThroughputSolverState>,
    mut runtime: ResMut<LogisticsThroughputRuntimeWitness>,
) {
    let before = queue.applied_total;
    queue.drain_apply(|result| {
        for (idx, load) in &result.edge_load {
            if *idx < solver.load.len() {
                solver.load[*idx] = solver.load[*idx].max(*load);
            }
        }
    });
    if queue.applied_total > before {
        runtime.saw_async_district_solve = true;
    }
}

/// Move staging → pending so apply runs on the **next** frame (LOG-D-04 runtime_check).
pub fn promote_async_district_staging_system(mut queue: ResMut<AsyncDistrictSolveQueue>) {
    queue.promote_staging();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn async_district_results_apply_on_main_thread_only() {
        let mut queue = AsyncDistrictSolveQueue::default();
        queue.post(DistrictSolveResult {
            district_id: 1,
            edge_load: vec![(0, 2.5)],
        });
        let mut applied = 0.0f32;
        queue.drain_apply(|r| {
            applied = r.edge_load[0].1;
        });
        assert!((applied - 2.5).abs() < 1e-4);
        assert_eq!(queue.applied_total, 1);
    }

    #[test]
    fn deferred_job_applies_after_promote() {
        let mut queue = AsyncDistrictSolveQueue::default();
        queue.post_deferred(DistrictSolveResult {
            district_id: 2,
            edge_load: vec![(1, 3.0)],
        });
        assert!(queue.pending.is_empty());
        queue.drain_apply(|_| {});
        assert_eq!(queue.applied_total, 0);
        queue.promote_staging();
        assert_eq!(queue.pending.len(), 1);
        queue.drain_apply(|_| {});
        assert_eq!(queue.applied_total, 1);
    }

    #[test]
    fn compute_district_solve_result_packs_active_loads() {
        let r = compute_district_solve_result(1, &[0, 2], &[1.5, 9.0, 0.25]);
        assert_eq!(r.district_id, 1);
        assert_eq!(r.edge_load, vec![(0, 1.5), (2, 0.25)]);
    }
}
