//! Mobility profiles — facts + derived → movement hints. See [`mobility_profile_matrix_v1.md`](../../../prompts/designer_questions/terrain_world/ontology/mobility_profile_matrix_v1.md).
//!
//! **Aggregation (v1):** multiplicative `cost_multiplier` across matching rules, `stuck_risk = max(...)`,
//! **any** matching `blocked: true` (or `slope_grade > max_grade`) vetoes movement.

use std::fmt;

use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::terrain::material::{TagRegistry, TagSet};

/// Per-tile evaluation result — **not** stored as terrain tags.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovementHint {
    pub cost_mul: f32,
    pub blocked: bool,
    pub stuck_risk: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct MobilityProfilesFile {
    pub schema_version: u32,
    pub profiles: Vec<MobilityProfileFile>,
}

#[derive(Clone, Debug, Deserialize)]
struct MobilityProfileFile {
    id: String,
    #[serde(default)]
    traction_requirement: f32,
    max_grade: f32,
    #[serde(default)]
    amphibious: bool,
    rules: Vec<MobilityRuleFile>,
}

#[derive(Clone, Debug, Deserialize)]
struct MobilityRuleFile {
    when_all: Vec<String>,
    #[serde(default)]
    when_any: Vec<String>,
    #[serde(default = "default_one")]
    cost_multiplier: f32,
    #[serde(default)]
    stuck_risk: f32,
    #[serde(default)]
    blocked: bool,
}

fn default_one() -> f32 {
    1.0
}

#[derive(Clone, Debug)]
pub struct MobilityProfile {
    pub id: String,
    pub traction_requirement: f32,
    pub max_grade: f32,
    pub amphibious: bool,
    pub rules: Vec<MobilityRule>,
}

#[derive(Clone, Debug)]
pub struct MobilityRule {
    pub when_all: Vec<String>,
    pub when_any: Vec<String>,
    pub cost_multiplier: f32,
    pub stuck_risk: f32,
    pub blocked: bool,
}

#[derive(Asset, TypePath, Clone, Debug)]
pub struct MobilityProfileRegistry {
    pub schema_version: u32,
    pub profiles: Vec<MobilityProfile>,
}

impl MobilityProfileRegistry {
    pub fn load_from_ron(path: &str) -> std::io::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let file: MobilityProfilesFile = ron::de::from_str(&s).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("RON: {e}"))
        })?;
        Ok(Self::from_file(file))
    }

    fn from_file(file: MobilityProfilesFile) -> Self {
        let profiles = file
            .profiles
            .into_iter()
            .map(|p| MobilityProfile {
                id: p.id,
                traction_requirement: p.traction_requirement,
                max_grade: p.max_grade,
                amphibious: p.amphibious,
                rules: p
                    .rules
                    .into_iter()
                    .map(|r| MobilityRule {
                        when_all: r.when_all,
                        when_any: r.when_any,
                        cost_multiplier: r.cost_multiplier,
                        stuck_risk: r.stuck_risk,
                        blocked: r.blocked,
                    })
                    .collect(),
            })
            .collect();
        Self {
            schema_version: file.schema_version,
            profiles,
        }
    }

    pub fn profile_index(&self, id: &str) -> Option<usize> {
        self.profiles.iter().position(|p| p.id == id)
    }
}

fn rule_matches(rule: &MobilityRule, cell_tags: &TagSet, tag_reg: &TagRegistry) -> bool {
    let all_ok = rule.when_all.iter().all(|name| {
        tag_reg
            .tag_id(name)
            .is_some_and(|tid| cell_tags.contains(tid))
    });
    if !all_ok {
        return false;
    }
    if rule.when_any.is_empty() {
        return true;
    }
    rule.when_any.iter().any(|name| {
        tag_reg
            .tag_id(name)
            .is_some_and(|tid| cell_tags.contains(tid))
    })
}

/// Evaluate mobility for one cell. `traction_index` is a future stub (0..1); pass `1.0` until sim provides it.
///
/// `material_traction_scale`: **interpretation** multiplier from material `sim.traction_mod` (defaults to **1.0**).
/// It scales **cost**, never hard-blocks — profiles and `blocked` rules own vetoes.
pub fn evaluate_tile(
    profile: &MobilityProfile,
    cell_tags: &TagSet,
    slope_grade: f32,
    traction_index: f32,
    tag_reg: &TagRegistry,
    material_traction_scale: f32,
) -> MovementHint {
    let mut blocked = slope_grade > profile.max_grade;
    if !profile.amphibious && profile.traction_requirement > 0.0 {
        blocked |= traction_index + 1e-6 < profile.traction_requirement;
    }

    let mut cost_mul = 1.0f32;
    let mut stuck_risk = 0.0f32;

    for rule in &profile.rules {
        if !rule_matches(rule, cell_tags, tag_reg) {
            continue;
        }
        if rule.blocked {
            blocked = true;
        }
        cost_mul *= rule.cost_multiplier;
        stuck_risk = stuck_risk.max(rule.stuck_risk);
    }

    let mt = material_traction_scale.clamp(0.05, 20.0);
    cost_mul *= mt;

    MovementHint {
        cost_mul,
        blocked,
        stuck_risk,
    }
}

/// Bevy loader `*.mobility_profiles.ron`.
#[derive(Default, TypePath)]
pub struct MobilityProfileRegistryLoader;

#[derive(Debug)]
pub enum MobilityProfileRegistryLoaderError {
    Io(std::io::Error),
    Ron(String),
}

impl fmt::Display for MobilityProfileRegistryLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Ron(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for MobilityProfileRegistryLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Ron(_) => None,
        }
    }
}

impl From<std::io::Error> for MobilityProfileRegistryLoaderError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl AssetLoader for MobilityProfileRegistryLoader {
    type Asset = MobilityProfileRegistry;
    type Settings = ();
    type Error = MobilityProfileRegistryLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let s = std::str::from_utf8(&bytes).map_err(|e| MobilityProfileRegistryLoaderError::Ron(e.to_string()))?;
        let file: MobilityProfilesFile = ron::de::from_str(s)
            .map_err(|e| MobilityProfileRegistryLoaderError::Ron(format!("RON: {e}")))?;
        Ok(MobilityProfileRegistry::from_file(file))
    }

    fn extensions(&self) -> &[&str] {
        &["mobility_profiles.ron"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn example_tag_registry() -> TagRegistry {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/tag_registry.example.json");
        TagRegistry::load_from_json(path.to_str().unwrap()).unwrap()
    }

    #[test]
    fn wheeled_blocked_on_steep_tag() {
        let tag_reg = example_tag_registry();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mob = MobilityProfileRegistry::load_from_ron(
            root
                .join("assets/config/terrain/mobility_profiles.example.ron")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let p = mob.profiles.iter().find(|x| x.id == "wheeled_logistics").unwrap();
        let mut ts = TagSet::default();
        if let Some(id) = tag_reg.tag_id("steep") {
            ts.insert(id);
        }
        let h = evaluate_tile(p, &ts, 0.1, 1.0, &tag_reg, 1.0);
        assert!(h.blocked);
    }

    #[test]
    fn wheeled_multiplies_mud_cost() {
        let tag_reg = example_tag_registry();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mob = MobilityProfileRegistry::load_from_ron(
            root
                .join("assets/config/terrain/mobility_profiles.example.ron")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let p = mob.profiles.iter().find(|x| x.id == "wheeled_logistics").unwrap();
        let mut ts = TagSet::default();
        if let Some(id) = tag_reg.tag_id("muddy") {
            ts.insert(id);
        }
        let h = evaluate_tile(p, &ts, 0.05, 1.0, &tag_reg, 1.0);
        assert!(!h.blocked);
        assert!((h.cost_mul - 4.0).abs() < 0.01);
        assert!((h.stuck_risk - 0.3).abs() < 0.01);
    }

    #[test]
    fn traction_mod_multiplies_rule_cost_without_blocking() {
        let tag_reg = example_tag_registry();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mob = MobilityProfileRegistry::load_from_ron(
            root
                .join("assets/config/terrain/mobility_profiles.example.ron")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let p = mob.profiles.iter().find(|x| x.id == "wheeled_logistics").unwrap();
        let mut ts = TagSet::default();
        if let Some(id) = tag_reg.tag_id("muddy") {
            ts.insert(id);
        }
        let h = evaluate_tile(p, &ts, 0.05, 1.0, &tag_reg, 0.5);
        assert!(!h.blocked);
        assert!((h.cost_mul - 2.0).abs() < 0.01, "expected 4 * 0.5, got {}", h.cost_mul);
    }

    #[test]
    fn slope_above_max_grade_blocks() {
        let tag_reg = example_tag_registry();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mob = MobilityProfileRegistry::load_from_ron(
            root
                .join("assets/config/terrain/mobility_profiles.example.ron")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let p = mob.profiles.iter().find(|x| x.id == "wheeled_logistics").unwrap();
        let ts = TagSet::default();
        let h = evaluate_tile(p, &ts, p.max_grade + 0.1, 1.0, &tag_reg, 1.0);
        assert!(h.blocked);
    }
}
