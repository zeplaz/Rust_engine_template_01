//! ARCH-BUILD-GRAMMAR-002 — hierarchical building grammar evaluator (T1 core).
//!
//! Contract: `generate(archetype_id, district_style, seed)` → footprint + slot overrides + rule chain.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use bevy::prelude::Resource;
use serde::Deserialize;

use super::footprint_grid::FootprintGrid;
use super::types::{ProceduralBuildingRequest, StylePackId};

pub const GRAMMARS_DIR: &str = "assets/configs/buildings/grammars";
pub const GRAMMAR_RULES_VERSION: &str = "building_grammar_v1";
pub const GRAMMAR_DIVERSITY_WITNESS_JSON: &str = "debug_runs/grammar_diversity_witness.json";
pub const PG_QUALITY_001_SEED_SWEEP: u64 = 64;

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
    pub usage: String,
    pub footprint_bounds: FootprintBounds,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MassingStrategy {
    pub id: String,
    pub weight: u32,
    #[serde(default = "default_ratio")]
    pub width_depth_ratio: f32,
    #[serde(default = "default_footprint_mode")]
    pub footprint_mode: String,
}

fn default_ratio() -> f32 {
    1.5
}

fn default_footprint_mode() -> String {
    "rect".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct MassingRule {
    pub strategies: Vec<MassingStrategy>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoofMassingOverride {
    pub massing_id: String,
    pub slot: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoofRule {
    pub default_slot: String,
    #[serde(default)]
    pub by_massing: Vec<RoofMassingOverride>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FacadeRule {
    #[serde(default)]
    pub window_slot: String,
    #[serde(default)]
    pub door_slot: String,
    #[serde(default)]
    pub wall_slot: String,
    #[serde(default)]
    pub placement_tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DetailRule {
    #[serde(default)]
    pub prop_slot: String,
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
    pub style_pack_id: String,
    #[serde(default)]
    pub style_tags: Vec<String>,
    #[serde(default)]
    pub zoning: String,
    /// Slot key → material_profile id (PG-MATERIAL-GENERATION-001).
    #[serde(default)]
    pub material_profiles: HashMap<String, String>,
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
}

#[derive(Debug, Clone)]
pub struct GrammarRuleStep {
    pub layer: &'static str,
    pub rule_id: String,
    pub detail: String,
}

/// Deterministic output of `generate(archetype, district_style, seed)`.
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
    /// Resolved slot → material_profile from district binding.
    pub material_profiles: HashMap<String, String>,
    /// Weathering band derived from age rule (APS / worker apply).
    pub weathering: String,
}

#[derive(Resource, Debug, Default)]
pub struct BuildingGrammarRegistry {
    pub grammars: HashMap<String, BuildingGrammar>,
    pub load_errors: Vec<String>,
}

#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

fn mix_seed(seed: u64, salt: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    seed.hash(&mut h);
    salt.hash(&mut h);
    h.finish()
}

fn pick_weighted_index<'a>(items: &'a [(u32, usize)], seed: u64) -> usize {
    let total: u32 = items.iter().map(|(w, _)| *w).sum();
    if total == 0 {
        return 0;
    }
    let roll = (seed % u64::from(total)) as u32;
    let mut acc = 0u32;
    for (weight, idx) in items {
        acc += *weight;
        if roll < acc {
            return *idx;
        }
    }
    items.last().map(|(_, i)| *i).unwrap_or(0)
}

impl BuildingGrammar {
    pub fn district_binding(&self, district_style: &str) -> Option<&DistrictStyleBinding> {
        self.district_styles
            .iter()
            .find(|d| d.id == district_style)
    }

    fn resolve_footprint(&self, strategy: &MassingStrategy, seed: u64) -> (u32, u32, u32) {
        let b = &self.archetype.footprint_bounds;
        let salt = format!("footprint:{}", strategy.id);
        let s = mix_seed(seed, &salt);

        let depth_span = b.max_depth.saturating_sub(b.min_depth) + 1;
        let depth = b.min_depth + (s as u32 % depth_span);

        let width = match strategy.id.as_str() {
            "long_hall" | "double_hall" => {
                let ratio = strategy.width_depth_ratio.max(1.1);
                let w = ((depth as f32) * ratio).round() as u32;
                w.clamp(b.min_width, b.max_width)
            }
            "l_shape" => {
                let w = depth + 2;
                w.clamp(b.min_width, b.max_width)
            }
            _ => {
                let width_span = b.max_width.saturating_sub(b.min_width) + 1;
                b.min_width + ((s >> 16) as u32 % width_span)
            }
        };

        let floor_span = b.max_floors.saturating_sub(b.min_floors) + 1;
        let floors = b.min_floors + ((s >> 32) as u32 % floor_span);

        (width.max(2), depth.max(2), floors.max(1))
    }

    fn roof_slot_for_massing(&self, massing_id: &str) -> &str {
        self.roof
            .by_massing
            .iter()
            .find(|r| r.massing_id == massing_id)
            .map(|r| r.slot.as_str())
            .unwrap_or(self.roof.default_slot.as_str())
    }

    /// Evaluate full grammar chain for one archetype + district + seed.
    pub fn generate(&self, district_style: &str, seed: u64) -> Result<GrammarGenerateResult, String> {
        let district = self
            .district_binding(district_style)
            .ok_or_else(|| format!("unknown district_style: {district_style}"))?;

        let massing_weights: Vec<(u32, usize)> = self
            .massing
            .strategies
            .iter()
            .enumerate()
            .map(|(i, s)| (s.weight, i))
            .collect();
        let mi = pick_weighted_index(&massing_weights, mix_seed(seed, "massing"));
        let strategy = &self.massing.strategies[mi];

        let (width, depth, floors) = self.resolve_footprint(strategy, seed);

        let age_weights: Vec<(u32, usize)> = self
            .age
            .bands
            .iter()
            .enumerate()
            .map(|(i, b)| (b.weight, i))
            .collect();
        let ai = pick_weighted_index(&age_weights, mix_seed(seed, "age"));
        let age_band = &self.age.bands[ai];

        let mut slot_overrides = HashMap::new();
        slot_overrides.insert(
            "roof_default".into(),
            self.roof_slot_for_massing(&strategy.id).into(),
        );
        if !self.facade.wall_slot.is_empty() {
            slot_overrides.insert("wall_1u".into(), self.facade.wall_slot.clone());
        }
        if !self.facade.door_slot.is_empty() {
            slot_overrides.insert("door_default".into(), self.facade.door_slot.clone());
        }
        if !self.facade.window_slot.is_empty() {
            slot_overrides.insert("window_1u".into(), self.facade.window_slot.clone());
        }

        let mut rule_chain = vec![
            GrammarRuleStep {
                layer: "archetype",
                rule_id: self.archetype.id.clone(),
                detail: format!("usage={}", self.archetype.usage),
            },
            GrammarRuleStep {
                layer: "district_style",
                rule_id: district.id.clone(),
                detail: format!("style_pack={}", district.style_pack_id),
            },
            GrammarRuleStep {
                layer: "massing",
                rule_id: strategy.id.clone(),
                detail: format!("{width}x{depth}x{floors} mode={}", strategy.footprint_mode),
            },
            GrammarRuleStep {
                layer: "roof",
                rule_id: self.roof_slot_for_massing(&strategy.id).into(),
                detail: "slot override for R token".into(),
            },
            GrammarRuleStep {
                layer: "facade",
                rule_id: "facade_v1".into(),
                detail: format!(
                    "tags={}",
                    self.facade.placement_tags.join(",")
                ),
            },
            GrammarRuleStep {
                layer: "detail",
                rule_id: if self.detail.prop_slot.is_empty() {
                    "none".into()
                } else {
                    self.detail.prop_slot.clone()
                },
                detail: format!("density={:.2}", self.detail.density),
            },
            GrammarRuleStep {
                layer: "age",
                rule_id: age_band.id.clone(),
                detail: format!("variant_tags={}", age_band.variant_tags.join(",")),
            },
        ];

        if strategy.footprint_mode == "l_shape" {
            rule_chain.push(GrammarRuleStep {
                layer: "massing",
                rule_id: "l_shape_v1".into(),
                detail: "asymmetric rect footprint (full L cutout in v2)".into(),
            });
        }

        let material_profiles = district.material_profiles.clone();
        let weathering = weathering_for_age_band(&age_band.id);

        Ok(GrammarGenerateResult {
            grammar_id: self.grammar_id.clone(),
            archetype_id: self.archetype.id.clone(),
            district_style: district_style.into(),
            seed,
            massing_strategy: strategy.id.clone(),
            footprint_mode: strategy.footprint_mode.clone(),
            width,
            depth,
            floors,
            style_pack_id: district.style_pack_id.clone(),
            slot_overrides,
            placement_tags: self.facade.placement_tags.clone(),
            variant_tags: age_band.variant_tags.clone(),
            detail_density: self.detail.density,
            age_band: age_band.id.clone(),
            rule_chain,
            material_profiles,
            weathering,
        })
    }
}

#[must_use]
fn weathering_for_age_band(age_band: &str) -> String {
    match age_band {
        "new" => "light".into(),
        "weathered" => "medium".into(),
        "abandoned" => "heavy".into(),
        _ => "medium".into(),
    }
}

impl GrammarGenerateResult {
    /// Style-pack slot key → material_profile (PG-MATERIAL-GENERATION-001).
    #[must_use]
    pub fn material_profile_for_slot(&self, slot_key: &str) -> Option<&str> {
        self.material_profiles
            .get(slot_key)
            .map(|s| s.as_str())
            .or_else(|| default_material_for_token_slot(slot_key, &self.style_pack_id))
    }
}

#[must_use]
fn default_material_for_token_slot(slot_key: &str, style_pack_id: &str) -> Option<&'static str> {
    if style_pack_id == "style_industrial_west" {
        return match slot_key {
            "wall_1u" | "wall_2u" => Some("steel_panel_01"),
            "door_default" | "door_wide" => Some("steel_door_warehouse_01"),
            "corner_outer" | "corner_inner" => Some("steel_corner_01"),
            "roof_default" | "roof_industrial" | "roof_flat" => Some("roof_metal_01"),
            "window_industrial" | "window_1u" => Some("glass_panel_01"),
            _ => None,
        };
    }
    match slot_key {
        "wall_1u" => Some("brick_red_01"),
        "door_default" => Some("wood_plank_01"),
        "corner_outer" => Some("brick_red_01"),
        "roof_default" => Some("roof_tile_01"),
        _ => None,
    }
}

impl GrammarGenerateResult {
    #[must_use]
    pub fn procedural_request(&self) -> ProceduralBuildingRequest {
        ProceduralBuildingRequest {
            archetype_id: self.archetype_id.clone(),
            width: self.width,
            depth: self.depth,
            floors: self.floors,
            style: StylePackId(self.style_pack_id.clone()),
            seed: self.seed,
        }
    }

    #[must_use]
    pub fn footprint_grid(&self) -> FootprintGrid {
        FootprintGrid::from_grammar(self)
    }

    /// Style-pack slot key for a footprint token (`W`/`D`/`C`/`R`).
    #[must_use]
    pub fn slot_key_for_token(&self, token: &str) -> Option<&str> {
        let base = match token {
            "W" => "wall_1u",
            "D" => "door_default",
            "C" => "corner_outer",
            "R" => "roof_default",
            _ => return None,
        };
        self.slot_overrides
            .get(base)
            .map(|s| s.as_str())
            .or(Some(base))
    }
}

/// Load all `*.ron` grammars under [`GRAMMARS_DIR`].
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

pub fn load_building_grammar_from_path(path: &Path) -> Result<BuildingGrammar, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    ron::from_str(&text).map_err(|e| format!("RON parse {}: {e}", path.display()))
}

#[must_use]
pub fn load_building_grammar_registry() -> BuildingGrammarRegistry {
    load_building_grammar_registry_from_dir(&repo_asset_path(GRAMMARS_DIR))
}

/// Resolve grammar by archetype id (e.g. `IndustrialWarehouse`) and evaluate.
pub fn generate(
    archetype_id: &str,
    district_style: &str,
    seed: u64,
) -> Result<GrammarGenerateResult, String> {
    let registry = load_building_grammar_registry();
    if !registry.load_errors.is_empty() {
        return Err(registry.load_errors.join("; "));
    }
    let grammar = registry
        .grammars
        .get(archetype_id)
        .ok_or_else(|| format!("no grammar for archetype: {archetype_id}"))?;
    grammar.generate(district_style, seed)
}

/// Reference tags for assembly snapshot (parity with MCP `grammar_reference_tags`).
#[must_use]
pub fn grammar_reference_tags(result: &GrammarGenerateResult) -> Vec<String> {
    let mut tags = vec![
        format!("grammar:{}", result.grammar_id),
        format!("archetype:{}", result.archetype_id),
        format!("district:{}", result.district_style),
        format!("massing:{}", result.massing_strategy),
        format!("age:{}", result.age_band),
        GRAMMAR_RULES_VERSION.to_owned(),
    ];
    for step in &result.rule_chain {
        tags.push(format!("chain:{}:{}", step.layer, step.rule_id));
    }
    tags
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

#[must_use]
pub fn pg_quality_001_collect_metrics(
    archetype_id: &str,
    district_style: &str,
    seed_max: u64,
) -> Result<PgQuality001Metrics, String> {
    let mut massing = HashSet::new();
    let mut roofs = HashSet::new();
    let mut silhouettes = HashSet::new();
    for seed in 0..seed_max {
        let r = generate(archetype_id, district_style, seed)?;
        massing.insert(r.massing_strategy.clone());
        let roof_slot = r
            .slot_overrides
            .get("roof_default")
            .cloned()
            .unwrap_or_else(|| "roof_default".into());
        roofs.insert(roof_slot);
        silhouettes.insert((r.width, r.depth, r.footprint_mode.clone()));
    }
    let mut massing_strategies: Vec<_> = massing.into_iter().collect();
    massing_strategies.sort();
    let mut roof_slots: Vec<_> = roofs.into_iter().collect();
    roof_slots.sort();
    Ok(PgQuality001Metrics {
        archetype_id: archetype_id.into(),
        district_style: district_style.into(),
        seeds_swept: seed_max,
        massing_strategy_count: massing_strategies.len(),
        roof_slot_count: roof_slots.len(),
        footprint_silhouette_count: silhouettes.len(),
        massing_strategies,
        roof_slots,
    })
}

#[must_use]
pub fn pg_quality_001_witness_green() -> bool {
    pg_quality_001_collect_metrics("IndustrialWarehouse", "industrial_west", PG_QUALITY_001_SEED_SWEEP)
        .ok()
        .is_some_and(|m| {
            m.massing_strategy_count >= 2
                && m.roof_slot_count >= 2
                && m.footprint_silhouette_count >= 2
        })
}

#[must_use]
pub fn build_pg_quality_001_witness_body() -> serde_json::Value {
    let metrics = pg_quality_001_collect_metrics(
        "IndustrialWarehouse",
        "industrial_west",
        PG_QUALITY_001_SEED_SWEEP,
    );
    let green = metrics
        .as_ref()
        .ok()
        .is_some_and(|m| m.massing_strategy_count >= 2 && m.roof_slot_count >= 2 && m.footprint_silhouette_count >= 2);
    let (metrics_ok, metrics_err) = match metrics {
        Ok(m) => (Some(m), None),
        Err(e) => (None, Some(e)),
    };
    serde_json::json!({
        "gate_id": "PG-QUALITY-001",
        "program_id": "PLAN-BUILDING-GRAMMAR-001",
        "green": green,
        "archetype_id": "IndustrialWarehouse",
        "district_style": "industrial_west",
        "seed_sweep": PG_QUALITY_001_SEED_SWEEP,
        "thresholds": {
            "massing_strategy_count_min": 2,
            "roof_slot_count_min": 2,
            "footprint_silhouette_count_min": 2,
        },
        "metrics": metrics_ok,
        "error": metrics_err,
    })
}

/// PG-QUALITY-002 — embed grammar massing diversity into PG-2 `procedural_assembly_live.json`.
#[must_use]
pub fn pg_quality_002_pg2_hook_body() -> serde_json::Value {
    let metrics = pg_quality_001_collect_metrics(
        "IndustrialWarehouse",
        "industrial_west",
        PG_QUALITY_001_SEED_SWEEP,
    );
    let grammar_metrics_green = metrics
        .as_ref()
        .ok()
        .is_some_and(|m| {
            m.massing_strategy_count >= 2
                && m.roof_slot_count >= 2
                && m.footprint_silhouette_count >= 2
        });
    let (metrics_ok, metrics_err) = match metrics {
        Ok(m) => (Some(m), None),
        Err(e) => (None, Some(e)),
    };
    serde_json::json!({
        "gate_id": "PG-QUALITY-002",
        "grammar_gate_id": "PG-QUALITY-001",
        "grammar_witness_path": GRAMMAR_DIVERSITY_WITNESS_JSON,
        "green": grammar_metrics_green,
        "pilot_archetype_id": "IndustrialWarehouse",
        "pilot_district_style": "industrial_west",
        "metrics": metrics_ok,
        "error": metrics_err,
    })
}

#[must_use]
pub fn pg_quality_002_pg2_hook_green() -> bool {
    pg_quality_002_pg2_hook_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Refresh PG-QUALITY-001 witness (`debug_runs/grammar_diversity_witness.json`).
#[must_use]
pub fn refresh_pg_quality_001_grammar_diversity_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_pg_quality_001_witness_body();
    let wrapped = wrap_debug_run(
        "construction",
        "refresh_pg_quality_001_grammar_diversity_witness",
        GRAMMAR_DIVERSITY_WITNESS_JSON,
        body,
    );
    write_debug_run_json(GRAMMAR_DIVERSITY_WITNESS_JSON, wrapped) && pg_quality_001_witness_green()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn industrial_warehouse_grammar_deterministic() {
        let registry = load_building_grammar_registry();
        assert!(
            registry.load_errors.is_empty(),
            "{:?}",
            registry.load_errors
        );
        let a = generate("IndustrialWarehouse", "industrial_west", 43).expect("generate a");
        let b = generate("IndustrialWarehouse", "industrial_west", 43).expect("generate b");
        assert_eq!(a.width, b.width);
        assert_eq!(a.massing_strategy, b.massing_strategy);
        assert_eq!(a.style_pack_id, "style_industrial_west");
        assert!(a.rule_chain.len() >= 5);
    }

    #[test]
    fn grammar_massing_strategies_vary_by_seed() {
        let mut strategies = std::collections::HashSet::new();
        for seed in 0..64 {
            let r = generate("IndustrialWarehouse", "industrial_west", seed).unwrap();
            strategies.insert(r.massing_strategy);
        }
        assert!(
            strategies.len() >= 2,
            "expected multiple massing strategies across seeds, got {strategies:?}"
        );
    }

    #[test]
    fn pg_quality_001_witness_metrics_green() {
        assert!(super::pg_quality_001_witness_green());
    }

    #[test]
    fn refresh_pg_quality_001_writes_grammar_diversity_witness() {
        assert!(super::refresh_pg_quality_001_grammar_diversity_witness());
        let path = super::repo_asset_path(super::GRAMMAR_DIVERSITY_WITNESS_JSON);
        let text = std::fs::read_to_string(path).expect("witness file");
        let body: serde_json::Value = serde_json::from_str(&text).expect("json");
        let green = body
            .get("green")
            .or_else(|| body.get("payload").and_then(|p| p.get("green")))
            .and_then(|v| v.as_bool());
        assert_eq!(green, Some(true));
    }
}
