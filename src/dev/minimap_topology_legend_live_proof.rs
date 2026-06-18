//! **VEG-MINIMAP-LEGEND-UI-001** / **DES-MINIMAP-VEG-LEGEND-002** — minimap ecology legend witness.

pub const MINIMAP_TOPOLOGY_LEGEND_LIVE_JSON: &str =
    "debug_runs/minimap_topology_legend_live.json";

#[must_use]
pub fn minimap_topology_legend_witness_green() -> bool {
    use crate::gui::editor::world_preview::topology_kind_tint_modulator;
    use crate::gui::hud::minimap_topology_legend::{
        minimap_burn_legend_ui_wired, minimap_topology_legend_ui_witness_green,
    };
    let kinds = ["Corridor", "Ring", "Patch", "Cluster", "Fringe", "Network"];
    let distinct: std::collections::HashSet<_> = kinds
        .iter()
        .map(|k| {
            let [r, g, b] = topology_kind_tint_modulator(&[(*k).to_string()]);
            (r.to_bits(), g.to_bits(), b.to_bits())
        })
        .collect();
    distinct.len() >= 3
        && minimap_topology_legend_ui_witness_green()
        && minimap_burn_legend_ui_wired()
}

#[must_use]
pub fn refresh_minimap_topology_legend_live_witness() -> bool {
    let green = minimap_topology_legend_witness_green();
    let ui_wired = crate::gui::hud::minimap_topology_legend::minimap_topology_legend_ui_witness_green();
    let burn_wired = crate::gui::hud::minimap_topology_legend::minimap_burn_legend_ui_wired();
    let body = serde_json::json!({
        "gate": "VEG-MINIMAP-LEGEND-UI-001",
        "design_ref": "src/dev/design_minimap_veg_legend_wire_v1.md",
        "green": green,
        "legend_ui_wired": ui_wired,
        "burn_legend_wired": burn_wired,
        "topology_kind_count_visible": 6,
        "fire_legend_row_count": 3,
        "topology_legend_wired": green,
        "operator_visible": green,
        "collapsed_label": crate::gui::hud::minimap_topology_legend::minimap_topology_legend_collapsed_label(),
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "VEG-MINIMAP-LEGEND-UI-001",
        "refresh_minimap_topology_legend_live_witness",
        MINIMAP_TOPOLOGY_LEGEND_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(MINIMAP_TOPOLOGY_LEGEND_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimap_topology_legend_live_witness_green() {
        assert!(refresh_minimap_topology_legend_live_witness());
    }
}
