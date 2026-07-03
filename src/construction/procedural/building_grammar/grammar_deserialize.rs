//! **CITY-G0-S1C-001** — RON registry load + post-deserialize validation.

use std::fs;
use std::path::{Path, PathBuf};

use super::grammar_types::{BuildingGrammar, BuildingGrammarRegistry, UsageId, GRAMMARS_DIR};

#[must_use]
pub fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

pub fn load_building_grammar_from_path(path: &Path) -> Result<BuildingGrammar, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let grammar: BuildingGrammar = ron::from_str(&text)
        .map_err(|e| format!("RON parse {}: {e}", path.display()))?;
    validate_building_grammar(&grammar)?;
    Ok(grammar)
}

pub fn validate_building_grammar(grammar: &BuildingGrammar) -> Result<(), String> {
    UsageId::try_new(grammar.archetype.usage.as_str())?;
    for strategy in &grammar.massing.strategies {
        super::grammar_types::MassingId::try_new(strategy.id.as_str())?;
    }
    Ok(())
}

#[must_use]
pub fn load_building_grammar_registry_from_dir(dir: &Path) -> BuildingGrammarRegistry {
    let mut registry = BuildingGrammarRegistry::default();
    if !dir.is_dir() {
        registry
            .load_errors
            .push(format!("grammars dir missing: {}", dir.display()));
        return registry;
    }
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ron") {
            continue;
        }
        match load_building_grammar_from_path(&path) {
            Ok(grammar) => {
                let key = grammar.archetype.id.clone();
                registry.grammars.insert(key, grammar);
            }
            Err(err) => registry
                .load_errors
                .push(format!("{}: {err}", path.display())),
        }
    }
    registry
}

#[must_use]
pub fn load_building_grammar_registry() -> BuildingGrammarRegistry {
    load_building_grammar_registry_from_dir(&repo_asset_path(GRAMMARS_DIR))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_rejects_invalid_usage_in_grammar() {
        let bad = r#"
(
    schema_version: 1,
    grammar_id: "bad_v1",
    archetype: (id: "Bad", usage: "not_a_real_usage", footprint_bounds: (min_width: 2, max_width: 4, min_depth: 2, max_depth: 4, min_floors: 1, max_floors: 1)),
    massing: (strategies: [(id: "long_hall", weight: 100)]),
    roof: (default_slot: "roof_default"),
    facade: (),
    detail: (),
    age: (bands: [(id: "new", weight: 100, variant_tags: ["clean"])]),
    district_styles: [(id: "industrial_west", style_pack_id: "style_industrial_west")],
)
"#;
        let parsed: Result<BuildingGrammar, _> = ron::from_str(bad);
        assert!(parsed.is_ok());
        assert!(validate_building_grammar(&parsed.unwrap()).is_err());
    }

    #[test]
    fn ship_grammars_load_without_errors() {
        let registry = load_building_grammar_registry();
        assert!(
            registry.load_errors.is_empty(),
            "{:?}",
            registry.load_errors
        );
        assert!(registry.grammars.len() >= 3);
    }
}
