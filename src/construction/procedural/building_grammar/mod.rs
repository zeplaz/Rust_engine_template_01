//! Building grammar — **CITY-G0-S1C-001** 3-way split with preserved public API.
//!
//! - `grammar_types` — typed ids (**CITY-G0-S11-001**)
//! - `grammar_deserialize` — RON load + validation
//! - `grammar_evaluation` — generate + witnesses

mod grammar_deserialize;
mod grammar_evaluation;
mod grammar_types;

pub use grammar_deserialize::load_building_grammar_registry;
pub use grammar_evaluation::{
    build_pg_quality_001_witness_body, city_g0_s11_typed_ids_witness_green,
    city_g0_s1c_split_witness_green, facility_binding_read_witness_body,
    facility_binding_read_witness_green, generate, generate_with_arch_dna_preset,
    grammar_reference_tags, pg_quality_001_witness_green, pg_quality_002_pg2_hook_body,
    pg_quality_002_pg2_hook_green, refresh_pg_quality_001_grammar_diversity_witness,
};
pub use grammar_types::{
    default_door_rhythm_for_massing, BuildingGrammar, BuildingGrammarRegistry, FacilityBindingV1,
    FacilityPowerTier, FacilityProgramAxes, GrammarGenerateResult, GrammarRuleStep, MassingId,
    MassingStrategy, PgQuality001Metrics, ProgramAxisLevel, ResolvedFacade, FACILITY_BINDING_G1_MIN,
    FACILITY_BINDING_SCHEMA, GRAMMAR_DIVERSITY_WITNESS_JSON, GRAMMAR_RULES_VERSION, GRAMMARS_DIR,
    PG_QUALITY_001_SEED_SWEEP,
};
