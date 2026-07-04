//! **CITY-G0-S11-001** — typed grammar IDs (MassingId, SlotId, UsageId, FootprintMode, CorridorType).

use std::collections::HashMap;
use std::fmt;

use bevy::prelude::Resource;
use serde::Deserialize;
use serde::Deserializer;

use crate::construction::procedural::types::{BuildingUsage, StylePackId};

/// RON grammars use bare tuples/strings for optional fields (not `Some(...)`).
pub fn deserialize_ron_optional_field<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Repr<T> {
        Value(T),
        Option(Option<T>),
    }
    match Repr::<T>::deserialize(deserializer) {
        Ok(Repr::Value(v)) => Ok(Some(v)),
        Ok(Repr::Option(v)) => Ok(v),
        Err(e) => Err(e),
    }
}

pub const GRAMMARS_DIR: &str = "assets/configs/buildings/grammars";
pub const GRAMMAR_RULES_VERSION: &str = "building_grammar_v1";
pub const GRAMMAR_DIVERSITY_WITNESS_JSON: &str = "debug_runs/grammar_diversity_witness.json";
pub const PG_QUALITY_001_SEED_SWEEP: u64 = 64;
pub const FACILITY_BINDING_SCHEMA: &str = "facility_binding_v1";
pub const FACILITY_BINDING_G1_MIN: usize = 2;

fn validate_snake_id(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(format!("{label} must be snake_case: {value}"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct MassingId(pub String);

impl MassingId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn try_new(value: &str) -> Result<Self, String> {
        validate_snake_id("MassingId", value)?;
        Ok(Self(value.to_owned()))
    }

    #[must_use]
    pub fn is_long_hall(&self) -> bool {
        matches!(self.0.as_str(), "long_hall" | "double_hall")
    }

    #[must_use]
    pub fn is_l_shape(&self) -> bool {
        self.0 == "l_shape"
    }

    #[must_use]
    pub fn is_yard_complex(&self) -> bool {
        self.0 == "yard_complex"
    }
}

impl fmt::Display for MassingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct SlotId(pub String);

impl SlotId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn try_new(value: &str) -> Result<Self, String> {
        validate_snake_id("SlotId", value)?;
        Ok(Self(value.to_owned()))
    }
}

impl fmt::Display for SlotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct UsageId(pub String);

impl UsageId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn try_new(value: &str) -> Result<Self, String> {
        validate_snake_id("UsageId", value)?;
        BuildingUsage::parse_tag(value)
            .ok_or_else(|| format!("unknown usage tag for UsageId: {value}"))?;
        Ok(Self(value.to_owned()))
    }

    #[must_use]
    pub fn building_usage(&self) -> Option<BuildingUsage> {
        BuildingUsage::parse_tag(&self.0)
    }
}

impl fmt::Display for UsageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Corridor classification — replaces ad-hoc `profile.contains("rail")` in grammar consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorridorType {
    Rail,
    Road,
    Yard,
    None,
}

#[must_use]
pub fn corridor_type_for_profile(profile: &str) -> CorridorType {
    let p = profile.to_ascii_lowercase();
    if p.contains("rail") {
        CorridorType::Rail
    } else if p.contains("road") || p.contains("street") || p.contains("highway") {
        CorridorType::Road
    } else if p.contains("yard") {
        CorridorType::Yard
    } else {
        CorridorType::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FootprintMode {
    Rect,
    #[serde(rename = "l_shape")]
    LShape,
    #[serde(rename = "yard_interior")]
    YardInterior,
}

impl FootprintMode {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rect => "rect",
            Self::LShape => "l_shape",
            Self::YardInterior => "yard_interior",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FootprintBounds {
    pub min_width: u32,
    pub max_width: u32,
    pub min_depth: u32,
    pub max_depth: u32,
    pub min_floors: u32,
    pub max_floors: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArchetypeRule {
    pub id: String,
    pub usage: UsageId,
    pub footprint_bounds: FootprintBounds,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MassingStrategy {
    pub id: MassingId,
    pub weight: u32,
    #[serde(default = "default_ratio")]
    pub width_depth_ratio: f32,
    #[serde(default = "default_footprint_mode")]
    pub footprint_mode: FootprintMode,
}

fn default_ratio() -> f32 {
    1.5
}

fn default_footprint_mode() -> FootprintMode {
    FootprintMode::Rect
}

#[derive(Debug, Clone, Deserialize)]
pub struct MassingRule {
    pub strategies: Vec<MassingStrategy>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoofMassingOverride {
    pub massing_id: MassingId,
    pub slot: SlotId,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoofRule {
    pub default_slot: SlotId,
    #[serde(default)]
    pub by_massing: Vec<RoofMassingOverride>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FacadeMassingOverride {
    pub massing_id: MassingId,
    #[serde(default = "default_empty_slot")]
    pub window_slot: SlotId,
    #[serde(default = "default_empty_slot")]
    pub door_slot: SlotId,
    #[serde(default = "default_empty_slot")]
    pub wall_slot: SlotId,
    #[serde(default)]
    pub placement_tags: Vec<String>,
    /// **BQ-H1** — door column rhythm: `linear_center` · `perimeter_only` · `leg_offset` · `loading_bay`.
    #[serde(default)]
    pub door_rhythm: String,
}

/// Massing-resolved facade slots + tags (**BQ-H1-FACADE-001**).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFacade {
    pub window_slot: SlotId,
    pub door_slot: SlotId,
    pub wall_slot: SlotId,
    pub placement_tags: Vec<String>,
    pub door_rhythm: String,
    pub massing_override_applied: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FacadeRule {
    #[serde(default = "default_empty_slot")]
    pub window_slot: SlotId,
    #[serde(default = "default_empty_slot")]
    pub door_slot: SlotId,
    #[serde(default = "default_empty_slot")]
    pub wall_slot: SlotId,
    #[serde(default)]
    pub placement_tags: Vec<String>,
    #[serde(default)]
    pub by_massing: Vec<FacadeMassingOverride>,
}

impl FacadeRule {
    #[must_use]
    pub fn resolve_for_massing(&self, massing_id: &MassingId) -> ResolvedFacade {
        let ov = self
            .by_massing
            .iter()
            .find(|entry| entry.massing_id == *massing_id);
        let pick_slot = |base: &SlotId, over: Option<&SlotId>| -> SlotId {
            if let Some(slot) = over {
                if !slot.as_str().is_empty() {
                    return slot.clone();
                }
            }
            base.clone()
        };
        let mut tags = self.placement_tags.clone();
        if let Some(entry) = ov {
            for tag in &entry.placement_tags {
                if !tags.iter().any(|t| t == tag) {
                    tags.push(tag.clone());
                }
            }
        }
        let door_rhythm = ov
            .and_then(|entry| {
                if entry.door_rhythm.is_empty() {
                    None
                } else {
                    Some(entry.door_rhythm.clone())
                }
            })
            .unwrap_or_else(|| default_door_rhythm_for_massing(massing_id));
        ResolvedFacade {
            window_slot: pick_slot(&self.window_slot, ov.map(|o| &o.window_slot)),
            door_slot: pick_slot(&self.door_slot, ov.map(|o| &o.door_slot)),
            wall_slot: pick_slot(&self.wall_slot, ov.map(|o| &o.wall_slot)),
            placement_tags: tags,
            door_rhythm,
            massing_override_applied: ov.is_some(),
        }
    }
}

#[must_use]
pub fn default_door_rhythm_for_massing(massing_id: &MassingId) -> String {
    if massing_id.is_long_hall() {
        "linear_center".into()
    } else if massing_id.is_l_shape() {
        "leg_offset".into()
    } else if massing_id.is_yard_complex() {
        "perimeter_only".into()
    } else {
        "default".into()
    }
}

fn default_empty_slot() -> SlotId {
    SlotId(String::new())
}

impl Default for SlotId {
    fn default() -> Self {
        Self(String::new())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DetailRule {
    #[serde(default = "default_empty_slot")]
    pub prop_slot: SlotId,
    #[serde(default)]
    pub density: f32,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgeBand {
    pub id: String,
    pub weight: u32,
    pub variant_tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgeRule {
    pub bands: Vec<AgeBand>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DistrictStyleBinding {
    pub id: String,
    pub style_pack_id: StylePackId,
    #[serde(default)]
    pub style_tags: Vec<String>,
    #[serde(default)]
    pub zoning: String,
    #[serde(default)]
    pub material_profiles: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FacilityPowerTier {
    Light,
    Medium,
    Heavy,
    Grid,
}

impl FacilityPowerTier {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Medium => "medium",
            Self::Heavy => "heavy",
            Self::Grid => "grid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgramAxisLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FacilityProgramAxes {
    #[serde(default, deserialize_with = "deserialize_ron_optional_field")]
    pub storage: Option<ProgramAxisLevel>,
    #[serde(default, deserialize_with = "deserialize_ron_optional_field")]
    pub loading: Option<ProgramAxisLevel>,
    #[serde(default, deserialize_with = "deserialize_ron_optional_field")]
    pub office: Option<ProgramAxisLevel>,
    #[serde(default, deserialize_with = "deserialize_ron_optional_field")]
    pub service: Option<ProgramAxisLevel>,
    #[serde(default, deserialize_with = "deserialize_ron_optional_field")]
    pub expansion: Option<ProgramAxisLevel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FacilityBindingV1 {
    pub catalog_id: String,
    pub chain_id: String,
    pub supply_chain_role: String,
    pub power_tier: FacilityPowerTier,
    #[serde(default, deserialize_with = "deserialize_ron_optional_field")]
    pub site_template_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_ron_optional_field")]
    pub program_axes: Option<FacilityProgramAxes>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuildingGrammar {
    pub schema_version: u32,
    pub grammar_id: String,
    pub archetype: ArchetypeRule,
    pub massing: MassingRule,
    pub roof: RoofRule,
    pub facade: FacadeRule,
    pub detail: DetailRule,
    pub age: AgeRule,
    pub district_styles: Vec<DistrictStyleBinding>,
    #[serde(default, deserialize_with = "deserialize_ron_optional_field")]
    pub facility_binding: Option<FacilityBindingV1>,
}

#[derive(Debug, Clone)]
pub struct GrammarRuleStep {
    pub layer: &'static str,
    pub rule_id: String,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct GrammarGenerateResult {
    pub grammar_id: String,
    pub archetype_id: String,
    pub district_style: String,
    pub seed: u64,
    pub massing_strategy: String,
    pub footprint_mode: String,
    pub width: u32,
    pub depth: u32,
    pub floors: u32,
    pub style_pack_id: String,
    pub slot_overrides: HashMap<String, String>,
    pub placement_tags: Vec<String>,
    pub variant_tags: Vec<String>,
    pub detail_density: f32,
    pub age_band: String,
    pub rule_chain: Vec<GrammarRuleStep>,
    pub material_profiles: HashMap<String, String>,
    pub weathering: String,
    pub arch_dna_preset_id: Option<String>,
    /// **BQ-H1** — massing-resolved door column rhythm for footprint_grid.
    pub door_rhythm: String,
}

#[derive(Resource, Debug, Default)]
pub struct BuildingGrammarRegistry {
    pub grammars: HashMap<String, BuildingGrammar>,
    pub load_errors: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PgQuality001Metrics {
    pub archetype_id: String,
    pub district_style: String,
    pub seeds_swept: u64,
    pub massing_strategy_count: usize,
    pub roof_slot_count: usize,
    pub footprint_silhouette_count: usize,
    pub massing_strategies: Vec<String>,
    pub roof_slots: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_id_rejects_unknown_tag() {
        assert!(UsageId::try_new("not_a_real_usage").is_err());
        assert!(UsageId::try_new("warehouse").is_ok());
    }

    #[test]
    fn corridor_type_for_profile_rail() {
        assert_eq!(
            corridor_type_for_profile("default_rail"),
            CorridorType::Rail
        );
        assert_eq!(corridor_type_for_profile("highway_2lane"), CorridorType::Road);
    }

    #[test]
    fn massing_id_validates_snake_case() {
        assert!(MassingId::try_new("long_hall").is_ok());
        assert!(MassingId::try_new("BadId").is_err());
    }
}
