//! Pressure-driven construction intent — validated against [`crate::terrain::generation::world_generator_enhanced::WorldGenParams`]
//! and drained into [`ApprovedBuildOrders`] for corridor / structure hooks.

use bevy::prelude::{warn, Entity, Res, ResMut, Resource};

use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Built structure class for missions + field transformers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureType {
    EconomicHub,
    SensorTower,
    FireSupport,
    SupplyDepot,
    CommandPost,
    Fortification,
    Generic,
}

/// Why a faction requested a build (not a guaranteed spawn).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildReason {
    ResourceDeficit,
    Logistics,
    Strategic,
    MissionNudge,
}

/// Authored / AI-proposed site in **world tile** coordinates (column, row / z).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuildSiteTile {
    pub x: u32,
    pub z: u32,
}

/// Emergent construction order — world systems validate before committing ECS / book rows.
#[derive(Clone, Debug)]
pub struct BuildOrder {
    pub faction: Entity,
    pub site: BuildSiteTile,
    pub structure: StructureType,
    pub priority: f32,
    pub reason: BuildReason,
}

/// Incoming proposals (AI / missions / scripts).
#[derive(Resource, Default)]
pub struct BuildOrderQueue {
    pub pending: Vec<BuildOrder>,
}

/// Bounds-checked orders ready for construction / ledger systems.
#[derive(Resource, Default)]
pub struct ApprovedBuildOrders(pub Vec<BuildOrder>);

/// Pop `BuildOrderQueue` into [`ApprovedBuildOrders`] when the site lies inside `WorldGenParams` tile bounds.
pub fn process_build_order_queue_system(
    mut pending: ResMut<BuildOrderQueue>,
    mut approved: ResMut<ApprovedBuildOrders>,
    params: Res<WorldGenParams>,
) {
    if pending.pending.is_empty() {
        return;
    }
    let w = params.width;
    let h = params.height;
    for order in pending.pending.drain(..) {
        if w > 0 && h > 0 && order.site.x < w && order.site.z < h {
            approved.0.push(order);
        } else {
            warn!(
                "BuildOrder rejected (out of bounds or zero world): site {:?} world {}×{}",
                order.site, w, h
            );
        }
    }
}
