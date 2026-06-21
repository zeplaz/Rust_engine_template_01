//! PLAN-POWER-GRID-CONSTRUCTION-UX-001 program close witness.

pub const POWER_GRID_CONSTRUCTION_UX_CLOSE_JSON: &str =
    "debug_runs/power_grid_construction_ux_close_live.json";

#[must_use]
pub fn refresh_power_grid_construction_ux_close_witness() -> bool {
    use crate::construction::power_line_cut_input_wired;
    use crate::dev::power_grid_track_bd_live_proof::refresh_power_grid_track_bd_live_witness;
    use crate::dev::power_grid_track_c_live_proof::refresh_power_grid_track_c_live_witness;
    use crate::dev::power_line_draw_live_proof::refresh_power_line_draw_live_witness;

    let green = refresh_power_line_draw_live_witness()
        && refresh_power_grid_track_bd_live_witness()
        && refresh_power_grid_track_c_live_witness()
        && power_line_cut_input_wired();

    let body = serde_json::json!({
        "gate": "PLAN-POWER-GRID-CONSTRUCTION-UX-001",
        "program_closed": green,
        "track_a": refresh_power_line_draw_live_witness(),
        "track_b_d": refresh_power_grid_track_bd_live_witness(),
        "track_c": refresh_power_grid_track_c_live_witness(),
        "demolish_cut_wired": power_line_cut_input_wired(),
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "PLAN-POWER-GRID-CONSTRUCTION-UX-001",
        "refresh_power_grid_construction_ux_close_witness",
        POWER_GRID_CONSTRUCTION_UX_CLOSE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(POWER_GRID_CONSTRUCTION_UX_CLOSE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_grid_construction_ux_close_witness_green() {
        assert!(refresh_power_grid_construction_ux_close_witness());
    }
}
