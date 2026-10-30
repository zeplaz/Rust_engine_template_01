//! **Faction AI orders** — what a faction asks for, and what the player-facing API answered.
//!
//! An [`AiOrder`] is API-agnostic intent. Adapters in [`super`] translate it into the same enqueue
//! call the player UI makes: `DeploymentOrderQueue::enqueue` (front), `LogisticsOrderRequest`
//! (logistics) and `CommitConstructionSiteEvent` (construction funnel). The first four variants are
//! the interim strategic-queue orders used only when the real API is not in the app.
//! Validation belongs to the API.

use bevy::prelude::{Entity, IVec2};
use serde::{Serialize, Serializer};

use super::decide::StrategicGoal;
use super::perception::FactionPerception;

/// Upper bound on orders a faction issues in one decision.
pub const MAX_ORDERS_PER_DECISION: usize = 3;

/// A formation entity, serialized as its bit pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormationRef(pub Entity);

impl Serialize for FormationRef {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(self.0.to_bits())
    }
}

/// How a deployment engages its point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum DeployKind {
    /// Move to a threatened point of our own (`DeploymentKind::Reinforce`).
    Reinforce,
    /// Push through an opportunity point (`DeploymentKind::Attack`).
    Attack,
    /// Dig in where a threatened point is held (`DeploymentKind::Fortify`).
    Fortify,
}

/// Why the economy brain asks for a building.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum BuildReason {
    /// Nothing of ours holds `cargo` that a consumer is short of; build a producer.
    InputShort { cargo: String },
    /// First productive building for a faction with none.
    Bootstrap,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum AiOrder {
    /// Send one formation to a pressure point (`DeploymentOrderQueue`).
    Deploy {
        formation: FormationRef,
        kind: DeployKind,
        theater: u32,
        tile: IVec2,
    },
    /// Place a catalog building through the construction commit funnel.
    Build {
        catalog_id: String,
        origin: IVec2,
        footprint: (u32, u32),
        reason: BuildReason,
    },
    /// Keep a sink's demand supplied from a source (`CreateStandingLine`).
    StandingLine {
        source: u32,
        sink: u32,
        cargo: String,
        priority: &'static str,
    },
    /// Urgent one-off haul for a starving sink (`CreateOneOffRun`).
    SupplyRun {
        source: u32,
        sink: u32,
        cargo: String,
        quantity: u32,
        priority: &'static str,
    },
    /// Register the faction's first depot (a logistics endpoint, like a front supply point).
    FoundDepot { tile: IVec2 },
    /// Reinforce a threatened front cell against `enemy`.
    Reinforce { sector: IVec2, tile: IVec2, enemy: u8 },
    /// Push into a front cell where this faction leads.
    Attack { sector: IVec2, tile: IVec2, enemy: u8 },
    /// Keep a front sector supplied from the rear anchor.
    SupplyLine { from: IVec2, to: IVec2 },
    /// Grow the economy at a calm held tile.
    Develop { tile: IVec2 },
}

impl AiOrder {
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Deploy { formation, kind, tile, .. } => format!(
                "{} formation {} to ({}, {})",
                match kind {
                    DeployKind::Reinforce => "reinforce with",
                    DeployKind::Attack => "attack with",
                    DeployKind::Fortify => "dig in with",
                },
                formation.0.index(),
                tile.x,
                tile.y
            ),
            Self::Build { catalog_id, origin, reason, .. } => format!(
                "build {catalog_id} at ({}, {}) [{}]",
                origin.x,
                origin.y,
                match reason {
                    BuildReason::InputShort { cargo } => format!("short of {cargo}"),
                    BuildReason::Bootstrap => "bootstrap".to_string(),
                }
            ),
            Self::StandingLine { source, sink, cargo, priority } => {
                format!("standing line {cargo} site {source} -> site {sink} ({priority})")
            }
            Self::SupplyRun { source, sink, cargo, quantity, priority } => {
                format!("run {quantity} {cargo} site {source} -> site {sink} ({priority})")
            }
            Self::FoundDepot { tile } => format!("found depot at ({}, {})", tile.x, tile.y),
            Self::Reinforce { tile, enemy, .. } => {
                format!("reinforce ({}, {}) vs faction {enemy}", tile.x, tile.y)
            }
            Self::Attack { tile, enemy, .. } => {
                format!("attack ({}, {}) vs faction {enemy}", tile.x, tile.y)
            }
            Self::SupplyLine { from, to } => {
                format!("supply line ({}, {}) → ({}, {})", from.x, from.y, to.x, to.y)
            }
            Self::Develop { tile } => format!("develop at ({}, {})", tile.x, tile.y),
        }
    }

    /// Mission wire string the corridor queue understands (`MissionKind::as_wire_str`).
    #[must_use]
    pub fn corridor_mission(&self) -> &'static str {
        match self {
            Self::Reinforce { .. } | Self::SupplyLine { .. } => {
                crate::strategic::MissionKind::SecureCorridor.as_wire_str()
            }
            _ => crate::strategic::MissionKind::MoveCorridor.as_wire_str(),
        }
    }
}

/// What the API did with an order.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub enum OrderOutcome {
    /// Accepted by the order's own API.
    Issued { api: &'static str, id: u64 },
    /// Handed to an event / message API that returns no id (construction commit, logistics request).
    Submitted { api: &'static str },
    /// The order's own API is not in this app; sent through an interim queue the player also uses.
    Fallback { api: &'static str, id: u64 },
    /// The API rejected it (reason from the API).
    Refused { reason: String },
    /// Not issued this decision.
    Deferred { reason: &'static str },
}

impl OrderOutcome {
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Issued { api, id } => format!("issued via {api} #{id}"),
            Self::Submitted { api } => format!("submitted via {api}"),
            Self::Fallback { api, id } => format!("queued via {api} #{id} (interim)"),
            Self::Refused { reason } => format!("refused: {reason}"),
            Self::Deferred { reason } => format!("deferred: {reason}"),
        }
    }

    #[must_use]
    pub fn reached_api(&self) -> bool {
        matches!(self, Self::Issued { .. } | Self::Submitted { .. } | Self::Fallback { .. })
    }
}

/// Turn a chosen goal into concrete orders. Pure and deterministic (targets come from the
/// perception's sorted helpers).
#[must_use]
pub fn plan_orders(goal: StrategicGoal, p: &FactionPerception) -> Vec<AiOrder> {
    let mut out = Vec::new();
    match goal {
        StrategicGoal::DefendFront => {
            if let Some(c) = p.most_threatened_front() {
                out.push(AiOrder::Reinforce {
                    sector: c.chunk,
                    tile: c.tile,
                    enemy: c.enemy_slot,
                });
            }
            // Defence needs supply behind it when the front is starved.
            if p.front_supply_shortfall() > 0.5 {
                push_supply(&mut out, p);
            }
        }
        StrategicGoal::Advance => {
            if let Some(c) = p.best_push_front() {
                out.push(AiOrder::Attack {
                    sector: c.chunk,
                    tile: c.tile,
                    enemy: c.enemy_slot,
                });
            }
        }
        StrategicGoal::SecureSupply => push_supply(&mut out, p),
        StrategicGoal::Develop => {
            if let Some(tile) = p.rear_anchor {
                out.push(AiOrder::Develop { tile });
            }
        }
        StrategicGoal::Hold => {}
    }
    out.truncate(MAX_ORDERS_PER_DECISION);
    out
}

fn push_supply(out: &mut Vec<AiOrder>, p: &FactionPerception) {
    if let (Some(from), Some(c)) = (p.rear_anchor, p.worst_supplied_front()) {
        out.push(AiOrder::SupplyLine { from, to: c.tile });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::faction::perception::{perceive_faction, test_support::two_faction_strip};

    #[test]
    fn fai_defend_targets_most_threatened_cell() {
        let mut ov = two_faction_strip();
        ov.threat[2][0] = 0.9;
        let p = perceive_faction(0, [&ov]);
        let orders = plan_orders(StrategicGoal::DefendFront, &p);
        assert_eq!(
            orders.first(),
            Some(&AiOrder::Reinforce {
                sector: IVec2::ZERO,
                tile: IVec2::new(2, 0),
                enemy: 1
            })
        );
        // Both front cells have zero logistics → supply line rides along.
        assert!(orders.iter().any(|o| matches!(o, AiOrder::SupplyLine { .. })));
    }

    #[test]
    fn fai_supply_runs_rear_to_front() {
        let ov = two_faction_strip();
        let p = perceive_faction(1, [&ov]);
        let orders = plan_orders(StrategicGoal::SecureSupply, &p);
        assert_eq!(
            orders,
            vec![AiOrder::SupplyLine {
                from: IVec2::new(3, 0),
                to: IVec2::new(1, 0)
            }]
        );
    }

    #[test]
    fn fai_hold_issues_nothing() {
        let ov = two_faction_strip();
        let p = perceive_faction(0, [&ov]);
        assert!(plan_orders(StrategicGoal::Hold, &p).is_empty());
    }
}
