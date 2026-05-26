//! Strategic command queue — M1 stub + M2 fixed-tick dispatch delay (**S7B-M2-001**).

use bevy::prelude::*;

use crate::systems::sim_control::SimStepStamp;

use super::comms_contract::{CommunicationPlane, DispatchMessage};

/// Fixed-tick delay before a queued command is visible to consumers (**D-S7-04 A**).
pub const DISPATCH_DELAY_TICKS: u32 = 8;

/// Read-only pending queue in simulation; M2 drains into [`Self::delivered`] after delay.
#[derive(Resource, Clone, Debug, Default)]
pub struct StrategicCommandQueue {
    pub pending: Vec<DispatchMessage>,
    pub delivered: Vec<DispatchMessage>,
    next_command_id: u64,
}

impl StrategicCommandQueue {
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn dispatch_delay_ticks(&self) -> u32 {
        DISPATCH_DELAY_TICKS
    }

    /// Enqueue with `deliver_after = issued_at.tick + DISPATCH_DELAY_TICKS`.
    pub fn enqueue_strategic(
        &mut self,
        issued_at: SimStepStamp,
        summary: impl Into<String>,
    ) -> DispatchMessage {
        enqueue_strategic_command(self, issued_at, summary)
    }

    /// Move messages whose `deliver_after` has elapsed into [`Self::delivered`].
    pub fn tick(&mut self, now: SimStepStamp) {
        tick_strategic_command_queue(self, now);
    }

    #[must_use]
    pub fn is_visible_to_consumer(&self, msg: &DispatchMessage, now: SimStepStamp) -> bool {
        now.tick >= msg.deliver_after.tick
    }
}

/// Enqueue a strategic command with fixed-tick delay.
pub fn enqueue_strategic_command(
    queue: &mut StrategicCommandQueue,
    issued_at: SimStepStamp,
    summary: impl Into<String>,
) -> DispatchMessage {
    let id = queue.next_command_id;
    queue.next_command_id = queue.next_command_id.wrapping_add(1);
    let deliver_after = SimStepStamp::new(
        issued_at.tick.saturating_add(u64::from(DISPATCH_DELAY_TICKS)),
        issued_at.sim_time_micros,
    );
    let msg = DispatchMessage {
        plane: CommunicationPlane::StrategicCommand,
        issued_at,
        deliver_after,
        command_id: id,
        summary: summary.into(),
    };
    queue.pending.push(msg.clone());
    msg
}

/// Drain pending messages whose delivery tick has been reached.
pub fn tick_strategic_command_queue(queue: &mut StrategicCommandQueue, now: SimStepStamp) {
    let mut ready = Vec::new();
    queue.pending.retain(|msg| {
        if now.tick >= msg.deliver_after.tick {
            ready.push(msg.clone());
            false
        } else {
            true
        }
    });
    queue.delivered.extend(ready);
}

#[must_use]
pub fn dispatch_delay_ticks() -> u32 {
    DISPATCH_DELAY_TICKS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_not_visible_before_deliver_after_tick() {
        let mut queue = StrategicCommandQueue::default();
        let issued = SimStepStamp::new(10, 0);
        let msg = queue.enqueue_strategic(issued, "secure corridor");
        assert_eq!(queue.pending_count(), 1);
        assert_eq!(queue.delivered.len(), 0);

        let before = SimStepStamp::new(10 + u64::from(DISPATCH_DELAY_TICKS) - 1, 0);
        assert!(!queue.is_visible_to_consumer(&msg, before));
        queue.tick(before);
        assert_eq!(queue.delivered.len(), 0);

        let at = SimStepStamp::new(10 + u64::from(DISPATCH_DELAY_TICKS), 0);
        assert!(queue.is_visible_to_consumer(&msg, at));
        queue.tick(at);
        assert_eq!(queue.delivered.len(), 1);
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn dispatch_delay_ticks_constant_is_positive() {
        assert!(dispatch_delay_ticks() > 0);
    }
}
