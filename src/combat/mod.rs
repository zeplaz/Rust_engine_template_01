//! **Combat / front domain** (fleet `front`, PLAN-FRONT-COMBAT-v1 — `src/dev/plan_front_combat_v1.md`).
//!
//! Pure, headless, deterministic cores live here; Bevy systems are thin wrappers.
//! Attaches to authoritative fields: `ChunkStrategicOverlay` (control / threat), `FrontlineState`,
//! construction sites (fortification), the `Zone` pass (formation presence). No second field store.

use bevy::prelude::*;

pub mod balance;
pub mod fortification;
pub mod front;
pub mod orders;
pub mod pressure;
pub mod supply;
pub mod relax;
pub mod theater;

pub use balance::{
    load_front_balance, load_front_balance_from_path, parse_front_balance, FortKind,
    FortificationProfile, FrontBalance, UnitKind, UnitProfile, FRONT_BALANCE_RON,
};
pub use front::{
    classify_cell, derive_fronts, hostility_from_stances, ControlGrid, FrontCell, FrontDerivation, FrontPolyline, FrontSegment,
    Hostility,
};
pub use relax::{control_relaxation_system, relax_step, ControlRelaxation, RelaxCell, RelaxGrid, RelaxParams};
pub use theater::{
    assign_stable_ids, derive_theaters_system, partition_segments, Theater, TheaterBook, TheaterCandidate, TheaterId,
};
pub use fortification::FortificationIndex;
pub use pressure::{
    score_cell, score_pressure_points_system, score_theater, CellFields, PressureDrivers, PressureKind, PressurePoint,
    PressurePointBook,
};
pub use orders::{DeploymentKind, DeploymentOrder, DeploymentOrderId, DeploymentOrderQueue, EnqueueError};
pub use supply::{front_demand_targets, publish_front_supply_demands_system, FrontSupplyPoint, FrontSupplyPoints};

/// Load outcome of [`FrontBalance`] (disk error keeps the compiled-in default).
#[derive(Resource, Clone, Debug, Default)]
pub struct FrontBalanceLoadReport {
    pub error: Option<String>,
}

/// Registers front / combat resources and systems. Added by [`crate::strategic::FactionPlugin`].
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<FrontBalance>() {
            let (balance, error) = load_front_balance();
            if let Some(err) = &error {
                warn!("front balance: using compiled-in default ({err})");
            }
            app.insert_resource(balance)
                .insert_resource(FrontBalanceLoadReport { error });
        }
        // FC-2: control relaxes over sim time; the zone pass stops stamping Control zones once this exists.
        app.init_resource::<ControlRelaxation>().add_systems(
            Update,
            control_relaxation_system
                .before(crate::strategic::apply_zones_to_strategic_overlays_system)
                .in_set(crate::strategic::StrategicFieldPipeline::ZoneAndReadModel),
        );
        // FC-3: theaters follow the derived front.
        app.init_resource::<TheaterBook>().add_systems(
            Update,
            derive_theaters_system
                .after(crate::strategic::derive_frontline_from_control_system)
                .in_set(crate::strategic::StrategicFieldPipeline::ZoneAndReadModel),
        );
        // FC-4: pressure points over the theaters.
        app.init_resource::<FortificationIndex>()
            .init_resource::<PressurePointBook>()
            .add_systems(
                Update,
                score_pressure_points_system
                    .after(derive_theaters_system)
                    .in_set(crate::strategic::StrategicFieldPipeline::ZoneAndReadModel),
            );
        // FC-6 groundwork: one order queue for UI + AI; fronts publish supply demand to logistics.
        app.init_resource::<DeploymentOrderQueue>()
            .init_resource::<FrontSupplyPoints>()
            .add_systems(
                Update,
                publish_front_supply_demands_system
                    .after(derive_theaters_system)
                    .in_set(crate::strategic::StrategicFieldPipeline::ZoneAndReadModel),
            );
    }
}
