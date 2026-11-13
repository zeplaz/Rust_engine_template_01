//! Procedural building data — module kit index (PG-1) + StylePack registry (PG-1/2).

mod arch_build_grammar_v0;
mod arch_grammar_v0_shim;
mod assembly_quality_gate;
mod assembly_snapshot;
mod building_grammar;
mod edge_adjacency;
mod facade_propagation;
mod footprint_grid;
mod golden_seed_set;
mod load;
mod module_contract;
mod module_index;
mod palette_catalog;
mod scale_chain;
mod tile_atlas_index;
mod tile_variant_resolver;
mod tile_visual_state;
mod types;
mod variant_recipe;

#[cfg(test)]
mod tests;

pub use arch_build_grammar_v0::{
    arch_dna_consumer_from_preset_id, arch_dna_consumer_from_snapshot_fields,
    arch_dna_consumer_wired, beta_with_world_transport_bias, build_arch_dna_massing_diversity_rows,
    list_arch_dna_preset_ids, load_arch_dna_preset,
    load_canonical_arch_dna_pilot_preset, load_preset_for_id, load_preset_from_path,
    program_graph_stub_for_preset, site_zones_for_preset, ArchDnaConsumerFields,
    ArchGrammarV0Preset, PressureFieldV0, ARCH_DNA_EXAMPLES_DIR, ARCH_GRAMMAR_V0_PRESET_JSON,
};
pub use arch_grammar_v0_shim::{
    validate_v0_preset_against_grammar, V0_KNOWN_MASSING_IDS,
};
pub use assembly_quality_gate::{
    compute_assembly_quality,
    AssemblyQualityScore, BQ_A2_PASS_SCORE,
    };
pub use assembly_snapshot::{
    assembly_id_for, assembly_snapshot_stable_hash, build_assembly_snapshot, build_assembly_snapshot_from_grammar,
    build_assembly_snapshot_from_grammar_with_preset, grammar_rule_chain_snapshot, outward_yaw_for_face, outward_yaw_rad,
    procedural_module_local_translation, procedural_roof_local_translation,
    procedural_wall_local_translation, snapshot_passes_auto_001_contract,
    staging_relative_path, write_assembly_snapshot, AssemblyGrammarRuleChain,
    AssemblyModulePlacement, AssemblySnapshot, BuildingLookV2Flags, ASSEMBLY_SNAPSHOT_SCHEMA,
    ASSEMBLY_SNAPSHOT_STAGING, PROCEDURAL_RULES_VERSION,
};
pub use building_grammar::{
    generate as generate_building_grammar, generate_with_arch_dna_preset, grammar_reference_tags,
    load_building_grammar_registry, pg_quality_002_pg2_hook_body,
    BuildingGrammar, BuildingGrammarRegistry, FacilityBindingV1, FacilityPowerTier,
    FacilityProgramAxes, GrammarGenerateResult, GrammarRuleStep, PgQuality001Metrics,
    ProgramAxisLevel, FACILITY_BINDING_G1_MIN, FACILITY_BINDING_SCHEMA, GRAMMARS_DIR,
    GRAMMAR_DIVERSITY_WITNESS_JSON, GRAMMAR_RULES_VERSION, PG_QUALITY_001_SEED_SWEEP,
};
pub use edge_adjacency::{
    check_footprint_adjacency,
    AdjacencyViolation, };

pub use footprint_grid::{
    exterior_faces, street_facing_door_column, ExteriorFace, FootprintCell, FootprintGrid, FootprintToken,
    RoofMode,
};
pub use golden_seed_set::{
    GoldenSeedEntry, GOLDEN_SEED_SET_V1,
};
pub use load::{
    default_style_packs_dir, init_style_pack_registry, load_style_pack_registry,
    load_style_packs_from_dir, STYLE_PACKS_DIR,
};
pub use module_contract::{
    grid_units_from_width_m, standard_wall_height_m,
    FLOOR_HEIGHT_M, GRID_UNIT_M, MODULE_CONTRACT_JSON, PIVOT_CONVENTION,
    SEAM_TOLERANCE_M,
};
pub use module_index::{
    default_module_index_ron_path, init_procedural_module_registry,
    load_procedural_module_registry, load_procedural_module_registry_from_path,
    DevelopmentTier, ProceduralModuleEntry,
    ProceduralModuleRegistry, StylePackResolveMeta, MODULE_INDEX_JSON,
    MODULE_INDEX_RON,
};
pub use palette_catalog::{
    init_palette_catalog_registry, load_palette_catalog_registry, palette_variation_pick_index,
    resolve_palette_variation,
    resolve_palette_variation_default, visual_variant_id, PaletteCatalog, PaletteCatalogRegistry,
    PaletteVariation, ResolvedPaletteVariation, PALETTE_CATALOG_INDEX_RON,
};
pub use scale_chain::{
    scale_chain_links, SCALE_AUTHORITY_DECISION,
};
pub use tile_atlas_index::{
    default_tile_atlas_index_ron_path, init_tile_atlas_registry, load_tile_atlas_registry,
    load_tile_atlas_registry_from_path, TileAtlasEntry, TileAtlasRegistry, TILE_ATLAS_INDEX_JSON,
    TILE_ATLAS_INDEX_RON,
};
pub use tile_variant_resolver::{
    init_variant_catalog,
    load_variant_catalog, palette_atlas_variant_key,
    production_atlas_covers_assembly,
    resolve_landscape_tile_from_extract_key,
    resolve_landscape_tile_from_topology, resolve_landscape_tile_variant, resolve_tile_variant,
    resolve_tile_variant_with_palette, ProceduralTilePrimaryActive, ResolvedLandscapeTileVariant,
    ResolvedTileVariant, TileAtlasDomain, TileVariantContext, VariantCatalog,
    VARIANT_CATALOG_RON,
};
pub use tile_visual_state::{facing_from_rotation_quarter_turns, VisualState};
pub use types::{
    BuildingArchetype, BuildingUsage, FallbackPolicy, MissingSlotReason, MissingSlotViolation,
    ProceduralAssemblyRequest, ProceduralBuildingRequest, StylePack, StylePackId,
    StylePackRegistry, StylePackSlotKey,
};
pub use variant_recipe::{
    expand_variant_recipes, recipes_from_catalog_keys, BuildingState, ConstructionState,
    DamageState, FireState, LightingState, OccupancyState, VariantLayer, VariantRecipe,
};

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
