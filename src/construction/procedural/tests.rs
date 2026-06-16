//! PG-1 StylePack + PG-2 assembly integration tests and live witness.

use super::{
    build_assembly_snapshot, load_procedural_module_registry, load_style_pack_registry,
    pg_quality_002_pg2_hook_body, pg_quality_002_pg2_hook_green,
    refresh_pg_quality_001_grammar_diversity_witness, snapshot_passes_auto_001_contract,
    staging_relative_path, write_assembly_snapshot, FootprintGrid, ProceduralBuildingRequest,
    StylePackId,
};
use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::render::extraction::{
    assemble_procedural_build_instances, ProceduralModuleSceneCatalog,
};

pub const PROCEDURAL_ASSEMBLY_LIVE_JSON: &str = "debug_runs/procedural_assembly_live.json";

#[must_use]
pub fn build_procedural_assembly_witness_body() -> serde_json::Value {
    let modules = load_procedural_module_registry();
    let packs = load_style_pack_registry();
    let style_pack_id = "style_victorian";
    let request = ProceduralBuildingRequest {
        archetype_id: "rect_perimeter".into(),
        width: 4,
        depth: 2,
        floors: 2,
        style: StylePackId(style_pack_id.into()),
        seed: 1,
        arch_dna_preset_id: None,
    };
    let pack = packs.get(style_pack_id);
    let grid = FootprintGrid::from_request(&request);
    let extract = pack.map(|p| {
        assemble_procedural_build_instances(
            &request,
            p,
            &grid,
            &modules,
            &ProceduralModuleSceneCatalog::default(),
        )
    });

    let packs_loaded = packs.load_errors.is_empty() && packs.len() == 7;
    let slots_resolve = packs.iter().all(|pack| {
        pack.module_ids().all(|module_id| {
            modules
                .resolve_module_id(module_id)
                .is_some_and(|e| !e.development_tier.is_smoke())
        })
    });

    let pg2_wired = extract.as_ref().is_some_and(|e| e.pg2_wired);
    let smoke_fallback_used = extract.as_ref().is_some_and(|e| e.smoke_fallback_used);
    let footprint_cells = extract.as_ref().map(|e| e.footprint_cells).unwrap_or(0);
    let module_ids_used = extract
        .as_ref()
        .map(|e| e.module_ids_used.clone())
        .unwrap_or_default();
    let no_greybox_jobs = module_ids_used.iter().all(|id| {
        modules
            .resolve_module_id(id)
            .is_none_or(|e| !e.batch_id.starts_with("kit_greybox"))
    });

    let (assembly_id, assembly_snapshot_path, module_placements_count, auto_001_contract) =
        match (pack, extract.as_ref()) {
            (Some(p), Some(_)) => {
                let snapshot = build_assembly_snapshot(&request, p, &grid, &modules);
                let contract_ok = snapshot_passes_auto_001_contract(&snapshot);
                let _ = write_assembly_snapshot(&snapshot);
                (
                    snapshot.assembly_id.clone(),
                    staging_relative_path(&snapshot),
                    snapshot.module_placements.len(),
                    contract_ok,
                )
            }
            _ => (String::new(), String::new(), 0, false),
        };

    let green = packs_loaded
        && slots_resolve
        && pg2_wired
        && !smoke_fallback_used
        && footprint_cells > 0
        && !module_ids_used.is_empty()
        && no_greybox_jobs
        && auto_001_contract
        && !assembly_id.is_empty();

    let grammar_diversity = pg_quality_002_pg2_hook_body();

    serde_json::json!({
        "pg2_wired": pg2_wired,
        "style_pack_id": style_pack_id,
        "module_ids_used": module_ids_used,
        "smoke_fallback_used": smoke_fallback_used,
        "footprint_cells": footprint_cells,
        "style_packs_loaded": packs_loaded,
        "style_pack_count": packs.len(),
        "slots_resolve_lod0": slots_resolve,
        "assembly_id": assembly_id,
        "assembly_snapshot_path": assembly_snapshot_path,
        "module_placements_count": module_placements_count,
        "auto_001_contract": auto_001_contract,
        "grammar_diversity": grammar_diversity,
        "grammar_diversity_green": grammar_diversity
            .get("green")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        "green": green,
    })
}

/// MCP-PG-2-WIT — write `debug_runs/procedural_assembly_live.json`.
#[must_use]
pub fn refresh_procedural_assembly_live_witness() -> bool {
    let _ = refresh_pg_quality_001_grammar_diversity_witness();
    let body = build_procedural_assembly_witness_body();
    let green = body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let wrapped = wrap_debug_run(
        "procedural_assembly",
        "refresh_procedural_assembly_live_witness",
        PROCEDURAL_ASSEMBLY_LIVE_JSON,
        body,
    );
    write_debug_run_json(PROCEDURAL_ASSEMBLY_LIVE_JSON, wrapped) && green
}

#[test]
fn style_pack_ron_loads_victorian_slots() {
    let packs = load_style_pack_registry();
    assert!(packs.load_errors.is_empty(), "{:?}", packs.load_errors);
    assert_eq!(packs.len(), 7);
    let victorian = packs.get("style_victorian").expect("style_victorian");
    assert_eq!(victorian.label, "Victorian");
    assert_eq!(victorian.resolve_slot_str("wall_1u"), Some("wall_brick_1u"));
    assert_eq!(
        victorian.resolve_slot_str("door_default"),
        Some("door_residential")
    );
    assert_eq!(
        victorian.resolve_slot_str("roof_default"),
        Some("roof_pitched_gable")
    );
}

#[test]
fn style_pack_rejects_duplicate_ids() {
    let dir = std::env::temp_dir().join(format!(
        "style_pack_dup_test_{}",
        std::process::id()
    ));
    let _ = std::fs::create_dir_all(&dir);
    let ron = include_str!("../../../assets/configs/buildings/style_packs/style_victorian.ron");
    std::fs::write(dir.join("style_victorian.ron"), ron).unwrap();
    std::fs::write(dir.join("style_victorian_copy.ron"), ron).unwrap();
    let reg = super::load_style_packs_from_dir(&dir);
    assert!(
        reg.load_errors
            .iter()
            .any(|e| e.contains("duplicate style_pack_id")),
        "expected duplicate id error: {:?}",
        reg.load_errors
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn style_pack_slot_module_id_non_empty() {
    let packs = load_style_pack_registry();
    assert!(packs.load_errors.is_empty(), "{:?}", packs.load_errors);
    for pack in packs.iter() {
        for module_id in pack.module_ids() {
            assert!(
                !module_id.is_empty(),
                "empty module_id in pack {}",
                pack.id.as_str()
            );
        }
    }
}

#[test]
fn style_pack_slots_resolve_via_module_registry() {
    let modules = load_procedural_module_registry();
    assert!(modules.load_errors.is_empty(), "{:?}", modules.load_errors);
    let packs = load_style_pack_registry();
    assert!(packs.load_errors.is_empty(), "{:?}", packs.load_errors);
    for pack in packs.iter() {
        for module_id in pack.module_ids() {
            assert!(
                modules.resolve_module_id(module_id).is_some(),
                "pack {} slot `{}` must resolve to lod0+ row",
                pack.id.as_str(),
                module_id
            );
        }
    }
}

#[test]
fn procedural_assembly_live_witness_green() {
    assert!(
        refresh_procedural_assembly_live_witness(),
        "procedural_assembly_live.json must be green"
    );
}

#[test]
fn pg_quality_002_links_grammar_diversity_to_pg2_witness() {
    assert!(
        pg_quality_002_pg2_hook_green(),
        "grammar massing diversity must pass thresholds for PG-2 hook"
    );
    assert!(refresh_procedural_assembly_live_witness());
    let text = std::fs::read_to_string(PROCEDURAL_ASSEMBLY_LIVE_JSON).expect("witness");
    let body: serde_json::Value = serde_json::from_str(&text).expect("json");
    let hook = body
        .get("grammar_diversity")
        .or_else(|| body.get("payload").and_then(|p| p.get("grammar_diversity")))
        .expect("grammar_diversity hook");
    assert_eq!(hook.get("gate_id").and_then(|v| v.as_str()), Some("PG-QUALITY-002"));
    assert_eq!(
        hook.get("grammar_gate_id").and_then(|v| v.as_str()),
        Some("PG-QUALITY-001")
    );
}

// --- module index + building registry integration (MCP-E0) ---

use crate::construction::building_definitions::{
    attach_procedural_glb_paths, load_building_definitions_from_dir, BuildingDefinitionRegistry,
    default_buildings_dir,
};

#[test]
fn registry_resolves_lod0_module_glb_by_catalog_id() {
    let modules = load_procedural_module_registry();
    assert!(modules.load_errors.is_empty(), "{:?}", modules.load_errors);

    let buildings = load_building_definitions_from_dir(default_buildings_dir());
    let asset = buildings.procedural_glb_asset(&modules, "wall_brick_1u");
    assert!(asset.is_some(), "lod0 module id lookup");
    assert!(asset.unwrap().contains("wall_brick_1u_lod0_run001"));
}

#[test]
fn smoke_only_module_not_resolved_for_stylepack() {
    let modules = load_procedural_module_registry();
    let buildings = load_building_definitions_from_dir(default_buildings_dir());
    assert!(buildings.procedural_glb_asset(&modules, "corner_brick_outer").is_none());
}

#[test]
fn attach_paths_from_procedural_module_id_field() {
    let modules = load_procedural_module_registry();
    let mut buildings = BuildingDefinitionRegistry::default();
    buildings.by_id.insert(
        "test_proc_building".into(),
        crate::construction::BuildingDefinition {
            id: "test_proc_building".into(),
            display_name: "Test".into(),
            footprint: crate::construction::FootprintMatrix::from_size(1, 1, true),
            construction_cost: 1,
            construction_time_ticks: 1,
            power_consumption: 0.0,
            power_generation: 0.0,
            workers_required: 0,
            site_archetype: crate::strategic::SiteArchetype::Factory,
            family: crate::construction::BuildingFamily::Industry,
            produces: Vec::new(),
            consumes: Vec::new(),
            supply_chain: None,
            supply_chain_role: None,
            concrete_type: None,
            utility_role: None,
            plant_definition_id: None,
            transfer_capacity_mva: 0.0,
            is_productive: false,
            procedural_module_id: Some("door_residential".into()),
            procedural_glb_path: None,
            procedural_glb_asset: None,
            grammar_archetype_id: None,
            arch_dna_preset: None,
            site_json_path: None,
            pilot_hover_hint: None,
            district_style: None,
        },
    );
    attach_procedural_glb_paths(&mut buildings, &modules);
    let def = buildings.get("test_proc_building").unwrap();
    assert_eq!(
        def.procedural_glb_path.as_deref(),
        modules.glb_path("door_residential")
    );
    assert!(def
        .procedural_glb_asset
        .as_deref()
        .unwrap_or("")
        .contains("door_residential_lod0_run001"));
}
