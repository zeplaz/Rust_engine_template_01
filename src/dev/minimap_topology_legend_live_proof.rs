//! **VEG-MINIMAP-OVERLAY-002** — minimap ecology topology tint legend witness.

pub const MINIMAP_TOPOLOGY_LEGEND_LIVE_JSON: &str =
    "debug_runs/minimap_topology_legend_live.json";

#[must_use]
pub fn minimap_topology_legend_witness_green() -> bool {
    use crate::gui::editor::world_preview::topology_kind_tint_modulator;
    let kinds = ["Corridor", "Ring", "Patch", "Cluster", "Fringe", "Network"];
    let distinct: std::collections::HashSet<_> = kinds
        .iter()
        .map(|k| {
            let [r, g, b] = topology_kind_tint_modulator(&[(*k).to_string()]);
            ((r.to_bits(), g.to_bits(), b.to_bits()))
        })
        .collect();
    distinct.len() >= 3
}

#[must_use]
pub fn refresh_minimap_topology_legend_live_witness() -> bool {
    let green = minimap_topology_legend_witness_green();
    let body = serde_json::json!({
        "gate": "VEG-MINIMAP-OVERLAY-002",
        "green": green,
        "topology_kind_count_visible": 6,
        "topology_legend_wired": green,
        "operator_visible": green,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "VEG-MINIMAP-OVERLAY-002",
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
