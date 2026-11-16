//! **Deployment orders** (PLAN-FRONT-COMBAT-v1 FC-6): the one queue UI and AI both enqueue into.
//! Enqueue-only; [`execute_deployment_orders_system`] is the single writer of formation posture / target
//! from orders and reports every result through [`DeploymentOutcomes`].
//!
//! **DEC-13** (aig3 5.10 *Action Execution*): [`DeploymentOrder::priority`] +
//! [`DeploymentOrderQueue::enqueue_interruptible`] implement preemption; [`expand_compound_orders`]
//! builds sequenced compound actions. Strategic delayed lane:
//! [`crate::strategic::StrategicCommandQueue`].

use std::collections::VecDeque;

use bevy::prelude::*;

use super::balance::FrontBalance;
use super::formation::{Formation, Posture};
use super::fortification::{plan_fortification_rows_with_analysis, FortifyRow};
use super::pressure::{PressureKind, PressurePoint, PressurePointBook};
use super::theater::{TheaterBook, TheaterId};
use crate::construction::{PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind};
use crate::strategic::{BuildSiteTile, LayerType, MAX_STRATEGIC_FACTION_SLOTS};
use crate::systems::sim_control::SimTimeMicros;

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
    /// DEC-13: higher priority preempts pending same-formation orders (0 = default FIFO).
    pub priority: u8,
}

impl DeploymentOrder {
    #[must_use]
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
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

    /// DEC-13: enqueue with preemption - drops pending same-formation orders with lower priority.
    pub fn enqueue_interruptible(
        &mut self,
        order: DeploymentOrder,
    ) -> Result<DeploymentOrderId, EnqueueError> {
        self.pending
            .retain(|(_, o)| !(o.formation == order.formation && o.priority < order.priority));
        self.enqueue(order)
    }

    /// Drain highest priority first, then earlier ids (DEC-13 action manager arbitration).
    pub fn drain_by_priority(&mut self) -> Vec<(DeploymentOrderId, DeploymentOrder)> {
        let mut items: Vec<_> = self.pending.drain(..).collect();
        items.sort_by(|a, b| b.1.priority.cmp(&a.1.priority).then(a.0.cmp(&b.0)));
        items
    }
}


/// DEC-13: expand a compound waypoint chain into sequenced Move...Hold orders (same priority).
#[must_use]
pub fn expand_compound_orders(
    formation: Entity,
    issued_by: u8,
    waypoints: &[IVec2],
    priority: u8,
) -> Vec<DeploymentOrder> {
    let mut out = Vec::with_capacity(waypoints.len().saturating_add(1));
    for &tile in waypoints {
        out.push(DeploymentOrder {
            formation,
            kind: DeploymentKind::Move(tile),
            issued_by,
            priority,
        });
    }
    out.push(DeploymentOrder {
        formation,
        kind: DeploymentKind::Hold,
        issued_by,
        priority,
    });
    out
}

/// Order that sends `formation` to a pressure point: reinforce a threatened one, attack through an opportunity.
#[must_use]
pub fn order_for_pressure_point(formation: Entity, point: &PressurePoint) -> DeploymentOrder {
    DeploymentOrder {
        formation,
        issued_by: point.side,
        kind: match point.kind {
            PressureKind::Threatened => DeploymentKind::Reinforce { theater: point.theater, tile: point.tile },
            PressureKind::Opportunity => DeploymentKind::Attack(point.tile),
        },
        priority: 0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeploymentReject {
    NoSuchFormation,
    /// The formation belongs to another faction.
    NotOwner,
    /// Broken formations take no orders until they rally.
    Broken,
    UnknownTheater,
    /// Nothing of the issuer's own to dig in on in that theater.
    NothingToHold,
}

/// One executed (or rejected) order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutcomeRecord {
    pub id: DeploymentOrderId,
    pub formation: Entity,
    pub kind: DeploymentKind,
    pub issued_by: u8,
    pub result: Result<(), DeploymentReject>,
    /// Sim time the executor ran, microseconds (0 when no `SimTimeMicros`).
    pub at_micros: u64,
}

const RECENT_CAP: usize = 128;
const OUTCOME_CAP: usize = 64;

/// Feedback from the executor, for UI and AI.
///
/// * `last` / `records`: the most recent pass only.
/// * `recent`: rolling history (capped) across passes, for "was my order refused?" lookups.
#[derive(Resource, Clone, Debug, Default)]
pub struct DeploymentOutcomes {
    pub last: Vec<(DeploymentOrderId, Result<(), DeploymentReject>)>,
    pub records: Vec<OutcomeRecord>,
    pub recent: Vec<OutcomeRecord>,
}

impl DeploymentOutcomes {
    /// Newest outcome for a formation.
    #[must_use]
    pub fn latest_for(&self, formation: Entity) -> Option<&OutcomeRecord> {
        self.recent.iter().rev().find(|r| r.formation == formation)
    }

    /// Outcomes of one faction's orders, newest last.
    pub fn for_faction(&self, slot: u8) -> impl Iterator<Item = &OutcomeRecord> {
        self.recent.iter().filter(move |r| r.issued_by == slot)
    }

    /// Why the formation's newest order was refused, if it was and that was within `window_s` of `now_micros`.
    /// An accepted newer order clears the refusal.
    #[must_use]
    pub fn recently_rejected(&self, formation: Entity, now_micros: u64, window_s: f32) -> Option<DeploymentReject> {
        let r = self.latest_for(formation)?;
        let age_s = now_micros.saturating_sub(r.at_micros) as f32 / 1_000_000.0;
        match r.result {
            Err(e) if age_s <= window_s => Some(e),
            _ => None,
        }
    }

    fn push(&mut self, rec: OutcomeRecord) {
        if self.records.len() < OUTCOME_CAP {
            self.last.push((rec.id, rec.result));
            self.records.push(rec);
        }
        self.recent.push(rec);
        if self.recent.len() > RECENT_CAP {
            self.recent.remove(0);
        }
    }
}

/// What an accepted order asks the executor to do beyond moving the formation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderEffect {
    None,
    /// Plan fortification rows for the issuer's side of this theater.
    PlanFortify(TheaterId),
}

/// Pure validation + effect of one order on one formation.
pub fn apply_deployment_order(
    f: &mut Formation,
    order: &DeploymentOrder,
    theaters: &TheaterBook,
) -> Result<OrderEffect, DeploymentReject> {
    if f.faction_slot != order.issued_by {
        return Err(DeploymentReject::NotOwner);
    }
    if f.posture == Posture::Retreat {
        return Err(DeploymentReject::Broken);
    }
    let go = |f: &mut Formation, posture: Posture, tile: IVec2| {
        f.posture = posture;
        f.target = (tile != f.tile).then_some(tile);
        f.move_progress = 0.0;
        f.fortify = false;
        f.dug_in = 0.0;
        if tile == f.tile {
            f.posture = Posture::Hold;
        }
    };
    match order.kind {
        DeploymentKind::Move(t) => go(f, Posture::Move, t),
        DeploymentKind::Attack(t) => go(f, Posture::Attack, t),
        DeploymentKind::Hold => {
            f.posture = Posture::Hold;
            f.target = None;
        }
        DeploymentKind::Reinforce { theater, tile } => {
            if theaters.get(theater).is_none() {
                return Err(DeploymentReject::UnknownTheater);
            }
            go(f, Posture::Move, tile);
        }
        DeploymentKind::Fortify(id) => {
            let t = theaters.get(id).ok_or(DeploymentReject::UnknownTheater)?;
            if t.aor_contains(f.tile, theaters.depth_tiles) {
                // Already in the area of responsibility: dig in where we stand.
                f.posture = Posture::Hold;
                f.target = None;
            } else {
                let Some(nearest) = t
                    .cells_of(f.faction_slot)
                    .min_by_key(|c| ((c.tile - f.tile).length_squared(), c.tile.y, c.tile.x))
                else {
                    return Err(DeploymentReject::NothingToHold);
                };
                f.posture = Posture::Move;
                f.target = Some(nearest.tile);
                f.move_progress = 0.0;
                f.dug_in = 0.0;
            }
            f.fortify = true;
            return Ok(OrderEffect::PlanFortify(id));
        }
    }
    Ok(OrderEffect::None)
}

/// Rows a Fortify order proposed (all factions), for the AI economy brain and UI to commit.
#[derive(Resource, Clone, Debug, Default)]
pub struct FortifyPlans {
    pub rows: Vec<(u8, TheaterId, FortifyRow)>,
}

/// Sole writer of formation posture / target from orders: drains the queue in issue order. A Fortify
/// order also records its planned rows; for the player (slot 0) they are queued as pending construction
/// entries (unapproved), so the construction funnel validates stock and commits them.
#[allow(clippy::too_many_arguments)]
pub fn execute_deployment_orders_system(
    mut queue: ResMut<DeploymentOrderQueue>,
    theaters: Res<TheaterBook>,
    balance: Option<Res<FrontBalance>>,
    pressure: Option<Res<PressurePointBook>>,
    analysis: Option<Res<crate::terrain::TerrainAnalysisBook>>,
    time: Option<Res<SimTimeMicros>>,
    mut outcomes: ResMut<DeploymentOutcomes>,
    mut plans: Option<ResMut<FortifyPlans>>,
    mut pending: Option<ResMut<PendingConstructionQueue>>,
    mut formations: Query<&mut Formation>,
) {
    if queue.is_empty() {
        return;
    }
    outcomes.last.clear();
    outcomes.records.clear();
    let now = time.map_or(0, |t| t.0);
    for (id, order) in queue.drain_by_priority() {
        let result = match formations.get_mut(order.formation) {
            Ok(mut f) => apply_deployment_order(&mut f, &order, &theaters),
            Err(_) => Err(DeploymentReject::NoSuchFormation),
        };
        if let (Ok(OrderEffect::PlanFortify(tid)), Some(balance), Some(plans)) = (result, balance.as_deref(), plans.as_deref_mut()) {
            if let Some(t) = theaters.get(tid) {
                let points = pressure.as_ref().map_or(&[][..], |p| p.points.as_slice());
                for row in plan_fortification_rows_with_analysis(
                    t,
                    order.issued_by,
                    points,
                    balance,
                    analysis.as_deref(),
                ) {
                    if plans.rows.iter().any(|(s, th, r)| *s == order.issued_by && *th == tid && *r == row) {
                        continue;
                    }
                    plans.rows.push((order.issued_by, tid, row));
                    if order.issued_by == 0 {
                        if let Some(q) = pending.as_deref_mut() {
                            queue_fortify_row(q, row);
                        }
                    }
                }
            }
        }
        outcomes.push(OutcomeRecord {
            id,
            formation: order.formation,
            kind: order.kind,
            issued_by: order.issued_by,
            result: result.map(|_| ()),
            at_micros: now,
        });
    }
}

/// One proposed row as an unapproved pending construction entry (skipped if an equal one is queued).
fn queue_fortify_row(q: &mut PendingConstructionQueue, row: FortifyRow) {
    let origin = BuildSiteTile { x: row.origin.x.max(0) as u32, z: row.origin.y.max(0) as u32 };
    let archetype = row.kind.site_archetype();
    if q.entries.iter().any(|e| e.archetype == archetype && e.origin == origin) {
        return;
    }
    q.push(PendingBuildBlueprint {
        kind: PendingEntryKind::BuildSite,
        label: format!("fortify:{},{}", origin.x, origin.z),
        archetype,
        origin,
        footprint: row.kind.footprint(),
        layer: LayerType::Surface,
        rotation_quarter_turns: 0,
        mirror_x: false,
        approved: false,
        catalog_id: None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(issued_by: u8) -> DeploymentOrder {
        DeploymentOrder { formation: Entity::PLACEHOLDER, kind: DeploymentKind::Hold, issued_by, priority: 0 }
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

    #[test]
    fn rejection_is_remembered_per_formation_until_a_newer_order_is_accepted() {
        let e = Entity::PLACEHOLDER;
        let mut o = DeploymentOutcomes::default();
        let rec = |id, result, at| OutcomeRecord {
            id,
            formation: e,
            kind: DeploymentKind::Hold,
            issued_by: 1,
            result,
            at_micros: at,
        };
        o.push(rec(1, Err(DeploymentReject::Broken), 10_000_000));
        assert_eq!(o.recently_rejected(e, 20_000_000, 60.0), Some(DeploymentReject::Broken));
        assert_eq!(o.recently_rejected(e, 200_000_000, 60.0), None, "expired");
        o.push(rec(2, Ok(()), 30_000_000));
        assert_eq!(o.recently_rejected(e, 31_000_000, 60.0), None, "newer accepted order clears it");
        assert_eq!(o.for_faction(1).count(), 2);
    }
    #[test]
    fn dec13_priority_interrupts_and_compounds() {
        let mut q = DeploymentOrderQueue::default();
        let f = Entity::from_bits(7);
        let low = DeploymentOrder {
            formation: f,
            kind: DeploymentKind::Hold,
            issued_by: 0,
            priority: 1,
        };
        let high = DeploymentOrder {
            formation: f,
            kind: DeploymentKind::Attack(IVec2::new(1, 1)),
            issued_by: 0,
            priority: 5,
        };
        q.enqueue(low).unwrap();
        q.enqueue_interruptible(high).unwrap();
        assert_eq!(q.len(), 1);
        let drained = q.drain_by_priority();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].1.priority, 5);

        let compound = expand_compound_orders(f, 0, &[IVec2::new(2, 2), IVec2::new(3, 3)], 2);
        assert_eq!(compound.len(), 3);
        assert!(matches!(compound[0].kind, DeploymentKind::Move(_)));
        assert!(matches!(compound[2].kind, DeploymentKind::Hold));
        assert!(compound.iter().all(|o| o.priority == 2));
    }

}
