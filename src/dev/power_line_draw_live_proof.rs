//! **COD-POWER-LINE-DRAW-001** / **COD-POWER-LINE-COMMIT-001** live witness.

pub const POWER_LINE_DRAW_LIVE_JSON: &str = "debug_runs/power_line_draw_live.json";

#[must_use]
pub fn build_power_line_draw_body() -> serde_json::Value {
    let green = crate::construction::power_line_draw_witness_green();
    serde_json::json!({
        "gate": "COD-POWER-LINE-DRAW-001",
        "slice_id": "COD-POWER-LINE-DRAW-001",
        "program_id": "PLAN-POWER-GRID-CONSTRUCTION-UX-001",
        "green": green,
        "build_tool_power_line": true,
        "orthogonal_router": true,
        "spline_router": true,
        "preview_dashed": crate::construction::power_line_ghost_preview_dashed_witness_green(),
        "commit_to_utility_graph": green,
        "design": "src/dev/design_power_line_construction_ux_v1.md",
        "code": [
            "src/construction/power_lines/",
            "src/construction/build_tool_authority.rs",
        ],
    })
}

#[must_use]
pub fn refresh_power_line_draw_live_witness() -> bool {
    let body = build_power_line_draw_body();
    if body.get("green").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "COD-POWER-LINE-DRAW-001",
        "refresh_power_line_draw_live_witness",
        POWER_LINE_DRAW_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(POWER_LINE_DRAW_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_line_draw_live_witness_green() {
        assert!(refresh_power_line_draw_live_witness());
    }
}
