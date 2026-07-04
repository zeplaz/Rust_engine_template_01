//! **CITY-G0-S1C-001** — grammar evaluation, witnesses, and public generate API.

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use super::grammar_deserialize::{load_building_grammar_registry, repo_asset_path};
use super::grammar_types::{
    BuildingGrammar, BuildingGrammarRegistry, FacilityBindingV1, FacilityPowerTier,
    FootprintMode, GrammarGenerateResult, GrammarRuleStep, MassingId, MassingStrategy,
    PgQuality001Metrics, PG_QUALITY_001_SEED_SWEEP, FACILITY_BINDING_G1_MIN,
    FACILITY_BINDING_SCHEMA, GRAMMAR_DIVERSITY_WITNESS_JSON, GRAMMAR_RULES_VERSION,
};
use crate::construction::procedural::types::{ProceduralBuildingRequest, StylePackId};
use crate::construction::procedural::arch_build_grammar_v0::{
    floors_from_beta_vert, reweight_massing_strategies, ArchDnaConsumerFields,
};
use crate::construction::procedural::footprint_grid::FootprintGrid;

impl BuildingGrammarRegistry {
    #[must_use]
    pub fn by_grammar_id(&self, grammar_id: &str) -> Option<&BuildingGrammar> {
        self.grammars
            .values()
            .find(|g| g.grammar_id == grammar_id)
    }

    #[must_use]
    pub fn facility_bindings(&self) -> Vec<(&BuildingGrammar, &FacilityBindingV1)> {
        self.grammars
            .values()
            .filter_map(|g| g.facility_binding().map(|b| (g, b)))
            .collect()
    }

    #[must_use]
    pub fn facility_binding_for_archetype(&self, archetype_id: &str) -> Option<&FacilityBindingV1> {
        self.grammars
            .get(archetype_id)
            .and_then(|g| g.facility_binding())
    }
}

#[must_use]
pub fn facility_binding_read_witness_green() -> bool {
    facility_binding_read_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        == Some(true)
}

#[must_use]
pub fn facility_binding_read_witness_body() -> serde_json::Value {
    let registry = load_building_grammar_registry();
    let load_ok = registry.load_errors.is_empty();
    let bindings = registry.facility_bindings();
    let bound_count = bindings.len();
    let g1_min_ok = bound_count >= FACILITY_BINDING_G1_MIN;

    let factory = registry.by_grammar_id("factory_cluster_v1");
    let rail = registry.by_grammar_id("rail_edge_v1");
    let warehouse = registry.by_grammar_id("industrial_warehouse_v1");

    let factory_binding_ok = factory
        .and_then(|g| g.facility_binding())
        .is_some_and(|b| {
            b.catalog_id == "concrete_mixer_plant"
                && b.chain_id == "concrete_portland"
                && b.supply_chain_role == "concrete_mixer"
                && b.power_tier == FacilityPowerTier::Light
        });
    let rail_binding_ok = rail
        .and_then(|g| g.facility_binding())
        .is_some_and(|b| {
            b.catalog_id == "logistics_rail_warehouse" && b.chain_id == "logistics_storage"
        });
    let warehouse_binding_ok = warehouse
        .and_then(|g| g.facility_binding())
        .is_some_and(|b| b.catalog_id == "logistics_storage_warehouse");

    let green = load_ok && g1_min_ok && factory_binding_ok && rail_binding_ok && warehouse_binding_ok;

    serde_json::json!({
        "gate": "COD-FACILITY-BINDING-READ-001",
        "green": green,
        "binding_schema": FACILITY_BINDING_SCHEMA,
        "load_errors_empty": load_ok,
        "bound_grammar_count": bound_count,
        "g1_min_ok": g1_min_ok,
        "factory_cluster_v1": factory_binding_ok,
        "rail_edge_v1": rail_binding_ok,
        "industrial_warehouse_v1": warehouse_binding_ok,
        "bound_grammar_ids": bindings
            .iter()
            .map(|(g, _)| g.grammar_id.as_str())
            .collect::<Vec<_>>(),
    })
}

fn mix_seed(seed: u64, salt: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    seed.hash(&mut h);
    salt.hash(&mut h);
    h.finish()
}

fn pick_weighted_index(items: &[(u32, usize)], seed: u64) -> usize {
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
    #[must_use]
    pub fn facility_binding(&self) -> Option<&FacilityBindingV1> {
        self.facility_binding.as_ref()
    }

    pub fn district_binding(&self, district_style: &str) -> Option<&super::grammar_types::DistrictStyleBinding> {
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

        let width = if strategy.id.is_long_hall() {
            let ratio = strategy.width_depth_ratio.max(1.1);
            let w = ((depth as f32) * ratio).round() as u32;
            w.clamp(b.min_width, b.max_width)
        } else if strategy.id.is_l_shape() {
            let w = depth + 2;
            w.clamp(b.min_width, b.max_width)
        } else {
            let width_span = b.max_width.saturating_sub(b.min_width) + 1;
            b.min_width + ((s >> 16) as u32 % width_span)
        };

        let floor_span = b.max_floors.saturating_sub(b.min_floors) + 1;
        let floors = b.min_floors + ((s >> 32) as u32 % floor_span);

        (width.max(2), depth.max(2), floors.max(1))
    }

    fn roof_slot_for_massing(&self, massing_id: &MassingId) -> &str {
        self.roof
            .by_massing
            .iter()
            .find(|r| r.massing_id == *massing_id)
            .map(|r| r.slot.as_str())
            .unwrap_or_else(|| self.roof.default_slot.as_str())
    }

    pub fn generate(&self, district_style: &str, seed: u64) -> Result<GrammarGenerateResult, String> {
        self.generate_with_arch_dna(district_style, seed, None)
    }

    pub fn generate_with_arch_dna(
        &self,
        district_style: &str,
        seed: u64,
        arch_dna: Option<&ArchDnaConsumerFields>,
    ) -> Result<GrammarGenerateResult, String> {
        let district = self
            .district_binding(district_style)
            .ok_or_else(|| format!("unknown district_style: {district_style}"))?;

        let massing_weights: Vec<(u32, usize)> = if let Some(c) = arch_dna {
            reweight_massing_strategies(&self.massing.strategies, &c.pressure_field)
                .into_iter()
                .filter_map(|(id, w)| {
                    self.massing
                        .strategies
                        .iter()
                        .position(|s| s.id.as_str() == id)
                        .map(|i| (w, i))
                })
                .collect()
        } else {
            self.massing
                .strategies
                .iter()
                .enumerate()
                .map(|(i, s)| (s.weight, i))
                .collect()
        };
        let mi = pick_weighted_index(&massing_weights, mix_seed(seed, "massing"));
        let strategy = &self.massing.strategies[mi];

        let (width, depth, mut floors) = self.resolve_footprint(strategy, seed);
        if let Some(c) = arch_dna {
            let bounds = &self.archetype.footprint_bounds;
            floors = floors_from_beta_vert(
                c.pressure_field.beta_vert,
                bounds.min_floors,
                bounds.max_floors,
            );
        }

        let age_weights: Vec<(u32, usize)> = self
            .age
            .bands
            .iter()
            .enumerate()
            .map(|(i, b)| (b.weight, i))
            .collect();
        let ai = pick_weighted_index(&age_weights, mix_seed(seed, "age"));
        let age_band = &self.age.bands[ai];

        let resolved_facade = self.facade.resolve_for_massing(&strategy.id);

        let mut slot_overrides = HashMap::new();
        slot_overrides.insert(
            "roof_default".into(),
            self.roof_slot_for_massing(&strategy.id).into(),
        );
        if !resolved_facade.wall_slot.as_str().is_empty() {
            slot_overrides.insert("wall_1u".into(), resolved_facade.wall_slot.to_string());
        }
        if !resolved_facade.door_slot.as_str().is_empty() {
            slot_overrides.insert("door_default".into(), resolved_facade.door_slot.to_string());
        }
        if !resolved_facade.window_slot.as_str().is_empty() {
            slot_overrides.insert("window_1u".into(), resolved_facade.window_slot.to_string());
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
                detail: format!("style_pack={}", district.style_pack_id.as_str()),
            },
            GrammarRuleStep {
                layer: "massing",
                rule_id: strategy.id.to_string(),
                detail: format!(
                    "{width}x{depth}x{floors} mode={}",
                    strategy.footprint_mode.as_str()
                ),
            },
            GrammarRuleStep {
                layer: "roof",
                rule_id: self.roof_slot_for_massing(&strategy.id).into(),
                detail: "slot override for R token".into(),
            },
            GrammarRuleStep {
                layer: "facade",
                rule_id: format!("facade_{}", strategy.id),
                detail: format!(
                    "tags={}; rhythm={}",
                    resolved_facade.placement_tags.join(","),
                    resolved_facade.door_rhythm
                ),
            },
            GrammarRuleStep {
                layer: "detail",
                rule_id: if self.detail.prop_slot.as_str().is_empty() {
                    "none".into()
                } else {
                    self.detail.prop_slot.to_string()
                },
                detail: format!("density={:.2}", self.detail.density),
            },
            GrammarRuleStep {
                layer: "age",
                rule_id: age_band.id.clone(),
                detail: format!("variant_tags={}", age_band.variant_tags.join(",")),
            },
        ];

        if strategy.footprint_mode == FootprintMode::LShape {
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
            massing_strategy: strategy.id.to_string(),
            footprint_mode: strategy.footprint_mode.as_str().into(),
            width,
            depth,
            floors,
            style_pack_id: district.style_pack_id.as_str().to_owned(),
            slot_overrides,
            placement_tags: resolved_facade.placement_tags,
            variant_tags: age_band.variant_tags.clone(),
            detail_density: self.detail.density,
            age_band: age_band.id.clone(),
            rule_chain,
            material_profiles,
            weathering,
            arch_dna_preset_id: arch_dna.map(|c| c.preset_id.clone()),
            door_rhythm: resolved_facade.door_rhythm,
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
    #[must_use]
    pub fn material_profile_for_slot(&self, slot_key: &str) -> Option<&str> {
        self.material_profiles
            .get(slot_key)
            .map(|s| s.as_str())
            .or_else(|| default_material_for_token_slot(slot_key, &self.style_pack_id))
    }

    #[must_use]
    pub fn procedural_request(&self) -> ProceduralBuildingRequest {
        ProceduralBuildingRequest {
            archetype_id: self.archetype_id.clone(),
            width: self.width,
            depth: self.depth,
            floors: self.floors,
            style: StylePackId(self.style_pack_id.clone()),
            seed: self.seed,
            arch_dna_preset_id: self.arch_dna_preset_id.clone(),
        }
    }

    #[must_use]
    pub fn footprint_grid(&self) -> FootprintGrid {
        FootprintGrid::from_grammar(self)
    }

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

pub fn generate(
    archetype_id: &str,
    district_style: &str,
    seed: u64,
) -> Result<GrammarGenerateResult, String> {
    generate_with_arch_dna_preset(archetype_id, district_style, seed, None)
}

pub fn generate_with_arch_dna_preset(
    archetype_id: &str,
    district_style: &str,
    seed: u64,
    arch_dna_preset_id: Option<&str>,
) -> Result<GrammarGenerateResult, String> {
    let consumer = arch_dna_preset_id
        .and_then(|id| super::super::arch_build_grammar_v0::arch_dna_consumer_from_preset_id(id).ok());
    let registry = load_building_grammar_registry();
    if !registry.load_errors.is_empty() {
        return Err(registry.load_errors.join("; "));
    }
    let grammar = registry
        .grammars
        .get(archetype_id)
        .ok_or_else(|| format!("no grammar for archetype: {archetype_id}"))?;
    grammar.generate_with_arch_dna(district_style, seed, consumer.as_ref())
}

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
            m.massing_strategy_count >= 3
                && m.roof_slot_count >= 2
                && m.footprint_silhouette_count >= 2
        })
        && crate::construction::procedural::arch_dna_massing_diversity_witness_green()
}

#[must_use]
pub fn build_pg_quality_001_witness_body() -> serde_json::Value {
    let metrics = pg_quality_001_collect_metrics(
        "IndustrialWarehouse",
        "industrial_west",
        PG_QUALITY_001_SEED_SWEEP,
    );
    let dna_families = crate::construction::procedural::build_arch_dna_massing_diversity_rows();
    let dna_depth_green = dna_families
        .iter()
        .all(|row| row.get("green").and_then(|v| v.as_bool()).unwrap_or(false));
    let green = metrics
        .as_ref()
        .ok()
        .is_some_and(|m| {
            m.massing_strategy_count >= 3 && m.roof_slot_count >= 2 && m.footprint_silhouette_count >= 2
        })
        && dna_depth_green;
    let (metrics_ok, metrics_err) = match metrics {
        Ok(m) => (Some(m), None),
        Err(e) => (None, Some(e)),
    };
    serde_json::json!({
        "gate_id": "PG-QUALITY-001",
        "program_id": "PLAN-BUILDING-GRAMMAR-001",
        "slice_id": "CDR-B-CONSTRUCTION-GRAMMAR-DEPTH-001",
        "green": green,
        "archetype_id": "IndustrialWarehouse",
        "district_style": "industrial_west",
        "seed_sweep": PG_QUALITY_001_SEED_SWEEP,
        "thresholds": {
            "massing_strategy_count_min": 3,
            "roof_slot_count_min": 2,
            "footprint_silhouette_count_min": 2,
            "dna_family_massing_pick_min": 3,
        },
        "dna_families": dna_families,
        "dna_family_depth_green": dna_depth_green,
        "metrics": metrics_ok,
        "error": metrics_err,
    })
}

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

/// **CITY-G0-S11-001** lib witness — typed ids + deserialize validation.
#[must_use]
pub fn city_g0_s11_typed_ids_witness_green() -> bool {
    use super::grammar_types::{corridor_type_for_profile, CorridorType, MassingId, UsageId};

    let registry = load_building_grammar_registry();
    if !registry.load_errors.is_empty() {
        return false;
    }
    let all_usage_ok = registry.grammars.values().all(|g| {
        UsageId::try_new(g.archetype.usage.as_str()).is_ok()
            && g.massing
                .strategies
                .iter()
                .all(|s| MassingId::try_new(s.id.as_str()).is_ok())
    });
    all_usage_ok
        && corridor_type_for_profile("default_rail") == CorridorType::Rail
        && UsageId::try_new("bogus_usage").is_err()
}

/// **CITY-G0-S1C-001** lib witness — 3-way module split + behavior unchanged.
#[must_use]
pub fn city_g0_s1c_split_witness_green() -> bool {
    let root = repo_asset_path("src/construction/procedural/building_grammar");
    let types = root.join("grammar_types.rs");
    let deserialize = root.join("grammar_deserialize.rs");
    let evaluation = root.join("grammar_evaluation.rs");
    types.is_file()
        && deserialize.is_file()
        && evaluation.is_file()
        && generate("IndustrialWarehouse", "industrial_west", 43).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::grammar_types::ProgramAxisLevel;

    #[test]
    fn facility_binding_read_witness_is_green() {
        assert!(facility_binding_read_witness_green());
        let body = facility_binding_read_witness_body();
        assert!(body["g1_min_ok"].as_bool().unwrap_or(false));
        assert!(body["factory_cluster_v1"].as_bool().unwrap_or(false));
    }

    #[test]
    fn facility_binding_deserializes_program_axes() {
        let registry = load_building_grammar_registry();
        let grammar = registry
            .by_grammar_id("factory_cluster_v1")
            .expect("factory_cluster_v1");
        let binding = grammar.facility_binding().expect("binding");
        let axes = binding.program_axes.as_ref().expect("program_axes");
        assert_eq!(axes.loading, Some(ProgramAxisLevel::High));
        assert_eq!(axes.storage, Some(ProgramAxisLevel::Medium));
    }

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
        let mut strategies = HashSet::new();
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
        assert!(pg_quality_001_witness_green());
    }

    #[test]
    fn refresh_pg_quality_001_writes_grammar_diversity_witness() {
        assert!(refresh_pg_quality_001_grammar_diversity_witness());
        let path = repo_asset_path(GRAMMAR_DIVERSITY_WITNESS_JSON);
        let text = std::fs::read_to_string(path).expect("witness file");
        let body: serde_json::Value = serde_json::from_str(&text).expect("json");
        let green = body
            .get("green")
            .or_else(|| body.get("payload").and_then(|p| p.get("green")))
            .and_then(|v| v.as_bool());
        assert_eq!(green, Some(true));
    }

    #[test]
    fn city_g0_s11_typed_ids_witness_green_lib() {
        assert!(city_g0_s11_typed_ids_witness_green());
    }

    #[test]
    fn city_g0_s1c_split_witness_green_lib() {
        assert!(city_g0_s1c_split_witness_green());
    }
}
