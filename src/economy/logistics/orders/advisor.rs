//! Supply advisor — turns unserved draw demands into standing-line suggestions (plan §4, LO-6).
//!
//! Pure function over the snapshot: the AI calls it to issue lines, the player panel offers it
//! as "auto-supply". It only *proposes* commands; callers send them as [`LogisticsOrderRequest`]s.
//!
//! [`LogisticsOrderRequest`]: super::plugin::LogisticsOrderRequest

use super::model::{CargoKind, OwnerId, SiteId};
use super::sim::LogisticsOrderCommand;
use super::snapshot::{DemandRow, LogisticsOrdersSnapshot};
use super::source_pick::best_source_open_goal;

/// Why a demand got no suggestion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdvisorGap {
    /// No depot or plant of this owner holds the cargo.
    NoSourceWithStock { site: SiteId, cargo: String },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SupplyAdvice {
    pub commands: Vec<LogisticsOrderCommand>,
    pub gaps: Vec<AdvisorGap>,
}

/// Best source for a demand: an owner depot/plant other than the sink holding the cargo.
/// One open-goal search from the sink (NAV-09): full buffer cover, then travel ticks, then id.
#[must_use]
pub fn best_source_for(snap: &LogisticsOrdersSnapshot, demand: &DemandRow) -> Option<SiteId> {
    best_source_open_goal(snap, demand)
}

/// Standing lines for every unserved demand of `owner`, most urgent first.
#[must_use]
pub fn suggest_lines_for_unserved(snap: &LogisticsOrdersSnapshot, owner: OwnerId) -> SupplyAdvice {
    let mut demands: Vec<&DemandRow> = snap.unserved_demands(owner).collect();
    // Starving before low before satisfied; then priority; then site id (deterministic).
    demands.sort_by(|a, b| {
        b.health
            .severity()
            .cmp(&a.health.severity())
            .then(b.priority.cmp(&a.priority))
            .then(a.site.cmp(&b.site))
            .then(a.cargo.cmp(&b.cargo))
    });
    let mut advice = SupplyAdvice::default();
    for d in demands {
        match best_source_for(snap, d) {
            Some(source) => advice.commands.push(LogisticsOrderCommand::CreateStandingLine {
                owner,
                source,
                sink: d.site,
                cargo: CargoKind::new(d.cargo.clone()),
                priority: d.priority,
                vehicle_cap: None,
            }),
            None => advice.gaps.push(AdvisorGap::NoSourceWithStock {
                site: d.site,
                cargo: d.cargo.clone(),
            }),
        }
    }
    advice
}
