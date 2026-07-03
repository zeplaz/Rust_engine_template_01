//! Runtime building defs from `assets/configs/buildings/*.json` (Round 3-A).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use crate::entities::types::s_flagz::ConcreteType;
use crate::strategic::SiteArchetype;

use super::build_tool_authority::BuildingArchetypeId;
use super::building_catalog::{
    ApartmentForm, BuildingFamily, BuildingIntentPreview, FootprintMatrix,
};

const BUILDINGS_DIR: &str = "assets/configs/buildings";
const MOCK_SHAPES_RON: &str = "assets/configs/buildings/_mock_shapes.ron";

/// Resolve engine-owned asset paths from crate root (works in tests and `cargo run` from repo).
#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

#[derive(Debug, Clone, Deserialize)]
struct MockShapesFile {
    shapes: Vec<MockShapeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct MockShapeEntry {
    id: String,
    label: String,
    width: u32,
    depth: u32,
    cells: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
struct BuildingDefinitionFile {
    #[serde(default)]
    asset_name: String,
    #[serde(default)]
    segment: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    is_power: bool,
    #[serde(default)]
    is_productive: bool,
    #[serde(default = "default_one")]
    building_size_x: u32,
    #[serde(default = "default_one")]
    building_size_y: u32,
    #[serde(default)]
    building_height: u32,
    #[serde(default)]
    construction_cost: u32,
    #[serde(default)]
    power_generation: f32,
    #[serde(default)]
    power_consumption: f32,
    #[serde(default)]
    produces_resources: Vec<String>,
    #[serde(default)]
    consumes_resources: Vec<String>,
    #[serde(default)]
    supply_chain: Option<String>,
    #[serde(default)]
    supply_chain_role: Option<String>,
    #[serde(default)]
    concrete_type: Option<String>,
    #[serde(default)]
    utility_role: Option<String>,
    #[serde(default)]
    plant_definition_id: Option<String>,
    #[serde(default)]
    transfer_capacity_mva: f32,
    /// MCP module catalog id — mesh resolved from `_module_index.ron` when set.
    #[serde(default)]
    procedural_module_id: Option<String>,
}

fn default_one() -> u32 {
    1
}

#[derive(Debug, Clone)]
pub struct BuildingDefinition {
    pub id: String,
    pub display_name: String,
    pub footprint: FootprintMatrix,
    pub construction_cost: u32,
    pub construction_time_ticks: u32,
    pub power_consumption: f32,
    pub power_generation: f32,
    pub workers_required: u32,
    pub site_archetype: SiteArchetype,
    pub family: BuildingFamily,
    pub produces: Vec<String>,
    pub consumes: Vec<String>,
    /// e.g. `concrete_portland`, `aluminum_primary` — see `assets/configs/industrial_supply_chains.json`.
    pub supply_chain: Option<String>,
    pub supply_chain_role: Option<super::supply_chain_role::IndustrialSupplyChainRole>,
    pub concrete_type: Option<ConcreteType>,
    pub utility_role: Option<super::utility_infrastructure_role::UtilityInfrastructureRole>,
    pub plant_definition_id: Option<String>,
    pub transfer_capacity_mva: f32,
    pub is_productive: bool,
    /// Catalog module id (`wall_concrete_2u`, …) from MCP promote/register.
    pub procedural_module_id: Option<String>,
    /// Repo-relative GLB from module index (set at registry load).
    pub procedural_glb_path: Option<String>,
    /// Bevy asset path (no `assets/` prefix) for `AssetServer::load`.
    pub procedural_glb_asset: Option<String>,
    /// Grammar pilot — PG-2 archetype id from catalog (`grammar_archetype_id` field).
    pub grammar_archetype_id: Option<String>,
    pub arch_dna_preset: Option<String>,
    pub site_json_path: Option<String>,
    pub pilot_hover_hint: Option<String>,
    pub district_style: Option<String>,
}

#[derive(Resource, Debug, Default)]
pub struct BuildingDefinitionRegistry {
    pub by_id: HashMap<String, BuildingDefinition>,
    /// `INDUSTRIAL-GOV-01` — productive industry rows missing `supply_chain_role` / `utility_role`.
    pub governance_violations: Vec<String>,
}

impl BuildingDefinitionRegistry {
    pub fn get(&self, id: &str) -> Option<&BuildingDefinition> {
        self.by_id.get(id)
    }

    pub fn intent_preview(&self, id: &str) -> Option<BuildingIntentPreview> {
        self.by_id.get(id).map(intent_from_definition)
    }

    pub fn ids_by_family(&self, family: BuildingFamily) -> Vec<&str> {
        self.by_id
            .values()
            .filter(|d| d.family == family)
            .map(|d| d.id.as_str())
            .collect()
    }
}

#[must_use]
pub fn intent_from_definition(def: &BuildingDefinition) -> BuildingIntentPreview {
    BuildingIntentPreview {
        label: def.display_name.clone(),
        family: def.family,
        footprint: def.footprint.clone(),
        construction_cost: def.construction_cost,
        construction_time_ticks: def.construction_time_ticks,
        power_consumption: def.power_consumption,
        workers_required: def.workers_required,
        unit_kinds: Vec::new(),
        apartment_form: None,
        catalog_id: Some(def.id.clone()),
        arch_dna_preset_id: def.arch_dna_preset.clone(),
    }
}

#[must_use]
pub fn intent_from_archetype(
    archetype: BuildingArchetypeId,
    registry: &BuildingDefinitionRegistry,
) -> BuildingIntentPreview {
    let family = match archetype {
        BuildingArchetypeId::Housing => BuildingFamily::Residential,
        BuildingArchetypeId::Office | BuildingArchetypeId::Retail => BuildingFamily::Retail,
        BuildingArchetypeId::Factory | BuildingArchetypeId::Depot => BuildingFamily::Industry,
        BuildingArchetypeId::PowerPlant | BuildingArchetypeId::WaterPlant => BuildingFamily::Power,
    };
    if let Some(id) = registry
        .ids_by_family(family)
        .into_iter()
        .find(|id| !id.starts_with("builtin:"))
        .or_else(|| registry.ids_by_family(family).first().copied())
    {
        if let Some(p) = registry.intent_preview(id) {
            return p;
        }
    }
    let label = format!("{archetype:?}");
    BuildingIntentPreview {
        label: label.clone(),
        family,
        footprint: FootprintMatrix::from_size(
            archetype.footprint().width,
            archetype.footprint().depth,
            true,
        ),
        construction_cost: 500,
        construction_time_ticks: 20,
        power_consumption: 10.0,
        workers_required: 2,
        unit_kinds: Vec::new(),
        apartment_form: None,
        catalog_id: None,
        arch_dna_preset_id: None,
    }
}

#[must_use]
pub fn intent_from_apartment_form(form: ApartmentForm, registry: &BuildingDefinitionRegistry) -> BuildingIntentPreview {
    let id = match form {
        ApartmentForm::Duplex => "builtin:duplex",
        ApartmentForm::Quadplex => "builtin:quadplex",
        ApartmentForm::HighRise => "builtin:highrise",
        ApartmentForm::ThreeStoryBlock => "builtin:3story",
        ApartmentForm::FiveStoryBlock => "builtin:5story",
    };
    if let Some(p) = registry.intent_preview(id) {
        return p;
    }
    let mut p = super::building_catalog::default_preview_for_apartment(form);
    p.catalog_id = Some(id.into());
    p
}

/// `INDUSTRIAL-GOV-01` — productive industry must declare chain or utility role.
#[must_use]
pub fn check_industrial_governance(def: &BuildingDefinition) -> Option<String> {
    if !def.is_productive || def.family != BuildingFamily::Industry {
        return None;
    }
    if def.supply_chain_role.is_some() || def.utility_role.is_some() {
        return None;
    }
    if def.site_archetype == SiteArchetype::PowerPlant {
        return None;
    }
    Some(format!(
        "{}: productive industry missing supply_chain_role or utility_role",
        def.id
    ))
}

fn infer_archetype(raw: &BuildingDefinitionFile, name_lower: &str) -> SiteArchetype {
    if raw
        .utility_role
        .as_deref()
        .and_then(super::utility_infrastructure_role::UtilityInfrastructureRole::from_json)
        .is_some()
        || raw.is_power
        || raw.power_generation > 0.0
    {
        return SiteArchetype::PowerPlant;
    }
    if raw.is_productive || !raw.produces_resources.is_empty() {
        return SiteArchetype::Factory;
    }
    let water_utility = name_lower.contains("water plant")
        || name_lower.contains("waterworks")
        || (name_lower.contains("water") && raw.produces_resources.is_empty());
    if water_utility
        && raw.consumes_resources.iter().any(|c| c.eq_ignore_ascii_case("water"))
        && raw.power_consumption > 20.0
    {
        return SiteArchetype::WaterPlant;
    }
    if raw.building_size_x <= 2 && raw.building_size_y <= 2 && raw.produces_resources.is_empty() {
        return SiteArchetype::CivilHousing;
    }
    SiteArchetype::Factory
}

fn infer_family(archetype: SiteArchetype, raw: &BuildingDefinitionFile) -> BuildingFamily {
    match archetype {
        SiteArchetype::CivilHousing => BuildingFamily::Residential,
        SiteArchetype::WaterPlant => BuildingFamily::Power,
        SiteArchetype::PowerPlant => BuildingFamily::Power,
        SiteArchetype::RailDepot => BuildingFamily::Rail,
        SiteArchetype::Factory => {
            if raw.segment.eq_ignore_ascii_case("civilian") && raw.produces_resources.is_empty() {
                BuildingFamily::Retail
            } else {
                BuildingFamily::Industry
            }
        }
        _ => BuildingFamily::Civic,
    }
}

fn parse_concrete_type(s: &str) -> Option<ConcreteType> {
    match s {
        "Limecrete" => Some(ConcreteType::Limecrete),
        "Portland" => Some(ConcreteType::Portland),
        "Geopolymer" => Some(ConcreteType::Geopolymer),
        "Gypsum" => Some(ConcreteType::Gypsum),
        _ => None,
    }
}

fn file_to_definition(id: String, raw: BuildingDefinitionFile) -> BuildingDefinition {
    let display_name = if !raw.asset_name.is_empty() {
        raw.asset_name.clone()
    } else if !raw.description.is_empty() {
        raw.description.clone()
    } else {
        id.clone()
    };
    let name_lower = display_name.to_lowercase();
    let site_archetype = infer_archetype(&raw, &name_lower);
    let family = infer_family(site_archetype, &raw);
    let w = raw.building_size_x.max(1);
    let d = raw.building_size_y.max(1);
    BuildingDefinition {
        id: id.clone(),
        display_name,
        footprint: FootprintMatrix::from_size(w, d, true),
        construction_cost: raw.construction_cost.max(1),
        construction_time_ticks: (raw.construction_cost / 10).max(30) + raw.building_height.saturating_mul(2),
        power_consumption: raw.power_consumption,
        power_generation: raw.power_generation,
        workers_required: if raw.is_productive { 4 } else { 0 },
        site_archetype,
        family,
        produces: raw.produces_resources,
        consumes: raw.consumes_resources,
        supply_chain: raw.supply_chain,
        supply_chain_role: raw
            .supply_chain_role
            .as_deref()
            .and_then(super::supply_chain_role::IndustrialSupplyChainRole::from_json_role),
        concrete_type: raw.concrete_type.as_deref().and_then(parse_concrete_type),
        utility_role: raw
            .utility_role
            .as_deref()
            .and_then(super::utility_infrastructure_role::UtilityInfrastructureRole::from_json)
            .or_else(|| {
                super::utility_infrastructure_role::UtilityInfrastructureRole::from_catalog_id(
                    id.as_str(),
                )
            }),
        plant_definition_id: raw.plant_definition_id.clone(),
        transfer_capacity_mva: raw.transfer_capacity_mva,
        is_productive: raw.is_productive,
        procedural_module_id: raw.procedural_module_id.clone(),
        procedural_glb_path: None,
        procedural_glb_asset: None,
        grammar_archetype_id: None,
        arch_dna_preset: None,
        site_json_path: None,
        pilot_hover_hint: None,
        district_style: None,
    }
}

fn register_mock_shapes_from_ron(registry: &mut BuildingDefinitionRegistry) {
    let path = repo_asset_path(MOCK_SHAPES_RON);
    let Ok(text) = fs::read_to_string(&path) else {
        return;
    };
    let Ok(file) = ron::from_str::<MockShapesFile>(&text) else {
        warn!("mock shapes RON parse failed: {}", path.display());
        return;
    };
    for shape in file.shapes {
        let expected = (shape.width as usize) * (shape.depth as usize);
        if shape.cells.len() != expected {
            warn!(
                "mock shape {} cell count {} != {}×{}",
                shape.id,
                shape.cells.len(),
                shape.width,
                shape.depth
            );
            continue;
        }
        let footprint = FootprintMatrix {
            width: shape.width,
            depth: shape.depth,
            cells: shape.cells,
        };
        let id = format!("mock:{}", shape.id);
        let def = BuildingDefinition {
            id: id.clone(),
            display_name: shape.label,
            footprint,
            construction_cost: 1,
            construction_time_ticks: 1,
            power_consumption: 0.0,
            power_generation: 0.0,
            workers_required: 0,
            site_archetype: SiteArchetype::Factory,
            family: BuildingFamily::Industry,
            produces: Vec::new(),
            consumes: Vec::new(),
            supply_chain: None,
            supply_chain_role: None,
            concrete_type: None,
            utility_role: None,
            plant_definition_id: None,
            transfer_capacity_mva: 0.0,
            is_productive: false,
            procedural_module_id: None,
            procedural_glb_path: None,
            procedural_glb_asset: None,
            grammar_archetype_id: None,
            arch_dna_preset: None,
            site_json_path: None,
            pilot_hover_hint: None,
            district_style: None,
        };
        registry.by_id.insert(id, def);
    }
}

fn register_pilot_catalog_from_ron(registry: &mut BuildingDefinitionRegistry) {
    use super::pilot_catalog::{pilot_building_registration, PilotCatalog};

    let catalog = PilotCatalog::load_from_disk();
    for pilot in &catalog.pilots {
        let (display_name, cost, time, power, workers, site_archetype, family, is_productive) =
            pilot_building_registration(pilot);
        let def = BuildingDefinition {
            id: pilot.catalog_id.clone(),
            display_name,
            footprint: pilot.footprint.clone(),
            construction_cost: cost,
            construction_time_ticks: time,
            power_consumption: power,
            power_generation: 0.0,
            workers_required: workers,
            site_archetype,
            family,
            produces: Vec::new(),
            consumes: if is_productive {
                vec!["Electricity".into()]
            } else {
                Vec::new()
            },
            supply_chain: None,
            supply_chain_role: None,
            concrete_type: None,
            utility_role: None,
            plant_definition_id: None,
            transfer_capacity_mva: 0.0,
            is_productive,
            procedural_module_id: None,
            procedural_glb_path: None,
            procedural_glb_asset: None,
            grammar_archetype_id: pilot.grammar_archetype_id.clone(),
            arch_dna_preset: pilot.arch_dna_preset.clone(),
            site_json_path: pilot.site_json_path.clone(),
            pilot_hover_hint: pilot.hover_hint.clone(),
            district_style: pilot.district_style.clone(),
        };
        registry.by_id.insert(def.id.clone(), def);
    }
}

fn register_builtin_apartments(registry: &mut BuildingDefinitionRegistry) {
    for (id, form) in [
        ("builtin:duplex", ApartmentForm::Duplex),
        ("builtin:quadplex", ApartmentForm::Quadplex),
        ("builtin:highrise", ApartmentForm::HighRise),
        ("builtin:3story", ApartmentForm::ThreeStoryBlock),
        ("builtin:5story", ApartmentForm::FiveStoryBlock),
    ] {
        let mut preview = super::building_catalog::default_preview_for_apartment(form);
        preview.catalog_id = Some(id.into());
        let def = BuildingDefinition {
            id: id.into(),
            display_name: preview.label.clone(),
            footprint: preview.footprint.clone(),
            construction_cost: preview.construction_cost,
            construction_time_ticks: preview.construction_time_ticks,
            power_consumption: preview.power_consumption,
            power_generation: 0.0,
            workers_required: preview.workers_required,
            site_archetype: SiteArchetype::CivilHousing,
            family: BuildingFamily::Residential,
            produces: Vec::new(),
            consumes: Vec::new(),
            supply_chain: None,
            supply_chain_role: None,
            concrete_type: None,
            utility_role: None,
            plant_definition_id: None,
            transfer_capacity_mva: 0.0,
            is_productive: false,
            procedural_module_id: None,
            procedural_glb_path: None,
            procedural_glb_asset: None,
            grammar_archetype_id: None,
            arch_dna_preset: None,
            site_json_path: None,
            pilot_hover_hint: None,
            district_style: None,
        };
        registry.by_id.insert(def.id.clone(), def);
    }
}

impl BuildingDefinitionRegistry {
    /// Resolve Bevy asset path: building row `procedural_module_id` or direct module catalog id.
    #[must_use]
    pub fn procedural_glb_asset<'a>(
        &'a self,
        modules: &'a super::procedural::ProceduralModuleRegistry,
        catalog_or_module_id: &str,
    ) -> Option<&'a str> {
        if let Some(def) = self.get(catalog_or_module_id) {
            if let Some(asset) = def.procedural_glb_asset.as_deref() {
                return Some(asset);
            }
            if let Some(module_id) = def.procedural_module_id.as_deref() {
                if let Some(asset) = modules.stylepack_glb_asset(module_id) {
                    return Some(asset);
                }
            }
        }
        modules.stylepack_glb_asset(catalog_or_module_id)
    }
}

#[must_use]
pub fn load_building_definitions_from_dir(dir: impl AsRef<Path>) -> BuildingDefinitionRegistry {
    let mut registry = BuildingDefinitionRegistry::default();
    register_builtin_apartments(&mut registry);

    let dir = dir.as_ref();
    let Ok(entries) = fs::read_dir(dir) else {
        return registry;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name.starts_with('_') {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(raw) = serde_json::from_str::<BuildingDefinitionFile>(&text) else {
            continue;
        };
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name)
            .to_string();
        let def = file_to_definition(id, raw);
        if let Some(msg) = check_industrial_governance(&def) {
            registry.governance_violations.push(msg);
        }
        registry.by_id.insert(def.id.clone(), def);
    }
    register_mock_shapes_from_ron(&mut registry);
    register_pilot_catalog_from_ron(&mut registry);
    registry
}

#[must_use]
pub fn default_buildings_dir() -> PathBuf {
    repo_asset_path(BUILDINGS_DIR)
}

/// UI-P2A-CODER-B — `_mock_shapes.ron` roundtrip matches registry footprint.
#[must_use]
pub fn mock_shapes_parity_green() -> bool {
    let reg = load_building_definitions_from_dir(default_buildings_dir());
    let Some(def) = reg.get("mock:shape_t_3x3") else {
        return false;
    };
    def.footprint.width == 3
        && def.footprint.depth == 3
        && def.footprint.cells.len() == 9
        && def.footprint.cells[0] == 1
        && def.footprint.cells[4] == 1
        && def.footprint.cells[3] == 0
}

pub fn attach_procedural_glb_paths(
    buildings: &mut BuildingDefinitionRegistry,
    modules: &super::procedural::ProceduralModuleRegistry,
) {
    for def in buildings.by_id.values_mut() {
        let Some(module_id) = def.procedural_module_id.as_deref() else {
            continue;
        };
        let Some(entry) = modules.stylepack_entry(module_id) else {
            continue;
        };
        def.procedural_glb_path = Some(entry.glb_path.clone());
        def.procedural_glb_asset = Some(entry.glb_asset.clone());
    }
}

pub fn init_building_definition_registry(
    mut commands: Commands,
    modules: Option<Res<super::procedural::ProceduralModuleRegistry>>,
) {
    let mut registry = load_building_definitions_from_dir(default_buildings_dir());
    if let Some(modules) = modules {
        attach_procedural_glb_paths(&mut registry, &modules);
    }
    commands.insert_resource(registry);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governance_catches_productive_industry_without_role() {
        let raw = BuildingDefinitionFile {
            asset_name: "Bad Mega Factory".into(),
            segment: "Industrial".into(),
            description: String::new(),
            is_power: false,
            is_productive: true,
            building_size_x: 4,
            building_size_y: 4,
            building_height: 1,
            construction_cost: 100,
            power_generation: 0.0,
            power_consumption: 50.0,
            produces_resources: vec!["Concrete".into()],
            consumes_resources: vec!["Electricity".into()],
            supply_chain: None,
            supply_chain_role: None,
            concrete_type: None,
            utility_role: None,
            plant_definition_id: None,
            transfer_capacity_mva: 0.0,
            procedural_module_id: None,
        };
        let def = file_to_definition("test_bad_mega".into(), raw);
        assert_eq!(def.family, BuildingFamily::Industry);
        assert!(check_industrial_governance(&def).is_some());
    }

    #[test]
    fn governance_accepts_supply_chain_step() {
        let reg = load_building_definitions_from_dir(default_buildings_dir());
        assert!(
            !reg.governance_violations.iter().any(|v| v.contains("aluminum_smelter1")),
            "smelter should have role: {:?}",
            reg.governance_violations
        );
    }

    #[test]
    fn loads_mock_shape_footprints_from_ron() {
        let reg = load_building_definitions_from_dir(default_buildings_dir());
        assert!(
            reg.by_id.contains_key("mock:shape_t_3x3"),
            "mock shapes from _mock_shapes.ron"
        );
    }

    #[test]
    fn mock_shape_ron_roundtrip_matches_registry_footprint() {
        let reg = load_building_definitions_from_dir(default_buildings_dir());
        let def = reg.get("mock:shape_t_3x3").expect("T shape");
        assert_eq!(def.footprint.width, 3);
        assert_eq!(def.footprint.depth, 3);
        assert_eq!(def.footprint.cells.len(), 9);
        assert_eq!(def.footprint.cells[0], 1);
        assert_eq!(def.footprint.cells[4], 1);
        assert_eq!(def.footprint.cells[3], 0);
    }

    #[test]
    fn loads_example_building_json_when_present() {
        let reg = load_building_definitions_from_dir(default_buildings_dir());
        assert!(reg.by_id.contains_key("builtin:duplex"));
        if Path::new("assets/configs/buildings/concrete_basic_production_plant.json").exists() {
            let def = reg.get("concrete_basic_production_plant").expect("plant def");
            assert_eq!(def.footprint.width, 3);
            assert_eq!(def.site_archetype, SiteArchetype::Factory);
        }
    }
}
