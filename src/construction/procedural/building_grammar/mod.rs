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
    generate, generate_with_arch_dna_preset,
    grammar_reference_tags, pg_quality_002_pg2_hook_body,
    };
pub use grammar_types::{
    BuildingGrammar, BuildingGrammarRegistry, FacilityBindingV1,
    FacilityPowerTier, FacilityProgramAxes, GrammarGenerateResult, GrammarRuleStep,
    MassingStrategy, PgQuality001Metrics, ProgramAxisLevel,
    FACILITY_BINDING_G1_MIN, FACILITY_BINDING_SCHEMA, GRAMMARS_DIR, GRAMMAR_DIVERSITY_WITNESS_JSON,
    GRAMMAR_RULES_VERSION, PG_QUALITY_001_SEED_SWEEP,
};
