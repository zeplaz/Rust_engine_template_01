//! **CITY-G1-C1-001** — field-driven block typing (BlockArchetype + RON threshold table).

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::district::{DevelopmentPressure, DistrictMetrics};
use super::ids::BlockId;
use super::seed_chain::block_seed;
use super::zoning::ZoningClass;

pub const BLOCK_ARCHETYPE_THRESHOLDS_RON: &str =
    "assets/configs/settlement/block_archetype_thresholds_v1.ron";
pub const CITY_G1_C1_LIVE_JSON: &str = "debug_runs/city_g1_c1_001_live.json";

/// Block-scale grammar band — maps to BlockRecipe ids in G1-C3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BlockArchetype {
    ForestPark,
    LowDensityRes,
    MediumDensityRes,
    HighDensityCommercial,
    Industrial,
    Civic,
}

impl BlockArchetype {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ForestPark => "forest_park",
            Self::LowDensityRes => "low_density_res",
            Self::MediumDensityRes => "medium_density_res",
            Self::HighDensityCommercial => "high_density_commercial",
            Self::Industrial => "industrial",
            Self::Civic => "civic",
        }
    }

    #[must_use]
    pub fn recipe_id(self) -> &'static str {
        match self {
            Self::ForestPark => "block_recipe_forest_park_v1",
            Self::LowDensityRes => "block_recipe_low_density_res_v1",
            Self::MediumDensityRes => "block_recipe_medium_density_res_v1",
            Self::HighDensityCommercial => "block_recipe_high_density_commercial_v1",
            Self::Industrial => "block_recipe_industrial_yard_v1",
            Self::Civic => "block_recipe_civic_plaza_v1",
        }
    }
}

/// Inputs for threshold resolution (C1 field-driven typing).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockScore {
    pub pressure_residential: f32,
    pub pressure_commercial: f32,
    pub pressure_industrial: f32,
    pub saturation: f32,
    pub transport_access: f32,
    pub zoning: ZoningClass,
    /// Seeded tie-break jitter in `[0, 1)` — does not change band unless scores tie.
    pub noise_jitter: f32,
}

impl BlockScore {
    #[must_use]
    pub fn from_district_fields(
        metrics: &DistrictMetrics,
        pressure: &DevelopmentPressure,
        saturation: f32,
        zoning: ZoningClass,
        noise_jitter: f32,
    ) -> Self {
        Self {
            pressure_residential: pressure.residential.clamp(0.0, 1.0),
            pressure_commercial: pressure.commercial.clamp(0.0, 1.0),
            pressure_industrial: pressure.industrial.clamp(0.0, 1.0),
            saturation: saturation.clamp(0.0, 1.0),
            transport_access: metrics.transport_access.clamp(0.0, 1.0),
            zoning,
            noise_jitter: noise_jitter.clamp(0.0, 1.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockArchetypeBand {
    pub archetype: BlockArchetype,
    #[serde(default)]
    pub min_pressure_residential: Option<f32>,
    #[serde(default)]
    pub max_pressure_residential: Option<f32>,
    #[serde(default)]
    pub min_pressure_commercial: Option<f32>,
    #[serde(default)]
    pub max_pressure_commercial: Option<f32>,
    #[serde(default)]
    pub min_pressure_industrial: Option<f32>,
    #[serde(default)]
    pub max_pressure_industrial: Option<f32>,
    #[serde(default)]
    pub min_saturation: Option<f32>,
    #[serde(default)]
    pub max_saturation: Option<f32>,
    #[serde(default)]
    pub min_transport_access: Option<f32>,
    #[serde(default)]
    pub max_transport_access: Option<f32>,
    #[serde(default)]
    pub zoning_any_of: Vec<ZoningClass>,
}

impl BlockArchetypeBand {
    #[must_use]
    pub fn matches(&self, score: &BlockScore) -> bool {
        if let Some(min) = self.min_pressure_residential {
            if score.pressure_residential < min {
                return false;
            }
        }
        if let Some(max) = self.max_pressure_residential {
            if score.pressure_residential > max {
                return false;
            }
        }
        if let Some(min) = self.min_pressure_commercial {
            if score.pressure_commercial < min {
                return false;
            }
        }
        if let Some(max) = self.max_pressure_commercial {
            if score.pressure_commercial > max {
                return false;
            }
        }
        if let Some(min) = self.min_pressure_industrial {
            if score.pressure_industrial < min {
                return false;
            }
        }
        if let Some(max) = self.max_pressure_industrial {
            if score.pressure_industrial > max {
                return false;
            }
        }
        if let Some(min) = self.min_saturation {
            if score.saturation < min {
                return false;
            }
        }
        if let Some(max) = self.max_saturation {
            if score.saturation > max {
                return false;
            }
        }
        if let Some(min) = self.min_transport_access {
            if score.transport_access < min {
                return false;
            }
        }
        if let Some(max) = self.max_transport_access {
            if score.transport_access > max {
                return false;
            }
        }
        if !self.zoning_any_of.is_empty() && !self.zoning_any_of.contains(&score.zoning) {
            return false;
        }
        true
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockArchetypeThresholdTable {
    pub schema_version: u32,
    pub table_id: String,
    pub default_archetype: BlockArchetype,
    pub bands: Vec<BlockArchetypeBand>,
}

impl BlockArchetypeThresholdTable {
    #[must_use]
    pub fn resolve(&self, score: &BlockScore) -> BlockArchetype {
        let mut matches: Vec<BlockArchetype> = self
            .bands
            .iter()
            .filter(|b| b.matches(score))
            .map(|b| b.archetype)
            .collect();
        if matches.is_empty() {
            return self.default_archetype;
        }
        if matches.len() == 1 {
            return matches[0];
        }
        matches.sort_by_key(|a| a.as_str());
        let idx = ((score.noise_jitter * matches.len() as f32).floor() as usize).min(matches.len() - 1);
        matches[idx]
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct BlockArchetypeRegistry {
    pub table: Option<BlockArchetypeThresholdTable>,
    pub load_errors: Vec<String>,
}

impl BlockArchetypeRegistry {
    #[must_use]
    pub fn resolve(&self, score: &BlockScore) -> BlockArchetype {
        self.table
            .as_ref()
            .map(|t| t.resolve(score))
            .unwrap_or(BlockArchetype::MediumDensityRes)
    }
}

#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

pub fn load_block_archetype_threshold_table_from_path(
    path: &Path,
) -> Result<BlockArchetypeThresholdTable, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    ron::from_str(&text).map_err(|e| format!("RON parse {}: {e}", path.display()))
}

#[must_use]
pub fn load_block_archetype_threshold_table() -> BlockArchetypeThresholdTable {
    load_block_archetype_threshold_table_from_path(&repo_asset_path(BLOCK_ARCHETYPE_THRESHOLDS_RON))
        .unwrap_or_else(|err| {
            panic!("block archetype thresholds must load in tests/dev: {err}");
        })
}

#[must_use]
pub fn load_block_archetype_registry() -> BlockArchetypeRegistry {
    let path = repo_asset_path(BLOCK_ARCHETYPE_THRESHOLDS_RON);
    match load_block_archetype_threshold_table_from_path(&path) {
        Ok(table) => BlockArchetypeRegistry {
            table: Some(table),
            load_errors: Vec::new(),
        },
        Err(err) => BlockArchetypeRegistry {
            table: None,
            load_errors: vec![err],
        },
    }
}

/// Seeded jitter for tie-break only (C4 block_seed → `[0,1)`).
#[must_use]
pub fn noise_jitter_from_block_seed(block_seed: u64) -> f32 {
    let mixed = super::seed_chain::mix_u64(block_seed, "block_archetype_jitter", "v1");
    (mixed as f64 / u64::MAX as f64) as f32
}

#[must_use]
pub fn resolve_block_archetype(
    table: &BlockArchetypeThresholdTable,
    score: &BlockScore,
) -> BlockArchetype {
    table.resolve(score)
}

#[must_use]
pub fn fixture_score_for_archetype(archetype: BlockArchetype) -> BlockScore {
    match archetype {
        BlockArchetype::ForestPark => BlockScore {
            pressure_residential: 0.08,
            pressure_commercial: 0.05,
            pressure_industrial: 0.04,
            saturation: 0.10,
            transport_access: 0.25,
            zoning: ZoningClass::Rural,
            noise_jitter: 0.0,
        },
        BlockArchetype::LowDensityRes => BlockScore {
            pressure_residential: 0.40,
            pressure_commercial: 0.12,
            pressure_industrial: 0.05,
            saturation: 0.22,
            transport_access: 0.55,
            zoning: ZoningClass::Residential,
            noise_jitter: 0.0,
        },
        BlockArchetype::MediumDensityRes => BlockScore {
            pressure_residential: 0.55,
            pressure_commercial: 0.20,
            pressure_industrial: 0.10,
            saturation: 0.42,
            transport_access: 0.60,
            zoning: ZoningClass::Residential,
            noise_jitter: 0.0,
        },
        BlockArchetype::HighDensityCommercial => BlockScore {
            pressure_residential: 0.25,
            pressure_commercial: 0.72,
            pressure_industrial: 0.15,
            saturation: 0.55,
            transport_access: 0.78,
            zoning: ZoningClass::Commercial,
            noise_jitter: 0.0,
        },
        BlockArchetype::Industrial => BlockScore {
            pressure_residential: 0.10,
            pressure_commercial: 0.18,
            pressure_industrial: 0.68,
            saturation: 0.30,
            transport_access: 0.72,
            zoning: ZoningClass::Industrial,
            noise_jitter: 0.0,
        },
        BlockArchetype::Civic => BlockScore {
            pressure_residential: 0.22,
            pressure_commercial: 0.35,
            pressure_industrial: 0.08,
            saturation: 0.28,
            transport_access: 0.82,
            zoning: ZoningClass::Civic,
            noise_jitter: 0.0,
        },
    }
}

#[must_use]
pub fn city_g1_c1_001_per_band_tests_green() -> bool {
    let table = load_block_archetype_threshold_table();
    [
        BlockArchetype::ForestPark,
        BlockArchetype::LowDensityRes,
        BlockArchetype::MediumDensityRes,
        BlockArchetype::HighDensityCommercial,
        BlockArchetype::Industrial,
        BlockArchetype::Civic,
    ]
    .into_iter()
    .all(|expected| {
        let score = fixture_score_for_archetype(expected);
        resolve_block_archetype(&table, &score) == expected
    })
}

#[must_use]
pub fn build_city_g1_c1_001_witness_body() -> serde_json::Value {
    use crate::construction::procedural::city_g0_wit_001_determinism_witness_green;
    use crate::strategic::settlement::city_g1_c4_001_seed_chain_witness_green;

    let registry = load_block_archetype_registry();
    let table_ok = registry.load_errors.is_empty() && registry.table.is_some();
    let per_band = city_g1_c1_001_per_band_tests_green();
    let g0_wit = city_g0_wit_001_determinism_witness_green();
    let g1_c4 = city_g1_c4_001_seed_chain_witness_green();

    let band_results: Vec<_> = [
        BlockArchetype::ForestPark,
        BlockArchetype::LowDensityRes,
        BlockArchetype::MediumDensityRes,
        BlockArchetype::HighDensityCommercial,
        BlockArchetype::Industrial,
        BlockArchetype::Civic,
    ]
    .into_iter()
    .map(|expected| {
        let score = fixture_score_for_archetype(expected);
        let resolved = registry.resolve(&score);
        serde_json::json!({
            "expected": expected.as_str(),
            "resolved": resolved.as_str(),
            "recipe_id": resolved.recipe_id(),
            "ok": resolved == expected,
        })
    })
    .collect();

    let jitter_demo = {
        let block = BlockId("tie_break_demo".into());
        let bs = block_seed(
            super::seed_chain::town_seed(
                super::seed_chain::DEFAULT_WORLD_SEED,
                &super::ids::TownId(super::seed_chain::DEFAULT_TOWN_ID.into()),
            ),
            &block,
        );
        noise_jitter_from_block_seed(bs)
    };

    let green = table_ok && per_band && g0_wit && g1_c4;

    serde_json::json!({
        "gate": "CITY-G1-C1-001",
        "issue": "CITY-C1",
        "green": green,
        "table_ok": table_ok,
        "per_band_ok": per_band,
        "city_g0_wit_still_green": g0_wit,
        "city_g1_c4_still_green": g1_c4,
        "threshold_ron": BLOCK_ARCHETYPE_THRESHOLDS_RON,
        "band_results": band_results,
        "noise_jitter_demo": jitter_demo,
    })
}

#[must_use]
pub fn city_g1_c1_001_block_archetype_witness_green() -> bool {
    build_city_g1_c1_001_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn refresh_city_g1_c1_001_block_archetype_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_city_g1_c1_001_witness_body();
    let green = body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let wrapped = wrap_debug_run(
        "CITY-G1-C1-001",
        "refresh_city_g1_c1_001_block_archetype_witness",
        CITY_G1_C1_LIVE_JSON,
        body,
    );
    write_debug_run_json(CITY_G1_C1_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_ron_loads() {
        let table = load_block_archetype_threshold_table();
        assert_eq!(table.schema_version, 1);
        assert!(!table.bands.is_empty());
    }

    #[test]
    fn each_block_archetype_band_resolves() {
        assert!(city_g1_c1_001_per_band_tests_green());
    }

    #[test]
    fn noise_jitter_is_deterministic() {
        let a = noise_jitter_from_block_seed(12345);
        let b = noise_jitter_from_block_seed(12345);
        assert!((a - b).abs() < f32::EPSILON);
        assert!((0.0..1.0).contains(&a));
    }

    #[test]
    fn recipe_ids_are_stable() {
        assert_eq!(
            BlockArchetype::Industrial.recipe_id(),
            "block_recipe_industrial_yard_v1"
        );
    }
}
