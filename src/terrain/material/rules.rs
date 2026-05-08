use std::fmt;

use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::terrain::family::{TerrainFamilyId, TerrainFamilyRegistry};

#[derive(Clone, Debug)]
pub struct MaterialRule {
    pub required: Vec<String>,
    pub forbidden: Vec<String>,
    pub family_filter: Option<TerrainFamilyId>,
    pub result_name: String,
    pub priority: i32,
    pub rule_index: u32,
}

#[derive(Asset, TypePath, Clone, Debug)]
pub struct RuleSet {
    pub schema_version: u32,
    pub rules: Vec<MaterialRule>,
}

#[derive(Deserialize)]
pub(crate) struct RuleSetFile {
    schema_version: u32,
    rules: Vec<RuleEntryFile>,
}

#[derive(Deserialize)]
struct RuleEntryFile {
    required: Vec<String>,
    forbidden: Vec<String>,
    #[serde(default)]
    family_filter: Option<String>,
    #[serde(alias = "result")]
    result_name: String,
    priority: i32,
}

fn default_family_registry_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/config/terrain/terrain_family_registry.example.json")
}

impl RuleSet {
    pub fn load_from_ron(path: &str) -> std::io::Result<Self> {
        let families = TerrainFamilyRegistry::load_from_json(
            default_family_registry_path().to_str().unwrap(),
        )?;
        let s = std::fs::read_to_string(path)?;
        let file: RuleSetFile = ron::de::from_str(&s).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("RON: {e}"))
        })?;
        Self::from_file(file, &families).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub(crate) fn from_file(file: RuleSetFile, families: &TerrainFamilyRegistry) -> Result<Self, String> {
        let mut rules: Vec<MaterialRule> = file
            .rules
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                let family_filter = match r.family_filter {
                    None => None,
                    Some(ref name) => Some(families.require_id(name)?),
                };
                Ok(MaterialRule {
                    required: r.required,
                    forbidden: r.forbidden,
                    family_filter,
                    result_name: r.result_name,
                    priority: r.priority,
                    rule_index: i as u32,
                })
            })
            .collect::<Result<_, String>>()?;
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

/// Bevy loader for `*.material_rules.ron`.
#[derive(Default, TypePath)]
pub struct RuleSetLoader;

#[derive(Debug)]
pub enum RuleSetLoaderError {
    Io(std::io::Error),
    Ron(String),
}

impl fmt::Display for RuleSetLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Ron(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for RuleSetLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Ron(_) => None,
        }
    }
}

impl From<std::io::Error> for RuleSetLoaderError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl AssetLoader for RuleSetLoader {
    type Asset = RuleSet;
    type Settings = ();
    type Error = RuleSetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let s = std::str::from_utf8(&bytes).map_err(|e| RuleSetLoaderError::Ron(e.to_string()))?;
        let file: RuleSetFile = ron::de::from_str(s)
            .map_err(|e| RuleSetLoaderError::Ron(format!("RON: {e}")))?;
        let families = TerrainFamilyRegistry::load_from_json(
            default_family_registry_path().to_str().ok_or_else(|| {
                RuleSetLoaderError::Ron("family registry path not utf-8".into())
            })?,
        )
        .map_err(|e| RuleSetLoaderError::Ron(e.to_string()))?;
        RuleSet::from_file(file, &families).map_err(RuleSetLoaderError::Ron)
    }

    fn extensions(&self) -> &[&str] {
        &["material_rules.ron"]
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

    #[test]
    fn rule_set_loader_extensions() {
        let loader = RuleSetLoader::default();
        assert!(loader.extensions().contains(&"material_rules.ron"));
    }
}
