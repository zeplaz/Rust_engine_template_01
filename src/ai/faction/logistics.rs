//! **Logistics brain** — keeps a faction's sites and front supply points supplied through the same
//! order commands the player's logistics panel sends (`CreateStandingLine` via the supply advisor,
//! `CreateOneOffRun` for urgent gaps). Pure over a [`LogisticsOrdersSnapshot`]; the tick system
//! turns the actions into [`LogisticsOrderRequest`] messages.
//!
//! [`LogisticsOrderRequest`]: crate::economy::logistics::orders::LogisticsOrderRequest

use std::collections::BTreeMap;

use crate::economy::logistics::orders::{
    best_source_for, suggest_lines_for_unserved, CargoKind, ConsumerKind, DemandHealth, DemandRow,
    LogisticsOrderCommand, LogisticsOrdersSnapshot, OwnerId, Priority, SiteId, SiteRole,
};

use super::decide::{FactionDoctrine, ScoreTerm, StrategicGoal};

/// Standing lines one decision may create.
pub const MAX_LINES_PER_DECISION: usize = 4;
/// One-off runs one decision may create.
pub const MAX_RUNS_PER_DECISION: usize = 2;
/// A sink must have been starved this many order-sim ticks before a one-off run is justified.
pub const URGENT_STARVED_TICKS: u64 = 20;
/// Largest single run (units); trucks make several trips.
pub const RUN_QUANTITY_CAP: u32 = 60;

/// What the brain wants to send. Both map 1:1 to a [`LogisticsOrderCommand`].
#[derive(Clone, Debug, PartialEq)]
pub enum LogisticsAction {
    Line {
        source: SiteId,
        sink: SiteId,
        cargo: String,
        priority: Priority,
    },
    Run {
        source: SiteId,
        sink: SiteId,
        cargo: String,
        quantity: u32,
        priority: Priority,
    },
}

impl LogisticsAction {
    #[must_use]
    pub fn to_command(&self, owner: OwnerId) -> LogisticsOrderCommand {
        match self {
            Self::Line { source, sink, cargo, priority } => LogisticsOrderCommand::CreateStandingLine {
                owner,
                source: *source,
                sink: *sink,
                cargo: CargoKind::new(cargo.clone()),
                priority: *priority,
                vehicle_cap: None,
            },
            Self::Run { source, sink, cargo, quantity, priority } => LogisticsOrderCommand::CreateOneOffRun {
                owner,
                source: *source,
                sink: *sink,
                cargo: CargoKind::new(cargo.clone()),
                quantity: *quantity,
                priority: *priority,
                vehicle_cap: None,
            },
        }
    }
}

/// One scored action with the terms that justified it.
#[derive(Clone, Debug, PartialEq)]
pub struct ScoredAction {
    pub action: LogisticsAction,
    pub subject: String,
    pub total: f32,
    pub terms: Vec<ScoreTerm>,
}

/// What this faction ordered recently, so a demand is not ordered twice before the snapshot catches up.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LogisticsMemory {
    /// `(sink, cargo, is_run)` → tick until which the order counts as in flight.
    recent: BTreeMap<(u32, String, bool), u64>,
}

impl LogisticsMemory {
    #[must_use]
    pub fn recently(&self, sink: SiteId, cargo: &str, run: bool, now: u64) -> bool {
        self.recent
            .get(&(sink.0, cargo.to_string(), run))
            .is_some_and(|until| *until > now)
    }

    pub fn note(&mut self, sink: SiteId, cargo: &str, run: bool, until: u64) {
        self.recent.insert((sink.0, cargo.to_string(), run), until);
    }

    pub fn prune(&mut self, now: u64) {
        self.recent.retain(|_, until| *until > now);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.recent.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.recent.is_empty()
    }
}

fn severity(h: DemandHealth) -> f32 {
    match h {
        DemandHealth::Satisfied => 0.0,
        DemandHealth::Low => 0.5,
        DemandHealth::Starving => 1.0,
    }
}

fn demand_of<'a>(snap: &'a LogisticsOrdersSnapshot, sink: SiteId, cargo: &str) -> Option<&'a DemandRow> {
    snap.demands.iter().find(|d| d.site == sink && d.cargo == cargo)
}

fn source_available(snap: &LogisticsOrdersSnapshot, source: SiteId, cargo: &str) -> u32 {
    snap.site(source)
        .and_then(|s| s.stock.iter().find(|r| r.cargo == cargo))
        .map_or(0, |r| r.on_hand.saturating_sub(r.reserved_out))
}

fn site_distance_km(snap: &LogisticsOrdersSnapshot, a: SiteId, b: SiteId) -> f32 {
    match (snap.site(a), snap.site(b)) {
        (Some(a), Some(b)) => {
            let (dx, dz) = (a.world_xz[0] - b.world_xz[0], a.world_xz[1] - b.world_xz[1]);
            (dx * dx + dz * dz).sqrt() / 1000.0
        }
        _ => 0.0,
    }
}

fn consumer_terms(d: &DemandRow, doctrine: &FactionDoctrine, goal: StrategicGoal) -> Vec<ScoreTerm> {
    let front = if d.consumer == ConsumerKind::Front { 1.0 } else { 0.0 };
    let goal_front = if goal == StrategicGoal::SecureSupply || goal == StrategicGoal::DefendFront {
        1.0
    } else {
        0.0
    };
    vec![
        ScoreTerm { label: "demand severity", input: severity(d.health), weight: 0.6 },
        ScoreTerm { label: "front sustainment", input: front, weight: 0.3 + 0.4 * doctrine.logistics_focus },
        ScoreTerm { label: "goal emphasis", input: front * goal_front, weight: 0.2 },
        ScoreTerm {
            label: "site under construction",
            input: if d.consumer == ConsumerKind::Construction { 1.0 } else { 0.0 },
            weight: 0.15,
        },
        ScoreTerm { label: "priority", input: f32::from(d.priority as u8) / 3.0, weight: 0.15 },
    ]
}

/// Orders to keep `owner`'s demands served. Deterministic; reads only the snapshot and `memory`.
///
/// `lines` is false when the faction's `AutoSupplyPolicy` already creates standing lines for its
/// unserved demands (the default for AI factions): the brain then only adds urgent one-off runs,
/// so no demand is ordered twice.
#[must_use]
pub fn plan_logistics(
    snap: &LogisticsOrdersSnapshot,
    owner: OwnerId,
    doctrine: &FactionDoctrine,
    goal: StrategicGoal,
    memory: &LogisticsMemory,
    now: u64,
    lines: bool,
) -> Vec<ScoredAction> {
    let mut lines_out: Vec<ScoredAction> = Vec::new();
    let advice = if lines { suggest_lines_for_unserved(snap, owner).commands } else { Vec::new() };
    for cmd in advice {
        let LogisticsOrderCommand::CreateStandingLine { source, sink, cargo, priority, .. } = cmd else {
            continue;
        };
        let Some(d) = demand_of(snap, sink, cargo.as_str()) else { continue };
        if memory.recently(sink, cargo.as_str(), false, now) {
            continue;
        }
        let mut terms = consumer_terms(d, doctrine, goal);
        terms.push(ScoreTerm {
            label: "source distance km",
            input: site_distance_km(snap, source, sink),
            weight: -0.05,
        });
        let total = terms.iter().map(ScoreTerm::value).sum();
        lines_out.push(ScoredAction {
            subject: format!("line {} to site {}", cargo.as_str(), sink.0),
            action: LogisticsAction::Line { source, sink, cargo: cargo.0, priority },
            total,
            terms,
        });
    }
    lines_out.sort_by(|a, b| b.total.total_cmp(&a.total).then_with(|| a.subject.cmp(&b.subject)));
    lines_out.truncate(MAX_LINES_PER_DECISION);

    let mut runs: Vec<ScoredAction> = Vec::new();
    for d in snap.demands.iter().filter(|d| d.owner == owner && d.health == DemandHealth::Starving) {
        if d.starved_ticks < URGENT_STARVED_TICKS || memory.recently(d.site, &d.cargo, true, now) {
            continue;
        }
        let run_open = snap
            .one_off_runs()
            .any(|o| o.sink == d.site && o.cargo == d.cargo && !o.status.is_terminal());
        if run_open {
            continue;
        }
        let Some(source) = best_source_for(snap, d) else { continue };
        let deficit = d.buffer_target.saturating_sub(d.on_hand.saturating_add(d.inbound));
        let quantity = deficit.min(source_available(snap, source, &d.cargo)).min(RUN_QUANTITY_CAP);
        if quantity == 0 {
            continue;
        }
        let priority = if d.consumer == ConsumerKind::Front { Priority::Critical } else { Priority::High };
        let terms = vec![
            ScoreTerm { label: "starved for", input: (d.starved_ticks as f32 / 100.0).min(1.0), weight: 0.6 },
            ScoreTerm { label: "buffer deficit", input: deficit as f32 / d.buffer_target.max(1) as f32, weight: 0.4 },
            ScoreTerm {
                label: "front sustainment",
                input: if d.consumer == ConsumerKind::Front { 1.0 } else { 0.0 },
                weight: 0.3 + 0.4 * doctrine.logistics_focus,
            },
            ScoreTerm { label: "source distance km", input: site_distance_km(snap, source, d.site), weight: -0.05 },
        ];
        let total = terms.iter().map(ScoreTerm::value).sum();
        runs.push(ScoredAction {
            subject: format!("run {} to site {}", d.cargo, d.site.0),
            action: LogisticsAction::Run { source, sink: d.site, cargo: d.cargo.clone(), quantity, priority },
            total,
            terms,
        });
    }
    runs.sort_by(|a, b| b.total.total_cmp(&a.total).then_with(|| a.subject.cmp(&b.subject)));
    runs.truncate(MAX_RUNS_PER_DECISION);

    lines_out.extend(runs);
    lines_out
}

/// Mean fill (`on_hand / buffer_target`, 0..1) of `owner`'s front supply points; `None` when the
/// faction has no front demand published yet.
#[must_use]
pub fn front_supply_fill(snap: &LogisticsOrdersSnapshot, owner: OwnerId) -> Option<f32> {
    let mut sum = 0.0_f32;
    let mut n = 0u32;
    for site in snap.sites_of(owner, SiteRole::Front) {
        for d in snap.demands.iter().filter(|d| d.site == site.id) {
            sum += d.fill_ratio().clamp(0.0, 1.0);
            n += 1;
        }
    }
    (n > 0).then(|| sum / n as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::orders::{SiteRow, StockRow};

    fn site(id: u32, owner: u8, role: SiteRole, x: f32, stock: &[(&str, u32)]) -> SiteRow {
        SiteRow {
            id: SiteId(id),
            name: format!("s{id}"),
            owner: OwnerId(owner),
            role,
            world_xz: [x, 0.0],
            tile: [0, 0],
            stock: stock
                .iter()
                .map(|(c, n)| StockRow { cargo: (*c).into(), unit: "t", on_hand: *n, reserved_out: 0, inbound: 0 })
                .collect(),
        }
    }

    fn demand(site: u32, owner: u8, cargo: &str, consumer: ConsumerKind, health: DemandHealth, starved: u64) -> DemandRow {
        DemandRow {
            site: SiteId(site),
            site_name: format!("s{site}"),
            owner: OwnerId(owner),
            cargo: cargo.into(),
            unit: "t",
            consumer,
            on_hand: 0,
            inbound: 0,
            buffer_target: 40,
            reorder_point: 20,
            draw_rate_per_min: 0,
            priority: Priority::Normal,
            health,
            served_by: 0,
            starved_ticks: starved,
            shortfall_total: 0,
            time_to_empty_ticks: None,
            round_trip_ticks: None,
            trucks_needed: None,
        }
    }

    fn snap() -> LogisticsOrdersSnapshot {
        LogisticsOrdersSnapshot {
            sites: vec![
                site(1, 1, SiteRole::Depot, 0.0, &[("ammo", 100), ("Concrete", 100)]),
                site(2, 1, SiteRole::Front, 500.0, &[]),
                site(3, 1, SiteRole::Construction, 100.0, &[]),
                site(4, 2, SiteRole::Front, 900.0, &[]),
            ],
            demands: vec![
                demand(2, 1, "ammo", ConsumerKind::Front, DemandHealth::Starving, 50),
                demand(3, 1, "Concrete", ConsumerKind::Construction, DemandHealth::Low, 0),
                demand(4, 2, "ammo", ConsumerKind::Front, DemandHealth::Starving, 50),
            ],
            ..LogisticsOrdersSnapshot::default()
        }
    }

    #[test]
    fn serves_only_own_unserved_demands_front_first() {
        let plan = plan_logistics(
            &snap(),
            OwnerId(1),
            &FactionDoctrine::default(),
            StrategicGoal::SecureSupply,
            &LogisticsMemory::default(),
            0,
            true,
        );
        let lines: Vec<_> = plan.iter().filter(|a| matches!(a.action, LogisticsAction::Line { .. })).collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].subject.contains("ammo"), "front ammo outranks construction concrete: {lines:?}");
        assert!(plan.iter().all(|a| match &a.action {
            LogisticsAction::Line { sink, .. } | LogisticsAction::Run { sink, .. } => sink.0 != 4,
        }));
        assert!(lines[0].terms.iter().any(|t| t.label == "front sustainment" && t.value() > 0.0));
    }

    #[test]
    fn starving_front_gets_an_urgent_run_once() {
        let mut mem = LogisticsMemory::default();
        let doctrine = FactionDoctrine::default();
        let plan = plan_logistics(&snap(), OwnerId(1), &doctrine, StrategicGoal::Hold, &mem, 0, false);
        let run = plan
            .iter()
            .find_map(|a| match &a.action {
                LogisticsAction::Run { sink, quantity, priority, .. } => Some((*sink, *quantity, *priority)),
                LogisticsAction::Line { .. } => None,
            })
            .expect("an urgent run");
        assert_eq!(run, (SiteId(2), 40, Priority::Critical));
        mem.note(SiteId(2), "ammo", true, 100);
        let again = plan_logistics(&snap(), OwnerId(1), &doctrine, StrategicGoal::Hold, &mem, 10, false);
        assert!(again.iter().all(|a| !matches!(a.action, LogisticsAction::Run { .. })));
    }

    #[test]
    fn auto_supplied_factions_only_get_runs() {
        let plan = plan_logistics(
            &snap(),
            OwnerId(1),
            &FactionDoctrine::default(),
            StrategicGoal::Hold,
            &LogisticsMemory::default(),
            0,
            false,
        );
        assert!(!plan.is_empty());
        assert!(plan.iter().all(|a| matches!(a.action, LogisticsAction::Run { .. })));
    }

    #[test]
    fn memory_expires() {
        let mut mem = LogisticsMemory::default();
        mem.note(SiteId(1), "ammo", false, 50);
        assert!(mem.recently(SiteId(1), "ammo", false, 49));
        assert!(!mem.recently(SiteId(1), "ammo", false, 50));
        mem.prune(50);
        assert!(mem.is_empty());
    }

    #[test]
    fn front_fill_is_mean_demand_fill() {
        let mut s = snap();
        s.demands[0].on_hand = 20;
        let fill = front_supply_fill(&s, OwnerId(1)).expect("front demand");
        assert!((fill - 0.5).abs() < 1e-6, "{fill}");
        assert_eq!(front_supply_fill(&s, OwnerId(9)), None);
    }
}
