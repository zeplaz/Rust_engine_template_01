//! Map overlay for logistics orders (PLAN-LOGISTICS-ORDERS-v1 §6 "Map").
//!
//! Standing lines draw solid, one-off runs dashed, waiting orders in the warning colour;
//! vehicles are dots interpolated along their current leg (filled = loaded, ring = empty).
//! Reads only [`LogisticsOrdersSnapshotRes`]; painted on the egui background layer so panels
//! stay on top. Map space is tiles (`world_xz / METRES_PER_TILE`).

use bevy::prelude::*;

use crate::construction::ConstructionMapProjection;
use crate::economy::logistics::orders::snapshot::METRES_PER_TILE;
use crate::economy::logistics::orders::{
    LogisticsOrdersSnapshot, LogisticsOrdersSnapshotRes, OrderStatus, SiteRole,
};
use crate::engine::states::BaseState;
use crate::gui::style::UiPalette;
use crate::gui::{MainWorldCameraOrthoTrace, MapCameraDesiredRes, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::logistics_orders_panel::LogisticsOrdersPanelState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteTone {
    Flowing,
    Waiting,
    Paused,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RouteStroke {
    pub from: Vec2,
    pub to: Vec2,
    pub standing: bool,
    pub tone: RouteTone,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VehicleDot {
    pub at: Vec2,
    pub loaded: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SiteMarker {
    pub at: Vec2,
    pub name: String,
    pub role: SiteRole,
}

/// Overlay primitives in tile space.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OrdersOverlayFrame {
    pub routes: Vec<RouteStroke>,
    pub vehicles: Vec<VehicleDot>,
    pub sites: Vec<SiteMarker>,
}

fn tile_xz(world_xz: [f32; 2]) -> Vec2 {
    Vec2::new(world_xz[0], world_xz[1]) / METRES_PER_TILE
}

/// Build overlay primitives for live orders and their vehicles.
#[must_use]
pub fn build_orders_overlay(snap: &LogisticsOrdersSnapshot, with_sites: bool) -> OrdersOverlayFrame {
    let pos = |id| snap.site(id).map(|s| tile_xz(s.world_xz));
    let mut frame = OrdersOverlayFrame::default();
    for o in snap.orders.iter().filter(|o| !o.status.is_terminal()) {
        let (Some(from), Some(to)) = (pos(o.source), pos(o.sink)) else {
            continue;
        };
        let tone = match o.status {
            OrderStatus::Waiting(_) => RouteTone::Waiting,
            OrderStatus::Paused | OrderStatus::Cancelling => RouteTone::Paused,
            _ => RouteTone::Flowing,
        };
        frame.routes.push(RouteStroke {
            from,
            to,
            standing: o.standing,
            tone,
        });
    }
    for v in snap.vehicles.iter().filter(|v| v.order.is_some()) {
        let (Some(a), Some(b)) = (pos(v.from), pos(v.to)) else {
            continue;
        };
        frame.vehicles.push(VehicleDot {
            at: a.lerp(b, v.fraction.clamp(0.0, 1.0)),
            loaded: matches!(v.phase, "hauling" | "unloading"),
        });
    }
    if with_sites {
        for s in &snap.sites {
            frame.sites.push(SiteMarker {
                at: tile_xz(s.world_xz),
                name: s.name.clone(),
                role: s.role,
            });
        }
    }
    frame
}

fn map_session(state: Option<&State<BaseState>>) -> bool {
    state.is_some_and(|s| matches!(s.get(), BaseState::Editor | BaseState::Simulation))
}

#[allow(clippy::too_many_arguments)]

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::orders::{
        OrderId, OrderRow, OwnerId, Priority, SiteId, SiteRow, VehicleId, VehicleRow,
        CargoClassMask,
    };

    fn site(id: u32, x: f32) -> SiteRow {
        SiteRow {
            id: SiteId(id),
            name: format!("S{id}"),
            owner: OwnerId::PLAYER,
            role: SiteRole::Depot,
            world_xz: [x, 0.0],
            tile: [0, 0],
            stock: vec![],
        }
    }

    fn order(id: u32, standing: bool, status: OrderStatus) -> OrderRow {
        OrderRow {
            id: OrderId(id),
            owner: OwnerId::PLAYER,
            standing,
            source: SiteId(1),
            sink: SiteId(2),
            source_name: String::new(),
            sink_name: String::new(),
            cargo: "ammo".into(),
            unit: "t",
            priority: Priority::Normal,
            status,
            status_label: String::new(),
            quantity: None,
            delivered: 0,
            in_flight: 0,
            remaining: 0,
            trips_done: 0,
            trips_planned: 0,
            vehicles: 0,
            vehicle_cap: None,
            next_eta_ticks: None,
            fill_ratio: None,
            throughput_per_min: 0.0,
            lost: 0,
            created_tick: 0,
            finished_tick: None,
        }
    }

    #[test]
    fn overlay_maps_metres_to_tiles_and_skips_finished_orders() {
        let snap = LogisticsOrdersSnapshot {
            sites: vec![site(1, 0.0), site(2, 160.0)],
            orders: vec![
                order(1, true, OrderStatus::Active),
                order(2, false, OrderStatus::Waiting(crate::economy::logistics::orders::WaitReason::NoVehicles)),
                order(3, false, OrderStatus::Completed),
            ],
            vehicles: vec![VehicleRow {
                id: VehicleId(1),
                owner: OwnerId::PLAYER,
                class_mask: CargoClassMask::FREIGHT,
                capacity_units: 10,
                phase: "hauling",
                order: Some(OrderId(1)),
                cargo: Some("ammo".into()),
                load: 10,
                at: SiteId(1),
                from: SiteId(1),
                to: SiteId(2),
                fraction: 0.5,
            }],
            ..Default::default()
        };
        let f = build_orders_overlay(&snap, true);
        assert_eq!(f.routes.len(), 2);
        assert_eq!(f.routes[0].to, Vec2::new(10.0, 0.0));
        assert!(f.routes[0].standing);
        assert_eq!(f.routes[1].tone, RouteTone::Waiting);
        assert_eq!(f.vehicles, vec![VehicleDot { at: Vec2::new(5.0, 0.0), loaded: true }]);
        assert_eq!(f.sites.len(), 2);
        assert!(build_orders_overlay(&snap, false).sites.is_empty());
    }
}
