//! Async district throughput solve scaffold (LOG-D-04). Jobs post off-thread; only main-thread apply mutates ECS.

use bevy::prelude::*;
use std::collections::VecDeque;

/// Posted solve result — applied next frame on the main thread.
#[derive(Clone, Debug)]
pub struct DistrictSolveResult {
    pub district_id: u32,
    pub edge_load: Vec<(usize, f32)>,
}

/// LOG-D-04: queue + apply boundary (no off-thread field mutation).
#[derive(Resource, Default)]
pub struct AsyncDistrictSolveQueue {
    pub pending: VecDeque<DistrictSolveResult>,
    pub applied_total: u64,
}

impl AsyncDistrictSolveQueue {
    pub fn post(&mut self, result: DistrictSolveResult) {
        self.pending.push_back(result);
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

pub fn apply_async_district_solve_results_system(
    mut queue: ResMut<AsyncDistrictSolveQueue>,
    mut solver: ResMut<super::types::ThroughputSolverState>,
) {
    queue.drain_apply(|result| {
        for (idx, load) in &result.edge_load {
            if *idx < solver.load.len() {
                solver.load[*idx] = solver.load[*idx].max(*load);
            }
        }
    });
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
}
