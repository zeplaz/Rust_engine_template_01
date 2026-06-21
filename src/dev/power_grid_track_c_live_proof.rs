//! Track C rollup — damage segment + repair queue witness.

pub const POWER_GRID_TRACK_C_LIVE_JSON: &str = "debug_runs/power_grid_track_c_live.json";

#[must_use]
pub fn refresh_power_grid_track_c_live_witness() -> bool {
    use crate::construction::{
        cut_power_line_segment, power_damage_segment_witness_green, power_repair_queue_witness_green,
        PowerLineDamageBook, PowerLineSegmentHealth, PowerRepairQueue,
    };
    use crate::gui::hud::context_tray_power_repair_egui::{
        power_repair_panel_tier_tray_logistics, power_repair_panel_wired,
    };
    use crate::infrastructure::utility::{
        fixture_utility_network_snapshot, hydrate_utility_graph_from_snapshot,
    };
    use crate::construction::preview_island_offline_from_cut;

    let snap = fixture_utility_network_snapshot();
    let graph = hydrate_utility_graph_from_snapshot(&snap);
    let mut book = PowerLineDamageBook::default();
    for edge in &graph.power_edges {
        book.ensure_segment(edge.link_id);
    }
    let link = graph.power_edges.first().map(|e| e.link_id).unwrap_or(11);
    let preview_offline = preview_island_offline_from_cut(&graph, &snap, &book, link);
    cut_power_line_segment(&mut book, link);

    let mut queue = PowerRepairQueue::default();
    let seg = book.segments.get(&link).cloned().unwrap_or_else(|| {
        let mut s = PowerLineSegmentHealth::new(link);
        s.destroyed = true;
        s
    });
    queue.enqueue_damaged_segment(&seg, 2);

    let green = power_damage_segment_witness_green()
        && power_repair_queue_witness_green()
        && power_repair_panel_wired()
        && power_repair_panel_tier_tray_logistics()
        && book.destroyed_link_ids().contains(&link)
        && !queue.jobs.is_empty()
        && preview_offline > 0;

    let body = serde_json::json!({
        "gate": "PLAN-POWER-GRID-CONSTRUCTION-UX-001",
        "track_c_green": green,
        "COD-POWER-DAMAGE-SEGMENT-001": power_damage_segment_witness_green(),
        "COD-POWER-REPAIR-QUEUE-001": power_repair_queue_witness_green(),
        "power_repair_panel_wired": power_repair_panel_wired(),
        "panel_tier": "tray_logistics",
        "floating_repair_window": false,
        "priority_range": [1, 100],
        "cut_preview_offline": preview_offline,
        "repair_jobs": queue.jobs.len(),
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "PLAN-POWER-GRID-CONSTRUCTION-UX-001",
        "refresh_power_grid_track_c_live_witness",
        POWER_GRID_TRACK_C_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(POWER_GRID_TRACK_C_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_grid_track_c_live_witness_green() {
        assert!(refresh_power_grid_track_c_live_witness());
    }
}
