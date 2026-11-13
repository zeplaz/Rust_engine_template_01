//! **BQ-H3-V0-RETIRE-001** — freeze `arch_build_grammar_v0` behind validation shim.
//!
//! v0 DNA reweighting remains opt-in via `arch_dna_preset_id`; unknown massing override ids
//! in presets are rejected before they can bias T1 grammar picks.

use super::arch_build_grammar_v0::ArchGrammarV0Preset;
use super::building_grammar::BuildingGrammar;

/// Massing ids that v0 bias tables know about (`arch_build_grammar_v0::massing_bias`).
pub const V0_KNOWN_MASSING_IDS: &[&str] = &["long_hall", "double_hall", "l_shape", "yard_complex"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V0PresetValidation {
    pub preset_id: String,
    pub grammar_id: String,
    pub ok: bool,
    pub unknown_massing_overrides: Vec<String>,
    pub unknown_grammar_massings: Vec<String>,
}

#[must_use]
pub fn validate_v0_preset_against_grammar(
    preset: &ArchGrammarV0Preset,
    grammar: &BuildingGrammar,
) -> V0PresetValidation {
    let grammar_massings: Vec<String> = grammar
        .massing
        .strategies
        .iter()
        .map(|s| s.id.as_str().to_owned())
        .collect();
    let unknown_overrides: Vec<String> = preset
        .massing_weight_override
        .iter()
        .filter(|o| !grammar_massings.iter().any(|id| id == &o.id))
        .map(|o| o.id.clone())
        .collect();
    let unknown_v0_bias: Vec<String> = grammar_massings
        .iter()
        .filter(|id| !V0_KNOWN_MASSING_IDS.contains(&id.as_str()))
        .cloned()
        .collect();
    let ok = unknown_overrides.is_empty();
    V0PresetValidation {
        preset_id: preset.preset_id.clone(),
        grammar_id: preset.grammar_id.clone(),
        ok,
        unknown_massing_overrides: unknown_overrides,
        unknown_grammar_massings: unknown_v0_bias,
    }
}



#[cfg(test)]
mod tests {
    


}
