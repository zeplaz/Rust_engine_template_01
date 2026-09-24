//! Manufactured deployable staging gate (**COD-DEPLOYABLE-PLACE-001**).
//!
//! `allows_commit` reads [`SiteStagingStock`] (+ pending reservations + in-transit).
//! Successful commits only **enqueue** [`PendingSiteStagingDebits`] — debit via
//! `try_debit` runs in `LogisticsSimulationSet::FreightDispatch` (never UI).

use std::sync::atomic::{AtomicU32, Ordering};

use bevy::prelude::*;

use crate::economy::logistics::{
    InTransitLedger, PendingSiteStagingDebits, SiteStagingStock,
};
use crate::entities::production::core::{
    BUFFER_TAG_DRAGON_TEETH_UNIT, BUFFER_TAG_MINE_UNIT,
};
use crate::strategic::{FootprintTiles, SiteArchetype, SitePlacementValidation};

use super::build_tool_authority::{BuildTool, DefenseKind};

/// Designer-locked validation token → [`crate::gui::hud::sim_hud_copy::REASON_INSUFFICIENT_STAGED`].
pub const INSUFFICIENT_STAGED_ERROR: &str = "insufficient_staged_stock";
/// Designer-locked validation token → [`crate::gui::hud::sim_hud_copy::REASON_STOCK_IN_TRANSIT`].
pub const STOCK_IN_TRANSIT_ERROR: &str = "stock_still_in_transit";

static PLACE_BLOCKED_NO_STOCK: AtomicU32 = AtomicU32::new(0);
static PLACE_OK_DEBIT: AtomicU32 = AtomicU32::new(0);
static AI_COMMIT_GATED: AtomicU32 = AtomicU32::new(0);

#[inline]
pub fn place_blocked_no_stock_count() -> u32 {
    PLACE_BLOCKED_NO_STOCK.load(Ordering::Relaxed)
}

#[inline]
pub fn place_ok_debit_count() -> u32 {
    PLACE_OK_DEBIT.load(Ordering::Relaxed)
}

#[inline]
pub fn ai_commit_gated_count() -> u32 {
    AI_COMMIT_GATED.load(Ordering::Relaxed)
}

#[inline]
pub(crate) fn note_blocked_no_stock() {
    PLACE_BLOCKED_NO_STOCK.fetch_add(1, Ordering::Relaxed);
}

#[inline]
fn note_ok_debit() {
    PLACE_OK_DEBIT.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn note_ai_commit_gated() {
    AI_COMMIT_GATED.fetch_add(1, Ordering::Relaxed);
}

/// Buffer tag + footprint need for a manufactured deployable, if any.
#[must_use]
pub fn deployable_staging_spec(
    kind: DefenseKind,
    footprint: FootprintTiles,
) -> Option<(&'static str, f32)> {
    match kind {
        // Heavy: one prefab lot covers the strip.
        DefenseKind::DragonTeeth => Some((BUFFER_TAG_DRAGON_TEETH_UNIT, 1.0)),
        // Light: one unit per three tiles (3×3 field → 3 units = one manufacture tick).
        DefenseKind::Minefield => {
            let tiles = (footprint.width.max(1) * footprint.depth.max(1)) as f32;
            Some((BUFFER_TAG_MINE_UNIT, (tiles / 3.0).max(1.0)))
        }
        _ => None,
    }
}

#[must_use]
pub fn deployable_staging_spec_for_archetype(
    archetype: SiteArchetype,
    footprint: FootprintTiles,
) -> Option<(&'static str, f32)> {
    let kind = match archetype {
        SiteArchetype::DragonTeeth => DefenseKind::DragonTeeth,
        SiteArchetype::Minefield => DefenseKind::Minefield,
        _ => return None,
    };
    deployable_staging_spec(kind, footprint)
}

#[must_use]
pub fn available_staged(
    staging: &SiteStagingStock,
    pending: &PendingSiteStagingDebits,
    tag: &str,
) -> f32 {
    (staging.get(tag) - pending.reserved(tag)).max(0.0)
}

#[must_use]
pub fn in_transit_amount(ledger: &InTransitLedger, tag: &str) -> f32 {
    ledger
        .lots
        .iter()
        .filter(|l| l.buffer_tag == tag)
        .map(|l| l.amount)
        .sum()
}

/// Gate `allows_commit` for DragonTeeth / Minefield when staged stock &lt; need.
/// Wall / trench / bunker / non-defense: no-op.
pub fn apply_deployable_staging_gate(
    tool: BuildTool,
    footprint: FootprintTiles,
    staging: &SiteStagingStock,
    pending: &PendingSiteStagingDebits,
    ledger: &InTransitLedger,
    report: &mut SitePlacementValidation,
) {
    let BuildTool::Defense(kind) = tool else {
        return;
    };
    let Some((tag, need)) = deployable_staging_spec(kind, footprint) else {
        return;
    };
    let avail = available_staged(staging, pending, tag);
    if avail + 1e-4 >= need {
        return;
    }
    report.allows_commit = false;
    let transit = in_transit_amount(ledger, tag);
    let err = if transit > 1e-4 {
        STOCK_IN_TRANSIT_ERROR
    } else {
        INSUFFICIENT_STAGED_ERROR
    };
    if !report.errors.iter().any(|e| e == err) {
        report.errors.push(err.to_string());
    }
}

/// Same gate for AI / archetype-driven place (shared commit funnel).
pub fn apply_deployable_staging_gate_archetype(
    archetype: SiteArchetype,
    footprint: FootprintTiles,
    staging: &SiteStagingStock,
    pending: &PendingSiteStagingDebits,
    ledger: &InTransitLedger,
    report: &mut SitePlacementValidation,
) {
    let Some((tag, need)) = deployable_staging_spec_for_archetype(archetype, footprint) else {
        return;
    };
    let avail = available_staged(staging, pending, tag);
    if avail + 1e-4 >= need {
        return;
    }
    report.allows_commit = false;
    let transit = in_transit_amount(ledger, tag);
    let err = if transit > 1e-4 {
        STOCK_IN_TRANSIT_ERROR
    } else {
        INSUFFICIENT_STAGED_ERROR
    };
    if !report.errors.iter().any(|e| e == err) {
        report.errors.push(err.to_string());
    }
}

/// Enqueue FreightDispatch debit for a successful deployable commit. Other tools: Ok(false).
pub fn enqueue_deployable_staging_debit_on_commit(
    tool: BuildTool,
    footprint: FootprintTiles,
    staging: &SiteStagingStock,
    pending: &mut PendingSiteStagingDebits,
) -> Result<bool, ()> {
    let BuildTool::Defense(kind) = tool else {
        return Ok(false);
    };
    let Some((tag, need)) = deployable_staging_spec(kind, footprint) else {
        return Ok(false);
    };
    let avail = available_staged(staging, pending, tag);
    if avail + 1e-4 < need {
        note_blocked_no_stock();
        return Err(());
    }
    pending.enqueue(tag, need);
    note_ok_debit();
    Ok(true)
}

/// AI / archetype path — enqueue debit before [`CommitConstructionSiteEvent`].
pub fn enqueue_deployable_staging_debit_for_archetype(
    archetype: SiteArchetype,
    footprint: FootprintTiles,
    staging: &SiteStagingStock,
    pending: &mut PendingSiteStagingDebits,
) -> Result<bool, ()> {
    let Some((tag, need)) = deployable_staging_spec_for_archetype(archetype, footprint) else {
        return Ok(false);
    };
    let avail = available_staged(staging, pending, tag);
    if avail + 1e-4 < need {
        note_blocked_no_stock();
        note_ai_commit_gated();
        return Err(());
    }
    pending.enqueue(tag, need);
    note_ok_debit();
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::apply_pending_site_staging_debits_system;

    #[test]
    fn dragon_teeth_need_one_heavy_unit() {
        let fp = DefenseKind::DragonTeeth.footprint();
        let (tag, need) = deployable_staging_spec(DefenseKind::DragonTeeth, fp).unwrap();
        assert_eq!(tag, BUFFER_TAG_DRAGON_TEETH_UNIT);
        assert!((need - 1.0).abs() < 1e-4);
    }

    #[test]
    fn minefield_need_three_light_units() {
        let fp = DefenseKind::Minefield.footprint();
        let (tag, need) = deployable_staging_spec(DefenseKind::Minefield, fp).unwrap();
        assert_eq!(tag, BUFFER_TAG_MINE_UNIT);
        assert!((need - 3.0).abs() < 1e-4);
    }

    #[test]
    fn gate_blocks_without_stock_and_enqueues_debit_when_ready() {
        let fp = DefenseKind::DragonTeeth.footprint();
        let mut staging = SiteStagingStock::default();
        let mut pending = PendingSiteStagingDebits::default();
        let ledger = InTransitLedger::default();
        let mut report = SitePlacementValidation {
            allows_commit: true,
            ..Default::default()
        };
        apply_deployable_staging_gate(
            BuildTool::Defense(DefenseKind::DragonTeeth),
            fp,
            &staging,
            &pending,
            &ledger,
            &mut report,
        );
        assert!(!report.allows_commit);
        assert!(report.errors.iter().any(|e| e == INSUFFICIENT_STAGED_ERROR));

        staging.credit(BUFFER_TAG_DRAGON_TEETH_UNIT, 1.0);
        report.allows_commit = true;
        report.errors.clear();
        apply_deployable_staging_gate(
            BuildTool::Defense(DefenseKind::DragonTeeth),
            fp,
            &staging,
            &pending,
            &ledger,
            &mut report,
        );
        assert!(report.allows_commit);

        assert!(enqueue_deployable_staging_debit_on_commit(
            BuildTool::Defense(DefenseKind::DragonTeeth),
            fp,
            &staging,
            &mut pending,
        )
        .unwrap());
        assert_eq!(pending.queue.len(), 1);

        let mut app = App::new();
        app.insert_resource(staging)
            .insert_resource(pending)
            .add_systems(Update, apply_pending_site_staging_debits_system);
        app.update();
        assert!(
            app.world()
                .resource::<SiteStagingStock>()
                .get(BUFFER_TAG_DRAGON_TEETH_UNIT)
                < 1e-4
        );
        assert!(app
            .world()
            .resource::<PendingSiteStagingDebits>()
            .queue
            .is_empty());
    }

    #[test]
    fn wall_trench_bunker_untouched_by_staging_gate() {
        let staging = SiteStagingStock::default();
        let pending = PendingSiteStagingDebits::default();
        let ledger = InTransitLedger::default();
        for kind in [
            DefenseKind::DefensiveWall,
            DefenseKind::TrenchLine,
            DefenseKind::Bunker,
        ] {
            let mut report = SitePlacementValidation {
                allows_commit: true,
                ..Default::default()
            };
            apply_deployable_staging_gate(
                BuildTool::Defense(kind),
                kind.footprint(),
                &staging,
                &pending,
                &ledger,
                &mut report,
            );
            assert!(report.allows_commit, "{kind:?}");
            assert!(report.errors.is_empty(), "{kind:?}");
        }
    }

    #[test]
    fn in_transit_uses_locked_reason_token() {
        let fp = DefenseKind::Minefield.footprint();
        let staging = SiteStagingStock::default();
        let pending = PendingSiteStagingDebits::default();
        let mut ledger = InTransitLedger::default();
        ledger.lots.push(crate::economy::logistics::FreightLot {
            destination: Entity::PLACEHOLDER,
            buffer_tag: BUFFER_TAG_MINE_UNIT.into(),
            amount: 3.0,
            route: crate::economy::logistics::RouteHandle {
                id: 0,
                topology_revision: 0,
            },
            path: crate::economy::logistics::RoutePath {
                first_edge: 0,
                edge_count: 1,
            },
            progress_edge: 0,
            remaining_ticks: 2,
            movement: crate::economy::logistics::FreightMovementModel::Continuous,
        });
        let mut report = SitePlacementValidation {
            allows_commit: true,
            ..Default::default()
        };
        apply_deployable_staging_gate(
            BuildTool::Defense(DefenseKind::Minefield),
            fp,
            &staging,
            &pending,
            &ledger,
            &mut report,
        );
        assert!(!report.allows_commit);
        assert!(report.errors.iter().any(|e| e == STOCK_IN_TRANSIT_ERROR));
    }
}
