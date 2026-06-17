//! Procedural building data — module kit index (PG-1) + StylePack registry (PG-1/2).

mod assembly_snapshot;
mod arch_build_grammar_v0;
mod building_grammar;
mod footprint_grid;
mod load;
mod module_index;
mod tile_atlas_index;
mod tile_variant_resolver;
mod tile_visual_state;
mod variant_recipe;
mod types;

#[cfg(test)]
mod tests;

pub use arch_build_grammar_v0::{
    arch_dna_consumer_from_preset_id, arch_dna_consumer_from_snapshot_fields,
    arch_dna_consumer_wired, beta_with_world_transport_bias, build_read_grammar_v0_003_witness_body,
    build_read_grammar_v0_003_witness_green, build_read_consumer_mcp_001_witness_green,
    arch_dna_massing_diversity_witness_green, build_arch_dna_massing_diversity_rows,
    load_arch_dna_preset, list_arch_dna_preset_ids, load_preset_from_path, load_preset_for_id,
    load_logistics_rail_warehouse_v0_preset, program_graph_stub_for_preset,
    site_zones_for_preset,
    ArchDnaConsumerFields, ArchGrammarV0Preset, PressureFieldV0, ARCH_DNA_EXAMPLES_DIR,
    ARCH_GRAMMAR_V0_PRESET_JSON,
};
pub use building_grammar::{
    build_pg_quality_001_witness_body, generate as generate_building_grammar,
    generate_with_arch_dna_preset, grammar_reference_tags, load_building_grammar_registry, pg_quality_001_witness_green,
    pg_quality_002_pg2_hook_body, pg_quality_002_pg2_hook_green,
    refresh_pg_quality_001_grammar_diversity_witness, BuildingGrammar, BuildingGrammarRegistry,
    GrammarGenerateResult, GrammarRuleStep, PgQuality001Metrics, GRAMMAR_DIVERSITY_WITNESS_JSON,
    GRAMMAR_RULES_VERSION, GRAMMARS_DIR, PG_QUALITY_001_SEED_SWEEP,
};
pub use assembly_snapshot::{
    assembly_id_for, build_assembly_snapshot, build_assembly_snapshot_from_grammar,
    build_assembly_snapshot_from_grammar_with_preset, grammar_rule_chain_snapshot, snapshot_passes_auto_001_contract, staging_relative_path,
    write_assembly_snapshot, procedural_module_local_translation, AssemblyGrammarRuleChain,
    AssemblyModulePlacement, AssemblySnapshot, ASSEMBLY_SNAPSHOT_SCHEMA, ASSEMBLY_SNAPSHOT_STAGING,
    PROCEDURAL_RULES_VERSION,
};
pub use footprint_grid::{FootprintCell, FootprintGrid, FootprintToken};
pub use load::{
    default_style_packs_dir, init_style_pack_registry, load_style_pack_registry,
    load_style_packs_from_dir, STYLE_PACKS_DIR,
};
pub use module_index::{
    default_module_index_ron_path, init_procedural_module_registry, load_procedural_module_registry,
    load_procedural_module_registry_from_path, DevelopmentTier, ProceduralModuleEntry,
    ProceduralModuleRegistry, MODULE_INDEX_JSON, MODULE_INDEX_RON,
};
pub use tile_atlas_index::{
    default_tile_atlas_index_ron_path, init_tile_atlas_registry, load_tile_atlas_registry,
    load_tile_atlas_registry_from_path, TileAtlasEntry, TileAtlasRegistry, TILE_ATLAS_INDEX_JSON,
    TILE_ATLAS_INDEX_RON,
};
pub use tile_visual_state::{
    facing_from_rotation_quarter_turns, VisualState,
};
pub use variant_recipe::{
    expand_variant_recipes, recipes_from_catalog_keys, BuildingState, ConstructionState,
    DamageState, FireState, LightingState, OccupancyState, VariantLayer, VariantRecipe,
};
pub use tile_variant_resolver::{
    build_procedural_tiles_runtime_witness_body, init_variant_catalog, load_variant_catalog,
    landscape_tile_resolver_witness_green, production_atlas_covers_assembly,
    procedural_tiles_runtime_witness_green, refresh_procedural_tiles_runtime_live_witness,
    resolve_landscape_tile_from_extract_key, resolve_landscape_tile_from_topology,
    resolve_landscape_tile_variant, resolve_tile_variant, ProceduralTilePrimaryActive,
    ResolvedLandscapeTileVariant, ResolvedTileVariant, TileAtlasDomain, TileVariantContext,
    VariantCatalog, VARIANT_CATALOG_RON, PROCEDURAL_TILES_RUNTIME_LIVE_JSON,
};
pub use types::{
    BuildingArchetype, BuildingUsage, FallbackPolicy, ProceduralAssemblyRequest,
    ProceduralBuildingRequest, StylePack, StylePackId, StylePackRegistry, StylePackSlotKey,
};

/// **PROC-PG-2-TAIL-001** — lod0+ tier filter; assembly records `source_tier`.
#[must_use]
pub fn procedural_pg2_tail_001_witness_green() -> bool {
    if !procedural_pg2_assembly_wired_witness_green() {
        return false;
    }
    let modules = load_procedural_module_registry();
    let packs = load_style_pack_registry();
    if !modules.load_errors.is_empty() || !packs.load_errors.is_empty() {
        return false;
    }
    let Some(pack) = packs.get("style_victorian") else {
        return false;
    };
    let request = ProceduralBuildingRequest {
        archetype_id: "rect_perimeter".into(),
        width: 4,
        depth: 2,
        floors: 2,
        style: StylePackId("style_victorian".into()),
        seed: 1,
        arch_dna_preset_id: None,
    };
    let grid = FootprintGrid::from_request(&request);
    let snap = build_assembly_snapshot(&request, pack, &grid, &modules);
    let tier_ok = matches!(snap.source_tier.as_str(), "lod0" | "production");
    tier_ok && !snap.module_placements.is_empty()
}

/// PG-2 rollup helper for construction stage witness (no file I/O).
#[must_use]
pub fn procedural_pg2_assembly_wired_witness_green() -> bool {
    use crate::render::extraction::{
        assemble_procedural_build_instances, ProceduralModuleSceneCatalog,
    };

    let modules = load_procedural_module_registry();
    let packs = load_style_pack_registry();
    if !modules.load_errors.is_empty() || !packs.load_errors.is_empty() {
        return false;
    }
    let Some(pack) = packs.get("style_victorian") else {
        return false;
    };
    let request = ProceduralBuildingRequest {
        archetype_id: "rect_perimeter".into(),
        width: 4,
        depth: 2,
        floors: 2,
        style: StylePackId("style_victorian".into()),
        seed: 1,
        arch_dna_preset_id: None,
    };
    let grid = FootprintGrid::from_request(&request);
    let extract = assemble_procedural_build_instances(
        &request,
        pack,
        &grid,
        &modules,
        &ProceduralModuleSceneCatalog::default(),
    );
    extract.pg2_wired && !extract.smoke_fallback_used && extract.footprint_cells > 0
}

/// District tag for grammar regen from a committed style pack id.
#[must_use]
pub fn district_style_for_pack(style_pack_id: &str) -> &'static str {
    match style_pack_id {
        "style_industrial_west" => "industrial_west",
        "style_industrial_soviet" => "industrial_soviet",
        "style_victorian" => "victorian",
        "style_colonial" => "colonial",
        "style_modern" => "modern",
        "style_military" => "military",
        _ => "industrial_west",
    }
}

/// PG-2 footprint authority: grammar L-matrix for pilot commits, rect perimeter otherwise.
#[must_use]
pub fn footprint_grid_for_assembly(request: &ProceduralBuildingRequest) -> FootprintGrid {
    if request.archetype_id == "rect_perimeter" {
        return FootprintGrid::from_request(request);
    }
    let district = district_style_for_pack(request.style.as_str());
    if let Ok(grammar) = generate_with_arch_dna_preset(
        &request.archetype_id,
        district,
        request.seed,
        request.arch_dna_preset_id.as_deref(),
    ) {
        return FootprintGrid::from_grammar(&grammar);
    }
    FootprintGrid::from_request(request)
}
