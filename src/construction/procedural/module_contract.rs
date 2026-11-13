//! BQ-C1 module geometric contract — grid constants mirrored from `module_contract_v1.json`.


pub const MODULE_CONTRACT_JSON: &str = "tools/mcp/schemas/module_contract_v1.json";
pub const GRID_UNIT_M: f32 = 4.0;
pub const FLOOR_HEIGHT_M: f32 = 3.0;
pub const PIVOT_CONVENTION: &str = "bottom_center";
pub const SEAM_TOLERANCE_M: f32 = 0.01;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_units_from_width_m_one_cell() {
        assert_eq!(grid_units_from_width_m(4.0), 1);
        assert_eq!(grid_units_from_width_m(8.0), 2);
    }
}
