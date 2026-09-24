//! **COD-DEPLOYABLE-RECIPE-001** + **COD-DEPLOYABLE-LOGISTICS-001** + **COD-DEPLOYABLE-PLACE-001**
//! Custom MFG recipes → FreightLot haul → SiteStagingStock → staging-gated place + AI.

use bevy::prelude::*;

use crate::ai::construction::ConstructionAiConfig;
use crate::construction::{
    apply_deployable_staging_gate, apply_deployable_staging_gate_archetype,
    enqueue_deployable_staging_debit_for_archetype, enqueue_deployable_staging_debit_on_commit,
    BuildTool, DefenseKind, INSUFFICIENT_STAGED_ERROR,
};
use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::economy::logistics::{
    apply_pending_site_staging_debits_system, commit_freight_arrivals_system,
    deployable_haul_transit_ticks, dispatch_deployable_from_manufacturing_system, InTransitLedger,
    PendingSiteStagingDebits, SiteStagingStock,
};
use crate::entities::production::core::{
    manufacturing_deployable_bundle, tick_manufacturing_nodes, ManufacturingBlueprintRegistry,
    ManufacturingDomain, ManufacturingOutputBuffers, BUFFER_TAG_DRAGON_TEETH_UNIT,
    BUFFER_TAG_MINE_UNIT, MFG_DRAGON_TEETH_V1, MFG_MINE_UNIT_V1,
};
use crate::strategic::{FootprintTiles, SiteArchetype, SitePlacementValidation};
use crate::systems::sim_control::SimControlState;

pub const COD_DEPLOYABLE_PIPELINE_LIVE_JSON: &str = "debug_runs/cod_deployable_pipeline_live.json";

#[derive(Debug, Clone, Copy)]
pub struct CodDeployableRecipeProof {
    pub recipes_registered: bool,
    pub tick_credits_tagged_buffers: bool,
    pub dragon_teeth_credited: f32,
    pub mine_unit_credited: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CodDeployableLogisticsProof {
    pub recipe: CodDeployableRecipeProof,
    pub in_transit: bool,
    pub staged: bool,
    pub no_same_tick_teleport: bool,
    pub in_transit_dragon_teeth: f32,
    pub in_transit_mine_unit: f32,
    pub staged_dragon_teeth: f32,
    pub staged_mine_unit: f32,
}

/// Lib harness: registry has Custom recipes; one tick credits plant-local tagged buffers.
#[must_use]
pub fn cod_deployable_recipe_lib_green() -> CodDeployableRecipeProof {
    let reg = ManufacturingBlueprintRegistry::default();
    let teeth_bp = reg.get(MFG_DRAGON_TEETH_V1);
    let mine_bp = reg.get(MFG_MINE_UNIT_V1);
    let recipes_registered = matches!(
        (teeth_bp, mine_bp),
        (Some(t), Some(m))
            if t.domain == ManufacturingDomain::Custom
                && m.domain == ManufacturingDomain::Custom
                && t.output_buffer_tag.as_deref() == Some(BUFFER_TAG_DRAGON_TEETH_UNIT)
                && m.output_buffer_tag.as_deref() == Some(BUFFER_TAG_MINE_UNIT)
    );

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<SimControlState>()
        .init_resource::<ManufacturingBlueprintRegistry>()
        .add_systems(Update, tick_manufacturing_nodes);

    let (node, buffers) = manufacturing_deployable_bundle(MFG_DRAGON_TEETH_V1).unwrap();
    let e_teeth = app.world_mut().spawn((node, buffers)).id();
    let (node_m, buffers_m) = manufacturing_deployable_bundle(MFG_MINE_UNIT_V1).unwrap();
    let e_mine = app.world_mut().spawn((node_m, buffers_m)).id();

    app.update();

    let dragon_teeth_credited = app
        .world()
        .get::<ManufacturingOutputBuffers>(e_teeth)
        .map(|b| b.get(BUFFER_TAG_DRAGON_TEETH_UNIT))
        .unwrap_or(0.0);
    let mine_unit_credited = app
        .world()
        .get::<ManufacturingOutputBuffers>(e_mine)
        .map(|b| b.get(BUFFER_TAG_MINE_UNIT))
        .unwrap_or(0.0);
    let tick_credits_tagged_buffers =
        dragon_teeth_credited > 0.0 && mine_unit_credited > 0.0;

    CodDeployableRecipeProof {
        recipes_registered,
        tick_credits_tagged_buffers,
        dragon_teeth_credited,
        mine_unit_credited,
    }
}

/// Manufacture → dispatch FreightLot → transit ticks → SiteStagingStock credit (no teleport).
#[must_use]
pub fn cod_deployable_logistics_lib_green() -> CodDeployableLogisticsProof {
    let recipe = cod_deployable_recipe_lib_green();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<SimControlState>()
        .init_resource::<ManufacturingBlueprintRegistry>()
        .init_resource::<InTransitLedger>()
        .init_resource::<SiteStagingStock>()
        .add_systems(
            Update,
            (
                tick_manufacturing_nodes,
                // Arrivals before dispatch — same causality as LogisticsSimulationSet::FreightDispatch.
                commit_freight_arrivals_system,
                dispatch_deployable_from_manufacturing_system,
            )
                .chain(),
        );

    let (node, buffers) = manufacturing_deployable_bundle(MFG_DRAGON_TEETH_V1).unwrap();
    app.world_mut().spawn((node, buffers));
    let (node_m, buffers_m) = manufacturing_deployable_bundle(MFG_MINE_UNIT_V1).unwrap();
    app.world_mut().spawn((node_m, buffers_m));

    // Tick 1: credit plant buffers + dispatch into ledger (arrivals already ran — empty).
    app.update();

    let (in_transit_dragon_teeth, in_transit_mine_unit, remaining_min) = {
        let ledger = app.world().resource::<InTransitLedger>();
        let mut teeth = 0.0;
        let mut mines = 0.0;
        let mut rem = u16::MAX;
        for lot in &ledger.lots {
            if lot.buffer_tag == BUFFER_TAG_DRAGON_TEETH_UNIT {
                teeth += lot.amount;
            } else if lot.buffer_tag == BUFFER_TAG_MINE_UNIT {
                mines += lot.amount;
            }
            rem = rem.min(lot.remaining_ticks);
        }
        (teeth, mines, if ledger.lots.is_empty() { 0 } else { rem })
    };
    let staging_after_dispatch = {
        let s = app.world().resource::<SiteStagingStock>();
        s.get(BUFFER_TAG_DRAGON_TEETH_UNIT) + s.get(BUFFER_TAG_MINE_UNIT)
    };
    let in_transit = in_transit_dragon_teeth > 0.0 && in_transit_mine_unit > 0.0;
    let no_same_tick_teleport = in_transit
        && staging_after_dispatch < 1e-4
        && remaining_min >= deployable_haul_transit_ticks();

    // Stop further manufacture so the haul can drain cleanly (dispatch finds empty buffers).
    app.world_mut().resource_mut::<SimControlState>().paused = true;

    // Advance until lots clear (transit + credit). Cap avoids hang if ticks mis-set.
    for _ in 0..8 {
        if app.world().resource::<InTransitLedger>().lots.is_empty() {
            break;
        }
        app.update();
    }

    let staged_dragon_teeth = app
        .world()
        .resource::<SiteStagingStock>()
        .get(BUFFER_TAG_DRAGON_TEETH_UNIT);
    let staged_mine_unit = app
        .world()
        .resource::<SiteStagingStock>()
        .get(BUFFER_TAG_MINE_UNIT);
    let staged = staged_dragon_teeth > 0.0
        && staged_mine_unit > 0.0
        && app.world().resource::<InTransitLedger>().lots.is_empty();

    CodDeployableLogisticsProof {
        recipe,
        in_transit,
        staged,
        no_same_tick_teleport,
        in_transit_dragon_teeth,
        in_transit_mine_unit,
        staged_dragon_teeth,
        staged_mine_unit,
    }
}

/// Place + AI gate proof on top of staged stock (COD-DEPLOYABLE-PLACE-001).
#[derive(Debug, Clone, Copy)]
pub struct CodDeployablePlaceProof {
    pub kind_map_ok: bool,
    pub place_blocked_no_stock: bool,
    pub place_ok: bool,
    pub debit_applied: bool,
    pub ai_commit_gated: bool,
    pub wall_concrete_untouched: bool,
}

#[must_use]
pub fn cod_deployable_place_lib_green(logistics: &CodDeployableLogisticsProof) -> CodDeployablePlaceProof {
    let kind_map_ok = DefenseKind::DragonTeeth.site_archetype() == SiteArchetype::DragonTeeth
        && DefenseKind::Minefield.site_archetype() == SiteArchetype::Minefield
        && DefenseKind::DragonTeeth.is_manufactured_deployable()
        && DefenseKind::Minefield.is_manufactured_deployable()
        && BuildTool::Defense(DefenseKind::DragonTeeth).uses_two_click_place()
        && BuildTool::Defense(DefenseKind::Minefield).uses_two_click_place();

    // Blocked without stock.
    let mut staging = SiteStagingStock::default();
    let mut pending = PendingSiteStagingDebits::default();
    let ledger = InTransitLedger::default();
    let teeth_fp = DefenseKind::DragonTeeth.footprint();
    let mut blocked = SitePlacementValidation {
        allows_commit: true,
        ..Default::default()
    };
    apply_deployable_staging_gate(
        BuildTool::Defense(DefenseKind::DragonTeeth),
        teeth_fp,
        &staging,
        &pending,
        &ledger,
        &mut blocked,
    );
    let place_blocked_no_stock = !blocked.allows_commit
        && blocked.errors.iter().any(|e| e == INSUFFICIENT_STAGED_ERROR);

    // Credit from logistics amounts → place ok + enqueue debit → FreightDispatch apply.
    staging.credit(BUFFER_TAG_DRAGON_TEETH_UNIT, logistics.staged_dragon_teeth.max(1.0));
    staging.credit(BUFFER_TAG_MINE_UNIT, logistics.staged_mine_unit.max(3.0));
    let mut ok_report = SitePlacementValidation {
        allows_commit: true,
        ..Default::default()
    };
    apply_deployable_staging_gate(
        BuildTool::Defense(DefenseKind::DragonTeeth),
        teeth_fp,
        &staging,
        &pending,
        &ledger,
        &mut ok_report,
    );
    let place_ok_gate = ok_report.allows_commit;
    let enqueue_ok = enqueue_deployable_staging_debit_on_commit(
        BuildTool::Defense(DefenseKind::DragonTeeth),
        teeth_fp,
        &staging,
        &mut pending,
    )
    .unwrap_or(false);
    let before = staging.get(BUFFER_TAG_DRAGON_TEETH_UNIT);
    let mut app = App::new();
    app.insert_resource(staging)
        .insert_resource(pending)
        .add_systems(Update, apply_pending_site_staging_debits_system);
    app.update();
    let after = app
        .world()
        .resource::<SiteStagingStock>()
        .get(BUFFER_TAG_DRAGON_TEETH_UNIT);
    let debit_applied = enqueue_ok && place_ok_gate && after + 1e-4 < before;
    let place_ok = place_ok_gate && debit_applied;

    // AI gated without stock (same funnel gate).
    let empty_staging = SiteStagingStock::default();
    let empty_pending = PendingSiteStagingDebits::default();
    let mut ai_report = SitePlacementValidation {
        allows_commit: true,
        ..Default::default()
    };
    apply_deployable_staging_gate_archetype(
        SiteArchetype::Minefield,
        DefenseKind::Minefield.footprint(),
        &empty_staging,
        &empty_pending,
        &ledger,
        &mut ai_report,
    );
    let ai_commit_gated = !ai_report.allows_commit
        && enqueue_deployable_staging_debit_for_archetype(
            SiteArchetype::Minefield,
            DefenseKind::Minefield.footprint(),
            &empty_staging,
            &mut PendingSiteStagingDebits::default(),
        )
        .is_err();

    // Wall / trench / bunker not blocked by empty staging.
    let mut wall_report = SitePlacementValidation {
        allows_commit: true,
        ..Default::default()
    };
    apply_deployable_staging_gate(
        BuildTool::Defense(DefenseKind::DefensiveWall),
        DefenseKind::DefensiveWall.footprint(),
        &empty_staging,
        &empty_pending,
        &ledger,
        &mut wall_report,
    );
    let wall_concrete_untouched = wall_report.allows_commit && wall_report.errors.is_empty();

    let _ = ConstructionAiConfig {
        archetype: SiteArchetype::DragonTeeth,
        footprint: FootprintTiles {
            width: 3,
            depth: 1,
        },
        ..Default::default()
    };

    CodDeployablePlaceProof {
        kind_map_ok,
        place_blocked_no_stock,
        place_ok,
        debit_applied,
        ai_commit_gated,
        wall_concrete_untouched,
    }
}

/// Full pipeline witness: recipe → haul → stage → place (+ AI gate).
#[must_use]
pub fn refresh_cod_deployable_pipeline_live_witness() -> bool {
    let proof = cod_deployable_logistics_lib_green();
    let place = cod_deployable_place_lib_green(&proof);
    let recipe_green = proof.recipe.recipes_registered && proof.recipe.tick_credits_tagged_buffers;
    let logistics_green = proof.in_transit
        && proof.staged
        && proof.no_same_tick_teleport
        && recipe_green;
    let place_green = place.kind_map_ok
        && place.place_blocked_no_stock
        && place.place_ok
        && place.debit_applied
        && place.ai_commit_gated
        && place.wall_concrete_untouched;
    let pipeline_complete = logistics_green && place_green;
    let body = serde_json::json!({
        "gate_id": "COD-DEPLOYABLE-PLACE-001",
        "program": "PLAN-MIL-DEPLOYABLE-DEFENSE-v1",
        "designer_pass": "d9851e12-4830-4637-baac-6f5a8bb625f1",
        "charter": "src/dev/design_mil_deployable_defense_v1.md",
        "plan": "src/dev/plan_mil_deployable_defense_v1.md",
        "authority_owner": "commit_freight_arrivals_system + apply_pending_site_staging_debits_system",
        "site_staging_stock_writer": "FreightDispatch (credit arrivals + debit pending)",
        "site_staging_stock_ui_writes": false,
        "recipes_registered": proof.recipe.recipes_registered,
        "tick_credits_tagged_buffers": proof.recipe.tick_credits_tagged_buffers,
        "recipes": [
            {
                "id": MFG_DRAGON_TEETH_V1,
                "domain": "Custom",
                "cargo": "heavy_unit",
                "output_buffer_tag": BUFFER_TAG_DRAGON_TEETH_UNIT,
                "credited": proof.recipe.dragon_teeth_credited,
            },
            {
                "id": MFG_MINE_UNIT_V1,
                "domain": "Custom",
                "cargo": "light_unit",
                "output_buffer_tag": BUFFER_TAG_MINE_UNIT,
                "credited": proof.recipe.mine_unit_credited,
            }
        ],
        "in_transit": proof.in_transit,
        "in_transit_amounts": {
            BUFFER_TAG_DRAGON_TEETH_UNIT: proof.in_transit_dragon_teeth,
            BUFFER_TAG_MINE_UNIT: proof.in_transit_mine_unit,
        },
        "staged": proof.staged,
        "staged_amounts": {
            BUFFER_TAG_DRAGON_TEETH_UNIT: proof.staged_dragon_teeth,
            BUFFER_TAG_MINE_UNIT: proof.staged_mine_unit,
        },
        "no_same_tick_teleport": proof.no_same_tick_teleport,
        "recipe_slice_green": recipe_green,
        "logistics_slice_green": logistics_green,
        "kind_map_ok": place.kind_map_ok,
        "place_ok": place.place_ok,
        "place_blocked_no_stock": place.place_blocked_no_stock,
        "place_blocked_no_stock_count": crate::construction::place_blocked_no_stock_count(),
        "place_ok_debit_count": crate::construction::place_ok_debit_count(),
        "ai_commit_gated_count": crate::construction::ai_commit_gated_count(),
        "debit_applied_freight_dispatch": place.debit_applied,
        "ai_commit_gated": place.ai_commit_gated,
        "wall_concrete_untouched": place.wall_concrete_untouched,
        "pipeline_complete": pipeline_complete,
        "green": pipeline_complete,
        "unblocks": [],
        "residual_next": {
            "id": null,
            "deferred": [
                "DR-MIL-MUNITIONS-INDUSTRY",
                "DR-MIL-MINE-COMBAT",
                "DR-MIL-DEPLOYABLE-ART"
            ],
            "closed": [
                {
                    "id": "DR-MIL-TRENCH-MATERIAL",
                    "closed_by": "COD-TRENCH-MATERIAL-GATE-001",
                    "witness": "debug_runs/cod_trench_material_gate_live.json"
                }
            ],
            "do_not": [
                "UI write SiteStagingStock",
                "parallel place bus",
                "ConstructionPlanQueue for defense",
                "Phase 9 / mine combat / art"
            ]
        },
        "debt_not_taken": [
            "Phase 9 munitions industry (DR-MIL-MUNITIONS-INDUSTRY)",
            "mine combat (DR-MIL-MINE-COMBAT)",
            "deployable art (DR-MIL-DEPLOYABLE-ART)"
        ]
    });
    let wrapped = wrap_debug_run(
        "COD_DEPLOYABLE_PLACE_001",
        "refresh_cod_deployable_pipeline_live_witness",
        COD_DEPLOYABLE_PIPELINE_LIVE_JSON,
        body,
    );
    pipeline_complete && write_debug_run_json(COD_DEPLOYABLE_PIPELINE_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cod_deployable_recipe_witness_green() {
        let p = cod_deployable_recipe_lib_green();
        assert!(p.recipes_registered);
        assert!(p.tick_credits_tagged_buffers);
    }

    #[test]
    fn cod_deployable_logistics_no_teleport_then_stage() {
        let p = cod_deployable_logistics_lib_green();
        assert!(p.recipe.recipes_registered);
        assert!(p.in_transit, "must enter InTransitLedger");
        assert!(p.no_same_tick_teleport, "must not credit staging same tick as dispatch");
        assert!(p.staged, "arrivals must credit SiteStagingStock");
        assert!(p.staged_dragon_teeth > 0.0);
        assert!(p.staged_mine_unit > 0.0);
    }

    #[test]
    fn cod_deployable_place_gated_and_debits() {
        let logistics = cod_deployable_logistics_lib_green();
        let p = cod_deployable_place_lib_green(&logistics);
        assert!(p.kind_map_ok);
        assert!(p.place_blocked_no_stock);
        assert!(p.place_ok);
        assert!(p.debit_applied);
        assert!(p.ai_commit_gated);
        assert!(p.wall_concrete_untouched);
    }

    #[test]
    fn cod_deployable_pipeline_witness_green() {
        assert!(refresh_cod_deployable_pipeline_live_witness());
    }
}
