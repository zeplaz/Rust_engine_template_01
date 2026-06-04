//! Transport + utility edge overlay draw requests (INFRA-E6-003).

use bevy::prelude::*;

use crate::systems::transport::TransportEdgeDirectory;

#[derive(Clone, Debug)]
pub struct InfrastructureEdgeOverlay {
    pub head: Vec3,
    pub tail: Vec3,
    pub profile: String,
    pub utility_type: Option<String>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct InfrastructureOverlayDrawRequests {
    pub edges: Vec<InfrastructureEdgeOverlay>,
}

pub fn collect_transport_overlay_edges_system(
    directory: Res<TransportEdgeDirectory>,
    mut overlays: ResMut<InfrastructureOverlayDrawRequests>,
) {
    overlays.edges.clear();
    for meta in directory.by_edge.values() {
        if meta.control_points.len() < 2 {
            continue;
        }
        let head = meta.control_points[0];
        let tail = *meta.control_points.last().unwrap();
        overlays.edges.push(InfrastructureEdgeOverlay {
            head: Vec3::from_array(head),
            tail: Vec3::from_array(tail),
            profile: meta.profile.clone(),
            utility_type: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::transport::{TransportEdgeId, TransportEdgeMeta};

    #[test]
    fn overlay_collects_transport_edges() {
        let mut app = App::new();
        app.init_resource::<TransportEdgeDirectory>()
            .init_resource::<InfrastructureOverlayDrawRequests>()
            .add_systems(Update, collect_transport_overlay_edges_system);
        {
            let mut dir = app.world_mut().resource_mut::<TransportEdgeDirectory>();
            dir.by_edge.insert(
                TransportEdgeId(1),
                TransportEdgeMeta {
                    profile: "default_road".into(),
                    control_points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                    head_key: "a".into(),
                    tail_key: "b".into(),
                    ..Default::default()
                },
            );
        }
        app.update();
        assert_eq!(
            app.world()
                .resource::<InfrastructureOverlayDrawRequests>()
                .edges
                .len(),
            1
        );
    }
}
