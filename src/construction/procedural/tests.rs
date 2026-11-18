//! PG-1 StylePack + PG-2 assembly integration tests.

use super::{
    load_procedural_module_registry, load_style_pack_registry,
};






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
    let dir = std::env::temp_dir().join(format!("style_pack_dup_test_{}", std::process::id()));
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





// --- module index + building registry integration (MCP-E0) ---

use crate::construction::building_definitions::{
    attach_procedural_glb_paths, default_buildings_dir, load_building_definitions_from_dir,
    BuildingDefinitionRegistry,
};

#[test]
fn registry_resolves_module_glb_by_catalog_id() {
    let modules = load_procedural_module_registry();
    assert!(modules.load_errors.is_empty(), "{:?}", modules.load_errors);

    let buildings = load_building_definitions_from_dir(default_buildings_dir());
    let asset = buildings.procedural_glb_asset(&modules, "wall_brick_1u");
    assert!(asset.is_some(), "module id lookup");
    // lod0 twins retired (BQ-LOD0-TWIN-PRUNE-001): catalog ids resolve to production GLBs.
    assert!(asset.unwrap().contains("wall_brick_1u_production_run001"));
}

#[test]
fn smoke_only_module_not_resolved_for_stylepack() {
    let modules = load_procedural_module_registry();
    let buildings = load_building_definitions_from_dir(default_buildings_dir());
    assert!(buildings
        .procedural_glb_asset(&modules, "corner_brick_outer")
        .is_none());
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
            crest_m: None,
            dam_kind: None,
            required_research_ids: Vec::new(),
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
        .contains("door_residential_production_run001"));
}
