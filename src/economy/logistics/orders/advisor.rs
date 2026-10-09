//! Supply advisor — turns unserved draw demands into standing-line suggestions (plan §4, LO-6).
//!
//! Pure function over the snapshot: the AI calls it to issue lines, the player panel offers it
//! as "auto-supply". It only *proposes* commands; callers send them as [`LogisticsOrderRequest`]s.
//!
//! [`LogisticsOrderRequest`]: super::plugin::LogisticsOrderRequest

use super::model::{CargoKind, OwnerId, SiteId};
use super::sim::LogisticsOrderCommand;
use super::snapshot::{DemandRow, LogisticsOrdersSnapshot, SiteRole, SiteRow};

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

fn dist2(a: &SiteRow, b: &SiteRow) -> f32 {
    let dx = a.world_xz[0] - b.world_xz[0];
    let dz = a.world_xz[1] - b.world_xz[1];
    dx * dx + dz * dz
}

/// Best source for a demand: an owner depot/plant other than the sink holding the cargo.
/// Prefers sources that can cover the full buffer target, then the nearest, then the lowest id.
#[must_use]
pub fn best_source_for(snap: &LogisticsOrdersSnapshot, demand: &DemandRow) -> Option<SiteId> {
    let sink = snap.site(demand.site)?;
    snap.sites
        .iter()
        .filter(|s| {
            s.owner == demand.owner
                && s.id != demand.site
                && matches!(s.role, SiteRole::Depot | SiteRole::Plant)
        })
        .filter_map(|s| {
            let available = s
                .stock
                .iter()
                .find(|r| r.cargo == demand.cargo)
                .map(|r| r.on_hand.saturating_sub(r.reserved_out))
                .unwrap_or(0);
            (available > 0).then_some((available < demand.buffer_target, dist2(s, sink), s.id))
        })
        .min_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.total_cmp(&b.1))
                .then(a.2.cmp(&b.2))
        })
        .map(|(_, _, id)| id)
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
