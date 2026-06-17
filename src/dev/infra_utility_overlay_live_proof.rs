//! **INFRA-UTILITY-OVERLAY-001** / **CDR-B-INFRA-OVERLAY-POLISH-001** — overlay collection witness.

pub const INFRA_UTILITY_OVERLAY_LIVE_JSON: &str = "debug_runs/infra_utility_overlay_live.json";

#[must_use]
pub fn refresh_infra_utility_overlay_live_witness() -> bool {
    use bevy::app::App;
    use bevy::prelude::*;
    use crate::infrastructure::utility::{
        fixture_utility_network_snapshot, hydrate_utility_graph_from_snapshot, UtilityGraph,
    };
    use crate::render::{
        collect_infrastructure_overlay_edges_system, infra_overlay_polish_green,
        infrastructure_overlay_polish_witness_fields, InfrastructureOverlayDrawRequests,
        InfrastructureOverlaySettings,
    };
    use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeMeta};

    let mut app = App::new();
    app.init_resource::<TransportEdgeDirectory>()
        .init_resource::<UtilityGraph>()
        .init_resource::<InfrastructureOverlayDrawRequests>()
        .init_resource::<InfrastructureOverlaySettings>()
        .add_systems(Update, collect_infrastructure_overlay_edges_system);
    {
        let snap = fixture_utility_network_snapshot();
        let graph = hydrate_utility_graph_from_snapshot(&snap);
        *app.world_mut().resource_mut::<UtilityGraph>() = graph;
        let mut dir = app.world_mut().resource_mut::<TransportEdgeDirectory>();
        dir.by_edge.insert(
            crate::systems::transport::TransportEdgeId(1),
            TransportEdgeMeta {
                control_points: vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]],
                profile: "default_road".into(),
                corridor_class: crate::systems::transport::CorridorClass::Road,
                allowed_agents: vec!["wheeled".into()],
                head_key: "t0_0".into(),
                tail_key: "t10_0".into(),
            },
        );
        let mut settings = app.world_mut().resource_mut::<InfrastructureOverlaySettings>();
        settings.enabled = true;
        settings.road = true;
        settings.power = true;
    }
    app.update();
    let edges = app
        .world()
        .resource::<InfrastructureOverlayDrawRequests>()
        .edges
        .len();
    let polish = infrastructure_overlay_polish_witness_fields(
        app.world().resource::<InfrastructureOverlaySettings>(),
    );
    let polish_ok = infra_overlay_polish_green();
    let green = edges >= 2 && polish_ok;
    let body = serde_json::json!({
        "gate": "INFRA-UTILITY-OVERLAY-001",
        "green": green,
        "overlay_edge_rows": edges,
        "sim_hud_overlay_wired": edges >= 1,
        "slice_id": "CDR-B-INFRA-OVERLAY-POLISH-001",
        "overlay_readability_polish": polish.get("overlay_readability_polish"),
        "legend_row_count": polish.get("legend_row_count"),
        "power_stroke_rgb": polish.get("power_stroke_rgb"),
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "INFRA-UTILITY-OVERLAY-001",
        "refresh_infra_utility_overlay_live_witness",
        INFRA_UTILITY_OVERLAY_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(INFRA_UTILITY_OVERLAY_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infra_utility_overlay_live_witness_green() {
        assert!(refresh_infra_utility_overlay_live_witness());
    }
}
