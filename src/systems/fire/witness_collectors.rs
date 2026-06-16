//! Live witness collectors for fire ecology F1 (DEV-CONTAIN-004).
//!
//! File I/O writer: [`crate::dev::runtime_witness::fire`].

use bevy::prelude::*;

/// Rolling sim metrics for fuel-gated ignition and heat stability (not render instance count).
#[derive(Resource, Clone, Debug, Default)]
pub struct FireEcologyWitness {
    pub frames_sampled: u64,
    pub chunks_sampled: u32,
    pub chunks_with_heat: u32,
    pub chunks_fuel_gated: u32,
    pub fuel_gated_spark_cells: u64,
    pub ungated_spark_cells: u64,
    pub mean_fuel: f32,
    pub mean_old_growth: f32,
    pub mean_heat: f32,
    pub max_heat: f32,
    pub heat_stable_low_frames: u32,
    pub heat_spike_frames: u32,
    pub proof_json: bool,
    sum_fuel: f64,
    sum_old_growth: f64,
    sum_heat: f64,
    chunk_samples: u64,
}

impl FireEcologyWitness {
    pub fn accumulate_chunk(&mut self, mean_heat: f32, mean_fuel: f32, old_growth: f32, fuel_ok: bool) {
        self.chunk_samples = self.chunk_samples.saturating_add(1);
        self.sum_heat += f64::from(mean_heat);
        self.sum_fuel += f64::from(mean_fuel);
        self.sum_old_growth += f64::from(old_growth);
        if mean_heat > 0.04 {
            self.chunks_with_heat = self.chunks_with_heat.saturating_add(1);
        }
        if mean_heat > self.max_heat {
            self.max_heat = mean_heat;
        }
        if !fuel_ok && mean_heat < 0.02 {
            self.heat_stable_low_frames = self.heat_stable_low_frames.saturating_add(1);
        }
        if self.mean_heat > 0.02 && mean_heat > self.mean_heat * 4.0 + 0.08 {
            self.heat_spike_frames = self.heat_spike_frames.saturating_add(1);
        }
    }

    pub fn finalize_frame_means(&mut self) {
        self.frames_sampled = self.frames_sampled.saturating_add(1);
        let n = self.chunk_samples.max(1) as f64;
        self.mean_heat = (self.sum_heat / n) as f32;
        self.mean_fuel = (self.sum_fuel / n) as f32;
        self.mean_old_growth = (self.sum_old_growth / n) as f32;
        self.sum_heat = 0.0;
        self.sum_fuel = 0.0;
        self.sum_old_growth = 0.0;
        self.chunk_samples = 0;
    }

    #[must_use]
    pub fn f1_fuel_gate_active(&self) -> bool {
        self.frames_sampled > 0
            && (self.fuel_gated_spark_cells > 0 || self.chunks_fuel_gated > 0)
    }

    #[must_use]
    pub fn heat_mostly_stable(&self) -> bool {
        self.frames_sampled >= 30
            && self.heat_spike_frames <= self.frames_sampled as u32 / 8
    }
}

pub fn build_fire_ecology_proof_payload(witness: &FireEcologyWitness) -> serde_json::Value {
    let gated_ratio = if witness.ungated_spark_cells + witness.fuel_gated_spark_cells > 0 {
        witness.fuel_gated_spark_cells as f64
            / (witness.ungated_spark_cells + witness.fuel_gated_spark_cells) as f64
    } else {
        0.0
    };
    serde_json::json!({
        "profile": "FIRE_ECOLOGY_F1",
        "green": witness.f1_fuel_gate_active() && witness.heat_mostly_stable(),
        "f1_green": witness.f1_fuel_gate_active() && witness.heat_mostly_stable(),
        "witness": {
            "fuel_gate_active": witness.f1_fuel_gate_active(),
            "heat_mostly_stable": witness.heat_mostly_stable(),
            "fuel_gated_ignitions": witness.fuel_gated_spark_cells,
            "ungated_spark_cells": witness.ungated_spark_cells,
            "fuel_gated_spark_ratio": gated_ratio,
            "mean_fuel": witness.mean_fuel,
            "mean_old_growth": witness.mean_old_growth,
            "mean_heat": witness.mean_heat,
            "max_heat": witness.max_heat,
            "chunks_with_heat": witness.chunks_with_heat,
            "chunks_fuel_gated": witness.chunks_fuel_gated,
            "chunks_sampled": witness.chunks_sampled,
            "heat_stable_low_frames": witness.heat_stable_low_frames,
            "heat_spike_frames": witness.heat_spike_frames,
            "frames_sampled": witness.frames_sampled,
            "proof_json": witness.proof_json,
        },
    })
}

pub use crate::dev::runtime_witness::fire::{
    write_fire_ecology_live_proof_system, FireEcologyLiveProofState, FIRE_ECOLOGY_JSON,
};

pub fn finalize_fire_ecology_witness_frame(mut witness: ResMut<FireEcologyWitness>) {
    witness.finalize_frame_means();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use bevy::input::InputPlugin;
    use crate::gui::InputBindings;
    use crate::systems::chunk_environment_persist::ChunkEnvironmentPersistPlugin;
    use crate::systems::chunk_sim_lod::ChunkSimLodPlugin;
    use crate::systems::ecology::EcologyPlugin;
    use crate::systems::sim_control::SimControlPlugin;
    use crate::systems::weather::WeatherSimulationPlugin;
    use crate::engine::states::BaseState;
    use crate::systems::ecology::VegetationField;
    use crate::systems::weather::ChunkWeather;
    use crate::terrain::generation::{Chunk, ChunkCellMatrix};
    use bevy::math::UVec2;

    fn proof_output_path() -> std::path::PathBuf {
        std::env::var_os("CARGO_MANIFEST_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("debug_runs")
            .join("fire_ecology_live.json")
    }

    fn assemble_fire_proof_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin, InputPlugin));
        app.init_resource::<InputBindings>();
        app.init_state::<BaseState>();
        app.insert_state(BaseState::Simulation);
        app.add_plugins(SimControlPlugin);
        crate::systems::chunk_environment_set::configure_chunk_environment_sets(&mut app);
        app.add_plugins((
            ChunkEnvironmentPersistPlugin,
            ChunkSimLodPlugin,
            WeatherSimulationPlugin,
            EcologyPlugin,
            super::super::FirePlugin,
        ));
        app.init_resource::<FireEcologyLiveProofState>();
        app.world_mut()
            .resource_mut::<FireEcologyLiveProofState>()
            .write_interval = 5;
        let mut matrix = ChunkCellMatrix::new(UVec2::new(4, 4));
        for m in matrix.moisture.iter_mut() {
            *m = 0.18;
        }
        for t in matrix.temperature.iter_mut() {
            *t = 0.42;
        }
        app.world_mut().spawn((
            Chunk {
                coord: IVec2::ZERO,
            },
            matrix,
            VegetationField {
                ground_fuel: 0.08,
                old_growth: 0.04,
                dryness: 0.75,
                ..Default::default()
            },
            ChunkWeather::default(),
        ));
        app
    }

    #[test]
    fn simulation_writes_fire_ecology_live_json() {
        let _ = fs::remove_file(proof_output_path());
        let mut app = assemble_fire_proof_app();
        for _ in 0..24 {
            app.update();
        }
        let path = proof_output_path();
        assert!(path.exists(), "expected {:?}", path);
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert_eq!(json["profile"], "FIRE_ECOLOGY_F1");
        assert!(app.world().resource::<FireEcologyLiveProofState>().written);
    }
}
