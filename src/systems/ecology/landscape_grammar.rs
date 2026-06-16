//! LG-1 — landscape grammar evaluator (topology graph + read-only λ blend + planning glyph witness).
//!
//! Charter: `src/dev/plan_landscape_grammar_exec_001_v1.md` §3.
//! Pilot preset: `agri_riparian_v0` on chunk `(12, 0)`.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{ChunkEcology, VegetationField};
use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::Chunk;

pub const LANDSCAPE_PRESETS_DIR: &str = "assets/configs/landscape/presets";
pub const LG1_PILOT_PRESET_ID: &str = "agri_riparian_v0";
pub const LG1_PILOT_CHUNK: IVec2 = IVec2::new(12, 0);
pub const LANDSCAPE_GRAMMAR_LG1_LIVE_JSON: &str = "debug_runs/landscape_grammar_lg1_live.json";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LandDnaV0 {
    pub H: String,
    pub S: String,
    pub E: String,
    pub T: String,
    pub D: String,
    pub L: String,
    pub A: String,
    pub M: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub struct PressureFieldV0 {
    pub lambda_moisture: f32,
    pub lambda_slope: f32,
    pub lambda_exposure: f32,
    pub lambda_disturbance: f32,
    pub lambda_access: f32,
    pub lambda_security: f32,
    pub lambda_productivity: f32,
    pub lambda_legibility: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LandscapeProgramV0 {
    pub class: String,
    #[serde(default)]
    pub district_ref: Option<String>,
    #[serde(default)]
    pub required_topologies: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopologyNodeV0 {
    pub id: String,
    pub topology_kind: String,
    #[serde(default)]
    pub topology_kind_id: Option<String>,
    pub preset_id: String,
    #[serde(default)]
    pub scale_band: Option<String>,
    #[serde(default)]
    pub anchor_ref: Option<String>,
    #[serde(default)]
    pub anchor_node_label: Option<String>,
    #[serde(default)]
    pub parent_topology_id: Option<String>,
    #[serde(default)]
    pub operator_stack: Vec<String>,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default)]
    pub children: Vec<TopologyNodeV0>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FlatTopologyNode {
    pub id: String,
    pub topology_kind: String,
    pub preset_id: String,
    pub depth: usize,
    pub planning_glyph: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LandscapeGrammarPreset {
    pub schema_version: String,
    pub preset_id: String,
    pub chart_id: String,
    pub land_dna: LandDnaV0,
    pub pressure_field: PressureFieldV0,
    pub landscape_program: LandscapeProgramV0,
    pub topology_graph: Vec<TopologyNodeV0>,
    /// MACRO-* composition recipes (lexicon §1.17) — expanded before flatten.
    #[serde(default)]
    pub composition_macros: Vec<String>,
}

#[derive(Resource, Debug, Default)]
pub struct LandscapeGrammarCatalog {
    pub presets: HashMap<String, LandscapeGrammarPreset>,
    pub load_errors: Vec<String>,
}

#[derive(Component, Debug, Clone)]
pub struct LandscapeProgramOnChunk {
    pub preset_id: String,
    pub evaluation: LandscapeProgramEvaluation,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanningGlyphOverlay {
    pub node_id: String,
    pub topology_kind: String,
    pub preset_id: String,
    pub glyph: String,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LandscapeProgramEvaluation {
    pub preset_id: String,
    pub chart_id: String,
    pub chunk_coord: [i32; 2],
    pub topology_kind_count: usize,
    pub nested_depth_max: usize,
    pub flat_node_count: usize,
    pub topology_kinds: Vec<String>,
    pub preset_lambda: PressureFieldV0,
    pub effective_lambda: PressureFieldV0,
    pub lambda_blended: bool,
    pub planning_glyph_overlays: Vec<PlanningGlyphOverlay>,
    pub required_topologies_met: bool,
    pub missing_required_topologies: Vec<String>,
}

#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn lambda_differs(a: PressureFieldV0, b: PressureFieldV0, eps: f32) -> bool {
    (a.lambda_moisture - b.lambda_moisture).abs() > eps
        || (a.lambda_slope - b.lambda_slope).abs() > eps
        || (a.lambda_exposure - b.lambda_exposure).abs() > eps
        || (a.lambda_disturbance - b.lambda_disturbance).abs() > eps
        || (a.lambda_access - b.lambda_access).abs() > eps
        || (a.lambda_security - b.lambda_security).abs() > eps
        || (a.lambda_productivity - b.lambda_productivity).abs() > eps
        || (a.lambda_legibility - b.lambda_legibility).abs() > eps
}

#[must_use]
pub fn planning_glyph_for_node(node: &TopologyNodeV0) -> String {
    if let Some(glyph) = node
        .metadata
        .get("glyph_planning")
        .and_then(|v| v.as_str())
    {
        return glyph.to_string();
    }

    match (node.topology_kind.as_str(), node.preset_id.as_str()) {
        ("Corridor", "CORRIDOR_RIPARIAN") => "≈≈≈╬≈≈≈".into(),
        ("Ring", "RING_SHELTERBELT") => "(█)".into(),
        ("Network", _) => "◊═◊".into(),
        ("Patch", _) => "█▓▒".into(),
        ("Fringe", _) => "▒│".into(),
        ("Cluster", _) => "◊◊".into(),
        ("Line", _) => "──".into(),
        _ => "◈".into(),
    }
}

fn collect_topology_nodes(nodes: &[TopologyNodeV0], out: &mut Vec<TopologyNodeV0>) {
    for node in nodes {
        out.push(node.clone());
        collect_topology_nodes(&node.children, out);
    }
}

#[must_use]
fn parent_chain_depth(node: &TopologyNodeV0, by_id: &HashMap<&str, &TopologyNodeV0>) -> usize {
    let mut depth = 0usize;
    let mut parent = node.parent_topology_id.as_deref();
    while let Some(pid) = parent {
        depth += 1;
        parent = by_id
            .get(pid)
            .and_then(|n| n.parent_topology_id.as_deref());
    }
    depth
}

/// MACRO-* registry — each recipe appends a deterministic topology subgraph (VEG-COMPOSITE-EVAL-001).
#[must_use]
pub fn macro_topology_subgraph(macro_id: &str) -> Vec<TopologyNodeV0> {
    let node = |id: &str, kind: &str, preset: &str| TopologyNodeV0 {
        id: id.to_string(),
        topology_kind: kind.to_string(),
        topology_kind_id: None,
        preset_id: preset.to_string(),
        scale_band: None,
        anchor_ref: None,
        anchor_node_label: None,
        parent_topology_id: None,
        operator_stack: Vec::new(),
        metadata: Value::Null,
        children: Vec::new(),
    };
    match macro_id {
        "MACRO-RIPARIAN-AXIS" => vec![node("macro_riparian_axis", "Corridor", "CORRIDOR_RIPARIAN")],
        "MACRO-AG-PARCEL" => vec![node("macro_ag_parcel", "Patch", "PATCH_IRREGULAR")],
        "MACRO-SHELTER-LEE" => vec![node("macro_shelter_lee", "Ring", "RING_SHELTERBELT")],
        "MACRO-WIND-ALLEY" => vec![node("macro_wind_alley", "Corridor", "CORRIDOR_WIND")],
        "MACRO-REGROWTH-CHAIN" => vec![node("macro_regrowth", "Cluster", "CLUSTER_REGROWTH")],
        _ => Vec::new(),
    }
}

#[must_use]
pub fn effective_topology_graph(preset: &LandscapeGrammarPreset) -> Vec<TopologyNodeV0> {
    let mut graph = preset.topology_graph.clone();
    for macro_id in &preset.composition_macros {
        graph.extend(macro_topology_subgraph(macro_id));
    }
    graph
}

pub fn flatten_topology_graph(nodes: &[TopologyNodeV0], out: &mut Vec<FlatTopologyNode>) {
    let mut collected = Vec::new();
    collect_topology_nodes(nodes, &mut collected);
    let by_id: HashMap<&str, &TopologyNodeV0> = collected
        .iter()
        .map(|n| (n.id.as_str(), n))
        .collect();
    for node in &collected {
        out.push(FlatTopologyNode {
            id: node.id.clone(),
            topology_kind: node.topology_kind.clone(),
            preset_id: node.preset_id.clone(),
            depth: parent_chain_depth(node, &by_id),
            planning_glyph: planning_glyph_for_node(node),
        });
    }
}

/// Read-only external λ inputs (hydrology / transport / construction) — VEG-λ-INPUTS-001.
#[derive(Clone, Copy, Debug, Default)]
pub struct LambdaExternalInputs {
    pub hydrology_bias: f32,
    pub transport_access: f32,
    pub construction_pressure: f32,
}

/// Read-only λ blend from ecology + weather; never mutates source components.
#[must_use]
pub fn blend_lambda_readonly(
    preset: PressureFieldV0,
    ecology: &ChunkEcology,
    veg: &VegetationField,
    weather: &ChunkWeather,
) -> PressureFieldV0 {
    blend_lambda_with_inputs(
        preset,
        ecology,
        veg,
        weather,
        &LambdaExternalInputs::default(),
    )
}

/// Ecology + weather + read-only external field biases (never writes ECS).
#[must_use]
pub fn blend_lambda_with_inputs(
    preset: PressureFieldV0,
    ecology: &ChunkEcology,
    veg: &VegetationField,
    weather: &ChunkWeather,
    inputs: &LambdaExternalInputs,
) -> PressureFieldV0 {
    let mut base = PressureFieldV0 {
        lambda_moisture: clamp01(
            preset.lambda_moisture + weather.soil_moisture * 0.12 - veg.dryness * 0.05,
        ),
        lambda_slope: clamp01(preset.lambda_slope + ecology.root_strength * 0.08),
        lambda_exposure: clamp01(preset.lambda_exposure + veg.dryness * 0.15 + weather.wind_speed * 0.02),
        lambda_disturbance: clamp01(
            preset.lambda_disturbance + veg.burn_severity * 0.25 + ecology.fire_risk * 0.08,
        ),
        lambda_access: preset.lambda_access,
        lambda_security: preset.lambda_security,
        lambda_productivity: clamp01(preset.lambda_productivity + ecology.biomass * 0.1),
        lambda_legibility: clamp01(preset.lambda_legibility + veg.fragmentation * 0.05),
    };
    base.lambda_moisture = clamp01(base.lambda_moisture + inputs.hydrology_bias * 0.08);
    base.lambda_access = clamp01(base.lambda_access + inputs.transport_access * 0.12);
    base.lambda_disturbance = clamp01(base.lambda_disturbance + inputs.construction_pressure * 0.1);
    base
}

#[must_use]
pub fn evaluate_landscape_program(
    preset: &LandscapeGrammarPreset,
    chunk: IVec2,
    ecology: &ChunkEcology,
    veg: &VegetationField,
    weather: &ChunkWeather,
) -> LandscapeProgramEvaluation {
    evaluate_landscape_program_with_inputs(
        preset,
        chunk,
        ecology,
        veg,
        weather,
        &LambdaExternalInputs::default(),
    )
}

#[must_use]
pub fn evaluate_landscape_program_with_inputs(
    preset: &LandscapeGrammarPreset,
    chunk: IVec2,
    ecology: &ChunkEcology,
    veg: &VegetationField,
    weather: &ChunkWeather,
    inputs: &LambdaExternalInputs,
) -> LandscapeProgramEvaluation {
    let mut flat = Vec::new();
    let graph = effective_topology_graph(preset);
    flatten_topology_graph(&graph, &mut flat);

    let topology_kinds: Vec<String> = flat
        .iter()
        .map(|n| n.topology_kind.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let preset_ids: HashSet<&str> = flat.iter().map(|n| n.preset_id.as_str()).collect();
    let required = &preset.landscape_program.required_topologies;
    let missing_required_topologies: Vec<String> = required
        .iter()
        .filter(|id| !preset_ids.contains(id.as_str()))
        .cloned()
        .collect();

    let preset_lambda = preset.pressure_field;
    let effective_lambda = blend_lambda_with_inputs(preset_lambda, ecology, veg, weather, inputs);
    let lambda_blended = lambda_differs(preset_lambda, effective_lambda, 1e-4);

    let planning_glyph_overlays: Vec<PlanningGlyphOverlay> = flat
        .iter()
        .map(|n| PlanningGlyphOverlay {
            node_id: n.id.clone(),
            topology_kind: n.topology_kind.clone(),
            preset_id: n.preset_id.clone(),
            glyph: n.planning_glyph.clone(),
            depth: n.depth,
        })
        .collect();

    let nested_depth_max = flat.iter().map(|n| n.depth).max().unwrap_or(0);

    LandscapeProgramEvaluation {
        preset_id: preset.preset_id.clone(),
        chart_id: preset.chart_id.clone(),
        chunk_coord: [chunk.x, chunk.y],
        topology_kind_count: topology_kinds.len(),
        nested_depth_max,
        flat_node_count: flat.len(),
        topology_kinds,
        preset_lambda,
        effective_lambda,
        lambda_blended,
        planning_glyph_overlays,
        required_topologies_met: missing_required_topologies.is_empty(),
        missing_required_topologies,
    }
}

pub fn load_landscape_preset_from_path(path: &Path) -> Result<LandscapeGrammarPreset, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("{path:?}: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("{path:?}: {e}"))
}

pub fn load_landscape_grammar_catalog_from_dir(dir: &Path) -> LandscapeGrammarCatalog {
    let mut catalog = LandscapeGrammarCatalog::default();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            catalog
                .load_errors
                .push(format!("read_dir {dir:?}: {e}"));
            return catalog;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        match load_landscape_preset_from_path(&path) {
            Ok(preset) => {
                catalog.presets.insert(preset.preset_id.clone(), preset);
            }
            Err(err) => catalog.load_errors.push(err),
        }
    }
    catalog
}

#[must_use]
pub fn load_landscape_grammar_catalog() -> LandscapeGrammarCatalog {
    load_landscape_grammar_catalog_from_dir(&repo_asset_path(LANDSCAPE_PRESETS_DIR))
}

#[must_use]
pub fn lg1_evaluation_green(eval: &LandscapeProgramEvaluation) -> bool {
    eval.preset_id == LG1_PILOT_PRESET_ID
        && eval.topology_kind_count >= 4
        && eval.nested_depth_max >= 2
        && eval.required_topologies_met
        && eval.planning_glyph_overlays.len() >= 4
        && eval.lambda_blended
}

#[must_use]
pub fn build_lg1_witness_body(eval: &LandscapeProgramEvaluation) -> Value {
    let green = lg1_evaluation_green(eval);
    json!({
        "lane": "LG-1",
        "pilot_preset_id": LG1_PILOT_PRESET_ID,
        "pilot_chunk": LG1_PILOT_CHUNK,
        "chart_id": eval.chart_id,
        "topology_kind_count": eval.topology_kind_count,
        "topology_kinds": eval.topology_kinds,
        "nested_depth_max": eval.nested_depth_max,
        "flat_node_count": eval.flat_node_count,
        "required_topologies_met": eval.required_topologies_met,
        "missing_required_topologies": eval.missing_required_topologies,
        "lambda_blended": eval.lambda_blended,
        "preset_lambda": eval.preset_lambda,
        "effective_lambda": eval.effective_lambda,
        "planning_glyph_overlays": eval.planning_glyph_overlays,
        "green": green,
    })
}

/// LG-1 witness — `debug_runs/landscape_grammar_lg1_live.json`.
#[must_use]
pub fn refresh_lg1_witness(eval: &LandscapeProgramEvaluation) -> bool {
    let body = build_lg1_witness_body(eval);
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "landscape_grammar_lg1",
        "refresh_lg1_witness",
        LANDSCAPE_GRAMMAR_LG1_LIVE_JSON,
        body,
    );
    write_debug_run_json(LANDSCAPE_GRAMMAR_LG1_LIVE_JSON, wrapped) && green
}

pub const LANDSCAPE_GRAMMAR_COMPOSITE_LIVE_JSON: &str =
    "debug_runs/landscape_grammar_composite_live.json";

/// VEG-COMPOSITE-EVAL-001 — MACRO-* registry expands to topology subgraph before flatten.
#[must_use]
pub fn refresh_composite_eval_witness(preset: &LandscapeGrammarPreset) -> bool {
    let expanded = effective_topology_graph(preset);
    let macro_nodes: u32 = preset
        .composition_macros
        .iter()
        .map(|m| macro_topology_subgraph(m).len() as u32)
        .sum();
    let green = !preset.composition_macros.is_empty()
        && macro_nodes >= preset.composition_macros.len() as u32
        && expanded.len() >= preset.topology_graph.len();
    let body = json!({
        "gate": "VEG-COMPOSITE-EVAL-001",
        "green": green,
        "composition_macros": preset.composition_macros,
        "macro_subgraph_nodes": macro_nodes,
        "effective_graph_nodes": expanded.len(),
        "base_graph_nodes": preset.topology_graph.len(),
    });
    let wrapped = wrap_debug_run(
        "VEG-COMPOSITE-EVAL-001",
        "refresh_composite_eval_witness",
        LANDSCAPE_GRAMMAR_COMPOSITE_LIVE_JSON,
        body,
    );
    write_debug_run_json(LANDSCAPE_GRAMMAR_COMPOSITE_LIVE_JSON, wrapped) && green
}

fn startup_load_landscape_catalog(mut commands: Commands) {
    commands.insert_resource(load_landscape_grammar_catalog());
}

pub fn attach_landscape_program_pilot(
    catalog: Res<LandscapeGrammarCatalog>,
    mut commands: Commands,
    q: Query<
        (Entity, &Chunk, &ChunkEcology, &VegetationField, &ChunkWeather),
        Without<LandscapeProgramOnChunk>,
    >,
) {
    let Some(preset) = catalog.presets.get(LG1_PILOT_PRESET_ID) else {
        return;
    };
    for (entity, chunk, ecology, veg, weather) in &q {
        if chunk.coord != LG1_PILOT_CHUNK {
            continue;
        }
        let evaluation = evaluate_landscape_program(preset, chunk.coord, ecology, veg, weather);
        commands.entity(entity).insert(LandscapeProgramOnChunk {
            preset_id: LG1_PILOT_PRESET_ID.to_string(),
            evaluation,
        });
    }
}

pub fn landscape_grammar_plugin(app: &mut App) {
    app.add_systems(Startup, startup_load_landscape_catalog)
        .add_systems(
            Update,
            attach_landscape_program_pilot.in_set(ChunkEnvironmentSet::Ecology),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pilot_preset() -> LandscapeGrammarPreset {
        let path = repo_asset_path(&format!(
            "{LANDSCAPE_PRESETS_DIR}/{LG1_PILOT_PRESET_ID}.json"
        ));
        load_landscape_preset_from_path(&path)
            .unwrap_or_else(|e| panic!("load pilot preset: {e}"))
    }

    fn sample_eval() -> LandscapeProgramEvaluation {
        let preset = pilot_preset();
        let ecology = ChunkEcology::default();
        let veg = VegetationField::default();
        let weather = ChunkWeather::default();
        evaluate_landscape_program(&preset, LG1_PILOT_CHUNK, &ecology, &veg, &weather)
    }

    #[test]
    fn agri_riparian_catalog_loads_without_errors() {
        let catalog = load_landscape_grammar_catalog();
        assert!(
            catalog.load_errors.is_empty(),
            "{:?}",
            catalog.load_errors
        );
        assert!(catalog.presets.contains_key(LG1_PILOT_PRESET_ID));
    }

    #[test]
    fn agri_riparian_flattens_four_topology_kinds_and_nested_depth() {
        let preset = pilot_preset();
        let eval = sample_eval();
        assert!(eval.topology_kind_count >= 4, "{:?}", eval.topology_kinds);
        assert!(eval.nested_depth_max >= 2, "depth={}", eval.nested_depth_max);
        assert!(eval.required_topologies_met);
        assert_eq!(preset.landscape_program.class, "agricultural");
    }

    #[test]
    fn blend_lambda_is_read_only_and_differs_from_preset() {
        let preset = pilot_preset();
        let ecology = ChunkEcology {
            biomass: 0.62,
            root_strength: 0.55,
            fire_risk: 0.18,
            ..Default::default()
        };
        let veg = VegetationField {
            dryness: 0.48,
            burn_severity: 0.12,
            fragmentation: 0.52,
            ..Default::default()
        };
        let weather = ChunkWeather {
            soil_moisture: 0.72,
            wind_speed: 4.5,
            ..Default::default()
        };
        let blended = blend_lambda_readonly(preset.pressure_field, &ecology, &veg, &weather);
        assert!(lambda_differs(preset.pressure_field, blended, 1e-4));
    }

    #[test]
    fn planning_glyphs_are_deterministic_per_node() {
        let preset = pilot_preset();
        let mut first = Vec::new();
        flatten_topology_graph(&preset.topology_graph, &mut first);
        let mut second = Vec::new();
        flatten_topology_graph(&preset.topology_graph, &mut second);
        assert_eq!(first, second);
        assert!(first.iter().any(|n| n.planning_glyph.contains('≈')));
        assert!(first.iter().any(|n| n.topology_kind == "Cluster"));
    }

    #[test]
    fn lambda_external_inputs_shift_effective_lambda() {
        let preset = pilot_preset();
        let ecology = ChunkEcology::default();
        let veg = VegetationField::default();
        let weather = ChunkWeather::default();
        let base = blend_lambda_readonly(preset.pressure_field, &ecology, &veg, &weather);
        let inputs = LambdaExternalInputs {
            hydrology_bias: 0.9,
            transport_access: 0.8,
            construction_pressure: 0.7,
        };
        let shifted = blend_lambda_with_inputs(preset.pressure_field, &ecology, &veg, &weather, &inputs);
        assert!(lambda_differs(base, shifted, 1e-4));
    }

    #[test]
    fn lg1_witness_writes_green_json() {
        let eval = sample_eval();
        assert!(lg1_evaluation_green(&eval), "{:?}", eval);
        assert!(refresh_lg1_witness(&eval));
        let path = repo_asset_path(LANDSCAPE_GRAMMAR_LG1_LIVE_JSON);
        let raw = fs::read_to_string(&path).expect("witness file");
        let doc: Value = serde_json::from_str(&raw).expect("witness json");
        assert_eq!(
            doc.get("green").and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}
