//! **Deployment orders** (PLAN-FRONT-COMBAT-v1 FC-6 groundwork): the one queue UI and AI both
//! enqueue into. Enqueue-only; the executor (single writer of formation posture/target) lands with
//! FC-5 formations and drains [`DeploymentOrderQueue::drain`].

use std::collections::VecDeque;

use bevy::prelude::*;

use super::theater::TheaterId;
use crate::strategic::MAX_STRATEGIC_FACTION_SLOTS;

/// What a formation is told to do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeploymentKind {
    Move(IVec2),
    Hold,
    /// Reinforce a pressure point (its tile) in a theater.
    Reinforce { theater: TheaterId, tile: IVec2 },
    Attack(IVec2),
    /// Plan fortifications for the issuer's side of a theater.
    Fortify(TheaterId),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeploymentOrder {
    pub formation: Entity,
    pub kind: DeploymentKind,
    /// Faction slot issuing the order (player = 0, AI factions use their slot).
    pub issued_by: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnqueueError {
    UnknownFaction,
    QueueFull,
}

/// Monotonic order number, unique per queue.
pub type DeploymentOrderId = u64;

#[derive(Resource, Clone, Debug)]
pub struct DeploymentOrderQueue {
    pending: VecDeque<(DeploymentOrderId, DeploymentOrder)>,
    next_id: DeploymentOrderId,
    /// Hard cap so a misbehaving issuer cannot grow the queue without bound.
    pub capacity: usize,
}

impl Default for DeploymentOrderQueue {
    fn default() -> Self {
        Self { pending: VecDeque::new(), next_id: 1, capacity: 4096 }
    }
}

impl DeploymentOrderQueue {
    /// The only way in. Pure: validates the issuer slot and capacity, returns the order number.
    pub fn enqueue(&mut self, order: DeploymentOrder) -> Result<DeploymentOrderId, EnqueueError> {
        if order.issued_by as usize >= MAX_STRATEGIC_FACTION_SLOTS {
            return Err(EnqueueError::UnknownFaction);
        }
        if self.pending.len() >= self.capacity {
            return Err(EnqueueError::QueueFull);
        }
        let id = self.next_id;
        self.next_id += 1;
        self.pending.push_back((id, order));
        Ok(id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &(DeploymentOrderId, DeploymentOrder)> {
        self.pending.iter()
    }

    /// Executor-side: take every pending order in issue order.
    pub fn drain(&mut self) -> Vec<(DeploymentOrderId, DeploymentOrder)> {
        self.pending.drain(..).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(issued_by: u8) -> DeploymentOrder {
        DeploymentOrder { formation: Entity::PLACEHOLDER, kind: DeploymentKind::Hold, issued_by }
    }

    #[test]
    fn enqueue_validates_and_preserves_issue_order() {
        let mut q = DeploymentOrderQueue::default();
        let a = q.enqueue(order(0)).unwrap();
        let b = q.enqueue(order(3)).unwrap();
        assert!(b > a);
        assert_eq!(q.enqueue(order(200)), Err(EnqueueError::UnknownFaction));
        let drained = q.drain();
        assert_eq!(drained.iter().map(|(id, _)| *id).collect::<Vec<_>>(), vec![a, b]);
        assert!(q.is_empty());
        q.capacity = 1;
        q.enqueue(order(0)).unwrap();
        assert_eq!(q.enqueue(order(0)), Err(EnqueueError::QueueFull));
    }
}
