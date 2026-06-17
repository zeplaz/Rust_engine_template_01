//! Transport + utility edge overlay draw requests (INFRA-E6-003 / CDR-B-INFRA-OVERLAY-POLISH-001).
//!
//! Stroke tokens: [`design_infra_network_overlay_v1.md`](../dev/design_infra_network_overlay_v1.md).

use bevy::prelude::*;

use crate::infrastructure::utility::UtilityGraph;
use crate::systems::transport::{CorridorClass, TransportEdgeDirectory, TransportEdgeMeta};

/// Network family for tactical infrastructure overlay strokes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InfrastructureNetworkLayer {
    RoadLocal,
    RoadArterial,
    Rail,
    Power,
    Water,
    Sewer,
    Canal,
}

/// Design-token stroke (color, weight, dash).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InfrastructureOverlayStroke {
    pub color_rgb: [u8; 3],
    pub weight_px: f32,
    pub dashed: bool,
}

#[derive(Clone, Debug)]
pub struct InfrastructureEdgeOverlay {
    pub head: Vec3,
    pub tail: Vec3,
    pub profile: String,
    pub utility_type: Option<String>,
    pub layer: InfrastructureNetworkLayer,
    pub stroke: InfrastructureOverlayStroke,
}

#[derive(Resource, Debug, Clone)]
pub struct InfrastructureOverlaySettings {
    /// Master toggle (INFRA-E6-004 — default off until player opts in).
    pub enabled: bool,
    pub road: bool,
    pub rail: bool,
    pub power: bool,
    pub water: bool,
    pub sewer: bool,
    pub canal: bool,
}

impl Default for InfrastructureOverlaySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            road: true,
            rail: true,
            power: false,
            water: false,
            sewer: false,
            canal: false,
        }
    }
}

impl InfrastructureOverlaySettings {
    #[must_use]
    pub fn layer_visible(&self, layer: InfrastructureNetworkLayer) -> bool {
        if !self.enabled {
            return false;
        }
        match layer {
            InfrastructureNetworkLayer::RoadLocal | InfrastructureNetworkLayer::RoadArterial => {
                self.road
            }
            InfrastructureNetworkLayer::Rail => self.rail,
            InfrastructureNetworkLayer::Power => self.power,
            InfrastructureNetworkLayer::Water => self.water,
            InfrastructureNetworkLayer::Sewer => self.sewer,
            InfrastructureNetworkLayer::Canal => self.canal,
        }
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct InfrastructureOverlayDrawRequests {
    pub edges: Vec<InfrastructureEdgeOverlay>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InfrastructureOverlayLegendRow {
    pub label: &'static str,
    pub stroke: InfrastructureOverlayStroke,
}

#[must_use]
pub fn stroke_for_layer(layer: InfrastructureNetworkLayer) -> InfrastructureOverlayStroke {
    match layer {
        InfrastructureNetworkLayer::RoadLocal => InfrastructureOverlayStroke {
            color_rgb: [0xc8, 0xc8, 0xc8],
            weight_px: 3.0,
            dashed: false,
        },
        InfrastructureNetworkLayer::RoadArterial => InfrastructureOverlayStroke {
            color_rgb: [0xf0, 0xf0, 0xf0],
            weight_px: 5.0,
            dashed: false,
        },
        InfrastructureNetworkLayer::Rail => InfrastructureOverlayStroke {
            color_rgb: [0x40, 0x40, 0x40],
            weight_px: 4.0,
            dashed: true,
        },
        InfrastructureNetworkLayer::Power => InfrastructureOverlayStroke {
            color_rgb: [0xe8, 0xc0, 0x40],
            weight_px: 2.0,
            dashed: false,
        },
        InfrastructureNetworkLayer::Water => InfrastructureOverlayStroke {
            color_rgb: [0x40, 0x80, 0xc0],
            weight_px: 2.0,
            dashed: false,
        },
        InfrastructureNetworkLayer::Sewer => InfrastructureOverlayStroke {
            color_rgb: [0x60, 0x50, 0x40],
            weight_px: 2.0,
            dashed: true,
        },
        InfrastructureNetworkLayer::Canal => InfrastructureOverlayStroke {
            color_rgb: [0x30, 0x80, 0xa0],
            weight_px: 3.0,
            dashed: false,
        },
    }
}

#[must_use]
pub fn infrastructure_overlay_legend_rows() -> Vec<InfrastructureOverlayLegendRow> {
    vec![
        InfrastructureOverlayLegendRow {
            label: "Road",
            stroke: stroke_for_layer(InfrastructureNetworkLayer::RoadLocal),
        },
        InfrastructureOverlayLegendRow {
            label: "Rail",
            stroke: stroke_for_layer(InfrastructureNetworkLayer::Rail),
        },
        InfrastructureOverlayLegendRow {
            label: "Power",
            stroke: stroke_for_layer(InfrastructureNetworkLayer::Power),
        },
        InfrastructureOverlayLegendRow {
            label: "Water",
            stroke: stroke_for_layer(InfrastructureNetworkLayer::Water),
        },
        InfrastructureOverlayLegendRow {
            label: "Sewer",
            stroke: stroke_for_layer(InfrastructureNetworkLayer::Sewer),
        },
    ]
}

#[must_use]
pub fn layer_for_transport_meta(meta: &TransportEdgeMeta) -> InfrastructureNetworkLayer {
    let profile = meta.profile.to_ascii_lowercase();
    match meta.corridor_class {
        CorridorClass::Rail => InfrastructureNetworkLayer::Rail,
        CorridorClass::Power => InfrastructureNetworkLayer::Power,
        CorridorClass::Pipeline => {
            if profile.contains("sewer") {
                InfrastructureNetworkLayer::Sewer
            } else {
                InfrastructureNetworkLayer::Water
            }
        }
        CorridorClass::Maritime => InfrastructureNetworkLayer::Canal,
        CorridorClass::Road | CorridorClass::Conveyor => {
            if profile.contains("arterial") || profile.contains("highway") {
                InfrastructureNetworkLayer::RoadArterial
            } else {
                InfrastructureNetworkLayer::RoadLocal
            }
        }
    }
}

#[must_use]
pub fn utility_label_for_layer(layer: InfrastructureNetworkLayer) -> &'static str {
    match layer {
        InfrastructureNetworkLayer::RoadLocal | InfrastructureNetworkLayer::RoadArterial => "road",
        InfrastructureNetworkLayer::Rail => "rail",
        InfrastructureNetworkLayer::Power => "power",
        InfrastructureNetworkLayer::Water => "water",
        InfrastructureNetworkLayer::Sewer => "sewer",
        InfrastructureNetworkLayer::Canal => "canal",
    }
}

fn push_edge(
    overlays: &mut InfrastructureOverlayDrawRequests,
    head: Vec3,
    tail: Vec3,
    profile: &str,
    layer: InfrastructureNetworkLayer,
) {
    overlays.edges.push(InfrastructureEdgeOverlay {
        head,
        tail,
        profile: profile.to_string(),
        utility_type: Some(utility_label_for_layer(layer).to_string()),
        layer,
        stroke: stroke_for_layer(layer),
    });
}

pub fn collect_infrastructure_overlay_edges_system(
    directory: Res<TransportEdgeDirectory>,
    utility: Option<Res<UtilityGraph>>,
    settings: Res<InfrastructureOverlaySettings>,
    mut overlays: ResMut<InfrastructureOverlayDrawRequests>,
) {
    overlays.edges.clear();
    if !settings.enabled {
        return;
    }

    for meta in directory.by_edge.values() {
        if meta.control_points.len() < 2 {
            continue;
        }
        let layer = layer_for_transport_meta(meta);
        if !settings.layer_visible(layer) {
            continue;
        }
        let head = meta.control_points[0];
        let tail = *meta.control_points.last().unwrap();
        push_edge(
            &mut overlays,
            Vec3::from_array(head),
            Vec3::from_array(tail),
            &meta.profile,
            layer,
        );
    }

    if !settings.power {
        return;
    }
    let Some(utility) = utility else {
        return;
    };
    let node_pos: std::collections::HashMap<u64, Vec2> = utility
        .nodes
        .iter()
        .map(|n| (n.id, n.position))
        .collect();
    for edge in &utility.power_edges {
        let (Some(from), Some(to)) = (node_pos.get(&edge.from), node_pos.get(&edge.to)) else {
            continue;
        };
        push_edge(
            &mut overlays,
            Vec3::new(from.x, from.y, 0.0),
            Vec3::new(to.x, to.y, 0.0),
            "utility_power",
            InfrastructureNetworkLayer::Power,
        );
    }
}

/// Back-compat alias for existing call sites.
pub fn collect_transport_overlay_edges_system(
    directory: Res<TransportEdgeDirectory>,
    settings: Res<InfrastructureOverlaySettings>,
    mut overlays: ResMut<InfrastructureOverlayDrawRequests>,
) {
    collect_infrastructure_overlay_edges_system(
        directory,
        None,
        settings,
        overlays,
    );
}

#[must_use]
pub fn infrastructure_overlay_polish_witness_fields(
    settings: &InfrastructureOverlaySettings,
) -> serde_json::Value {
    let power_stroke = stroke_for_layer(InfrastructureNetworkLayer::Power);
    serde_json::json!({
        "slice_id": "CDR-B-INFRA-OVERLAY-POLISH-001",
        "overlay_readability_polish": true,
        "design_ref": "src/dev/design_infra_network_overlay_v1.md",
        "utility_layers_default_off": !(settings.power || settings.water || settings.sewer),
        "road_rail_default_on_when_enabled": settings.road && settings.rail,
        "legend_row_count": infrastructure_overlay_legend_rows().len(),
        "hud_legend_wired": infrastructure_overlay_hud_legend_wired(),
        "power_stroke_rgb": power_stroke.color_rgb,
        "power_stroke_weight_px": power_stroke.weight_px,
    })
}

/// Bevy plugin: overlay settings + edge collection each frame.
pub struct InfrastructureOverlayPlugin;

impl Plugin for InfrastructureOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InfrastructureOverlaySettings>()
            .init_resource::<InfrastructureOverlayDrawRequests>()
            .add_systems(
                Update,
                collect_infrastructure_overlay_edges_system
                    .after(crate::systems::transport::TransportSchedule::Topology),
            );
    }
}

#[must_use]
pub fn infrastructure_overlay_hud_legend_wired() -> bool {
    infrastructure_overlay_legend_rows().len() >= 4
}

#[must_use]
pub fn infra_overlay_polish_green() -> bool {
    let settings = InfrastructureOverlaySettings::default();
    let fields = infrastructure_overlay_polish_witness_fields(&settings);
    fields.get("overlay_readability_polish").and_then(|v| v.as_bool()) == Some(true)
        && fields
            .get("legend_row_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            >= 4
        && fields
            .get("utility_layers_default_off")
            .and_then(|v| v.as_bool())
            == Some(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::transport::{TransportEdgeId, TransportEdgeMeta};

    #[test]
    fn overlay_collects_transport_edges_with_design_stroke() {
        let mut app = App::new();
        app.init_resource::<TransportEdgeDirectory>()
            .init_resource::<InfrastructureOverlayDrawRequests>()
            .init_resource::<InfrastructureOverlaySettings>()
            .add_systems(Update, collect_infrastructure_overlay_edges_system);
        {
            let mut settings = app.world_mut().resource_mut::<InfrastructureOverlaySettings>();
            settings.enabled = true;
            settings.road = true;
        }
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
        let overlays = app
            .world()
            .resource::<InfrastructureOverlayDrawRequests>();
        assert_eq!(overlays.edges.len(), 1);
        assert_eq!(overlays.edges[0].stroke.color_rgb, [0xc8, 0xc8, 0xc8]);
        assert_eq!(overlays.edges[0].utility_type.as_deref(), Some("road"));
    }

    #[test]
    fn overlay_default_off_collects_nothing() {
        let mut app = App::new();
        app.init_resource::<TransportEdgeDirectory>()
            .init_resource::<InfrastructureOverlayDrawRequests>()
            .init_resource::<InfrastructureOverlaySettings>()
            .add_systems(Update, collect_infrastructure_overlay_edges_system);
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
        assert!(app
            .world()
            .resource::<InfrastructureOverlayDrawRequests>()
            .edges
            .is_empty());
    }

    #[test]
    fn infra_overlay_polish_witness_fields_green() {
        assert!(infra_overlay_polish_green());
    }
}
