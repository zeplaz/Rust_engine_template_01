//! BQ-C1 module geometric contract — grid constants mirrored from `module_contract_v1.json`.

use serde::Deserialize;
use serde_json::{json, Value};

pub const MODULE_CONTRACT_JSON: &str = "tools/mcp/schemas/module_contract_v1.json";
pub const GRID_UNIT_M: f32 = 4.0;
pub const FLOOR_HEIGHT_M: f32 = 3.0;
pub const PIVOT_CONVENTION: &str = "bottom_center";
pub const SEAM_TOLERANCE_M: f32 = 0.01;

pub const BQ_C1_LIVE_JSON: &str = "debug_runs/bq_c1_contract_001_live.json";

#[derive(Debug, Clone, Deserialize)]
struct ModuleContractFile {
    schema_version: u32,
    grid_unit_m: f64,
    floor_height_m: f64,
    pivot_convention: String,
}

/// Grid cells for a module width in meters (rounded to nearest whole cell).
#[must_use]
pub fn grid_units_from_width_m(width_m: f32) -> u32 {
    ((width_m / GRID_UNIT_M).round().max(1.0)) as u32
}

/// Expected wall height for one floor band.
#[must_use]
pub fn standard_wall_height_m() -> f32 {
    FLOOR_HEIGHT_M
}

fn load_contract_file() -> Option<ModuleContractFile> {
    let text = std::fs::read_to_string(MODULE_CONTRACT_JSON).ok()?;
    serde_json::from_str(&text).ok()
}

/// Python/Rust/JSON parity for BQ-C1 witness.
#[must_use]
pub fn build_bq_c1_contract_witness_body() -> Value {
    let file = load_contract_file();
    let file_ok = file.is_some();
    let (grid_ok, floor_ok, pivot_ok) = if let Some(c) = file.as_ref() {
        (
            (c.grid_unit_m - f64::from(GRID_UNIT_M)).abs() < f64::EPSILON,
            (c.floor_height_m - f64::from(FLOOR_HEIGHT_M)).abs() < f64::EPSILON,
            c.pivot_convention == PIVOT_CONVENTION,
        )
    } else {
        (false, false, false)
    };
    let green = file_ok && grid_ok && floor_ok && pivot_ok;
    json!({
        "task_id": "BQ-C1-CONTRACT-001",
        "gate": "BQ-C1-CONTRACT-001",
        "green": green,
        "contract_json": MODULE_CONTRACT_JSON,
        "rust": {
            "grid_unit_m": GRID_UNIT_M,
            "floor_height_m": FLOOR_HEIGHT_M,
            "pivot_convention": PIVOT_CONVENTION,
            "seam_tolerance_m": SEAM_TOLERANCE_M,
        },
        "parity": {
            "contract_file_loaded": file_ok,
            "grid_unit_m": grid_ok,
            "floor_height_m": floor_ok,
            "pivot_convention": pivot_ok,
        },
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-C1",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bq_c1_contract_constants_match_json() {
        let body = build_bq_c1_contract_witness_body();
        assert_eq!(body.get("green").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn grid_units_from_width_m_one_cell() {
        assert_eq!(grid_units_from_width_m(4.0), 1);
        assert_eq!(grid_units_from_width_m(8.0), 2);
    }
}
