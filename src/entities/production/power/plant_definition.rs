//! Data-driven **plant definitions** loaded from `assets/config/power/plant_definitions.json`.
//! See `power_legacy_functional_parity_v1.md` §6 and spec `06_power_plants_data_scripting_v1.md`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::entities::production::power::power_states::PowerPlantType;
use crate::entities::types::OperationalStatus;

/// Root document in `plant_definitions.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantDefinitionFile {
    pub schema_version: u32,
    #[serde(default)]
    pub plants: Vec<PlantDefinition>,
}

/// One row in the registry (keyed by `id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantDefinition {
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    pub plant_type: PowerPlantType,
    #[serde(default)]
    pub narrative: PlantNarrative,
    #[serde(default)]
    pub output_model: OutputModel,
    #[serde(default)]
    pub operational: OperationalProfile,
    #[serde(default)]
    pub instance_template: InstanceTemplate,
    #[serde(default)]
    pub capabilities: CapabilityFlags,
    #[serde(default)]
    pub emissions: EmissionsProfile,
    #[serde(default)]
    pub economics: EconomicsProfile,
    #[serde(default)]
    pub research: ResearchGates,
    #[serde(default)]
    pub grid_interface: GridInterface,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlantNarrative {
    #[serde(default)]
    pub short_description: String,
    #[serde(default)]
    pub operator_notes: String,
    #[serde(default)]
    pub risk_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputModel {
    #[serde(default = "default_one")]
    pub nameplate_mw: f32,
    /// Same role as `PlantArchetype::efficiency_factor` until curves fully drive output.
    #[serde(default = "default_eff")]
    pub base_efficiency_factor: f32,
    #[serde(default)]
    pub part_load_curve_id: String,
    #[serde(default = "default_point_one")]
    pub min_stable_output_fraction: f32,
    #[serde(default)]
    pub ramp_up_mw_per_minute: f32,
    #[serde(default)]
    pub ramp_down_mw_per_minute: f32,
    #[serde(default)]
    pub inertia_constant_h: f32,
}

impl Default for OutputModel {
    fn default() -> Self {
        Self {
            nameplate_mw: default_one(),
            base_efficiency_factor: default_eff(),
            part_load_curve_id: String::new(),
            min_stable_output_fraction: default_point_one(),
            ramp_up_mw_per_minute: 0.0,
            ramp_down_mw_per_minute: 0.0,
            inertia_constant_h: 0.0,
        }
    }
}

fn default_one() -> f32 {
    1.0
}

fn default_eff() -> f32 {
    0.9
}

fn default_point_one() -> f32 {
    0.1
}

/// Per-`OperationalStatus` tuning. Missing keys fall back to type defaults in systems.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperationalProfile {
    #[serde(default)]
    pub status_modifiers: HashMap<OperationalStatus, StatusModifier>,
    #[serde(default)]
    pub transition_hints: Vec<TransitionHint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusModifier {
    /// Fraction of `PowerPlant.max_output` (or nameplate) when “allowed to generate”.
    #[serde(default = "default_one")]
    pub output_fraction: f32,
    /// Auxiliary house load as fraction of nameplate (for future `ElectricalComponent` bridge).
    #[serde(default)]
    pub aux_load_fraction: f32,
    /// Scalar on `base_efficiency_factor` for this status.
    #[serde(default = "default_one")]
    pub efficiency_multiplier: f32,
    #[serde(default)]
    pub ramp_limited: bool,
    #[serde(default)]
    pub allow_black_start: bool,
    #[serde(default)]
    pub tooling_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionHint {
    pub from: OperationalStatus,
    pub to: OperationalStatus,
    #[serde(default)]
    pub trigger: String,
    #[serde(default)]
    pub typical_duration_s: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstanceTemplate {
    #[serde(default)]
    pub reactor_units: u32,
    #[serde(default = "default_one")]
    pub unit_thermal_mw: f32,
    #[serde(default)]
    pub turbine_units: u32,
    #[serde(default)]
    pub hydro_head_m: f32,
    #[serde(default)]
    pub hydro_design_flow_m3s: f32,
    #[serde(default)]
    pub solar_array_area_m2: f32,
    #[serde(default)]
    pub panel_nominal_efficiency: f32,
    #[serde(default)]
    pub wind_rotor_diameter_m: f32,
    #[serde(default)]
    pub wind_hub_height_m: f32,
    #[serde(default)]
    pub geothermal_brine_inlet_c: f32,
    #[serde(default)]
    pub geothermal_brine_flow_kg_s: f32,
    #[serde(default)]
    pub fuel_feed_rate_kg_s: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityFlags {
    #[serde(default)]
    pub is_steam_cycle: bool,
    #[serde(default)]
    pub is_nuclear_containment: bool,
    #[serde(default)]
    pub is_variable_renewable: bool,
}

impl Default for CapabilityFlags {
    fn default() -> Self {
        Self {
            is_steam_cycle: false,
            is_nuclear_containment: false,
            is_variable_renewable: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmissionsProfile {
    #[serde(default)]
    pub co2_kg_per_mwh: f32,
    #[serde(default)]
    pub nox_g_per_mwh: f32,
    #[serde(default)]
    pub so2_g_per_mwh: f32,
    #[serde(default)]
    pub particulate_g_per_mwh: f32,
    #[serde(default)]
    pub thermal_outfall_kw_per_mw: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EconomicsProfile {
    #[serde(default)]
    pub capex_per_kw: f32,
    #[serde(default)]
    pub fixed_om_per_kw_year: f32,
    #[serde(default)]
    pub variable_om_per_mwh: f32,
    #[serde(default)]
    pub fuel_cost_per_mwh_thermal: f32,
    #[serde(default)]
    pub expected_availability: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResearchGates {
    #[serde(default)]
    pub required_research_ids: Vec<String>,
    #[serde(default)]
    pub unlock_notes: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GridInterface {
    #[serde(default = "default_pf")]
    pub power_factor: f32,
    #[serde(default)]
    pub inrush_multiplier: f32,
    #[serde(default)]
    pub allow_grid_forming: bool,
}

fn default_pf() -> f32 {
    0.95
}

impl PlantDefinition {
    /// Effective efficiency factor for output systems, before status and load derates.
    pub fn base_efficiency_factor(&self) -> f32 {
        self.output_model.base_efficiency_factor
    }

    pub fn status_efficiency_mult(&self, status: OperationalStatus) -> f32 {
        self.operational
            .status_modifiers
            .get(&status)
            .map(|m| m.efficiency_multiplier)
            .unwrap_or(1.0)
    }

    pub fn status_output_fraction(&self, status: OperationalStatus) -> f32 {
        self.operational
            .status_modifiers
            .get(&status)
            .map(|m| m.output_fraction)
            .unwrap_or_else(|| {
                if status == OperationalStatus::Operational {
                    1.0
                } else {
                    0.0
                }
            })
    }
}
