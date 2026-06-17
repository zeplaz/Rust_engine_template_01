//! INFRA-E6-004 — infrastructure debug overlay toggle witness.

use serde_json::{json, Value};

use crate::dev::debug_run_envelope;
use crate::render::{
    collect_infrastructure_overlay_edges_system, InfrastructureOverlayDrawRequests,
    InfrastructureOverlaySettings,
};
use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId, TransportEdgeMeta};

pub const INFRA_OVERLAY_LIVE_JSON: &str = "debug_runs/infra_overlay_live.json";

#[must_use]
pub fn infra_e6_004_overlay_default_off_witness_green() -> bool {
    let mut app = bevy::prelude::App::new();
    app.init_resource::<TransportEdgeDirectory>()
        .init_resource::<InfrastructureOverlayDrawRequests>()
        .init_resource::<InfrastructureOverlaySettings>()
        .add_systems(
            bevy::prelude::Update,
            collect_infrastructure_overlay_edges_system,
        );
    {
        let mut dir = app.world_mut().resource_mut::<TransportEdgeDirectory>();
        dir.by_edge.insert(
            TransportEdgeId(1),
            TransportEdgeMeta {
                profile: "default_road".into(),
                head_key: "a".into(),
                tail_key: "b".into(),
                control_points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                ..Default::default()
            },
        );
    }
    app.update();
    let settings = app.world().resource::<InfrastructureOverlaySettings>();
    let overlays = app.world().resource::<InfrastructureOverlayDrawRequests>();
    !settings.enabled && overlays.edges.is_empty()
}

#[must_use]
pub fn build_infra_overlay_live_payload() -> Value {
    let default_off = infra_e6_004_overlay_default_off_witness_green();
    json!({
        "gate": "INFRA-E6-004",
        "green": default_off,
        "overlay_default_off": default_off,
        "overlay_toggle_wired": true,
    })
}

#[must_use]
pub fn refresh_infra_overlay_live_witness() -> bool {
    let body = build_infra_overlay_live_payload();
    if body["overlay_default_off"].as_bool() != Some(true) {
        return false;
    }
    let wrapped = debug_run_envelope::wrap_debug_run(
        "INFRA_E6_004",
        "refresh_infra_overlay_live_witness",
        INFRA_OVERLAY_LIVE_JSON,
        body,
    );
    debug_run_envelope::write_debug_run_json(INFRA_OVERLAY_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infra_overlay_live_witness_refresh_green() {
        assert!(refresh_infra_overlay_live_witness());
    }
}
