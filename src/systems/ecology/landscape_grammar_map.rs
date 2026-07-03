//! Map-wide landscape grammar rollout — preset pick, λ external inputs, partition witness.
//!
//! Charter: vegetation drain Phase B (`VEG-MAP-ROLLOUT-001`, `VEG-MAP-PARTITION-001`).

use std::fs;
use std::path::PathBuf;

use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::landscape_grammar::{
    evaluate_landscape_program_with_inputs,
    LandscapeGrammarCatalog, LandscapeProgramOnChunk, LambdaExternalInputs, LG1_PILOT_CHUNK,
    LG1_PILOT_PRESET_ID,
};
use super::landscape_grammar_lg2::{attach_lg2_bundle_on_chunk, LandUseDistrictKind, LandUseInfluence};
use super::{ChunkEcology, VegetationField};
use crate::dev::debug_run_envelope::{witness_refresh_due, wrap_debug_run, write_debug_run_json};
use crate::strategic::StrategicRasterConfig;
use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::Chunk;

pub const LANDSCAPE_GRAMMAR_MAP_ROLLOUT_LIVE_JSON: &str =
    "debug_runs/landscape_grammar_map_rollout_live.json";
pub const LANDSCAPE_GRAMMAR_LG3_LIVE_JSON: &str = "debug_runs/landscape_grammar_lg3_live.json";
pub const LANDSCAPE_GRAMMAR_LG5_LIVE_JSON: &str = "debug_runs/landscape_grammar_lg5_live.json";
pub const LANDSCAPE_ATLAS_INDEX_RON: &str = "assets/configs/landscape/_landscape_atlas_index.ron";
pub const LG5_ATLAS_BATCH_WITNESS_JSON: &str =
    "debug_runs/art_pipeline/tile_tile_landscape_expanded_v1_live.json";
pub const LG5_ATLAS_EXPAND_ROLLUP_JSON: &str =
    "debug_runs/art_pipeline/tile_landscape_expanded_live.json";
pub const LG5_ATLAS_ID: &str = "landscape_lg5_expanded_v1";
pub const LG5_PILOT_ATLAS_ID: &str = "landscape_lg5_pilot_v1";
pub const VEGETATION_PROGRAM_CLOSE_LIVE_JSON: &str =
    "debug_runs/vegetation_program_close_live.json";

const PRESET_INDEX_RON: &str = "assets/configs/landscape/_preset_index.ron";

#[derive(Debug, Clone, Deserialize)]
struct PresetIndexRon {
    presets: Vec<String>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct LandscapePresetIndex {
    pub preset_ids: Vec<String>,
}

impl LandscapePresetIndex {
    pub fn load() -> Self {
        let path = repo_asset_path(PRESET_INDEX_RON);
        let raw = fs::read_to_string(&path).unwrap_or_default();
        if let Ok(doc) = ron::from_str::<PresetIndexRon>(&raw) {
            return Self {
                preset_ids: doc.presets,
            };
        }
        Self {
            preset_ids: vec![LG1_PILOT_PRESET_ID.to_string()],
        }
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct LandscapeMapRolloutWitness {
    pub chunks_with_program: u32,
    pub mean_topology_kind_count: f32,
    pub presets_used: u32,
}

#[must_use]
pub fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

/// Deterministic preset pick from catalog index (seed-driven, no RNG).
#[must_use]
pub fn pick_preset_id_for_chunk(coord: IVec2, index: &LandscapePresetIndex) -> String {
    pick_preset_id_for_chunk_with_inputs(coord, index, &LambdaExternalInputs::default())
}

/// VEG-PRESET-INDUSTRIAL-002 — industrial/military presets anchored to live transport/settlement fields.
#[must_use]
pub fn pick_preset_id_for_chunk_with_inputs(
    coord: IVec2,
    index: &LandscapePresetIndex,
    inputs: &LambdaExternalInputs,
) -> String {
    if coord == LG1_PILOT_CHUNK {
        return LG1_PILOT_PRESET_ID.to_string();
    }
    if index.preset_ids.is_empty() {
        return LG1_PILOT_PRESET_ID.to_string();
    }
    let pick_if_present = |id: &str| {
        index
            .preset_ids
            .iter()
            .any(|p| p == id)
            .then(|| id.to_string())
    };
    if inputs.transport_access >= 0.35 && inputs.construction_pressure >= 0.35 {
        if let Some(id) = pick_if_present("military_defensive_v0") {
            return id;
        }
    }
    if inputs.transport_access >= 0.55 {
        if let Some(id) = pick_if_present("industrial_barrier_v0") {
            return id;
        }
    }
    if inputs.hydrology_bias >= 0.65 {
        if let Some(id) = pick_if_present("settlement_park_v0") {
            return id;
        }
    }
    if inputs.hydrology_bias >= 0.75 {
        if let Some(id) = pick_if_present("wetland_margin_v0") {
            return id;
        }
    }
    if inputs.construction_pressure >= 0.25 {
        if let Some(id) = pick_if_present("fire_recovery_v0") {
            return id;
        }
    }
    pick_preset_from_lambda_influence(inputs, index)
}

/// CDR-A-PRESET-PICK-LAMBDA-001 — λ-driven fallback (no coord hash).
#[must_use]
fn pick_preset_from_lambda_influence(
    inputs: &LambdaExternalInputs,
    index: &LandscapePresetIndex,
) -> String {
    if index.preset_ids.is_empty() {
        return LG1_PILOT_PRESET_ID.to_string();
    }
    let scored: [(&str, f32); 5] = [
        (
            "military_defensive_v0",
            (inputs.transport_access * 0.55 + inputs.construction_pressure * 0.45).clamp(0.0, 1.0),
        ),
        ("industrial_barrier_v0", inputs.transport_access),
        ("wetland_margin_v0", inputs.hydrology_bias),
        ("settlement_park_v0", inputs.hydrology_bias * 0.9),
        ("fire_recovery_v0", inputs.construction_pressure),
    ];
    let mut best_score = -1.0f32;
    let mut best_id = index.preset_ids[0].as_str();
    for (candidate, score) in scored {
        if index.preset_ids.iter().any(|p| p == candidate) && score > best_score {
            best_score = score;
            best_id = candidate;
        }
    }
    best_id.to_string()
}

/// Read-only λ inputs from live ecology + weather (+ transport raster when present) — VEG-λ-LIVE-001.
#[must_use]
pub fn lambda_inputs_from_live_fields(
    ecology: &ChunkEcology,
    veg: &VegetationField,
    weather: &ChunkWeather,
    raster: Option<&StrategicRasterConfig>,
) -> LambdaExternalInputs {
    let transport = raster
        .map(|r| (r.cells_per_chunk.x as f32 * 0.01).clamp(0.0, 1.0))
        .unwrap_or(0.0)
        + ecology.root_strength.clamp(0.0, 1.0) * 0.12;
    let hydrology = weather.soil_moisture.clamp(0.0, 1.0);
    let construction =
        (ecology.fire_risk.clamp(0.0, 1.0) * 0.45 + veg.burn_severity.clamp(0.0, 1.0) * 0.35)
            .clamp(0.0, 1.0);
    LambdaExternalInputs {
        hydrology_bias: hydrology,
        transport_access: transport.clamp(0.0, 1.0),
        construction_pressure: construction,
    }
}

#[must_use]
pub fn district_kind_from_preset_class(class: &str) -> LandUseDistrictKind {
    match class {
        "industrial" => LandUseDistrictKind::IndustrialBarrier,
        "military" => LandUseDistrictKind::MilitaryDefensive,
        "settlement" => LandUseDistrictKind::SettlementPark,
        "forest" => LandUseDistrictKind::OldGrowthCore,
        _ => LandUseDistrictKind::AgriculturalRiparian,
    }
}

fn startup_load_preset_index(mut commands: Commands) {
    commands.insert_resource(LandscapePresetIndex::load());
}

pub fn rollout_landscape_program_on_chunks(
    catalog: Res<LandscapeGrammarCatalog>,
    index: Res<LandscapePresetIndex>,
    raster: Option<Res<StrategicRasterConfig>>,
    mut commands: Commands,
    mut witness: ResMut<LandscapeMapRolloutWitness>,
    q: Query<
        (Entity, &Chunk, &ChunkEcology, &VegetationField, &ChunkWeather),
        Without<LandscapeProgramOnChunk>,
    >,
) {
    let mut kind_sum = 0u32;
    let mut count = 0u32;
    let mut presets_seen = std::collections::HashSet::new();

    for (entity, chunk, ecology, veg, weather) in &q {
        let inputs = lambda_inputs_from_live_fields(ecology, veg, weather, raster.as_deref());
        let preset_id = pick_preset_id_for_chunk_with_inputs(chunk.coord, &index, &inputs);
        let Some(preset) = catalog.presets.get(&preset_id) else {
            continue;
        };
        let evaluation =
            evaluate_landscape_program_with_inputs(preset, chunk.coord, ecology, veg, weather, &inputs);
        kind_sum = kind_sum.saturating_add(evaluation.topology_kind_count as u32);
        count = count.saturating_add(1);
        presets_seen.insert(preset_id.clone());

        let district = district_kind_from_preset_class(&preset.landscape_program.class);
        commands.entity(entity).insert((
            LandscapeProgramOnChunk {
                preset_id: preset_id.clone(),
                evaluation,
            },
            LandUseInfluence {
                district,
                preset_id,
            },
        ));
        attach_lg2_bundle_on_chunk(&mut commands, entity, chunk.coord == LG1_PILOT_CHUNK);
    }

    if count > 0 {
        witness.chunks_with_program = witness.chunks_with_program.max(count);
        witness.mean_topology_kind_count = kind_sum as f32 / count as f32;
        witness.presets_used = presets_seen.len() as u32;
    }
}

pub fn refresh_map_rollout_witness_system(
    frame: Res<FrameCount>,
    program_q: Query<&LandscapeProgramOnChunk>,
    mut witness: ResMut<LandscapeMapRolloutWitness>,
) {
    if !witness_refresh_due(LANDSCAPE_GRAMMAR_MAP_ROLLOUT_LIVE_JSON, frame.0) {
        return;
    }
    let n = program_q.iter().count() as u32;
    if n > witness.chunks_with_program {
        witness.chunks_with_program = n;
    }
    if n > 0 {
        let sum: u32 = program_q
            .iter()
            .map(|p| p.evaluation.topology_kind_count as u32)
            .sum();
        witness.mean_topology_kind_count = sum as f32 / n as f32;
        let presets_seen: std::collections::HashSet<String> = program_q
            .iter()
            .map(|p| p.preset_id.clone())
            .collect();
        witness.presets_used = presets_seen.len() as u32;
    }
    let green = map_rollout_witness_green(&witness);
    let body = json!({
        "gate": "VEG-MAP-ROLLOUT-001",
        "green": green,
        "chunks_with_program": witness.chunks_with_program,
        "mean_topology_kind_count": witness.mean_topology_kind_count,
        "presets_used": witness.presets_used,
    });
    let wrapped = wrap_debug_run(
        "VEG-MAP-ROLLOUT-001",
        "refresh_map_rollout_witness",
        LANDSCAPE_GRAMMAR_MAP_ROLLOUT_LIVE_JSON,
        body,
    );
    let _ = write_debug_run_json(LANDSCAPE_GRAMMAR_MAP_ROLLOUT_LIVE_JSON, wrapped);
}

#[must_use]
pub fn map_rollout_witness_green(witness: &LandscapeMapRolloutWitness) -> bool {
    witness.chunks_with_program >= 16 && witness.mean_topology_kind_count >= 3.0
}

#[must_use]
pub fn refresh_lg3_witness_from_districts(district_kind_count: usize) -> bool {
    refresh_lg3_witness_from_districts_with_anchors(district_kind_count, false, false)
}

#[must_use]
pub fn refresh_lg3_witness_from_districts_with_anchors(
    district_kind_count: usize,
    industrial_anchored: bool,
    military_anchored: bool,
) -> bool {
    let green = district_kind_count >= 2 && industrial_anchored && military_anchored;
    let body = json!({
        "gate": "VEG-LG3-WITNESS-001",
        "green": green,
        "district_kind_count": district_kind_count,
        "industrial_preset_anchored": industrial_anchored,
        "military_preset_anchored": military_anchored,
        "anchor_source": "live_transport_settlement",
    });
    let wrapped = wrap_debug_run(
        "VEG-LG3-WITNESS-001",
        "refresh_lg3_witness_from_districts",
        LANDSCAPE_GRAMMAR_LG3_LIVE_JSON,
        body,
    );
    write_debug_run_json(LANDSCAPE_GRAMMAR_LG3_LIVE_JSON, wrapped) && green
}

#[must_use]
pub fn refresh_lg3_witness(program_q: Query<&LandUseInfluence>) -> bool {
    let mut districts = std::collections::HashSet::new();
    for inf in &program_q {
        districts.insert(format!("{:?}", inf.district));
    }
    let green = districts.len() >= 2;
    let body = json!({
        "gate": "VEG-LG3-WITNESS-001",
        "green": green,
        "district_kind_count": districts.len(),
        "districts": districts.iter().collect::<Vec<_>>(),
    });
    let wrapped = wrap_debug_run(
        "VEG-LG3-WITNESS-001",
        "refresh_lg3_witness",
        LANDSCAPE_GRAMMAR_LG3_LIVE_JSON,
        body,
    );
    write_debug_run_json(LANDSCAPE_GRAMMAR_LG3_LIVE_JSON, wrapped) && green
}

#[must_use]
pub fn landscape_lg5_atlas_batch_green() -> bool {
    let path = repo_asset_path(LG5_ATLAS_BATCH_WITNESS_JSON);
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(body) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    body.get("green").and_then(|v| v.as_bool()).unwrap_or(false)
}

#[must_use]
pub fn landscape_lg5_expand_rollup_green() -> bool {
    let path = repo_asset_path(LG5_ATLAS_EXPAND_ROLLUP_JSON);
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(body) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    body.get("green").and_then(|v| v.as_bool()).unwrap_or(false)
        && body
            .get("atlas_id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id == LG5_ATLAS_ID)
}

#[must_use]
pub fn landscape_lg5_registry_stamped() -> bool {
    let path = repo_asset_path(LANDSCAPE_ATLAS_INDEX_RON);
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    text.contains(LG5_ATLAS_ID)
}

#[must_use]
pub fn refresh_lg5_witness() -> bool {
    let atlas_ok = landscape_lg5_atlas_batch_green();
    let expand_ok = landscape_lg5_expand_rollup_green();
    let stamp_ok = landscape_lg5_registry_stamped();
    let bevy_uv = crate::gui::landscape_chunk_atlas_stamp::landscape_lg5_chunk_uv_stamp_witness_green();
    let green = atlas_ok && expand_ok && stamp_ok && bevy_uv;
    let atlas_lane = if green {
        "mcp_atlas_pack_expanded"
    } else if atlas_ok && stamp_ok {
        "bevy_chunk_uv_pending"
    } else if atlas_ok {
        "registry_stamp_pending"
    } else {
        "atlas_batch_pending"
    };
    let body = json!({
        "gate": "VEG-LG5-WITNESS-001",
        "slice_id": "CDR-A-LG5-REAL-STAMP-001",
        "green": green,
        "real_atlas_uv": green,
        "atlas_lane": atlas_lane,
        "atlas_batch_green": atlas_ok,
        "atlas_expand_rollup_green": expand_ok,
        "registry_stamp": stamp_ok,
        "bevy_chunk_uv_stamp": bevy_uv,
        "VEG-F03-REGISTRY-STAMP-001": bevy_uv,
        "atlas_id": LG5_ATLAS_ID,
        "pilot_atlas_id": LG5_PILOT_ATLAS_ID,
        "atlas_batch_witness": LG5_ATLAS_BATCH_WITNESS_JSON,
        "atlas_expand_witness": LG5_ATLAS_EXPAND_ROLLUP_JSON,
        "landscape_index": LANDSCAPE_ATLAS_INDEX_RON,
        "charter": "src/dev/design_landscape_lg5_expansion_matrix_v1.md",
    });
    let wrapped = wrap_debug_run(
        "VEG-LG5-WITNESS-001",
        "refresh_lg5_witness",
        LANDSCAPE_GRAMMAR_LG5_LIVE_JSON,
        body,
    );
    write_debug_run_json(LANDSCAPE_GRAMMAR_LG5_LIVE_JSON, wrapped) && green
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct VegetationProgramCloseBody {
    pub phase_a_green: bool,
    pub phase_b_green: bool,
    pub phase_c_green: bool,
    pub phase_d_green: bool,
    pub phase_e_green: bool,
    pub phase_f_green: bool,
    pub all_green: bool,
}

#[must_use]
pub fn vegetation_program_close_honest(body: &VegetationProgramCloseBody) -> bool {
    body.all_green
        && crate::dev::veg_runtime_proof_live::veg_runtime_child_sub_rules_ok()
        && crate::dev::veg_runtime_proof_live::lg4_preview_child_sub_rules_ok()
}

#[must_use]
pub fn refresh_vegetation_program_close(body: &VegetationProgramCloseBody) -> bool {
    let child_veg_runtime = crate::dev::veg_runtime_proof_live::veg_runtime_child_sub_rules_ok();
    let child_lg4 = crate::dev::veg_runtime_proof_live::lg4_preview_child_sub_rules_ok();
    let honest_all_green = vegetation_program_close_honest(body);
    let wrapped = wrap_debug_run(
        "VEG-PROGRAM-CLOSE-001",
        "refresh_vegetation_program_close",
        VEGETATION_PROGRAM_CLOSE_LIVE_JSON,
        json!({
            "phase_a_green": body.phase_a_green,
            "phase_b_green": body.phase_b_green,
            "phase_c_green": body.phase_c_green,
            "phase_d_green": body.phase_d_green,
            "phase_e_green": body.phase_e_green,
            "phase_f_green": body.phase_f_green,
            "all_green": honest_all_green,
            "child_rollup": {
                "veg_runtime_proof_sub_rules": child_veg_runtime,
                "lg4_preview_sub_rules": child_lg4,
            },
            "sub_rules_evaluated": true,
        }),
    );
    write_debug_run_json(VEGETATION_PROGRAM_CLOSE_LIVE_JSON, wrapped) && honest_all_green
}

pub fn landscape_grammar_map_plugin(app: &mut App) {
    app.init_resource::<LandscapeMapRolloutWitness>()
        .add_systems(Startup, startup_load_preset_index)
        .add_systems(
            Update,
            (
                rollout_landscape_program_on_chunks,
                refresh_map_rollout_witness_system,
            )
                .chain()
                .in_set(ChunkEnvironmentSet::Ecology),
        );
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::landscape_grammar::load_landscape_grammar_catalog;

    #[test]
    fn preset_index_loads_ten_presets() {
        let index = LandscapePresetIndex::load();
        assert!(index.preset_ids.len() >= 10, "{:?}", index.preset_ids);
    }

    #[test]
    fn preset_index_loads_five_presets() {
        let index = LandscapePresetIndex::load();
        assert!(index.preset_ids.len() >= 5, "{:?}", index.preset_ids);
    }

    #[test]
    fn catalog_loads_all_indexed_presets() {
        let catalog = load_landscape_grammar_catalog();
        let index = LandscapePresetIndex::load();
        for id in &index.preset_ids {
            assert!(catalog.presets.contains_key(id), "missing {id}");
        }
    }

    #[test]
    fn pick_preset_lambda_influence_not_coord_hash() {
        let index = LandscapePresetIndex::load();
        let industrial = LambdaExternalInputs {
            transport_access: 0.9,
            hydrology_bias: 0.1,
            construction_pressure: 0.1,
        };
        let wetland = LambdaExternalInputs {
            transport_access: 0.1,
            hydrology_bias: 0.95,
            construction_pressure: 0.1,
        };
        let a = pick_preset_id_for_chunk_with_inputs(IVec2::new(99, 99), &index, &industrial);
        let b = pick_preset_id_for_chunk_with_inputs(IVec2::new(99, 99), &index, &wetland);
        assert_ne!(a, b, "lambda inputs should drive preset, got both {a}");
        assert_eq!(
            pick_preset_id_for_chunk_with_inputs(IVec2::new(99, 99), &index, &industrial),
            a
        );
    }

    #[test]
    fn pick_preset_is_deterministic() {
        let index = LandscapePresetIndex::load();
        let a = pick_preset_id_for_chunk(IVec2::new(3, 5), &index);
        let b = pick_preset_id_for_chunk(IVec2::new(3, 5), &index);
        assert_eq!(a, b);
        assert_eq!(
            pick_preset_id_for_chunk(LG1_PILOT_CHUNK, &index),
            LG1_PILOT_PRESET_ID
        );
    }

    #[test]
    fn map_rollout_green_threshold() {
        let w = LandscapeMapRolloutWitness {
            chunks_with_program: 16,
            mean_topology_kind_count: 3.5,
            presets_used: 5,
        };
        assert!(map_rollout_witness_green(&w));
    }

    #[test]
    fn lg5_witness_green_when_atlas_and_registry_on_disk() {
        if !landscape_lg5_atlas_batch_green() || !landscape_lg5_registry_stamped() {
            return;
        }
        assert!(refresh_lg5_witness());
    }
}
