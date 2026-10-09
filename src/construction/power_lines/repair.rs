//! Power repair queue — jobs, priority, parts gate (COD-POWER-REPAIR-QUEUE-001).

use std::collections::HashSet;

use bevy::prelude::*;

use crate::infrastructure::utility::UtilityGraph;
use crate::systems::sim_control::SimControlState;

use super::damage::{PowerLineDamageBook, PowerLineSegmentHealth};

/// Reason stored when a job targets a link the commit funnel never wrote.
pub const POWER_REPAIR_UNCOMMITTED: &str = "line is not on the committed grid";

pub const POWER_REPAIR_PARTS_PER_SEGMENT: u32 = 2;
pub const POWER_REPAIR_TICKS_PER_JOB: u32 = 120;

#[derive(Clone, Debug, PartialEq)]
pub struct PowerRepairJob {
    pub id: u64,
    pub link_id: u64,
    pub label: String,
    pub priority: u8,
    pub parts_have: u32,
    pub parts_need: u32,
    pub ticks_remaining: u32,
    pub blocked_reason: Option<String>,
}

impl PowerRepairJob {
    #[must_use]
    pub fn parts_ready(&self) -> bool {
        self.parts_have >= self.parts_need && self.blocked_reason.is_none()
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct PowerRepairQueue {
    pub jobs: Vec<PowerRepairJob>,
    pub next_id: u64,
    pub completed: u32,
}

impl PowerRepairQueue {
    pub fn enqueue_damaged_segment(
        &mut self,
        seg: &PowerLineSegmentHealth,
        parts_have: u32,
    ) -> Option<u64> {
        if self.jobs.iter().any(|j| j.link_id == seg.link_id) {
            return None;
        }
        self.next_id = self.next_id.saturating_add(1);
        let id = self.next_id;
        let parts_need = POWER_REPAIR_PARTS_PER_SEGMENT;
        let blocked_reason = if parts_have >= parts_need {
            None
        } else {
            Some(format!("need {} spare parts", parts_need - parts_have))
        };
        self.jobs.push(PowerRepairJob {
            id,
            link_id: seg.link_id,
            label: format!("MV segment · link {}", seg.link_id),
            priority: 50,
            parts_have,
            parts_need,
            ticks_remaining: POWER_REPAIR_TICKS_PER_JOB,
            blocked_reason,
        });
        Some(id)
    }

    pub fn queue_all_damaged(&mut self, book: &PowerLineDamageBook, parts_have: u32) -> u32 {
        let mut added = 0_u32;
        for seg in book.segments.values() {
            if seg.destroyed || seg.hp < seg.max_hp {
                if self.enqueue_damaged_segment(seg, parts_have).is_some() {
                    added = added.saturating_add(1);
                }
            }
        }
        added
    }

    pub fn cancel(&mut self, job_id: u64) -> bool {
        let before = self.jobs.len();
        self.jobs.retain(|j| j.id != job_id);
        self.jobs.len() < before
    }

    pub fn sort_by_priority(&mut self) {
        self.jobs
            .sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.id.cmp(&b.id)));
    }
}

/// Advance repair jobs one sim step. Does not insert graph edges — that stays on the commit funnel.
pub fn advance_power_repair_queue(
    queue: &mut PowerRepairQueue,
    book: &mut PowerLineDamageBook,
    committed_links: &HashSet<u64>,
    sim_advancing: bool,
) {
    if !sim_advancing {
        return;
    }
    queue.sort_by_priority();
    let mut completed_ids = Vec::new();
    for job in &mut queue.jobs {
        if !committed_links.contains(&job.link_id) {
            job.blocked_reason = Some(POWER_REPAIR_UNCOMMITTED.to_string());
            continue;
        }
        if job.parts_have >= job.parts_need {
            job.blocked_reason = None;
        }
        if !job.parts_ready() {
            continue;
        }
        if job.ticks_remaining > 0 {
            job.ticks_remaining = job.ticks_remaining.saturating_sub(1);
        }
        if job.ticks_remaining == 0 {
            completed_ids.push((job.id, job.link_id));
        }
    }
    for (job_id, link_id) in completed_ids {
        if !committed_links.contains(&link_id) {
            continue;
        }
        book.restore_segment(link_id);
        queue.jobs.retain(|j| j.id != job_id);
        queue.completed = queue.completed.saturating_add(1);
    }
}

pub fn tick_power_repair_queue_system(
    mut queue: ResMut<PowerRepairQueue>,
    mut book: ResMut<PowerLineDamageBook>,
    graph: Option<Res<UtilityGraph>>,
    ctrl: Option<Res<SimControlState>>,
) {
    let sim_advancing = ctrl.as_deref().is_some_and(|c| c.should_tick());
    let committed = graph
        .as_deref()
        .map(|utility| {
            utility
                .power_edges
                .iter()
                .map(|edge| edge.link_id)
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    advance_power_repair_queue(&mut queue, &mut book, &committed, sim_advancing);
}

#[must_use]
pub fn power_repair_queue_witness_green() -> bool {
    let mut book = PowerLineDamageBook::default();
    let mut seg = PowerLineSegmentHealth::new(7);
    seg.hp = 40.0;
    book.segments.insert(7, seg);
    let mut queue = PowerRepairQueue::default();
    let id = queue.enqueue_damaged_segment(book.segments.get(&7).unwrap(), 2);
    id.is_some()
        && queue.jobs[0].priority <= 100
        && queue.jobs[0].parts_need == POWER_REPAIR_PARTS_PER_SEGMENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_repair_witness_green_lib() {
        assert!(power_repair_queue_witness_green());
    }

    fn ready_job(link_id: u64, ticks: u32) -> PowerRepairQueue {
        let mut book = PowerLineDamageBook::default();
        let mut seg = PowerLineSegmentHealth::new(link_id);
        seg.hp = 0.0;
        seg.destroyed = true;
        book.segments.insert(link_id, seg);
        let mut queue = PowerRepairQueue::default();
        queue
            .enqueue_damaged_segment(book.segments.get(&link_id).unwrap(), 2)
            .unwrap();
        queue.jobs[0].ticks_remaining = ticks;
        queue
    }

    fn graph_with_link(link_id: u64) -> crate::infrastructure::utility::graph::UtilityGraph {
        use crate::infrastructure::utility::graph::{UtilityGraph, UtilityGraphEdge};
        UtilityGraph {
            nodes: Vec::new(),
            power_edges: vec![UtilityGraphEdge {
                from: 1,
                to: 2,
                link_id,
                throughput: 1.0,
            }],
            hydrated_from_snapshot: true,
        }
    }

    #[test]
    fn grid_repair_ignores_an_uncommitted_cut() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        let mut book = PowerLineDamageBook::default();
        let mut seg = PowerLineSegmentHealth::new(7);
        seg.hp = 0.0;
        seg.destroyed = true;
        book.segments.insert(7, seg);
        app.insert_resource(book);
        app.insert_resource(ready_job(7, 1));
        app.insert_resource(graph_with_link(99));
        app.insert_resource(SimControlState {
            paused: false,
            steps_remaining: 0,
            speed: 1.0,
        });
        app.add_systems(Update, tick_power_repair_queue_system);
        app.update();
        let book = app.world().resource::<PowerLineDamageBook>();
        assert!(book.segments.get(&7).unwrap().circuit_open());
        let queue = app.world().resource::<PowerRepairQueue>();
        assert_eq!(queue.completed, 0);
        assert_eq!(queue.jobs.len(), 1);
        assert_eq!(
            queue.jobs[0].blocked_reason.as_deref(),
            Some(POWER_REPAIR_UNCOMMITTED)
        );
        assert_eq!(queue.jobs[0].ticks_remaining, 1);
    }

    #[test]
    fn grid_repair_holds_when_the_sim_is_paused() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        let mut book = PowerLineDamageBook::default();
        let mut seg = PowerLineSegmentHealth::new(7);
        seg.hp = 0.0;
        seg.destroyed = true;
        book.segments.insert(7, seg);
        app.insert_resource(book);
        app.insert_resource(ready_job(7, 4));
        app.insert_resource(graph_with_link(7));
        app.insert_resource(SimControlState {
            paused: true,
            steps_remaining: 0,
            speed: 1.0,
        });
        app.add_systems(Update, tick_power_repair_queue_system);
        app.update();
        let queue = app.world().resource::<PowerRepairQueue>();
        assert_eq!(queue.jobs[0].ticks_remaining, 4);
        assert_eq!(queue.completed, 0);
        assert!(app
            .world()
            .resource::<PowerLineDamageBook>()
            .segments
            .get(&7)
            .unwrap()
            .circuit_open());
    }

    #[test]
    fn grid_repair_recloses_a_committed_line() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        let mut book = PowerLineDamageBook::default();
        let mut seg = PowerLineSegmentHealth::new(7);
        seg.hp = 0.0;
        seg.destroyed = true;
        book.segments.insert(7, seg);
        app.insert_resource(book);
        app.insert_resource(ready_job(7, 1));
        let graph = graph_with_link(7);
        let edges_before = graph.power_edges.len();
        app.insert_resource(graph);
        app.insert_resource(SimControlState {
            paused: false,
            steps_remaining: 0,
            speed: 1.0,
        });
        app.add_systems(Update, tick_power_repair_queue_system);
        app.update();
        let book = app.world().resource::<PowerLineDamageBook>();
        assert!(!book.segments.get(&7).unwrap().circuit_open());
        assert!((book.segments.get(&7).unwrap().load_fraction() - 1.0).abs() < 1e-5);
        let queue = app.world().resource::<PowerRepairQueue>();
        assert!(queue.jobs.is_empty());
        assert_eq!(queue.completed, 1);
        assert_eq!(
            app.world().resource::<crate::infrastructure::utility::UtilityGraph>().power_edges.len(),
            edges_before
        );
    }

    #[test]
    fn grid_ready_parts_clear_a_stale_block() {
        let mut book = PowerLineDamageBook::default();
        let mut seg = PowerLineSegmentHealth::new(7);
        seg.hp = 10.0;
        book.segments.insert(7, seg);
        let mut queue = PowerRepairQueue::default();
        queue
            .enqueue_damaged_segment(book.segments.get(&7).unwrap(), 0)
            .unwrap();
        queue.jobs[0].parts_have = POWER_REPAIR_PARTS_PER_SEGMENT;
        queue.jobs[0].ticks_remaining = 1;
        assert!(queue.jobs[0].blocked_reason.is_some());
        let committed = HashSet::from([7_u64]);
        advance_power_repair_queue(&mut queue, &mut book, &committed, true);
        assert!(queue.jobs.is_empty());
        assert!(!book.segments.get(&7).unwrap().circuit_open());
        assert!((book.segments.get(&7).unwrap().hp - book.segments.get(&7).unwrap().max_hp).abs() < 1e-4);
    }

    #[test]
    fn link_paused_sim_does_not_advance_repair() {
        let mut book = PowerLineDamageBook::default();
        let mut seg = PowerLineSegmentHealth::new(7);
        seg.hp = 0.0;
        seg.destroyed = true;
        book.segments.insert(7, seg);
        let mut queue = ready_job(7, 4);
        let committed = HashSet::from([7_u64]);
        let paused = SimControlState {
            paused: true,
            steps_remaining: 0,
            speed: 1.0,
        };
        assert!(!paused.should_tick());
        advance_power_repair_queue(&mut queue, &mut book, &committed, paused.should_tick());
        assert_eq!(queue.jobs[0].ticks_remaining, 4);
        assert_eq!(queue.completed, 0);
        assert!(book.segments.get(&7).unwrap().circuit_open());
    }
}
