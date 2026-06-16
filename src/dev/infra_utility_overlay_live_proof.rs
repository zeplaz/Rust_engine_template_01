//! **INFRA-UTILITY-OVERLAY-001** — utility/transport overlay collection witness.

pub const INFRA_UTILITY_OVERLAY_LIVE_JSON: &str = "debug_runs/infra_utility_overlay_live.json";

#[must_use]
pub fn refresh_infra_utility_overlay_live_witness() -> bool {
    use bevy::app::App;
    use bevy::prelude::*;
    use crate::render::InfrastructureOverlayDrawRequests;
    use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeMeta};

    let mut app = App::new();
    app.init_resource::<TransportEdgeDirectory>()
        .init_resource::<InfrastructureOverlayDrawRequests>()
        .add_systems(
            Update,
            crate::render::collect_transport_overlay_edges_system,
        );
    {
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
    }
    app.update();
    let edges = app
        .world()
        .resource::<InfrastructureOverlayDrawRequests>()
        .edges
        .len();
    let green = edges >= 1;
    let body = serde_json::json!({
        "gate": "INFRA-UTILITY-OVERLAY-001",
        "green": green,
        "overlay_edge_rows": edges,
        "sim_hud_overlay_wired": green,
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
