//! ARCH-BUILD-GRAMMAR-v0 — DNA + pressure β massing re-rank (BUILD-READ-GRAMMAR-v0-003).
//!
//! Schema: `src/dev/arch_build_grammar_v0_baseline_v1.md` · example preset JSON.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::building_grammar::MassingStrategy;

pub const ARCH_GRAMMAR_V0_PRESET_JSON: &str =
    "tools/mcp/schemas/examples/arch_dna_logistics_rail_warehouse_v0.json";
pub const ARCH_DNA_EXAMPLES_DIR: &str = "tools/mcp/schemas/examples";

const K_BIAS: f32 = 40.0;

/// v0 pressure field (8 β keys).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub struct PressureFieldV0 {
    #[serde(rename = "beta_sym")]
    pub beta_sym: f32,
    #[serde(rename = "beta_irr")]
    pub beta_irr: f32,
    #[serde(rename = "beta_yard")]
    pub beta_yard: f32,
    #[serde(rename = "beta_svc")]
    pub beta_svc: f32,
    #[serde(rename = "beta_mod")]
    pub beta_mod: f32,
    #[serde(rename = "beta_exp")]
    pub beta_exp: f32,
    #[serde(rename = "beta_vert")]
    pub beta_vert: f32,
    #[serde(rename = "beta_roof")]
    pub beta_roof: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SiteZoneStub {
    pub id: String,
    pub role: String,
    #[serde(default)]
    pub footprint_hint: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArchGrammarV0Preset {
    pub schema_version: String,
    pub preset_id: String,
    pub grammar_id: String,
    #[serde(default)]
    pub arch_dna: serde_json::Value,
    pub pressure_field: PressureFieldV0,
    #[serde(default)]
    pub site_zones: Vec<SiteZoneStub>,
    #[serde(default)]
    pub massing_weight_override: Vec<MassingWeightOverride>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MassingWeightOverride {
    pub id: String,
    pub weight: u32,
}

struct MassingBias {
    yard: f32,
    svc: f32,
    sym: f32,
    exp: f32,
    irr: f32,
}

fn massing_bias(id: &str) -> MassingBias {
    match id {
        "long_hall" => MassingBias {
            yard: 0.0,
            svc: 0.0,
            sym: 1.0,
            exp: 0.0,
            irr: 0.0,
        },
        "double_hall" => MassingBias {
            yard: 0.0,
            svc: 0.0,
            sym: 0.8,
            exp: 0.2,
            irr: 0.0,
        },
        "l_shape" => MassingBias {
            yard: 0.6,
            svc: 1.0,
            sym: 0.0,
            exp: 0.3,
            irr: 0.8,
        },
        "yard_complex" => MassingBias {
            yard: 1.0,
            svc: 0.5,
            sym: 0.0,
            exp: 1.0,
            irr: 0.4,
        },
        _ => MassingBias {
            yard: 0.0,
            svc: 0.0,
            sym: 0.5,
            exp: 0.0,
            irr: 0.0,
        },
    }
}

/// Re-rank base massing strategy weights using v0 β formula (deterministic integer weights, sum 100).
#[must_use]
pub fn reweight_massing_strategies(
    strategies: &[MassingStrategy],
    pressure: &PressureFieldV0,
) -> Vec<(String, u32)> {
    let mut scored: Vec<(String, f32)> = strategies
        .iter()
        .map(|s| {
            let b = massing_bias(&s.id);
            let score = s.weight as f32
                + K_BIAS * pressure.beta_yard * b.yard
                + K_BIAS * pressure.beta_svc * b.svc
                + K_BIAS * pressure.beta_sym * b.sym
                + K_BIAS * pressure.beta_exp * b.exp
                + K_BIAS * pressure.beta_irr * b.irr;
            (s.id.clone(), score.max(0.0))
        })
        .collect();

    let total: f32 = scored.iter().map(|(_, w)| *w).sum();
    if total <= 0.0 {
        return strategies
            .iter()
            .map(|s| (s.id.clone(), s.weight.max(1)))
            .collect();
    }

    let mut out: Vec<(String, u32)> = scored
        .drain(..)
        .map(|(id, w)| (id, ((w / total) * 100.0).round() as u32))
        .collect();
    let sum: u32 = out.iter().map(|(_, w)| *w).sum();
    if sum != 100 {
        let delta = 100i32 - sum as i32;
        if let Some((_, w)) = out.first_mut() {
            *w = (*w as i32 + delta).max(1) as u32;
        }
    }
    out
}

/// Floors cap from βvert (v0 baseline §4).
#[must_use]
pub fn floors_from_beta_vert(
    beta_vert: f32,
    min_floors: u32,
    max_floors: u32,
) -> u32 {
    let raw = 1.0 + beta_vert.clamp(0.0, 1.0) * 3.0;
    raw.round().clamp(min_floors as f32, max_floors as f32) as u32
}

/// Shift+scale expansion link — βexp biases effective scale multiplier.
#[must_use]
pub fn expansion_scale_multiplier(beta_exp: f32) -> f32 {
    0.6 + 0.5 * beta_exp.clamp(0.0, 1.0)
}

#[must_use]
fn repo_path(rel: &str) -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| std::path::PathBuf::from(rel))
}

#[must_use]
pub fn load_preset_from_path(path: &Path) -> Result<ArchGrammarV0Preset, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

#[must_use]
pub fn preset_path_for_id(preset_id: &str) -> PathBuf {
    repo_path(&format!("{ARCH_DNA_EXAMPLES_DIR}/arch_dna_{preset_id}.json"))
}

#[must_use]
pub fn list_arch_dna_preset_ids() -> Vec<String> {
    let dir = repo_path(ARCH_DNA_EXAMPLES_DIR);
    let Ok(read) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut ids: Vec<String> = read
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_string();
            let rest = name.strip_prefix("arch_dna_")?;
            let id = rest.strip_suffix(".json")?;
            Some(id.to_string())
        })
        .collect();
    ids.sort_unstable();
    ids
}

#[must_use]
pub fn load_logistics_rail_warehouse_v0_preset() -> Result<ArchGrammarV0Preset, String> {
    load_preset_for_id("logistics_rail_warehouse_v0")
}

#[must_use]
pub fn load_preset_for_id(preset_id: &str) -> Result<ArchGrammarV0Preset, String> {
    load_preset_from_path(&preset_path_for_id(preset_id))
}

#[must_use]
pub fn site_zones_for_preset(preset_id: &str) -> Vec<SiteZoneStub> {
    load_preset_for_id(preset_id)
        .map(|p| p.site_zones)
        .unwrap_or_default()
}

/// BUILD-READ-CONSUMER-MCP-001 — snapshot DNA+β consumer fields (APS → Rust).
#[derive(Debug, Clone)]
pub struct ArchDnaConsumerFields {
    pub preset_id: String,
    pub grammar_id: Option<String>,
    pub arch_dna: serde_json::Value,
    pub pressure_field: PressureFieldV0,
}

/// MCP `consumer_contract.rust_consumer` entry point — preset JSON on disk.
#[must_use]
pub fn load_arch_dna_preset(preset_id: &str) -> Result<ArchGrammarV0Preset, String> {
    load_preset_for_id(preset_id)
}

#[must_use]
pub fn arch_dna_consumer_from_preset_id(preset_id: &str) -> Result<ArchDnaConsumerFields, String> {
    let preset = load_preset_for_id(preset_id)?;
    Ok(ArchDnaConsumerFields {
        preset_id: preset.preset_id,
        grammar_id: Some(preset.grammar_id),
        arch_dna: preset.arch_dna,
        pressure_field: preset.pressure_field,
    })
}

#[must_use]
pub fn arch_dna_consumer_from_snapshot_fields(
    preset_id: &str,
    grammar_id: Option<String>,
    arch_dna: serde_json::Value,
    pressure_field: Option<PressureFieldV0>,
) -> Option<ArchDnaConsumerFields> {
    let pressure = pressure_field.or_else(|| {
        load_preset_for_id(preset_id)
            .ok()
            .map(|p| p.pressure_field)
    })?;
    if arch_dna.is_null() || !arch_dna.as_object().is_some_and(|o| !o.is_empty()) {
        return None;
    }
    Some(ArchDnaConsumerFields {
        preset_id: preset_id.to_owned(),
        grammar_id,
        arch_dna,
        pressure_field: pressure,
    })
}

#[must_use]
pub fn arch_dna_consumer_wired(fields: &ArchDnaConsumerFields) -> bool {
    fields.arch_dna.as_object().is_some_and(|o| !o.is_empty())
        && fields.pressure_field.beta_yard > 0.0
}

/// BUILD-READ-CONSUMER-MCP-001 — Rust consumer wired on commit + snapshot roundtrip.
#[must_use]
pub fn build_read_consumer_mcp_001_witness_green() -> bool {
    build_read_consumer_mcp_001_self_check().is_ok()
}

fn build_read_consumer_mcp_001_self_check() -> Result<(), &'static str> {
    use super::assembly_snapshot::build_assembly_snapshot_from_grammar_with_preset;
    use super::load_procedural_module_registry;
    use super::load_style_pack_registry;

    let consumer = arch_dna_consumer_from_preset_id("logistics_rail_warehouse_v0")
        .map_err(|_| "consumer_load")?;
    if !arch_dna_consumer_wired(&consumer) {
        return Err("consumer_wired");
    }

    let modules = load_procedural_module_registry();
    if !modules.load_errors.is_empty() {
        return Err("modules");
    }
    let packs = load_style_pack_registry();
    if !packs.load_errors.is_empty() {
        return Err("packs");
    }

    let snapshot = build_assembly_snapshot_from_grammar_with_preset(
        "IndustrialWarehouse",
        "industrial_west",
        440013,
        Some("logistics_rail_warehouse_v0"),
        &modules,
        &packs,
    )
    .map_err(|_| "snapshot")?;

    if snapshot.arch_build_grammar_preset_id.as_deref()
        != Some("logistics_rail_warehouse_v0")
    {
        return Err("preset_on_snapshot");
    }
    if !snapshot.arch_dna_consumer_wired() {
        return Err("snapshot_consumer");
    }

    let grammar = super::building_grammar::generate_with_arch_dna_preset(
        "IndustrialWarehouse",
        "industrial_west",
        440013,
        Some("logistics_rail_warehouse_v0"),
    )
    .map_err(|_| "grammar")?;
    if grammar.arch_dna_preset_id.as_deref() != Some("logistics_rail_warehouse_v0") {
        return Err("preset_on_grammar");
    }

    Ok(())
}

/// BUILD-GRAMMAR-PROGRAM-001 — read-only ProgramGraph v1 stub from ARCH-DNA + β.
#[derive(Debug, Clone)]
pub struct ProgramGraphStubV1 {
    pub preset_id: String,
    pub site_zones: Vec<SiteZoneStub>,
    pub pressure_field: PressureFieldV0,
}

#[must_use]
pub fn program_graph_stub_for_preset(preset_id: &str) -> Option<ProgramGraphStubV1> {
    let preset = load_preset_for_id(preset_id).ok()?;
    Some(ProgramGraphStubV1 {
        preset_id: preset.preset_id,
        site_zones: preset.site_zones,
        pressure_field: preset.pressure_field,
    })
}

/// BUILD-GRAMMAR-β-WORLD-001 — blend preset β with live transport edge density (read-only).
#[must_use]
pub fn beta_with_world_transport_bias(
    preset: PressureFieldV0,
    transport_edge_count: u32,
) -> PressureFieldV0 {
    let access = (transport_edge_count as f32 * 0.04).clamp(0.0, 1.0);
    PressureFieldV0 {
        beta_yard: (preset.beta_yard + access * 0.12).clamp(0.0, 1.0),
        beta_sym: (preset.beta_sym + access * 0.08).clamp(0.0, 1.0),
        ..preset
    }
}

/// BUILD-READ-GRAMMAR-v0-003 — preset row for grammar diversity witness.
#[must_use]
pub fn build_read_grammar_v0_003_witness_body() -> serde_json::Value {
    let preset = load_logistics_rail_warehouse_v0_preset();
    let (ok, _preset_id, weights, pick_l_shape) = match preset {
        Ok(p) => {
            let registry = super::load_building_grammar_registry();
            let grammar = registry
                .grammars
                .values()
                .find(|g| g.grammar_id == p.grammar_id);
            let weights = grammar.map(|g| {
                reweight_massing_strategies(&g.massing.strategies, &p.pressure_field)
            });
            let top = weights.as_ref().and_then(|w| {
                w.iter()
                    .max_by_key(|(_, wt)| *wt)
                    .map(|(id, _)| id.clone())
            });
            let pick_l = top.as_deref() == Some("l_shape")
                || weights
                    .as_ref()
                    .and_then(|w| w.iter().find(|(id, _)| id == "l_shape"))
                    .map(|(_, wt)| *wt >= 26)
                    .unwrap_or(false);
            (true, p.preset_id, weights, pick_l)
        }
        Err(_e) => (false, String::new(), None, false),
    };
    serde_json::json!({
        "gate_id": "BUILD-READ-GRAMMAR-v0-003",
        "preset_id": "logistics_rail_warehouse_v0",
        "green": ok && pick_l_shape,
        "massing_weights_v0": weights,
        "rail_edge_picks_l_shape_or_yard": pick_l_shape,
        "beta_exp_multiplier": expansion_scale_multiplier(0.84),
        "load_ok": ok,
    })
}

#[must_use]
pub fn build_read_grammar_v0_003_witness_green() -> bool {
    build_read_grammar_v0_003_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logistics_rail_warehouse_v0_reweights_massing() {
        let preset = load_logistics_rail_warehouse_v0_preset().expect("preset json");
        let registry = super::super::load_building_grammar_registry();
        let grammar = registry
            .grammars
            .get("IndustrialWarehouse")
            .expect("industrial grammar");
        let weights = reweight_massing_strategies(&grammar.massing.strategies, &preset.pressure_field);
        let l = weights.iter().find(|(id, _)| id == "l_shape").map(|(_, w)| *w);
        assert!(l.unwrap_or(0) >= 26, "l_shape weight should rank high: {weights:?}");
        let sum: u32 = weights.iter().map(|(_, w)| *w).sum();
        assert_eq!(sum, 100);
    }

    #[test]
    fn build_read_grammar_v0_003_witness_green() {
        assert!(super::build_read_grammar_v0_003_witness_green());
    }

    #[test]
    fn beta_exp_expansion_multiplier_in_range() {
        let m = expansion_scale_multiplier(0.84);
        assert!((m - 1.02).abs() < 0.05);
    }
}
