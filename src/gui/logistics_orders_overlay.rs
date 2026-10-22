//! Map overlay for logistics orders (PLAN-LOGISTICS-ORDERS-v1 §6 "Map").
//!
//! Standing lines draw solid, one-off runs dashed, waiting orders in the warning colour;
//! vehicles are dots interpolated along their current leg (filled = loaded, ring = empty).
//! Built from the snapshot and drawn as world-map meshes by
//! `render::world_map_network_mesh::sync_world_map_orders_meshes`. Map space is tiles
//! (`world_xz / METRES_PER_TILE`) — the same `(x, row)` axes as roads and rails, so it must not
//! be scaled back to metres.

use bevy::prelude::*;

use crate::economy::logistics::orders::snapshot::METRES_PER_TILE;
use crate::economy::logistics::orders::{
    LogisticsOrdersSnapshot, OrderStatus, SiteRole,
};
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

impl OrdersOverlayFrame {
    /// Change key for redraw detection: routes, site markers and quantized vehicle positions
    /// (vehicles move every sim tick, so their positions must be part of it).
    #[must_use]
    pub fn change_key(&self) -> u64 {
        fn mix(key: &mut u64, n: u64) {
            *key = key.wrapping_mul(0x9E37_79B9).wrapping_add(n);
        }
        fn q(v: Vec2) -> u64 {
            let c = |f: f32| (f * 32.0).round() as i64 as u64;
            c(v.x) ^ c(v.y).rotate_left(21)
        }
        let mut key = 0x51_7C_C1u64;
        for r in &self.routes {
            mix(&mut key, q(r.from));
            mix(&mut key, q(r.to));
            mix(&mut key, u64::from(r.standing) + 2 * r.tone as u64);
        }
        for v in &self.vehicles {
            mix(&mut key, q(v.at));
            mix(&mut key, u64::from(v.loaded));
        }
        for s in &self.sites {
            mix(&mut key, q(s.at));
            mix(&mut key, s.role as u64);
        }
        mix(&mut key, (self.routes.len() + 7 * self.vehicles.len() + 31 * self.sites.len()) as u64);
        key
    }
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

        // A moving vehicle must change the redraw key.
        let mut moved = snap.clone();
        moved.vehicles[0].fraction = 0.75;
        assert_ne!(f.change_key(), build_orders_overlay(&moved, true).change_key());
    }
}
