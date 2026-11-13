//! In-situ concrete stock gate for poured fortifications (COD-WALL-CONCRETE-GATE-001 ·
//! COD-TRENCH-MATERIAL-GATE-001).
//!
//! Covers [`DefenseKind::DefensiveWall`], [`DefenseKind::TrenchLine`], and
//! [`DefenseKind::Bunker`]. Manufactured deployables (`DragonTeeth` / `Minefield`) are
//! staging-only — this module is a no-op for them.
//!
//! Reads/debits real industrial buffers (`ResourceType::Concrete` + tag `"Concrete"`) on
//! [`ResourceFlowNode`] — not an abstract `construction_cost` wallet.
//! Preview may show while blocked; commit only via [`CommitConstructionSiteEvent`].

use std::sync::atomic::{AtomicU32, Ordering};

use bevy::prelude::*;

use crate::economy::resource_flow::ResourceFlowNode;
use crate::entities::types::p_enumz::ResourceType;
use crate::strategic::{FootprintTiles, SitePlacementValidation};

use super::build_tool_authority::{BuildTool, DefenseKind};

/// Designer-locked validation token → [`crate::gui::hud::sim_hud_copy::REASON_INSUFFICIENT_CONCRETE`].
pub const INSUFFICIENT_CONCRETE_ERROR: &str = "insufficient_concrete";

/// Portland / mixer output tag (also maps to [`ResourceType::Concrete`]).
pub const CONCRETE_BUFFER_TAG: &str = "Concrete";

static PLACE_BLOCKED_NO_STOCK: AtomicU32 = AtomicU32::new(0);
static PLACE_OK_CONSUME: AtomicU32 = AtomicU32::new(0);





#[inline]
pub(crate) fn note_blocked_no_stock() {
    PLACE_BLOCKED_NO_STOCK.fetch_add(1, Ordering::Relaxed);
}

#[inline]
fn note_ok_consume() {
    PLACE_OK_CONSUME.fetch_add(1, Ordering::Relaxed);
}

/// Poured fortifications that debit concrete on commit (not manufactured deployables).
#[inline]
#[must_use]
pub const fn is_in_situ_concrete_defense(kind: DefenseKind) -> bool {
    matches!(
        kind,
        DefenseKind::DefensiveWall | DefenseKind::TrenchLine | DefenseKind::Bunker
    )
}

/// Concrete units on one industrial node (typed inventory + string buffer tag).
#[must_use]
pub fn concrete_units_on_node(node: &ResourceFlowNode) -> f32 {
    let typed = node
        .inventory
        .get(&ResourceType::Concrete)
        .copied()
        .unwrap_or(0.0);
    let tagged = node
        .buffer_by_tag
        .get(CONCRETE_BUFFER_TAG)
        .copied()
        .unwrap_or(0.0);
    typed + tagged
}

/// Sum pourable concrete across facility flow nodes.
#[must_use]
pub fn total_available_concrete<'a, I>(nodes: I) -> f32
where
    I: IntoIterator<Item = &'a ResourceFlowNode>,
{
    nodes.into_iter().map(concrete_units_on_node).sum()
}

/// Footprint need for an in-situ defense pour (1 unit per occupied tile).
#[must_use]
pub fn in_situ_defense_concrete_need(footprint: FootprintTiles) -> f32 {
    let w = footprint.width.max(1) as f32;
    let d = footprint.depth.max(1) as f32;
    w * d
}

/// Debit `need` from one node (typed inventory first, then tag). Returns amount taken.
fn debit_concrete_from_node(node: &mut ResourceFlowNode, need: f32) -> f32 {
    if need <= f32::EPSILON {
        return 0.0;
    }
    let mut taken = 0.0f32;
    let mut remaining = need;
    let typed = node
        .inventory
        .get(&ResourceType::Concrete)
        .copied()
        .unwrap_or(0.0);
    if typed > f32::EPSILON {
        let take = typed.min(remaining);
        node.inventory
            .insert(ResourceType::Concrete, (typed - take).max(0.0));
        taken += take;
        remaining -= take;
    }
    if remaining > f32::EPSILON {
        let tagged = node
            .buffer_by_tag
            .get(CONCRETE_BUFFER_TAG)
            .copied()
            .unwrap_or(0.0);
        if tagged > f32::EPSILON {
            let take = tagged.min(remaining);
            node.buffer_by_tag
                .insert(CONCRETE_BUFFER_TAG.to_string(), (tagged - take).max(0.0));
            taken += take;
        }
    }
    taken
}



/// Gate `allows_commit` for in-situ defense (wall / trench / bunker) when concrete stock
/// &lt; footprint need. Deployables / non-defense tools: no-op.
pub fn apply_defensive_wall_concrete_gate(
    tool: BuildTool,
    footprint: FootprintTiles,
    available: f32,
    report: &mut SitePlacementValidation,
) {
    let BuildTool::Defense(kind) = tool else {
        return;
    };
    if !is_in_situ_concrete_defense(kind) {
        return;
    }
    let need = in_situ_defense_concrete_need(footprint);
    if available + 1e-4 >= need {
        return;
    }
    report.allows_commit = false;
    if !report
        .errors
        .iter()
        .any(|e| e == INSUFFICIENT_CONCRETE_ERROR)
    {
        report.errors.push(INSUFFICIENT_CONCRETE_ERROR.to_string());
    }
}

/// Consume concrete for a successful in-situ defense commit. Deployables / other tools:
/// Ok(false) no debit.
pub fn consume_concrete_on_wall_commit(
    tool: BuildTool,
    footprint: FootprintTiles,
    nodes: &mut Query<&mut ResourceFlowNode>,
) -> Result<bool, ()> {
    let BuildTool::Defense(kind) = tool else {
        return Ok(false);
    };
    if !is_in_situ_concrete_defense(kind) {
        return Ok(false);
    }
    let need = in_situ_defense_concrete_need(footprint);
    let available: f32 = nodes.iter().map(|n| concrete_units_on_node(n)).sum();
    if available + 1e-4 < need {
        note_blocked_no_stock();
        return Err(());
    }
    let mut remaining = need;
    for mut node in nodes.iter_mut() {
        if remaining <= f32::EPSILON {
            break;
        }
        let took = debit_concrete_from_node(&mut node, remaining);
        remaining -= took;
    }
    if remaining > 1e-3 {
        note_blocked_no_stock();
        return Err(());
    }
    note_ok_consume();
    Ok(true)
}











#[cfg_attr(not(test), allow(dead_code))] // exercised by the concrete-stock unit tests
/// Debit `need` across nodes (typed inventory first, then tag). Returns false if short.
pub fn try_debit_concrete(nodes: &mut [&mut ResourceFlowNode], need: f32) -> bool {
    if need <= f32::EPSILON {
        return true;
    }
    let available: f32 = nodes.iter().map(|n| concrete_units_on_node(n)).sum();
    if available + 1e-4 < need {
        return false;
    }
    let mut remaining = need;
    for node in nodes.iter_mut() {
        if remaining <= f32::EPSILON {
            break;
        }
        let took = debit_concrete_from_node(node, remaining);
        remaining -= took;
    }
    remaining <= 1e-3
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::FootprintTiles;

    #[test]
    fn zero_concrete_blocks_in_situ_not_deployables() {
        let fp = FootprintTiles { width: 4, depth: 1 };
        let mut wall = SitePlacementValidation {
            valid: true,
            allows_commit: true,
            ..Default::default()
        };
        apply_defensive_wall_concrete_gate(
            BuildTool::Defense(DefenseKind::DefensiveWall),
            fp,
            0.0,
            &mut wall,
        );
        assert!(!wall.allows_commit);
        assert!(wall.errors.iter().any(|e| e == INSUFFICIENT_CONCRETE_ERROR));

        let mut trench = SitePlacementValidation {
            valid: true,
            allows_commit: true,
            ..Default::default()
        };
        apply_defensive_wall_concrete_gate(
            BuildTool::Defense(DefenseKind::TrenchLine),
            FootprintTiles { width: 6, depth: 1 },
            0.0,
            &mut trench,
        );
        assert!(!trench.allows_commit);
        assert!(trench
            .errors
            .iter()
            .any(|e| e == INSUFFICIENT_CONCRETE_ERROR));

        let mut bunker = SitePlacementValidation {
            valid: true,
            allows_commit: true,
            ..Default::default()
        };
        apply_defensive_wall_concrete_gate(
            BuildTool::Defense(DefenseKind::Bunker),
            FootprintTiles { width: 2, depth: 2 },
            0.0,
            &mut bunker,
        );
        assert!(!bunker.allows_commit);
        assert!(bunker
            .errors
            .iter()
            .any(|e| e == INSUFFICIENT_CONCRETE_ERROR));

        let mut teeth = SitePlacementValidation {
            valid: true,
            allows_commit: true,
            ..Default::default()
        };
        apply_defensive_wall_concrete_gate(
            BuildTool::Defense(DefenseKind::DragonTeeth),
            DefenseKind::DragonTeeth.footprint(),
            0.0,
            &mut teeth,
        );
        assert!(teeth.allows_commit);
        assert!(teeth.errors.is_empty());
    }

    #[test]
    fn debit_consumes_typed_concrete() {
        let need = 4.0;
        let mut node = ResourceFlowNode {
            catalog_id: "mixer".into(),
            inventory: std::collections::HashMap::from([(ResourceType::Concrete, 10.0)]),
            buffer_by_tag: std::collections::HashMap::new(),
            throughput_limit: 10.0,
            production: vec![],
            consumption: vec![],
        };
        assert!(try_debit_concrete(&mut [&mut node], need));
        assert!((concrete_units_on_node(&node) - 6.0).abs() < 1e-3);
    }




}
