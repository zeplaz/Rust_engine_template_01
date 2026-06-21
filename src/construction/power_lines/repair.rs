//! Power repair queue — jobs, priority, parts gate (COD-POWER-REPAIR-QUEUE-001).

use bevy::prelude::*;

use super::damage::{PowerLineDamageBook, PowerLineSegmentHealth};

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
        self.jobs.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.id.cmp(&b.id))
        });
    }
}

pub fn tick_power_repair_queue_system(
    mut queue: ResMut<PowerRepairQueue>,
    mut book: ResMut<PowerLineDamageBook>,
    tick: Option<Res<crate::systems::sim_control::SimTick>>,
) {
    let _ = tick;
    queue.sort_by_priority();
    let mut completed_ids = Vec::new();
    for job in &mut queue.jobs {
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
        book.restore_segment(link_id);
        queue.jobs.retain(|j| j.id != job_id);
        queue.completed = queue.completed.saturating_add(1);
    }
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
}
