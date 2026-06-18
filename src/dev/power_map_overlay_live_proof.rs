//! **COD-POWER-OVERLAY-RENDER-001** / **COD-POWER-ISLAND-HIGHLIGHT-001** — power map overlay witness.

pub const POWER_MAP_OVERLAY_LIVE_JSON: &str = "debug_runs/power_map_overlay_live.json";

#[must_use]
pub fn refresh_power_map_overlay_live_witness() -> bool {
    use bevy::app::App;
    use bevy::prelude::*;
    use crate::infrastructure::utility::graph::UtilityNetworkSnapshotResource;
    use crate::infrastructure::utility::{
        fixture_utility_network_snapshot, hydrate_utility_graph_from_snapshot,
        UtilityAuthoringMode, UtilityAuthoringTool, UtilityGraph, VoltageClass,
    };
    use crate::render::{
        collect_infrastructure_overlay_edges_system,
        refresh_power_island_from_damage_system, sync_power_overlay_auto_on_system,
        InfrastructureOverlayDrawRequests, InfrastructureOverlaySettings,
        PowerLineOverlayState, PowerMapOverlayPresentation, stroke_for_power_line_state,
        power_map_overlay_draw_witness_green, power_map_overlay_green,
        power_map_overlay_witness_fields,
    };
    use crate::systems::transport::TransportEdgeDirectory;

    let mut app = App::new();
    app.init_resource::<TransportEdgeDirectory>()
        .init_resource::<UtilityGraph>()
        .init_resource::<UtilityNetworkSnapshotResource>()
        .init_resource::<InfrastructureOverlayDrawRequests>()
        .init_resource::<InfrastructureOverlaySettings>()
        .init_resource::<PowerMapOverlayPresentation>()
        .init_resource::<UtilityAuthoringTool>()
        .add_systems(
            Update,
            (
                sync_power_overlay_auto_on_system,
                refresh_power_island_from_damage_system,
                collect_infrastructure_overlay_edges_system,
            )
                .chain(),
        );
    {
        let snap = fixture_utility_network_snapshot();
        let graph = hydrate_utility_graph_from_snapshot(&snap);
        *app.world_mut().resource_mut::<UtilityGraph>() = graph;
        *app.world_mut().resource_mut::<UtilityNetworkSnapshotResource>() =
            UtilityNetworkSnapshotResource(snap);
        let mut settings = app.world_mut().resource_mut::<InfrastructureOverlaySettings>();
        settings.enabled = true;
        settings.power = true;
        let mut presentation = app.world_mut().resource_mut::<PowerMapOverlayPresentation>();
        presentation.damaged_link_ids.insert(11);
        presentation.preview_segments.push((
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 4.0),
            VoltageClass::High,
        ));
        let mut authoring = app.world_mut().resource_mut::<UtilityAuthoringTool>();
        authoring.mode = UtilityAuthoringMode::PlacePower;
    }
    app.update();

    let edges = app
        .world()
        .resource::<InfrastructureOverlayDrawRequests>()
        .edges
        .len();
    let settings = app.world().resource::<InfrastructureOverlaySettings>().clone();
    let presentation = app.world().resource::<PowerMapOverlayPresentation>().clone();
    let authoring = app.world().resource::<UtilityAuthoringTool>().clone();
    let witness = power_map_overlay_witness_fields(&settings, &presentation, &authoring);
    let preview_stroke =
        stroke_for_power_line_state(VoltageClass::Medium, PowerLineOverlayState::Preview);
    let green = edges >= 2
        && power_map_overlay_green(&settings, &presentation, &authoring)
        && power_map_overlay_draw_witness_green()
        && preview_stroke.dashed;
    let body = serde_json::json!({
        "gate": "COD-POWER-OVERLAY-RENDER-001",
        "green": green,
        "overlay_power_edge_rows": edges,
        "power_overlay_auto_on_tool": witness.get("power_overlay_auto_on_tool"),
        "line_state_live": witness.get("line_state_live"),
        "line_state_preview_dashed": witness.get("line_state_preview_dashed"),
        "line_state_damaged_dash": witness.get("line_state_damaged_dash"),
        "line_state_destroyed_gap": witness.get("line_state_destroyed_gap"),
        "island_highlight_active": witness.get("island_highlight_active"),
        "island_offline_buildings": witness.get("island_offline_buildings"),
        "minimap_power_strokes": witness.get("minimap_power_strokes"),
        "map_draw_wired": witness.get("map_draw_wired"),
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "COD-POWER-OVERLAY-RENDER-001",
        "refresh_power_map_overlay_live_witness",
        POWER_MAP_OVERLAY_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(POWER_MAP_OVERLAY_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_map_overlay_live_witness_green() {
        assert!(refresh_power_map_overlay_live_witness());
    }
}
