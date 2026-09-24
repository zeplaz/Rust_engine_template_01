//! FIRE-ECOLOGY-REFRESH-001 / FIRE-VERIFY-ECOLOGY-001 — lib harness for `fire_ecology_live.json`.

use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::dev::runtime_witness::fire::{
    commit_fire_ecology_live_proof_unchecked, FireEcologyCommitLabels,
};
use crate::dev::sim_effect_spine_live_proof::sim_effect_spine_proof_state;
use crate::engine::states::BaseState;
use crate::gui::InputBindings;
use crate::sim::effects::build_sim_effect_spine_proof_payload;
use crate::systems::chunk_environment_persist::ChunkEnvironmentPersistPlugin;
use crate::systems::chunk_sim_lod::ChunkSimLodPlugin;
use crate::systems::ecology::VegetationField;
use crate::systems::fire::witness_collectors::FireEcologyWitness;
use crate::systems::fire::{chunk_fuel_profile_from_vegetation, FirePlugin};
use crate::systems::sim_control::SimControlPlugin;
use crate::systems::weather::{ChunkWeather, WeatherSimulationPlugin};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

fn harness_vegetation() -> VegetationField {
    VegetationField {
        canopy_density: 0.35,
        understory_density: 0.42,
        ground_fuel: 0.62,
        dryness: 0.88,
        fuel_load: 0.55,
        old_growth: 0.38,
        fragmentation: 0.12,
        smoke_absorption: 0.0,
        concealment: 0.0,
        burn_severity: 0.0,
        regrowth_stage: 0.0,
    }
}

fn assemble_fire_ecology_harness_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, InputPlugin));
    app.init_resource::<InputBindings>();
    app.init_state::<BaseState>();
    app.insert_state(BaseState::Simulation);
    app.add_plugins(SimControlPlugin);
    crate::systems::chunk_environment_set::configure_chunk_environment_sets(&mut app);
    // Ember emit needs the queue resource; full `SimEffectsPlugin` pulls grid/hydrology deps.
    app.init_resource::<crate::sim::effects::SimEffectQueue>();
    app.add_plugins((
        ChunkEnvironmentPersistPlugin,
        ChunkSimLodPlugin,
        WeatherSimulationPlugin,
        FirePlugin,
    ));
    app.init_resource::<crate::dev::runtime_witness::fire::FireEcologyLiveProofState>();
    app.world_mut()
        .resource_mut::<crate::dev::runtime_witness::fire::FireEcologyLiveProofState>()
        .cadence
        .write_interval = 1;

    let veg_burn = harness_vegetation();
    let profile_burn = chunk_fuel_profile_from_vegetation(&veg_burn);

    // Must match combustion::near_empty_vegetation_blocks_ignition_gate —
    // `..Default` leaves canopy/understory high enough to *pass* MIN_WILDLAND_FUEL_MASS
    // and leave f1_green dishonest (FIRE-F1-STRICT).
    let veg_gate = VegetationField {
        ground_fuel: 0.05,
        canopy_density: 0.05,
        understory_density: 0.05,
        dryness: 0.9,
        old_growth: 0.02,
        fuel_load: 0.05,
        ..Default::default()
    };
    let profile_gate = chunk_fuel_profile_from_vegetation(&veg_gate);

    let mut matrix_burn = ChunkCellMatrix::new(UVec2::new(4, 4));
    let mut matrix_gate = ChunkCellMatrix::new(UVec2::new(4, 4));
    for matrix in [&mut matrix_burn, &mut matrix_gate] {
        for m in matrix.moisture.iter_mut() {
            *m = 0.18;
        }
        for t in matrix.temperature.iter_mut() {
            *t = 0.42;
        }
        for e in matrix.elevation.iter_mut() {
            *e = 0.5;
        }
    }

    app.world_mut().spawn((
        Chunk {
            coord: IVec2::ZERO,
        },
        matrix_burn,
        veg_burn,
        profile_burn,
        ChunkWeather::default(),
    ));
    app.world_mut().spawn((
        Chunk {
            coord: IVec2::new(1, 0),
        },
        matrix_gate,
        veg_gate,
        profile_gate,
        ChunkWeather::default(),
    ));
    app
}

#[must_use]
pub fn run_fire_ecology_lib_harness() -> FireEcologyWitness {
    let mut app = assemble_fire_ecology_harness_app();
    app.update();
    {
        let world = app.world_mut();
        let mut query = world.query::<(&mut crate::systems::fire::ChunkFireOverlay, &Chunk)>();
        for (mut ovl, chunk) in query.iter_mut(world) {
            if chunk.coord != IVec2::ZERO {
                continue;
            }
            let mid = ovl.heat.len() / 2;
            if let Some(h) = ovl.heat.get_mut(mid) {
                *h = 0.72;
            }
        }
    }
    for _ in 0..64 {
        app.update();
    }
    app.world()
        .resource::<FireEcologyWitness>()
        .clone()
}

#[must_use]
pub fn fire_f2_fuel_spread_green(witness: &FireEcologyWitness) -> bool {
    witness.fuel_depleted_cells > 0 || witness.neighbor_spread_cells > 0
}

#[must_use]
pub fn fire_ecology_lib_harness_green(witness: &FireEcologyWitness) -> bool {
    let heat_ok = witness.heat_mostly_stable();
    let f1_ok = witness.f1_fuel_gate_active();
    let f2_ok = fire_f2_fuel_spread_green(witness);
    // Overall ecology green still accepts F2 spread; FIRE-F1-STRICT uses `f1_green` alone.
    heat_ok && (f1_ok || f2_ok)
}

/// **FIRE-F1-STRICT** — fuel/old-growth gate must have blocked real spark attempts.
#[must_use]
pub fn fire_ecology_f1_strict_green(witness: &FireEcologyWitness) -> bool {
    witness.f1_fuel_gate_active() && witness.heat_mostly_stable()
}

#[must_use]
pub fn refresh_fire_ecology_lib_harness_witness() -> bool {
    let witness = run_fire_ecology_lib_harness();
    if !fire_ecology_lib_harness_green(&witness) {
        return false;
    }

    // Same sole writer as runtime — nest spine after a real drain finalize (proof_state).
    let (spine_witness, spine_queue, spine_ledger, spine_faction_react) = sim_effect_spine_proof_state();
    let spine_payload = build_sim_effect_spine_proof_payload(
        &spine_witness,
        &spine_queue,
        &spine_ledger,
        Some(&spine_faction_react),
    );
    commit_fire_ecology_live_proof_unchecked(
        &witness,
        Some(&spine_payload),
        FireEcologyCommitLabels::lib_harness(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev::runtime_witness::fire::FIRE_ECOLOGY_JSON;
    use crate::systems::fire::{chunk_fuel_profile_from_vegetation, combustion::{fuel_ignition_gate, MIN_WILDLAND_FUEL_MASS}};

    #[test]
    fn harness_vegetation_profile_passes_fuel_gate() {
        let profile = chunk_fuel_profile_from_vegetation(&harness_vegetation());
        assert!(
            profile.wildland_fuel_mass >= MIN_WILDLAND_FUEL_MASS,
            "wildland={} should pass gate",
            profile.wildland_fuel_mass
        );
        assert!(fuel_ignition_gate(profile.wildland_fuel_mass) > 0.0);
    }

    #[test]
    fn fire_ecology_lib_harness_f2_fuel_spread_counters() {
        let witness = run_fire_ecology_lib_harness();
        assert!(fire_f2_fuel_spread_green(&witness), "depleted={} spread={}", witness.fuel_depleted_cells, witness.neighbor_spread_cells);
    }

    #[test]
    fn harness_gate_chunk_blocks_ignition_gate() {
        let veg_gate = VegetationField {
            ground_fuel: 0.05,
            canopy_density: 0.05,
            understory_density: 0.05,
            dryness: 0.9,
            old_growth: 0.02,
            fuel_load: 0.05,
            ..Default::default()
        };
        let profile = chunk_fuel_profile_from_vegetation(&veg_gate);
        assert!(
            profile.wildland_fuel_mass < MIN_WILDLAND_FUEL_MASS,
            "wildland={} must be below MIN={}",
            profile.wildland_fuel_mass,
            MIN_WILDLAND_FUEL_MASS
        );
        assert_eq!(fuel_ignition_gate(profile.wildland_fuel_mass), 0.0);
    }

    #[test]
    fn fire_ecology_lib_harness_meets_f1_green() {
        let witness = run_fire_ecology_lib_harness();
        assert!(
            witness.frames_sampled >= 30,
            "frames={}",
            witness.frames_sampled
        );
        assert!(
            fire_ecology_f1_strict_green(&witness),
            "FIRE-F1-STRICT: gated_cells={} chunks_gated={} ungated={} frames={}",
            witness.fuel_gated_spark_cells,
            witness.chunks_fuel_gated,
            witness.ungated_spark_cells,
            witness.frames_sampled,
        );
        assert!(fire_ecology_lib_harness_green(&witness));
    }

    #[test]
    fn fire_ecology_lib_harness_writes_green_json() {
        let _guard = crate::dev::runtime_witness::fire::ecology_witness_file_lock_for_tests();
        assert!(refresh_fire_ecology_lib_harness_witness());
        let raw = std::fs::read_to_string(
            std::env::var_os("CARGO_MANIFEST_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(FIRE_ECOLOGY_JSON),
        )
        .expect("read");
        let doc: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(doc.get("green").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            doc.get("f1_green").and_then(|v| v.as_bool()),
            Some(true),
            "FIRE-F1-STRICT exit: f1_green must be true"
        );
        assert!(doc.get("sim_effect_spine").is_some());
        assert_eq!(
            doc.pointer("/sim_effect_spine/queue_drain_ok")
                .and_then(|v| v.as_bool()),
            Some(true),
            "spine must nest drain metrics (not double-wrapped); got {:?}",
            doc.get("sim_effect_spine")
        );
        assert!(
            doc.pointer("/sim_effect_spine/sim_effect_spine").is_none(),
            "refuse double-nested sim_effect_spine"
        );
        let fuel_gate = doc
            .pointer("/witness/fuel_gate_active")
            .and_then(|v| v.as_bool());
        assert_eq!(fuel_gate, Some(true));
    }
}
