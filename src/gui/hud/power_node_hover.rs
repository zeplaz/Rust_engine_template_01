//! Map-attached hover cards for grid transformers + substations (COD-POWER-NODE-HOVER-001).

use std::collections::HashSet;

use bevy::prelude::*;

use crate::construction::{
    node_key_for_world, BuildingDefinitionRegistry, UtilityInfrastructureRole,
};
use crate::economy::activation::BuildingDefinitionRef;
use crate::infrastructure::utility::{
    facility_power_connected_from_graph, fixture_utility_network_snapshot,
    hydrate_utility_graph_from_snapshot, UtilityGraph, UtilityNetworkSnapshot,
};
use crate::infrastructure::VoltageClass;
use crate::render::{voltage_for_link, PowerMapOverlayPresentation};
use crate::strategic::{PlannedSite, SiteFootprint, SiteOperationalStats};

use super::power_grid_copy::{
    power_hover_capacity_fmt, power_hover_feeds_fmt, power_hover_links_fmt, power_hover_yard_fmt,
    POWER_HOVER_STATUS_DAMAGED, POWER_HOVER_STATUS_DESTROYED, POWER_HOVER_STATUS_OFFLINE,
    POWER_HOVER_STATUS_ONLINE, POWER_HOVER_STATUS_OVERLOAD, POWER_HOVER_SUBSTATION_TITLE,
    POWER_HOVER_TRANSFORMER_TITLE, POWER_HOVER_VOLTAGE_MIXED,
};

pub const HOVER_DWELL_SECS: f32 = 0.15;
pub const HOVER_HIDE_GRACE_SECS: f32 = 0.08;
pub const HOVER_OFFSET_X: f32 = 12.0;
pub const HOVER_OFFSET_Y: f32 = -8.0;
pub const HOVER_CARD_MIN_W: f32 = 220.0;
pub const HOVER_CARD_MAX_W: f32 = 280.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerNodeHoverStatus {
    Online,
    Offline,
    Damaged,
    Destroyed,
    Overload,
}

impl PowerNodeHoverStatus {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Online => POWER_HOVER_STATUS_ONLINE,
            Self::Offline => POWER_HOVER_STATUS_OFFLINE,
            Self::Damaged => POWER_HOVER_STATUS_DAMAGED,
            Self::Destroyed => POWER_HOVER_STATUS_DESTROYED,
            Self::Overload => POWER_HOVER_STATUS_OVERLOAD,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PowerNodeHoverCard {
    pub entity: Entity,
    pub catalog_id: String,
    pub role: UtilityInfrastructureRole,
    pub title: String,
    pub status: PowerNodeHoverStatus,
    pub voltage_label: String,
    pub load_pct: f32,
    pub capacity_line: String,
    pub feeds_line: String,
    pub links_line: Option<String>,
    pub yard_line: Option<String>,
}

#[derive(Resource, Debug, Default)]
pub struct PowerNodeHoverState {
    pub targeted_entity: Option<Entity>,
    pub dwell_secs: f32,
    pub hide_grace_secs: f32,
    pub card_visible: bool,
    pub card: Option<PowerNodeHoverCard>,
}

#[must_use]
pub fn power_node_hover_tier_map_attached() -> bool {
    true
}

#[must_use]
pub fn power_node_hover_dwell_ms() -> u32 {
    (HOVER_DWELL_SECS * 1000.0).round() as u32
}

#[must_use]
pub fn world_xy_to_map_tile(world: Vec2) -> IVec2 {
    IVec2::new(world.x.floor() as i32, world.y.floor() as i32)
}

#[must_use]
pub fn footprint_contains_tile(footprint: &SiteFootprint, tile: IVec2) -> bool {
    footprint.tiles.iter().any(|t| *t == tile)
}

#[must_use]
pub fn footprint_center_world(footprint: &SiteFootprint) -> Vec3 {
    if let Some(tile) = footprint.tiles.first() {
        Vec3::new(tile.x as f32 + 0.5, 0.0, tile.y as f32 + 0.5)
    } else {
        Vec3::ZERO
    }
}

#[must_use]
pub fn is_utility_hover_catalog(catalog_id: &str, registry: &BuildingDefinitionRegistry) -> bool {
    registry
        .get(catalog_id)
        .and_then(|d| UtilityInfrastructureRole::resolve(&d.id, d.utility_role))
        .is_some_and(|r| matches!(r, UtilityInfrastructureRole::Transformer | UtilityInfrastructureRole::Substation))
}

#[must_use]
pub fn pick_utility_hover_site_at_tile<'a>(
    sites: impl Iterator<Item = (Entity, &'a PlannedSite, &'a SiteFootprint)>,
    registry: &BuildingDefinitionRegistry,
    tile: IVec2,
) -> Option<(Entity, &'a PlannedSite, &'a SiteFootprint)> {
    sites
        .filter(|(_, planned, footprint)| {
            planned
                .catalog_id
                .as_deref()
                .is_some_and(|id| is_utility_hover_catalog(id, registry))
                && footprint_contains_tile(footprint, tile)
        })
        .min_by_key(|(_, _planned, footprint)| footprint.tiles.len())
}

#[must_use]
pub fn voltage_hover_label(
    role: UtilityInfrastructureRole,
    classes: &[VoltageClass],
) -> String {
    if classes.len() > 1 {
        let distinct: HashSet<_> = classes.iter().copied().collect();
        if distinct.len() > 1 {
            return POWER_HOVER_VOLTAGE_MIXED.to_string();
        }
    }
    let v = classes.first().copied().unwrap_or(VoltageClass::Medium);
    match role {
        UtilityInfrastructureRole::Substation => match v {
            VoltageClass::High => "Transmission in".to_string(),
            VoltageClass::Medium => "Medium in".to_string(),
            VoltageClass::Low => "Distribution in".to_string(),
        },
        UtilityInfrastructureRole::Transformer => match v {
            VoltageClass::Low => "Distribution".to_string(),
            VoltageClass::Medium => "Medium voltage".to_string(),
            VoltageClass::High => "Transmission".to_string(),
        },
        UtilityInfrastructureRole::PowerPlant => "Generation".to_string(),
    }
}

#[must_use]
pub fn incident_voltages_for_node(
    graph: &UtilityGraph,
    snap: &UtilityNetworkSnapshot,
    node_id: u64,
) -> Vec<VoltageClass> {
    graph
        .power_edges
        .iter()
        .filter(|e| e.from == node_id || e.to == node_id)
        .map(|e| voltage_for_link(snap, e.link_id))
        .collect()
}

#[must_use]
pub fn link_counts_for_node(graph: &UtilityGraph, node_id: u64) -> (u32, u32) {
    let mut lines = 0_u32;
    let mut upstream = 0_u32;
    for edge in &graph.power_edges {
        if edge.from != node_id && edge.to != node_id {
            continue;
        }
        lines += 1;
        let other = if edge.from == node_id {
            edge.to
        } else {
            edge.from
        };
        let plant = graph
            .nodes
            .iter()
            .find(|n| n.id == other)
            .is_some_and(|n| n.key.to_ascii_lowercase().contains("plant"));
        if plant {
            upstream += 1;
        }
    }
    (lines, upstream)
}

#[must_use]
pub fn downstream_consumer_count(graph: &UtilityGraph, node_id: u64) -> u32 {
    graph
        .power_edges
        .iter()
        .filter(|e| e.from == node_id || e.to == node_id)
        .map(|e| if e.from == node_id { e.to } else { e.from })
        .filter(|id| {
            graph
                .nodes
                .iter()
                .find(|n| n.id == *id)
                .is_some_and(|n| {
                    let k = n.key.to_ascii_lowercase();
                    !k.contains("plant") && !k.contains("sub_")
                })
        })
        .count() as u32
}

#[must_use]
pub fn node_incident_damaged(
    graph: &UtilityGraph,
    presentation: Option<&PowerMapOverlayPresentation>,
) -> bool {
    let Some(p) = presentation else {
        return false;
    };
    graph.power_edges.iter().any(|e| {
        p.damaged_link_ids.contains(&e.link_id) || p.destroyed_link_ids.contains(&e.link_id)
    })
}

#[must_use]
pub fn build_hover_card(
    entity: Entity,
    planned: &PlannedSite,
    footprint: &SiteFootprint,
    stats: &SiteOperationalStats,
    registry: &BuildingDefinitionRegistry,
    graph: Option<&UtilityGraph>,
    snap: Option<&UtilityNetworkSnapshot>,
    presentation: Option<&PowerMapOverlayPresentation>,
) -> Option<PowerNodeHoverCard> {
    let catalog_id = planned.catalog_id.as_deref()?;
    let def = registry.get(catalog_id)?;
    let role = UtilityInfrastructureRole::resolve(&def.id, def.utility_role)?;
    if !matches!(
        role,
        UtilityInfrastructureRole::Transformer | UtilityInfrastructureRole::Substation
    ) {
        return None;
    }

    let title = match role {
        UtilityInfrastructureRole::Transformer => POWER_HOVER_TRANSFORMER_TITLE.to_string(),
        UtilityInfrastructureRole::Substation => POWER_HOVER_SUBSTATION_TITLE.to_string(),
        UtilityInfrastructureRole::PowerPlant => def.display_name.clone(),
    };

    let center = footprint_center_world(footprint);
    let key = node_key_for_world(center);
    let node_id = graph.and_then(|g| g.nodes.iter().find(|n| n.key == key).map(|n| n.id));

    let (lines, upstream) = node_id
        .map(|id| link_counts_for_node(graph.unwrap(), id))
        .unwrap_or((0, 0));
    let feeds = node_id
        .map(|id| downstream_consumer_count(graph.unwrap(), id))
        .unwrap_or(0);

    let voltages = match (graph, snap, node_id) {
        (Some(g), Some(s), Some(id)) => incident_voltages_for_node(g, s, id),
        _ => vec![VoltageClass::Medium],
    };
    let voltage_label = voltage_hover_label(role, &voltages);

    let load_pct = (stats.power_ratio.clamp(0.0, 1.0) * 100.0).clamp(0.0, 100.0);
    let max_mva = def.transfer_capacity_mva.max(0.1);
    let used_mva = (load_pct / 100.0) * max_mva;
    let capacity_line = power_hover_capacity_fmt(used_mva, max_mva);
    let feeds_line = power_hover_feeds_fmt(feeds);
    let links_line = if lines > 0 || upstream > 0 {
        Some(power_hover_links_fmt(lines, upstream))
    } else {
        None
    };
    let yard_line = if role == UtilityInfrastructureRole::Substation {
        Some(power_hover_yard_fmt(def.footprint.width, def.footprint.depth))
    } else {
        None
    };

    let blocked: HashSet<u64> = presentation
        .map(|p| {
            p.damaged_link_ids
                .iter()
                .chain(p.destroyed_link_ids.iter())
                .copied()
                .collect()
        })
        .unwrap_or_default();
    let connected = graph
        .map(|g| facility_power_connected_from_graph(g, &Transform::from_translation(center), &blocked))
        .unwrap_or(stats.power_ratio > 0.0);
    let damaged = graph.is_some_and(|g| {
        node_id.is_some_and(|id| {
            g.power_edges
                .iter()
                .filter(|e| e.from == id || e.to == id)
                .any(|e| {
                    presentation.is_some_and(|p| {
                        p.damaged_link_ids.contains(&e.link_id)
                            || p.destroyed_link_ids.contains(&e.link_id)
                    })
                })
        }) || node_incident_damaged(g, presentation)
    });

    let status = if stats.integrity <= 0.0 {
        PowerNodeHoverStatus::Destroyed
    } else if damaged {
        PowerNodeHoverStatus::Damaged
    } else if load_pct > 90.0 {
        PowerNodeHoverStatus::Overload
    } else if !connected {
        PowerNodeHoverStatus::Offline
    } else {
        PowerNodeHoverStatus::Online
    };

    Some(PowerNodeHoverCard {
        entity,
        catalog_id: catalog_id.to_string(),
        role,
        title,
        status,
        voltage_label,
        load_pct,
        capacity_line,
        feeds_line,
        links_line,
        yard_line,
    })
}

pub fn sync_power_node_hover_pick(
    time: Res<Time>,
    gate: Res<crate::gui::hud::simulation_pointer_gate::SimulationMapPointerGate>,
    authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
    desired: Res<crate::gui::MapCameraDesired>,
    map_vp: Res<crate::gui::SimulationMapViewport>,
    params: Res<crate::terrain::generation::world_generator_enhanced::WorldGenParams>,
    registry: Res<BuildingDefinitionRegistry>,
    graph: Option<Res<UtilityGraph>>,
    snap: Option<Res<crate::infrastructure::utility::graph::UtilityNetworkSnapshotResource>>,
    presentation: Option<Res<PowerMapOverlayPresentation>>,
    sites: Query<(
        Entity,
        &PlannedSite,
        &SiteFootprint,
        &SiteOperationalStats,
        Option<&BuildingDefinitionRef>,
    )>,
    mut hover: ResMut<PowerNodeHoverState>,
) {
    let dt = time.delta_secs();
    if !map_vp.is_adequate_for_camera() || !gate.in_play_area || gate.egui_blocks {
        if hover.card_visible {
            hover.hide_grace_secs -= dt;
            if hover.hide_grace_secs <= 0.0 {
                hover.card_visible = false;
                hover.card = None;
            }
        } else {
            hover.targeted_entity = None;
            hover.dwell_secs = 0.0;
            hover.card = None;
        }
        return;
    }

    let proj = crate::construction::ConstructionMapProjection::resolve(
        authority.as_deref(),
        desired.as_ref(),
        map_vp.as_ref(),
        params.as_ref(),
    );
    let Some(world) = proj.cursor_world_xy(gate.cursor) else {
        hover.targeted_entity = None;
        hover.dwell_secs = 0.0;
        return;
    };
    let tile = world_xy_to_map_tile(world);

    let pick = pick_utility_hover_site_at_tile(
        sites.iter().map(|(e, p, f, _, _)| (e, p, f)),
        registry.as_ref(),
        tile,
    );

    match pick {
        Some((entity, planned, footprint)) => {
            hover.hide_grace_secs = HOVER_HIDE_GRACE_SECS;
            if hover.targeted_entity == Some(entity) {
                hover.dwell_secs += dt;
            } else {
                hover.targeted_entity = Some(entity);
                hover.dwell_secs = 0.0;
            }
            if hover.dwell_secs >= HOVER_DWELL_SECS {
                let stats = sites
                    .get(entity)
                    .map(|(_, _, _, stats, _)| stats.clone())
                    .unwrap_or_default();
                hover.card = build_hover_card(
                    entity,
                    planned,
                    footprint,
                    &stats,
                    registry.as_ref(),
                    graph.as_deref(),
                    snap.as_deref().map(|r| &r.0),
                    presentation.as_deref(),
                );
                hover.card_visible = hover.card.is_some();
            }
        }
        None => {
            if hover.targeted_entity.is_some() {
                hover.hide_grace_secs = HOVER_HIDE_GRACE_SECS;
            }
            hover.targeted_entity = None;
            hover.dwell_secs = 0.0;
            if hover.card_visible {
                hover.hide_grace_secs -= dt;
                if hover.hide_grace_secs <= 0.0 {
                    hover.card_visible = false;
                    hover.card = None;
                }
            }
        }
    }
}

#[must_use]
pub fn power_node_hover_witness_green() -> bool {
    let snap = fixture_utility_network_snapshot();
    let graph = hydrate_utility_graph_from_snapshot(&snap);
    let node_id = graph.nodes.first().map(|n| n.id).unwrap_or(0);
    let (lines, _) = link_counts_for_node(&graph, node_id);
    power_node_hover_tier_map_attached()
        && power_node_hover_dwell_ms() == 150
        && lines >= 1
        && voltage_hover_label(
            UtilityInfrastructureRole::Transformer,
            &[VoltageClass::Medium],
        ) == "Medium voltage"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_node_hover_witness_green_lib() {
        assert!(power_node_hover_witness_green());
    }
}
