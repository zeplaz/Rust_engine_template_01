//! `assembly_snapshot_v1` manifest — AUTO-001 contract shared with MCP tile pipeline.
//!
//! Engine PG-2 emits the same JSON shape as `rust_engine_mcp.assembly.generate_assembly_snapshot`.
//! Blender import stays in coder-mcp (`assembly_build` job).

use std::path::PathBuf;

use bevy::prelude::Vec3;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::arch_build_grammar_v0::{
    arch_dna_consumer_from_preset_id, arch_dna_consumer_from_snapshot_fields,
    arch_dna_consumer_wired, ArchDnaConsumerFields, PressureFieldV0,
};
use super::building_grammar::generate_with_arch_dna_preset;
use super::{
    grammar_reference_tags, DevelopmentTier, FootprintGrid, FootprintToken,
    GrammarGenerateResult, ProceduralBuildingRequest, ProceduralModuleRegistry,
    StylePack, StylePackRegistry, StylePackSlotKey, GRAMMAR_RULES_VERSION,
};

pub const ASSEMBLY_SNAPSHOT_SCHEMA: u32 = 1;
pub const PROCEDURAL_RULES_VERSION: &str = "pg2_wdc_v1";
pub const ASSEMBLY_SNAPSHOT_STAGING: &str = "assets/staging/assemblies";

/// One resolved module row in AUTO-001 `module_placements`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssemblyModulePlacement {
    pub module_id: String,
    pub job_id: String,
    pub slot_key: String,
    pub token: String,
    pub grid_x: u32,
    pub grid_y: u32,
    pub floor: u32,
    pub glb_path: String,
    pub position: [f64; 3],
    pub rotation_euler: [f64; 3],
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub material_profile: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weathering: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssemblyFootprintSnapshot {
    pub width: u32,
    pub depth: u32,
    pub floors: u32,
    pub wdc_cell_count: u32,
}

/// Flattened grammar chain for APS inspector (matches `assembly_snapshot_v1.grammar_rule_chain`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AssemblyGrammarRuleChain {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archetype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub massing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roof: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facade: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footprint_mode: Option<String>,
}

/// AUTO-001 assembly snapshot body (serde field names match JSON schema).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssemblySnapshot {
    pub schema_version: u32,
    pub assembly_id: String,
    pub style_pack_id: String,
    pub source_tier: String,
    pub procedural_rules_version: String,
    pub reference_tags: Vec<String>,
    pub seed: u64,
    pub footprint: AssemblyFootprintSnapshot,
    pub module_placements: Vec<AssemblyModulePlacement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archetype_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub district_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grammar_rule_chain: Option<AssemblyGrammarRuleChain>,
    /// BUILD-READ-CONSUMER-MCP-001 — APS snapshot DNA preset id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch_build_grammar_preset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch_build_grammar_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch_dna: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pressure_field: Option<PressureFieldV0>,
}

impl AssemblySnapshot {
    /// Resolve MCP consumer fields from embedded snapshot DNA+β (commit replay path).
    #[must_use]
    pub fn arch_dna_consumer(&self) -> Option<ArchDnaConsumerFields> {
        let preset_id = self.arch_build_grammar_preset_id.as_deref()?;
        arch_dna_consumer_from_snapshot_fields(
            preset_id,
            self.arch_build_grammar_id.clone(),
            self.arch_dna.clone().unwrap_or(serde_json::Value::Null),
            self.pressure_field,
        )
    }

    #[must_use]
    pub fn arch_dna_consumer_wired(&self) -> bool {
        self.arch_dna_consumer()
            .is_some_and(|c| arch_dna_consumer_wired(&c))
    }
}

#[must_use]
pub fn grammar_rule_chain_snapshot(result: &GrammarGenerateResult) -> AssemblyGrammarRuleChain {
    let mut chain = AssemblyGrammarRuleChain {
        footprint_mode: Some(result.footprint_mode.clone()),
        ..AssemblyGrammarRuleChain::default()
    };
    for step in &result.rule_chain {
        match step.layer {
            "archetype" if chain.archetype.is_none() => chain.archetype = Some(step.rule_id.clone()),
            "massing" if chain.massing.is_none() => chain.massing = Some(step.rule_id.clone()),
            "roof" if chain.roof.is_none() => chain.roof = Some(step.rule_id.clone()),
            "facade" if chain.facade.is_none() => chain.facade = Some(step.rule_id.clone()),
            "detail" if chain.detail.is_none() => chain.detail = Some(step.rule_id.clone()),
            "age" if chain.age.is_none() => chain.age = Some(step.rule_id.clone()),
            _ => {}
        }
    }
    chain
}

#[must_use]
fn slot_key_for_token(token: FootprintToken) -> Option<StylePackSlotKey> {
    match token {
        FootprintToken::Wall => Some(StylePackSlotKey::Wall1u),
        FootprintToken::Door => Some(StylePackSlotKey::DoorDefault),
        FootprintToken::Corner => Some(StylePackSlotKey::CornerOuter),
        FootprintToken::Roof => Some(StylePackSlotKey::RoofDefault),
        FootprintToken::Yard => None,
    }
}

/// Local 1 m grid placement for PG-2 module instances (matches AUTO-001 `position`).
#[must_use]
pub fn procedural_module_local_translation(x: u32, y: u32, floor: u32) -> Vec3 {
    Vec3::new(x as f32, (floor * 3) as f32, y as f32)
}

#[must_use]
fn grid_to_position(x: u32, y: u32, floor: u32) -> [f64; 3] {
    let v = procedural_module_local_translation(x, y, floor);
    [f64::from(v.x), f64::from(v.y), f64::from(v.z)]
}

/// Deterministic id — must match `rust_engine_mcp.assembly._assembly_id`.
#[must_use]
pub fn assembly_id_for(style_pack_id: &str, width: u32, depth: u32, floors: u32, seed: u64) -> String {
    let raw = format!("{style_pack_id}:{width}x{depth}x{floors}:s{seed}");
    let digest = Sha256::digest(raw.as_bytes());
    let hex: String = digest.iter().take(2).map(|b| format!("{b:02x}")).collect();
    let pack_suffix = style_pack_id.strip_prefix("style_").unwrap_or(style_pack_id);
    format!("{pack_suffix}_{width}x{depth}_s{seed}_{hex}")
}

#[must_use]
pub fn build_assembly_snapshot(
    request: &ProceduralBuildingRequest,
    style_pack: &StylePack,
    grid: &FootprintGrid,
    registry: &ProceduralModuleRegistry,
) -> AssemblySnapshot {
    build_assembly_snapshot_with_grammar(request, style_pack, grid, None, registry)
}

/// Grammar-first snapshot: evaluate `generate` → footprint grid → slot overrides → placements.
pub fn build_assembly_snapshot_from_grammar(
    archetype_id: &str,
    district_style: &str,
    seed: u64,
    registry: &ProceduralModuleRegistry,
    packs: &StylePackRegistry,
) -> Result<AssemblySnapshot, String> {
    build_assembly_snapshot_from_grammar_with_preset(
        archetype_id,
        district_style,
        seed,
        None,
        registry,
        packs,
    )
}

/// Grammar-first snapshot with optional ARCH-DNA preset consumer on commit path.
pub fn build_assembly_snapshot_from_grammar_with_preset(
    archetype_id: &str,
    district_style: &str,
    seed: u64,
    arch_dna_preset_id: Option<&str>,
    registry: &ProceduralModuleRegistry,
    packs: &StylePackRegistry,
) -> Result<AssemblySnapshot, String> {
    let grammar = generate_with_arch_dna_preset(
        archetype_id,
        district_style,
        seed,
        arch_dna_preset_id,
    )?;
    let request = grammar.procedural_request();
    let pack = packs
        .get(request.style.0.as_str())
        .ok_or_else(|| format!("missing style pack: {}", request.style.0))?;
    let grid = grammar.footprint_grid();
    Ok(build_assembly_snapshot_with_grammar(
        &request,
        pack,
        &grid,
        Some(&grammar),
        registry,
    ))
}

#[must_use]
fn build_assembly_snapshot_with_grammar(
    request: &ProceduralBuildingRequest,
    style_pack: &StylePack,
    grid: &FootprintGrid,
    grammar: Option<&GrammarGenerateResult>,
    registry: &ProceduralModuleRegistry,
) -> AssemblySnapshot {
    let style_pack_id = style_pack.id.as_str().to_owned();
    let mut placements = Vec::new();
    let mut source_tier = "lod0".to_owned();

    for cell in grid.facade_cells() {
        let Some(slot_key) = slot_key_for_token(cell.token) else {
            continue;
        };
        let Some(schema_token) = cell.token.as_schema_token() else {
            continue;
        };
        let slot_name = slot_key.ron_key();
        let effective_slot = grammar
            .and_then(|g| g.slot_overrides.get(slot_name))
            .map(|s| s.as_str())
            .unwrap_or(slot_name);
        let Some(raw_module_id) = style_pack.resolve_slot_str(effective_slot) else {
            continue;
        };
        let Some(entry) = registry.resolve_module_id(raw_module_id) else {
            continue;
        };
        if entry.development_tier.is_smoke() || entry.batch_id.starts_with("kit_greybox") {
            continue;
        }
        if entry.development_tier == DevelopmentTier::Production {
            source_tier = "production".to_owned();
        }
        let material_profile = grammar
            .and_then(|g| g.material_profile_for_slot(effective_slot))
            .map(str::to_owned)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| entry.material_profile.clone());
        let weathering = grammar.map(|g| g.weathering.clone());
        placements.push(AssemblyModulePlacement {
            module_id: entry.module_id.clone(),
            job_id: entry.job_id.clone(),
            slot_key: effective_slot.to_owned(),
            token: schema_token.to_owned(),
            grid_x: cell.x,
            grid_y: cell.y,
            floor: cell.floor,
            glb_path: entry.glb_path.clone(),
            position: grid_to_position(cell.x, cell.y, cell.floor),
            rotation_euler: [0.0, 0.0, 0.0],
            material_profile,
            weathering,
        });
    }

    let (procedural_rules_version, reference_tags, archetype_id, district_style, grammar_rule_chain) =
        if let Some(g) = grammar {
            (
                GRAMMAR_RULES_VERSION.to_owned(),
                grammar_reference_tags(g),
                Some(g.archetype_id.clone()),
                Some(g.district_style.clone()),
                Some(grammar_rule_chain_snapshot(g)),
            )
        } else {
            (
                PROCEDURAL_RULES_VERSION.to_owned(),
                Vec::new(),
                None,
                None,
                None,
            )
        };

    let (arch_build_grammar_preset_id, arch_build_grammar_id, arch_dna, pressure_field) =
        if let Some(g) = grammar {
            if let Some(preset_id) = g.arch_dna_preset_id.as_deref() {
                if let Ok(consumer) = arch_dna_consumer_from_preset_id(preset_id) {
                    (
                        Some(consumer.preset_id),
                        consumer.grammar_id,
                        Some(consumer.arch_dna),
                        Some(consumer.pressure_field),
                    )
                } else {
                    (None, None, None, None)
                }
            } else {
                (None, None, None, None)
            }
        } else if let Some(preset_id) = request.arch_dna_preset_id.as_deref() {
            if let Ok(consumer) = arch_dna_consumer_from_preset_id(preset_id) {
                (
                    Some(consumer.preset_id),
                    consumer.grammar_id,
                    Some(consumer.arch_dna),
                    Some(consumer.pressure_field),
                )
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };

    AssemblySnapshot {
        schema_version: ASSEMBLY_SNAPSHOT_SCHEMA,
        assembly_id: assembly_id_for(
            &style_pack_id,
            grid.width,
            grid.depth,
            grid.floors,
            request.seed,
        ),
        style_pack_id,
        source_tier,
        procedural_rules_version,
        reference_tags,
        seed: request.seed,
        footprint: AssemblyFootprintSnapshot {
            width: grid.width,
            depth: grid.depth,
            floors: grid.floors,
            wdc_cell_count: grid.wdc_cell_count(),
        },
        module_placements: placements,
        archetype_id,
        district_style,
        grammar_rule_chain,
        arch_build_grammar_preset_id,
        arch_build_grammar_id,
        arch_dna,
        pressure_field,
    }
}

#[must_use]
pub fn default_staging_dir() -> PathBuf {
    repo_asset_path(ASSEMBLY_SNAPSHOT_STAGING)
}

#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

/// Write `{assembly_id}.json` under `assets/staging/assemblies/`.
pub fn write_assembly_snapshot(snapshot: &AssemblySnapshot) -> std::io::Result<PathBuf> {
    let dir = default_staging_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.json", snapshot.assembly_id));
    let text = serde_json::to_string_pretty(snapshot).map_err(std::io::Error::other)?;
    std::fs::write(&path, text)?;
    Ok(path)
}

#[must_use]
pub fn staging_relative_path(snapshot: &AssemblySnapshot) -> String {
    format!(
        "{}/{}.json",
        ASSEMBLY_SNAPSHOT_STAGING, snapshot.assembly_id
    )
}

/// **CITY-G0-WIT-001** — fixed grammar→assembly spec for determinism witness.
pub const CITY_G0_WIT_ARCHETYPE: &str = "IndustrialWarehouse";
pub const CITY_G0_WIT_DISTRICT: &str = "industrial_west";
pub const CITY_G0_WIT_SEED: u64 = 43;
pub const CITY_G0_WIT_RUNS: u32 = 3;
pub const CITY_G0_WIT_LIVE_JSON: &str = "debug_runs/city_g0_wit_001_live.json";

/// Stable SHA-256 hex over canonical JSON bytes (field order = struct definition).
#[must_use]
pub fn assembly_snapshot_stable_hash(snapshot: &AssemblySnapshot) -> String {
    let bytes = serde_json::to_vec(snapshot).unwrap_or_default();
    let digest = Sha256::digest(&bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[must_use]
pub fn build_city_g0_wit_001_witness_body() -> serde_json::Value {
    use super::building_grammar::{
        city_g0_s11_typed_ids_witness_green, city_g0_s1c_split_witness_green,
    };

    let modules = super::load_procedural_module_registry();
    let packs = super::load_style_pack_registry();
    let registry_ok = modules.load_errors.is_empty() && packs.load_errors.is_empty();

    let mut run_hashes = Vec::new();
    let mut contract_ok = false;
    for _ in 0..CITY_G0_WIT_RUNS {
        match build_assembly_snapshot_from_grammar(
            CITY_G0_WIT_ARCHETYPE,
            CITY_G0_WIT_DISTRICT,
            CITY_G0_WIT_SEED,
            &modules,
            &packs,
        ) {
            Ok(snapshot) => {
                contract_ok = snapshot_passes_auto_001_contract(&snapshot);
                run_hashes.push(assembly_snapshot_stable_hash(&snapshot));
            }
            Err(err) => {
                return serde_json::json!({
                    "gate": "CITY-G0-WIT-001",
                    "green": false,
                    "error": err,
                    "registry_ok": registry_ok,
                });
            }
        }
    }

    let three_run_stable = run_hashes.len() == CITY_G0_WIT_RUNS as usize
        && run_hashes.windows(2).all(|w| w[0] == w[1]);

    let seed_sensitivity_ok = build_assembly_snapshot_from_grammar(
        CITY_G0_WIT_ARCHETYPE,
        CITY_G0_WIT_DISTRICT,
        CITY_G0_WIT_SEED.wrapping_add(1),
        &modules,
        &packs,
    )
    .ok()
    .map(|s| assembly_snapshot_stable_hash(&s))
    .is_some_and(|h| run_hashes.first().is_some_and(|base| h != *base));

    let s11 = city_g0_s11_typed_ids_witness_green();
    let s1c = city_g0_s1c_split_witness_green();
    let green = registry_ok && three_run_stable && contract_ok && seed_sensitivity_ok && s11 && s1c;

    serde_json::json!({
        "gate": "CITY-G0-WIT-001",
        "issue": "CITY-G0c",
        "green": green,
        "spec": {
            "archetype_id": CITY_G0_WIT_ARCHETYPE,
            "district_style": CITY_G0_WIT_DISTRICT,
            "seed": CITY_G0_WIT_SEED,
            "consecutive_runs": CITY_G0_WIT_RUNS,
        },
        "registry_ok": registry_ok,
        "auto_001_contract": contract_ok,
        "three_run_stable": three_run_stable,
        "seed_sensitivity_ok": seed_sensitivity_ok,
        "stable_hash": run_hashes.first(),
        "run_hashes": run_hashes,
        "g0_prerequisites": {
            "city_g0_s11": s11,
            "city_g0_s1c": s1c,
        },
    })
}

/// **CITY-G0-WIT-001** lib witness — same seed ⇒ identical assembly snapshot hash (3 runs).
#[must_use]
pub fn city_g0_wit_001_determinism_witness_green() -> bool {
    build_city_g0_wit_001_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Write `debug_runs/city_g0_wit_001_live.json`.
#[must_use]
pub fn refresh_city_g0_wit_001_grammar_determinism_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_city_g0_wit_001_witness_body();
    let green = body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let wrapped = wrap_debug_run(
        "CITY-G0-WIT-001",
        "refresh_city_g0_wit_001_grammar_determinism_witness",
        CITY_G0_WIT_LIVE_JSON,
        body,
    );
    write_debug_run_json(CITY_G0_WIT_LIVE_JSON, wrapped) && green
}

/// Required AUTO-001 keys present and placements non-empty.
#[must_use]
pub fn snapshot_passes_auto_001_contract(snapshot: &AssemblySnapshot) -> bool {
    snapshot.schema_version == ASSEMBLY_SNAPSHOT_SCHEMA
        && !snapshot.assembly_id.is_empty()
        && snapshot.style_pack_id.starts_with("style_")
        && snapshot.footprint.width >= 2
        && snapshot.footprint.depth >= 2
        && snapshot.footprint.floors >= 1
        && !snapshot.module_placements.is_empty()
        && snapshot
            .module_placements
            .iter()
            .all(|p| !p.module_id.is_empty() && !p.glb_path.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::procedural::{
        build_assembly_snapshot_from_grammar, load_procedural_module_registry, load_style_pack_registry,
        ProceduralBuildingRequest, StylePackId, GRAMMAR_RULES_VERSION,
    };

    fn victorian_request(width: u32, depth: u32, seed: u64) -> ProceduralBuildingRequest {
        ProceduralBuildingRequest {
            archetype_id: "rect_perimeter".into(),
            width,
            depth,
            floors: 2,
            style: StylePackId("style_victorian".into()),
            seed,
            arch_dna_preset_id: None,
        }
    }

    #[test]
    fn assembly_id_matches_mcp_sha256_contract() {
        assert_eq!(
            assembly_id_for("style_victorian", 4, 3, 2, 42),
            "victorian_4x3_s42_a7cb"
        );
    }

    #[test]
    fn assembly_snapshot_auto_001_shape_victorian_4x3_s42() {
        let modules = load_procedural_module_registry();
        assert!(modules.load_errors.is_empty(), "{:?}", modules.load_errors);
        let packs = load_style_pack_registry();
        let pack = packs.get("style_victorian").expect("style_victorian");
        let request = victorian_request(4, 3, 42);
        let grid = FootprintGrid::from_request(&request);
        let snapshot = build_assembly_snapshot(&request, pack, &grid, &modules);
        assert_eq!(snapshot.assembly_id, "victorian_4x3_s42_a7cb");
        assert_eq!(snapshot.procedural_rules_version, PROCEDURAL_RULES_VERSION);
        assert_eq!(snapshot.module_placements.len(), 30);
        assert!(snapshot_passes_auto_001_contract(&snapshot));
        let first = &snapshot.module_placements[0];
        assert_eq!(first.token, "C");
        assert_eq!(first.slot_key, "corner_outer");
        assert!(first.glb_path.contains("corner_L_lod0_run001"));
    }

    #[test]
    fn assembly_snapshot_grammar_wire_industrial_warehouse_s43() {
        let modules = load_procedural_module_registry();
        assert!(modules.load_errors.is_empty(), "{:?}", modules.load_errors);
        let packs = load_style_pack_registry();
        let snapshot = build_assembly_snapshot_from_grammar(
            "IndustrialWarehouse",
            "industrial_west",
            43,
            &modules,
            &packs,
        )
        .expect("grammar snapshot");
        assert_eq!(snapshot.procedural_rules_version, GRAMMAR_RULES_VERSION);
        assert_eq!(
            snapshot.archetype_id.as_deref(),
            Some("IndustrialWarehouse")
        );
        assert_eq!(snapshot.district_style.as_deref(), Some("industrial_west"));
        assert!(snapshot.grammar_rule_chain.is_some());
        assert!(
            snapshot
                .reference_tags
                .iter()
                .any(|t| t.starts_with("grammar:"))
        );
        assert!(snapshot_passes_auto_001_contract(&snapshot));
        assert!(snapshot.assembly_id.contains("industrial_west"));
        assert!(snapshot.assembly_id.contains("_s43_"));
        assert!(
            snapshot
                .module_placements
                .iter()
                .all(|p| !p.material_profile.is_empty()),
            "PG-MATERIAL-GENERATION-001: grammar must emit material_profile per placement"
        );
        assert!(
            snapshot
                .module_placements
                .iter()
                .all(|p| p.weathering.as_deref().is_some_and(|w| !w.is_empty()))
        );
    }

    #[test]
    fn grammar_snapshot_roof_slot_override_differs_by_massing() {
        let modules = load_procedural_module_registry();
        let packs = load_style_pack_registry();
        let a = build_assembly_snapshot_from_grammar(
            "IndustrialWarehouse",
            "industrial_west",
            0,
            &modules,
            &packs,
        )
        .expect("seed 0");
        let b = build_assembly_snapshot_from_grammar(
            "IndustrialWarehouse",
            "industrial_west",
            50,
            &modules,
            &packs,
        )
        .expect("seed 50");
        let mass_a = a.grammar_rule_chain.as_ref().and_then(|c| c.massing.clone());
        let mass_b = b.grammar_rule_chain.as_ref().and_then(|c| c.massing.clone());
        assert_ne!(mass_a, mass_b);
        let roof_a = a
            .module_placements
            .iter()
            .find(|p| p.token == "R")
            .map(|p| p.slot_key.as_str());
        let roof_b = b
            .module_placements
            .iter()
            .find(|p| p.token == "R")
            .map(|p| p.slot_key.as_str());
        assert_ne!(roof_a, roof_b);
    }

    #[test]
    fn city_g0_wit_001_assembly_snapshot_hash_stable_three_runs() {
        assert!(city_g0_wit_001_determinism_witness_green());
        let body = build_city_g0_wit_001_witness_body();
        assert!(body["three_run_stable"].as_bool().unwrap_or(false));
        let hashes = body["run_hashes"].as_array().expect("run_hashes");
        assert_eq!(hashes.len(), CITY_G0_WIT_RUNS as usize);
        assert!(body["seed_sensitivity_ok"].as_bool().unwrap_or(false));
    }

    #[test]
    fn write_assembly_snapshot_staging_roundtrip() {
        let modules = load_procedural_module_registry();
        let packs = load_style_pack_registry();
        let pack = packs.get("style_victorian").unwrap();
        let request = victorian_request(5, 3, 99);
        let grid = FootprintGrid::from_request(&request);
        let snapshot = build_assembly_snapshot(&request, pack, &grid, &modules);
        let path = write_assembly_snapshot(&snapshot).expect("write snapshot");
        assert!(path.is_file());
        let text = std::fs::read_to_string(&path).expect("read snapshot");
        assert!(!text.is_empty(), "snapshot file must not be empty");
        let loaded: AssemblySnapshot = serde_json::from_str(&text).unwrap();
        assert_eq!(loaded.assembly_id, snapshot.assembly_id);
        assert_eq!(loaded.module_placements.len(), snapshot.module_placements.len());
    }
}
