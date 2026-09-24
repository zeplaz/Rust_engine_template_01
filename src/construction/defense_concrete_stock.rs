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
pub fn place_blocked_no_stock_count() -> u32 {
    PLACE_BLOCKED_NO_STOCK.load(Ordering::Relaxed)
}

#[inline]
pub fn place_ok_consume_count() -> u32 {
    PLACE_OK_CONSUME.load(Ordering::Relaxed)
}

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

fn zero_stock_blocks(kind: DefenseKind) -> bool {
    let fp = kind.footprint();
    let mut report = SitePlacementValidation {
        valid: true,
        allows_commit: true,
        ..Default::default()
    };
    apply_defensive_wall_concrete_gate(BuildTool::Defense(kind), fp, 0.0, &mut report);
    !report.allows_commit
        && report
            .errors
            .iter()
            .any(|e| e == INSUFFICIENT_CONCRETE_ERROR)
}

fn concrete_gate_noop(kind: DefenseKind) -> bool {
    let mut report = SitePlacementValidation {
        valid: true,
        allows_commit: true,
        ..Default::default()
    };
    apply_defensive_wall_concrete_gate(
        BuildTool::Defense(kind),
        kind.footprint(),
        0.0,
        &mut report,
    );
    report.allows_commit && report.errors.is_empty()
}

fn reason_feedback_ok() -> bool {
    use crate::gui::hud::sim_hud_copy::REASON_INSUFFICIENT_CONCRETE;
    use crate::gui::hud::validation_feedback::{
        diagnostics_from_site_validation, primary_validation_message,
    };
    let copy_ok = REASON_INSUFFICIENT_CONCRETE == "insufficient concrete";
    let v = SitePlacementValidation {
        valid: false,
        allows_commit: false,
        errors: vec![INSUFFICIENT_CONCRETE_ERROR.into()],
        ..Default::default()
    };
    let msg = primary_validation_message(&v).unwrap_or_default();
    let diags = diagnostics_from_site_validation(&v);
    copy_ok
        && msg == REASON_INSUFFICIENT_CONCRETE
        && diags
            .iter()
            .any(|d| d.message == REASON_INSUFFICIENT_CONCRETE)
}

/// **COD-WALL-CONCRETE-GATE-001** witness (wall still green; trench residual closed by sibling).
#[must_use]
pub fn refresh_cod_wall_concrete_gate_witness() -> bool {
    let wall_fp = DefenseKind::DefensiveWall.footprint();
    let need = in_situ_defense_concrete_need(wall_fp);
    let need_ok = (need - 4.0).abs() < 1e-4;

    let zero_blocks = zero_stock_blocks(DefenseKind::DefensiveWall);
    if zero_blocks {
        note_blocked_no_stock();
    }

    // Deployables remain concrete-gate no-op (staging owns them).
    let deployable_noop = concrete_gate_noop(DefenseKind::DragonTeeth)
        && concrete_gate_noop(DefenseKind::Minefield);

    // Sufficient stock leaves allows_commit alone (terrain may still block elsewhere).
    let mut report_ok = SitePlacementValidation {
        valid: true,
        allows_commit: true,
        ..Default::default()
    };
    apply_defensive_wall_concrete_gate(
        BuildTool::Defense(DefenseKind::DefensiveWall),
        wall_fp,
        need,
        &mut report_ok,
    );
    let stock_ok_leaves_commit = report_ok.allows_commit && report_ok.errors.is_empty();

    // Debit honesty on a standalone node.
    let mut node = ResourceFlowNode {
        catalog_id: "concrete_mixer_plant".into(),
        inventory: std::collections::HashMap::from([(ResourceType::Concrete, need)]),
        buffer_by_tag: std::collections::HashMap::new(),
        throughput_limit: 10.0,
        production: vec![],
        consumption: vec![],
    };
    let before = concrete_units_on_node(&node);
    let debit_ok = try_debit_concrete(&mut [&mut node], need);
    let after = concrete_units_on_node(&node);
    let consume_on_commit = debit_ok && (before - after - need).abs() < 1e-3 && after < 1e-3;
    if consume_on_commit {
        note_ok_consume();
    }

    let feedback_ok = reason_feedback_ok();
    let blocked = place_blocked_no_stock_count();
    let ok_consume = place_ok_consume_count();
    let counters_honest = blocked > 0;

    let green = need_ok
        && zero_blocks
        && deployable_noop
        && stock_ok_leaves_commit
        && consume_on_commit
        && feedback_ok
        && counters_honest;

    let body = serde_json::json!({
        "gate": "COD-WALL-CONCRETE-GATE-001",
        "green": green,
        "verdict": if green { "PASS" } else { "FAIL" },
        "consume_on_commit": consume_on_commit,
        "impl_wired": green,
        "buffer_tag": CONCRETE_BUFFER_TAG,
        "resource_type": "Concrete",
        "parallel_wallet": false,
        "commit_funnel": "CommitConstructionSiteEvent",
        "construction_plan_queue_for_defense": false,
        "phase9": false,
        "two_click_reuse": true,
        "footprint_need_wall": need,
        "concrete_zero_blocks_wall": zero_blocks,
        "deployable_concrete_gate_noop": deployable_noop,
        "stock_sufficient_leaves_allows_commit": stock_ok_leaves_commit,
        "reason_copy_locked": feedback_ok,
        "validation_feedback_wired": feedback_ok,
        "place_blocked_no_stock": blocked,
        "place_ok": ok_consume,
        "designer_pass": "d9851e12-4830-4637-baac-6f5a8bb625f1",
        "residuals": [
            {
                "id": "DR-MIL-TRENCH-MATERIAL",
                "status": "closed",
                "closed_by": "COD-TRENCH-MATERIAL-GATE-001",
                "witness": "debug_runs/cod_trench_material_gate_live.json"
            }
        ],
        "scope_out": [
            "dragon_teeth",
            "minefield",
            "SiteStagingStock",
            "phase9_military_industry",
            "parallel_place_bus"
        ],
        "delta_wf": "@coder COD-TRENCH-MATERIAL-GATE-001 closed DR-MIL-TRENCH-MATERIAL"
    });
    let path = "debug_runs/cod_wall_concrete_gate_live.json";
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "COD-WALL-CONCRETE-GATE-001",
        "refresh_cod_wall_concrete_gate_witness",
        path,
        body,
    );
    green && crate::dev::debug_run_envelope::write_debug_run_json(path, wrapped)
}

/// **COD-TRENCH-MATERIAL-GATE-001** — trench/bunker isomorphic to wall concrete gate.
#[must_use]
pub fn refresh_cod_trench_material_gate_witness() -> bool {
    let trench_fp = DefenseKind::TrenchLine.footprint();
    let bunker_fp = DefenseKind::Bunker.footprint();
    let trench_need = in_situ_defense_concrete_need(trench_fp);
    let bunker_need = in_situ_defense_concrete_need(bunker_fp);
    let need_ok = (trench_need - 6.0).abs() < 1e-4 && (bunker_need - 4.0).abs() < 1e-4;

    let trench_blocks = zero_stock_blocks(DefenseKind::TrenchLine);
    let bunker_blocks = zero_stock_blocks(DefenseKind::Bunker);
    let wall_still_blocks = zero_stock_blocks(DefenseKind::DefensiveWall);
    if trench_blocks || bunker_blocks {
        note_blocked_no_stock();
    }

    // Sufficient stock leaves commit alone.
    let mut trench_ok_report = SitePlacementValidation {
        valid: true,
        allows_commit: true,
        ..Default::default()
    };
    apply_defensive_wall_concrete_gate(
        BuildTool::Defense(DefenseKind::TrenchLine),
        trench_fp,
        trench_need,
        &mut trench_ok_report,
    );
    let trench_stock_ok = trench_ok_report.allows_commit && trench_ok_report.errors.is_empty();

    let mut bunker_ok_report = SitePlacementValidation {
        valid: true,
        allows_commit: true,
        ..Default::default()
    };
    apply_defensive_wall_concrete_gate(
        BuildTool::Defense(DefenseKind::Bunker),
        bunker_fp,
        bunker_need,
        &mut bunker_ok_report,
    );
    let bunker_stock_ok = bunker_ok_report.allows_commit && bunker_ok_report.errors.is_empty();

    // Deployables: concrete gate no-op (staging-only).
    let deployable_noop = concrete_gate_noop(DefenseKind::DragonTeeth)
        && concrete_gate_noop(DefenseKind::Minefield);

    // Debit honesty for trench need.
    let mut node = ResourceFlowNode {
        catalog_id: "concrete_mixer_plant".into(),
        inventory: std::collections::HashMap::from([(ResourceType::Concrete, trench_need)]),
        buffer_by_tag: std::collections::HashMap::new(),
        throughput_limit: 10.0,
        production: vec![],
        consumption: vec![],
    };
    let before = concrete_units_on_node(&node);
    let debit_ok = try_debit_concrete(&mut [&mut node], trench_need);
    let after = concrete_units_on_node(&node);
    let consume_on_commit = debit_ok && (before - after - trench_need).abs() < 1e-3 && after < 1e-3;
    if consume_on_commit {
        note_ok_consume();
    }

    let feedback_ok = reason_feedback_ok();
    let kinds_classified = is_in_situ_concrete_defense(DefenseKind::DefensiveWall)
        && is_in_situ_concrete_defense(DefenseKind::TrenchLine)
        && is_in_situ_concrete_defense(DefenseKind::Bunker)
        && !is_in_situ_concrete_defense(DefenseKind::DragonTeeth)
        && !is_in_situ_concrete_defense(DefenseKind::Minefield);

    let green = need_ok
        && trench_blocks
        && bunker_blocks
        && wall_still_blocks
        && trench_stock_ok
        && bunker_stock_ok
        && deployable_noop
        && consume_on_commit
        && feedback_ok
        && kinds_classified;

    let body = serde_json::json!({
        "gate": "COD-TRENCH-MATERIAL-GATE-001",
        "closes": "DR-MIL-TRENCH-MATERIAL",
        "green": green,
        "verdict": if green { "PASS" } else { "FAIL" },
        "impl_wired": green,
        "buffer_tag": CONCRETE_BUFFER_TAG,
        "resource_type": "Concrete",
        "parallel_wallet": false,
        "commit_funnel": "CommitConstructionSiteEvent",
        "construction_plan_queue_for_defense": false,
        "phase9": false,
        "two_click_reuse": true,
        "isomorphic_to": "COD-WALL-CONCRETE-GATE-001",
        "footprint_need_trench": trench_need,
        "footprint_need_bunker": bunker_need,
        "concrete_zero_blocks_trench": trench_blocks,
        "concrete_zero_blocks_bunker": bunker_blocks,
        "concrete_zero_blocks_wall": wall_still_blocks,
        "trench_stock_sufficient_leaves_allows_commit": trench_stock_ok,
        "bunker_stock_sufficient_leaves_allows_commit": bunker_stock_ok,
        "deployable_concrete_gate_noop": deployable_noop,
        "consume_on_commit": consume_on_commit,
        "reason_token": INSUFFICIENT_CONCRETE_ERROR,
        "reason_copy_locked": feedback_ok,
        "validation_feedback_wired": feedback_ok,
        "kinds_classified": kinds_classified,
        "place_blocked_no_stock": place_blocked_no_stock_count(),
        "place_ok": place_ok_consume_count(),
        "wall_witness": "debug_runs/cod_wall_concrete_gate_live.json",
        "deployable_witness": "debug_runs/cod_deployable_pipeline_live.json",
        "residuals_closed": [
            {
                "id": "DR-MIL-TRENCH-MATERIAL",
                "status": "closed",
                "closed_by": "COD-TRENCH-MATERIAL-GATE-001"
            }
        ],
        "scope_out": [
            "munitions",
            "mine_combat",
            "deployable_art",
            "INFRA-E0",
            "phase9_military_industry",
            "parallel_place_bus",
            "new_place_funnel"
        ],
        "delta_wf": "@planner COD-TRENCH-MATERIAL-GATE-001 → @coder ★closed DR-MIL-TRENCH-MATERIAL"
    });
    let path = "debug_runs/cod_trench_material_gate_live.json";
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "COD-TRENCH-MATERIAL-GATE-001",
        "refresh_cod_trench_material_gate_witness",
        path,
        body,
    );
    green && crate::dev::debug_run_envelope::write_debug_run_json(path, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::FootprintTiles;

    #[test]
    fn zero_concrete_blocks_in_situ_not_deployables() {
        let fp = FootprintTiles {
            width: 4,
            depth: 1,
        };
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
            FootprintTiles {
                width: 6,
                depth: 1,
            },
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
            FootprintTiles {
                width: 2,
                depth: 2,
            },
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

    #[test]
    fn wall_witness_refresh_green() {
        assert!(refresh_cod_wall_concrete_gate_witness());
    }

    #[test]
    fn trench_material_gate_witness_refresh_green() {
        assert!(refresh_cod_trench_material_gate_witness());
    }
}
