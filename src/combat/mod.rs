//! **Combat / front domain** (fleet `front`, PLAN-FRONT-COMBAT-v1 — `src/dev/plan_front_combat_v1.md`).
//!
//! Pure, headless, deterministic cores live here; Bevy systems are thin wrappers.
//! Attaches to authoritative fields: `ChunkStrategicOverlay` (control / threat), `FrontlineState`,
//! construction sites (fortification), the `Zone` pass (formation presence). No second field store.

use bevy::prelude::*;

pub mod balance;
pub mod ballistics;
pub mod capability_toe;
pub mod drone_doctrine;
pub mod drone_missions;
pub mod drone_research;
pub mod drone_supply;
pub mod formation;
pub mod fortification;
pub mod front;
pub mod missile_supply;
pub mod orders;
pub mod personnel_supply;
pub mod pressure;
pub mod supply;
pub mod relax;
pub mod resolve;
pub mod theater;
pub mod weather;
pub mod weather_poll_sites;

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
pub use fortification::{
    fort_kind_of, plan_fortification_rows, rebuild_fortification_index_system, FortificationIndex, FortifyRow,
};
pub use pressure::{
    score_cell, score_pressure_points_system, score_theater, CellFields, PressureDrivers, PressureKind, PressurePoint,
    PressurePointBook,
};
pub use formation::{
    advance_combat_clock_system, advance_formations_system, dig_in_system, formation_supply_system, spawn_formation,
    sync_formation_zones_system, CombatClock, Formation, FormationFireZone, Posture,
};
pub use orders::{
    apply_deployment_order, execute_deployment_orders_system, order_for_pressure_point, DeploymentKind,
    DeploymentOrder, DeploymentOrderId, DeploymentOrderQueue, DeploymentOutcomes, DeploymentReject, EnqueueError,
    FortifyPlans, OrderEffect, OutcomeRecord,
};
pub use resolve::{combat_resolve_system, resolve_step, Combatant, Outcome};
pub use weather::{
    artillery_weather_stamp, engagement_visibility_scale, ArtilleryWeatherDump,
};
pub use weather_poll_sites::gather_weather_poll_sites;
pub use drone_missions::{
    apply_missile_sortie_launches_system, pulse_drone_sorties, step_drone_missions_system,
    AirOpZone, DroneAo, DroneCombatModifiers, DroneMissionDump, DroneMissionKind, DronePackage,
    DroneRole, MissionBook, MissionIntent, MissionIntentQueue,
};
pub use drone_research::DroneResearchState;
pub use drone_doctrine::{doctrine_preferred_role, FactionDroneDoctrine};
pub use capability_toe::{
    load_capability_toe_registry, CapabilityToeRegistry, MissileClass, CAPABILITY_TOE_RON,
};
pub use drone_supply::{
    publish_drone_cargo_demands_system, refill_drone_packages_from_front_system, DroneCargoDemandBook,
};
pub use ballistics::{BallisticFlightBook, BallisticKind, BallisticsPlugin};
pub use missile_supply::{
    publish_missile_cargo_demands_system, pulse_missile_sorties, MissileSortieLaunch,
    MissileSortieLaunchQueue, refill_missile_packages_from_front_system, MissileCargoDemandBook,
    MissilePackage, MissileSortieDump,
};
pub use personnel_supply::{
    publish_personnel_demands_system, refill_personnel_credits_from_front_system,
    PersonnelRestockBook,
};
pub use supply::{
    draw_front_supply_system, formation_draw, front_demand_targets, front_draw_kg_per_s, front_draw_t_per_min,
    publish_front_supply_demands_system, update_front_supply_coverage_system, FrontSupplyCoverage, FrontSupplyDraw,
    FrontSupplyPoint, FrontSupplyPoints, KG_PER_TONNE,
};

/// Load outcome of [`FrontBalance`] (disk error keeps the compiled-in default).
#[derive(Resource, Clone, Debug, Default)]
pub struct FrontBalanceLoadReport {
    pub error: Option<String>,
}

/// Registers front / combat resources and systems. Added by [`crate::strategic::FactionPlugin`].
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        crate::systems::sim_control::ensure_sim_step_clock(app);
        app.add_plugins(ballistics::BallisticsPlugin);
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
            .init_resource::<DeploymentOutcomes>()
            .init_resource::<FortifyPlans>()
            .init_resource::<FrontSupplyPoints>()
            .init_resource::<FrontSupplyCoverage>()
            .init_resource::<FrontSupplyDraw>()
            .add_systems(
                Update,
                (
                    publish_front_supply_demands_system.after(derive_theaters_system),
                    update_front_supply_coverage_system.after(publish_front_supply_demands_system),
                    draw_front_supply_system
                        .after(publish_front_supply_demands_system)
                        .after(crate::systems::sim_control::SimControlSystemSet::AdvanceSimTick),
                )
                    .in_set(crate::strategic::StrategicFieldPipeline::ZoneAndReadModel),
            );
        // FC-5: formations. Orders -> supply -> combat -> movement -> zones, all before the relaxation
        // that reads those zones, so a formation's presence this frame is what moves control.
        app.add_systems(
            Update,
            gather_weather_poll_sites
                .before(crate::systems::chunk_environment_set::ChunkEnvironmentSet::Weather)
                .in_set(crate::systems::chunk_environment_set::ChunkEnvironmentSet::Lod),
        );
        app.add_systems(
            Update,
            rebuild_fortification_index_system
                .before(combat_resolve_system)
                .in_set(crate::strategic::StrategicFieldPipeline::ZoneAndReadModel),
        );
        app.init_resource::<CombatClock>()
            .init_resource::<ArtilleryWeatherDump>()
            .init_resource::<MissionIntentQueue>()
            .init_resource::<MissionBook>()
            .init_resource::<DroneCombatModifiers>()
            .init_resource::<DroneMissionDump>()
            .init_resource::<DroneResearchState>()
            .init_resource::<FactionDroneDoctrine>()
            .init_resource::<DroneCargoDemandBook>()
            .init_resource::<MissileCargoDemandBook>()
            .init_resource::<MissileSortieDump>()
            .init_resource::<MissileSortieLaunchQueue>()
            .init_resource::<PersonnelRestockBook>()
            .init_resource::<CapabilityToeRegistry>()
            .add_systems(
                Startup,
                capability_toe::load_capability_toe_on_startup,
            )
            .add_systems(
            Update,
            (
                advance_combat_clock_system,
                execute_deployment_orders_system,
                // D1–D6 / P6: mission intents + weather-gated drone/missile sorties before resolve.
                step_drone_missions_system,
                refill_drone_packages_from_front_system,
                refill_missile_packages_from_front_system,
                refill_personnel_credits_from_front_system,
                formation_supply_system,
            )
                .chain()
                .before(combat_resolve_system)
                .in_set(crate::strategic::StrategicFieldPipeline::ZoneAndReadModel),
        )
        .add_systems(
            Update,
            apply_missile_sortie_launches_system
                .after(step_drone_missions_system)
                .before(combat_resolve_system)
                .in_set(crate::strategic::StrategicFieldPipeline::ZoneAndReadModel),
        )
        .add_systems(
            Update,
            (
                combat_resolve_system,
                advance_formations_system,
                dig_in_system,
                sync_formation_zones_system,
            )
                .chain()
                .after(formation_supply_system)
                .before(control_relaxation_system)
                .in_set(crate::strategic::StrategicFieldPipeline::ZoneAndReadModel),
        );
        app.add_systems(
            Update,
            (
                publish_drone_cargo_demands_system,
                publish_missile_cargo_demands_system,
                publish_personnel_demands_system,
            )
                .after(publish_front_supply_demands_system)
                .in_set(crate::strategic::StrategicFieldPipeline::ZoneAndReadModel),
        );
    }
}
