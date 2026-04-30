use serde::Deserialize;

use crate::terrain::biome::TerrainClass;

#[derive(Clone, Debug)]
pub struct MaterialRule {
    pub required: Vec<String>,
    pub forbidden: Vec<String>,
    pub family_filter: Option<TerrainClass>,
    pub result_name: String,
    pub priority: i32,
    pub rule_index: u32,
}

#[derive(Clone, Debug)]
pub struct RuleSet {
    pub schema_version: u32,
    pub rules: Vec<MaterialRule>,
}

#[derive(Deserialize)]
struct RuleSetFile {
    schema_version: u32,
    rules: Vec<RuleEntryFile>,
}

#[derive(Deserialize)]
struct RuleEntryFile {
    required: Vec<String>,
    forbidden: Vec<String>,
    #[serde(default)]
    family_filter: Option<TerrainClass>,
    #[serde(alias = "result")]
    result_name: String,
    priority: i32,
}

impl RuleSet {
    pub fn load_from_ron(path: &str) -> std::io::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let file: RuleSetFile = ron::de::from_str(&s).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("RON: {e}"))
        })?;
        let mut rules: Vec<MaterialRule> = file
            .rules
            .into_iter()
            .enumerate()
            .map(|(i, r)| MaterialRule {
                required: r.required,
                forbidden: r.forbidden,
                family_filter: r.family_filter,
                result_name: r.result_name,
                priority: r.priority,
                rule_index: i as u32,
            })
            .collect();
        rules.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.rule_index.cmp(&b.rule_index))
        });
        Ok(Self {
            schema_version: file.schema_version,
            rules,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_rules_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/material_rules.example.ron")
    }

    #[test]
    fn material_rules_load_example_ron() {
        let set = RuleSet::load_from_ron(example_rules_path().to_str().unwrap()).unwrap();
        assert_eq!(set.schema_version, 1);
        assert!(!set.rules.is_empty());
        let loam = set.rules.iter().find(|r| r.result_name == "loam_wet");
        assert!(loam.is_some());
        assert_eq!(loam.unwrap().priority, 20);
    }

    #[test]
    fn material_rule_set_sort_stable_under_equal_priority() {
        let mut rules = vec![
            MaterialRule {
                required: vec![],
                forbidden: vec![],
                family_filter: None,
                result_name: "second".into(),
                priority: 5,
                rule_index: 1,
            },
            MaterialRule {
                required: vec![],
                forbidden: vec![],
                family_filter: None,
                result_name: "first".into(),
                priority: 5,
                rule_index: 0,
            },
        ];
        rules.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.rule_index.cmp(&b.rule_index))
        });
        assert_eq!(rules[0].result_name, "first");
        assert_eq!(rules[1].result_name, "second");
    }
}
