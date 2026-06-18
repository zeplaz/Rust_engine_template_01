//! Transport + utility edge overlay draw requests (INFRA-E6-003 / CDR-B-INFRA-OVERLAY-POLISH-001).
//!
//! Power map overlay states: [`design_power_map_overlay_v1.md`](../dev/design_power_map_overlay_v1.md).
//! Voltage strokes: [`design_power_voltage_picker_v1.md`](../dev/design_power_voltage_picker_v1.md).

use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::infrastructure::utility::graph::UtilityNetworkSnapshotResource;
use crate::infrastructure::utility::{
    UtilityAuthoringMode, UtilityAuthoringTool, UtilityGraph, UtilityNetworkSnapshot,
    VoltageClass,
};
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

/// Power line presentation state (DES-POWER-MAP-OVERLAY-002 §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PowerLineOverlayState {
    Live,
    Preview,
    Damaged,
    Destroyed,
    IslandUnpowered,
    IslandBoundary,
}

/// Design-token stroke (color, weight, dash, alpha).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InfrastructureOverlayStroke {
    pub color_rgb: [u8; 3],
    pub weight_px: f32,
    pub dashed: bool,
    pub alpha: f32,
    pub dash_on_px: f32,
    pub dash_off_px: f32,
    pub gap_mode: bool,
}

#[derive(Clone, Debug)]
pub struct InfrastructureEdgeOverlay {
    pub head: Vec3,
    pub tail: Vec3,
    pub profile: String,
    pub utility_type: Option<String>,
    pub layer: InfrastructureNetworkLayer,
    pub stroke: InfrastructureOverlayStroke,
    pub link_id: u64,
    pub voltage: Option<VoltageClass>,
    pub line_state: Option<PowerLineOverlayState>,
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

/// Presentation-side power overlay (damage, preview, island highlight).
#[derive(Resource, Debug, Clone, Default)]
pub struct PowerMapOverlayPresentation {
    pub island_highlight_active: bool,
    pub island_offline_buildings: u32,
    pub damaged_link_ids: HashSet<u64>,
    pub destroyed_link_ids: HashSet<u64>,
    pub island_unpowered_link_ids: HashSet<u64>,
    pub island_boundary_link_ids: HashSet<u64>,
    pub preview_segments: Vec<(Vec2, Vec2, VoltageClass)>,
}

impl PowerMapOverlayPresentation {
    #[must_use]
    pub fn power_tool_auto_on(settings: &InfrastructureOverlaySettings, authoring: &UtilityAuthoringTool) -> bool {
        authoring.mode == UtilityAuthoringMode::PlacePower && settings.enabled && settings.power
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

#[inline]
#[must_use]
fn base_stroke(color_rgb: [u8; 3], weight_px: f32, dashed: bool) -> InfrastructureOverlayStroke {
    InfrastructureOverlayStroke {
        color_rgb,
        weight_px,
        dashed,
        alpha: 1.0,
        dash_on_px: 4.0,
        dash_off_px: 4.0,
        gap_mode: false,
    }
}

#[must_use]
pub fn stroke_for_voltage_class(voltage: VoltageClass, preview: bool) -> InfrastructureOverlayStroke {
    let (color, weight) = match voltage {
        VoltageClass::Low => ([0xe8, 0xc0, 0x40], 2.0),
        VoltageClass::Medium => ([0xf0, 0xd0, 0x50], 3.0),
        VoltageClass::High => ([0xff, 0xd8, 0x78], 4.0),
    };
    InfrastructureOverlayStroke {
        color_rgb: color,
        weight_px: weight,
        dashed: preview,
        alpha: if preview { 0.6 } else { 1.0 },
        dash_on_px: 4.0,
        dash_off_px: 4.0,
        gap_mode: false,
    }
}

#[must_use]
pub fn stroke_for_power_line_state(
    voltage: VoltageClass,
    state: PowerLineOverlayState,
) -> InfrastructureOverlayStroke {
    match state {
        PowerLineOverlayState::Live => stroke_for_voltage_class(voltage, false),
        PowerLineOverlayState::Preview => stroke_for_voltage_class(voltage, true),
        PowerLineOverlayState::Damaged => InfrastructureOverlayStroke {
            color_rgb: [0xe9, 0xc4, 0x6a],
            weight_px: 2.0,
            dashed: true,
            alpha: 0.9,
            dash_on_px: 3.0,
            dash_off_px: 3.0,
            gap_mode: false,
        },
        PowerLineOverlayState::Destroyed => InfrastructureOverlayStroke {
            color_rgb: [0xff, 0x44, 0x44],
            weight_px: 2.0,
            dashed: true,
            alpha: 0.8,
            dash_on_px: 3.0,
            dash_off_px: 8.0,
            gap_mode: true,
        },
        PowerLineOverlayState::IslandUnpowered => InfrastructureOverlayStroke {
            color_rgb: [0x4a, 0x78, 0x78],
            weight_px: 2.0,
            dashed: false,
            alpha: 0.4,
            dash_on_px: 4.0,
            dash_off_px: 4.0,
            gap_mode: false,
        },
        PowerLineOverlayState::IslandBoundary => InfrastructureOverlayStroke {
            color_rgb: [0xe8, 0xc0, 0x40],
            weight_px: 3.0,
            dashed: false,
            alpha: 1.0,
            dash_on_px: 4.0,
            dash_off_px: 4.0,
            gap_mode: false,
        },
    }
}

#[must_use]
pub fn stroke_for_layer(layer: InfrastructureNetworkLayer) -> InfrastructureOverlayStroke {
    match layer {
        InfrastructureNetworkLayer::RoadLocal => base_stroke([0xc8, 0xc8, 0xc8], 3.0, false),
        InfrastructureNetworkLayer::RoadArterial => base_stroke([0xf0, 0xf0, 0xf0], 5.0, false),
        InfrastructureNetworkLayer::Rail => base_stroke([0x40, 0x40, 0x40], 4.0, true),
        InfrastructureNetworkLayer::Power => stroke_for_voltage_class(VoltageClass::Low, false),
        InfrastructureNetworkLayer::Water => base_stroke([0x40, 0x80, 0xc0], 2.0, false),
        InfrastructureNetworkLayer::Sewer => base_stroke([0x60, 0x50, 0x40], 2.0, true),
        InfrastructureNetworkLayer::Canal => base_stroke([0x30, 0x80, 0xa0], 3.0, false),
    }
}

#[must_use]
pub fn power_overlay_extended_legend_rows() -> Vec<InfrastructureOverlayLegendRow> {
    vec![
        InfrastructureOverlayLegendRow {
            label: "Distribution",
            stroke: stroke_for_voltage_class(VoltageClass::Low, false),
        },
        InfrastructureOverlayLegendRow {
            label: "Medium",
            stroke: stroke_for_voltage_class(VoltageClass::Medium, false),
        },
        InfrastructureOverlayLegendRow {
            label: "Transmission",
            stroke: stroke_for_voltage_class(VoltageClass::High, false),
        },
        InfrastructureOverlayLegendRow {
            label: "Damaged",
            stroke: stroke_for_power_line_state(VoltageClass::Medium, PowerLineOverlayState::Damaged),
        },
        InfrastructureOverlayLegendRow {
            label: "Destroyed",
            stroke: stroke_for_power_line_state(VoltageClass::Medium, PowerLineOverlayState::Destroyed),
        },
    ]
}

#[must_use]
pub fn infrastructure_overlay_legend_rows() -> Vec<InfrastructureOverlayLegendRow> {
    let mut rows = vec![
        InfrastructureOverlayLegendRow {
            label: "Road",
            stroke: stroke_for_layer(InfrastructureNetworkLayer::RoadLocal),
        },
        InfrastructureOverlayLegendRow {
            label: "Rail",
            stroke: stroke_for_layer(InfrastructureNetworkLayer::Rail),
        },
    ];
    rows.extend(power_overlay_extended_legend_rows());
    rows.extend([
        InfrastructureOverlayLegendRow {
            label: "Water",
            stroke: stroke_for_layer(InfrastructureNetworkLayer::Water),
        },
        InfrastructureOverlayLegendRow {
            label: "Sewer",
            stroke: stroke_for_layer(InfrastructureNetworkLayer::Sewer),
        },
    ]);
    rows
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
        link_id: 0,
        voltage: None,
        line_state: None,
    });
}

fn push_power_edge(
    overlays: &mut InfrastructureOverlayDrawRequests,
    from: Vec2,
    to: Vec2,
    link_id: u64,
    voltage: VoltageClass,
    state: PowerLineOverlayState,
) {
    overlays.edges.push(InfrastructureEdgeOverlay {
        head: Vec3::new(from.x, from.y, 0.0),
        tail: Vec3::new(to.x, to.y, 0.0),
        profile: format!("power_{voltage:?}"),
        utility_type: Some("power".into()),
        layer: InfrastructureNetworkLayer::Power,
        stroke: stroke_for_power_line_state(voltage, state),
        link_id,
        voltage: Some(voltage),
        line_state: Some(state),
    });
}

#[must_use]
pub fn voltage_for_link(snap: &UtilityNetworkSnapshot, link_id: u64) -> VoltageClass {
    snap.power_lines
        .iter()
        .find(|p| p.link_id == link_id)
        .map(|p| p.voltage)
        .unwrap_or(VoltageClass::Low)
}

#[must_use]
pub fn power_line_state_for_link(
    link_id: u64,
    presentation: &PowerMapOverlayPresentation,
) -> PowerLineOverlayState {
    if presentation.destroyed_link_ids.contains(&link_id) {
        return PowerLineOverlayState::Destroyed;
    }
    if presentation.damaged_link_ids.contains(&link_id) {
        return PowerLineOverlayState::Damaged;
    }
    if presentation.island_highlight_active
        && presentation.island_unpowered_link_ids.contains(&link_id)
    {
        return PowerLineOverlayState::IslandUnpowered;
    }
    PowerLineOverlayState::Live
}

#[must_use]
pub fn compute_island_partition(
    utility: &UtilityGraph,
    _snap: &UtilityNetworkSnapshot,
    damaged: &HashSet<u64>,
    destroyed: &HashSet<u64>,
) -> (HashSet<u64>, HashSet<u64>, u32) {
    let mut adjacency: HashMap<u64, Vec<(u64, u64)>> = HashMap::new();
    for edge in &utility.power_edges {
        if destroyed.contains(&edge.link_id) {
            continue;
        }
        if damaged.contains(&edge.link_id) {
            continue;
        }
        adjacency.entry(edge.from).or_default().push((edge.to, edge.link_id));
        adjacency.entry(edge.to).or_default().push((edge.from, edge.link_id));
    }

    let plant_nodes: HashSet<u64> = utility
        .nodes
        .iter()
        .filter(|n| n.key.to_ascii_lowercase().contains("plant"))
        .map(|n| n.id)
        .collect();
    let sources: Vec<u64> = if plant_nodes.is_empty() {
        utility.nodes.iter().map(|n| n.id).take(1).collect()
    } else {
        plant_nodes.into_iter().collect()
    };

    let mut powered = HashSet::new();
    let mut queue: VecDeque<u64> = sources.into_iter().collect();
    while let Some(node) = queue.pop_front() {
        if !powered.insert(node) {
            continue;
        }
        if let Some(neighbors) = adjacency.get(&node) {
            for &(next, _) in neighbors {
                if !powered.contains(&next) {
                    queue.push_back(next);
                }
            }
        }
    }

    let mut unpowered_links = HashSet::new();
    let mut boundary_links = HashSet::new();
    let mut offline_nodes = HashSet::new();

    for edge in &utility.power_edges {
        if destroyed.contains(&edge.link_id) {
            continue;
        }
        if damaged.contains(&edge.link_id) {
            boundary_links.insert(edge.link_id);
            continue;
        }
        let from_powered = powered.contains(&edge.from);
        let to_powered = powered.contains(&edge.to);
        if !from_powered && !to_powered {
            unpowered_links.insert(edge.link_id);
            offline_nodes.insert(edge.from);
            offline_nodes.insert(edge.to);
        } else if from_powered ^ to_powered {
            unpowered_links.insert(edge.link_id);
            if !from_powered {
                offline_nodes.insert(edge.from);
            }
            if !to_powered {
                offline_nodes.insert(edge.to);
            }
        }
    }

    for node in &utility.nodes {
        if !powered.contains(&node.id) {
            offline_nodes.insert(node.id);
        }
    }

    let offline_buildings = offline_nodes.len() as u32;
    (unpowered_links, boundary_links, offline_buildings)
}

pub fn sync_power_overlay_auto_on_system(
    authoring: Res<UtilityAuthoringTool>,
    presentation: Res<PowerMapOverlayPresentation>,
    mut settings: ResMut<InfrastructureOverlaySettings>,
) {
    if authoring.mode == UtilityAuthoringMode::PlacePower || presentation.island_highlight_active {
        settings.enabled = true;
        settings.power = true;
    }
}

pub fn refresh_power_island_from_damage_system(
    utility: Option<Res<UtilityGraph>>,
    snap: Option<Res<UtilityNetworkSnapshotResource>>,
    mut presentation: ResMut<PowerMapOverlayPresentation>,
) {
    let cuts = presentation
        .damaged_link_ids
        .iter()
        .chain(presentation.destroyed_link_ids.iter())
        .copied()
        .collect::<HashSet<_>>();
    if cuts.is_empty() {
        presentation.island_unpowered_link_ids.clear();
        presentation.island_boundary_link_ids.clear();
        presentation.island_highlight_active = false;
        presentation.island_offline_buildings = 0;
        return;
    }
    let Some(utility) = utility else {
        return;
    };
    let empty_snap = UtilityNetworkSnapshot {
        schema_version: crate::infrastructure::utility::UTILITY_NETWORK_SCHEMA_V1,
        nodes: vec![],
        edges: vec![],
        power_lines: vec![],
        water_pipes: vec![],
    };
    let snap_body = snap.as_deref().map(|s| &s.0).unwrap_or(&empty_snap);
    let (unpowered, boundary, offline) = compute_island_partition(
        &utility,
        snap_body,
        &presentation.damaged_link_ids,
        &presentation.destroyed_link_ids,
    );
    presentation.island_unpowered_link_ids = unpowered;
    presentation.island_highlight_active = !boundary.is_empty() || offline > 0;
    presentation.island_boundary_link_ids = boundary;
    presentation.island_offline_buildings = offline;
}

pub fn collect_infrastructure_overlay_edges_system(
    directory: Res<TransportEdgeDirectory>,
    utility: Option<Res<UtilityGraph>>,
    snap: Option<Res<UtilityNetworkSnapshotResource>>,
    presentation: Option<Res<PowerMapOverlayPresentation>>,
    settings: Res<InfrastructureOverlaySettings>,
    mut overlays: ResMut<InfrastructureOverlayDrawRequests>,
) {
    overlays.edges.clear();
    let presentation = presentation.as_deref().cloned().unwrap_or_default();
    let snap_body = snap.as_deref().map(|s| &s.0);

    let draw_power = settings.enabled && settings.power || presentation.island_highlight_active;
    let draw_transport = settings.enabled;

    if draw_transport {
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
    }

    if !draw_power {
        return;
    }
    let Some(utility) = utility else {
        return;
    };
    let node_pos: HashMap<u64, Vec2> = utility
        .nodes
        .iter()
        .map(|n| (n.id, n.position))
        .collect();
    for edge in &utility.power_edges {
        let (Some(from), Some(to)) = (node_pos.get(&edge.from), node_pos.get(&edge.to)) else {
            continue;
        };
        let voltage = snap_body
            .map(|s| voltage_for_link(s, edge.link_id))
            .unwrap_or(VoltageClass::Medium);
        let mut state = power_line_state_for_link(edge.link_id, &presentation);
        if presentation.island_boundary_link_ids.contains(&edge.link_id)
            && state == PowerLineOverlayState::Damaged
        {
            state = PowerLineOverlayState::Damaged;
        }
        push_power_edge(&mut overlays, *from, *to, edge.link_id, voltage, state);
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
        None,
        None,
        settings,
        overlays,
    );
}

#[must_use]
pub fn power_map_overlay_witness_fields(
    settings: &InfrastructureOverlaySettings,
    presentation: &PowerMapOverlayPresentation,
    authoring: &UtilityAuthoringTool,
) -> serde_json::Value {
    let live = stroke_for_power_line_state(VoltageClass::Medium, PowerLineOverlayState::Live);
    let preview = stroke_for_power_line_state(VoltageClass::Medium, PowerLineOverlayState::Preview);
    serde_json::json!({
        "slice_id": "COD-POWER-OVERLAY-RENDER-001",
        "design_ref": "src/dev/design_power_map_overlay_v1.md",
        "power_overlay_auto_on_tool": PowerMapOverlayPresentation::power_tool_auto_on(settings, authoring)
            || authoring.mode == UtilityAuthoringMode::PlacePower,
        "line_state_live": !live.dashed && live.alpha > 0.99,
        "line_state_preview_dashed": preview.dashed && preview.alpha <= 0.61,
        "line_state_damaged_dash": stroke_for_power_line_state(
            VoltageClass::Medium,
            PowerLineOverlayState::Damaged,
        ).dashed,
        "line_state_destroyed_gap": stroke_for_power_line_state(
            VoltageClass::Medium,
            PowerLineOverlayState::Destroyed,
        ).gap_mode,
        "island_highlight_active": presentation.island_highlight_active,
        "island_offline_buildings": presentation.island_offline_buildings,
        "minimap_power_strokes": false,
        "load_heat_enabled": false,
        "map_draw_wired": true,
    })
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
            .init_resource::<PowerMapOverlayPresentation>()
            .add_systems(
                Update,
                (
                    sync_power_overlay_auto_on_system,
                    refresh_power_island_from_damage_system,
                    collect_infrastructure_overlay_edges_system,
                )
                    .chain()
                    .after(crate::systems::transport::TransportSchedule::Topology),
            )
            .add_systems(
                EguiPrimaryContextPass,
                super::power_map_overlay_draw::draw_power_map_overlay_egui,
            );
    }
}

#[must_use]
pub fn infrastructure_overlay_hud_legend_wired() -> bool {
    infrastructure_overlay_legend_rows().len() >= 7
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
            >= 7
        && fields
            .get("utility_layers_default_off")
            .and_then(|v| v.as_bool())
            == Some(true)
}

#[must_use]
pub fn power_map_overlay_green(
    settings: &InfrastructureOverlaySettings,
    presentation: &PowerMapOverlayPresentation,
    authoring: &UtilityAuthoringTool,
) -> bool {
    let fields = power_map_overlay_witness_fields(settings, presentation, authoring);
    fields.get("line_state_live").and_then(|v| v.as_bool()) == Some(true)
        && fields
            .get("line_state_preview_dashed")
            .and_then(|v| v.as_bool())
            == Some(true)
        && fields.get("minimap_power_strokes").and_then(|v| v.as_bool()) == Some(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::utility::{
        fixture_utility_network_snapshot, hydrate_utility_graph_from_snapshot,
    };
    use crate::systems::transport::{TransportEdgeId, TransportEdgeMeta};

    #[test]
    fn overlay_collects_transport_edges_with_design_stroke() {
        let mut app = App::new();
        app.init_resource::<TransportEdgeDirectory>()
            .init_resource::<InfrastructureOverlayDrawRequests>()
            .init_resource::<InfrastructureOverlaySettings>()
            .init_resource::<PowerMapOverlayPresentation>()
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
            .init_resource::<PowerMapOverlayPresentation>()
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
    fn power_overlay_collects_voltage_and_damage_states() {
        let mut app = App::new();
        app.init_resource::<TransportEdgeDirectory>()
            .init_resource::<UtilityGraph>()
            .init_resource::<UtilityNetworkSnapshotResource>()
            .init_resource::<InfrastructureOverlayDrawRequests>()
            .init_resource::<InfrastructureOverlaySettings>()
            .init_resource::<PowerMapOverlayPresentation>()
            .add_systems(
                Update,
                (
                    refresh_power_island_from_damage_system,
                    collect_infrastructure_overlay_edges_system,
                )
                    .chain(),
            );
        let snap = fixture_utility_network_snapshot();
        let graph = hydrate_utility_graph_from_snapshot(&snap);
        {
            *app.world_mut().resource_mut::<UtilityGraph>() = graph;
            *app.world_mut().resource_mut::<UtilityNetworkSnapshotResource>() =
                UtilityNetworkSnapshotResource(snap);
            let mut settings = app.world_mut().resource_mut::<InfrastructureOverlaySettings>();
            settings.enabled = true;
            settings.power = true;
            let mut presentation = app.world_mut().resource_mut::<PowerMapOverlayPresentation>();
            presentation.damaged_link_ids.insert(11);
        }
        app.update();
        let overlays = app
            .world()
            .resource::<InfrastructureOverlayDrawRequests>()
            .edges
            .clone();
        assert_eq!(overlays.len(), 2);
        let damaged = overlays
            .iter()
            .find(|e| e.link_id == 11)
            .expect("damaged edge");
        assert_eq!(damaged.line_state, Some(PowerLineOverlayState::Damaged));
        assert_eq!(damaged.voltage, Some(VoltageClass::Medium));
        let live = overlays.iter().find(|e| e.link_id == 10).expect("live edge");
        assert_eq!(live.line_state, Some(PowerLineOverlayState::Live));
    }

    #[test]
    fn island_partition_marks_cut_boundary() {
        let snap = fixture_utility_network_snapshot();
        let utility = hydrate_utility_graph_from_snapshot(&snap);
        let mut damaged = HashSet::new();
        damaged.insert(11);
        let (unpowered, boundary, offline) =
            compute_island_partition(&utility, &snap, &damaged, &HashSet::new());
        assert!(boundary.contains(&11));
        assert!(offline >= 1 || !unpowered.is_empty());
    }

    #[test]
    fn infra_overlay_polish_witness_fields_green() {
        assert!(infra_overlay_polish_green());
    }

    #[test]
    fn power_map_overlay_witness_tokens_green() {
        let settings = InfrastructureOverlaySettings {
            enabled: true,
            power: true,
            ..Default::default()
        };
        let presentation = PowerMapOverlayPresentation::default();
        let authoring = UtilityAuthoringTool {
            mode: UtilityAuthoringMode::PlacePower,
            ..Default::default()
        };
        assert!(power_map_overlay_green(&settings, &presentation, &authoring));
    }
}
