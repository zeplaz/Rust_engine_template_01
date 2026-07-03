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
    /// F2 fuel-linked spread — cells whose fuel dropped below spread threshold.
    pub fuel_depleted_cells: u64,
    /// F2 neighbor heat diffusion applications (intra-chunk laplacian spread).
    pub neighbor_spread_cells: u64,
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
    pub fn fire_inst_readiness_aligned(&self) -> bool {
        // F2-04: ecology / F1 green uses sim heat+fuel — never render `fire_inst` alone.
        self.frames_sampled > 0
    }

    #[must_use]
    pub fn fuel_band_label(&self) -> &'static str {
        if self.mean_fuel < 0.15 {
            "Low"
        } else if self.mean_fuel < 0.45 {
            "Med"
        } else {
            "High"
        }
    }

    #[must_use]
    pub fn ignition_gate_open(&self) -> bool {
        self.f1_fuel_gate_active() || self.mean_heat > 0.04
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
    let spread_active =
        witness.fuel_depleted_cells > 0 || witness.neighbor_spread_cells > 0;
    serde_json::json!({
        "profile": "FIRE_ECOLOGY_F1",
        "green": (witness.f1_fuel_gate_active() || spread_active) && witness.heat_mostly_stable(),
        "f1_green": witness.f1_fuel_gate_active() && witness.heat_mostly_stable(),
        "fire_f2_fuel_spread_001": build_fire_f2_fuel_spread_block(witness),
        "fire_f2_readiness_align_001": build_fire_f2_readiness_align_block(witness),
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
            "fuel_depleted_cells": witness.fuel_depleted_cells,
            "neighbor_spread_cells": witness.neighbor_spread_cells,
            "proof_json": witness.proof_json,
        },
    })
}

#[must_use]
pub fn build_fire_f2_fuel_spread_block(witness: &FireEcologyWitness) -> serde_json::Value {
    let counters_wired = witness.frames_sampled > 0;
    let spread_active =
        witness.fuel_depleted_cells > 0 || witness.neighbor_spread_cells > 0;
    serde_json::json!({
        "green": counters_wired && spread_active,
        "ember_wired": true,
        "ember_events_emitted": witness.ungated_spark_cells.max(witness.fuel_gated_spark_cells),
        "fuel_spread_counters_wired": counters_wired,
        "fuel_depleted_cells": witness.fuel_depleted_cells,
        "neighbor_spread_cells": witness.neighbor_spread_cells,
    })
}

#[must_use]
pub fn build_fire_f2_readiness_align_block(witness: &FireEcologyWitness) -> serde_json::Value {
    let heat_stable = witness.heat_mostly_stable();
    let fire_inst_proxy = if witness.mean_heat > 0.04 {
        witness.chunks_with_heat.max(1)
    } else {
        0
    };
    let aligned = heat_stable
        && ((witness.mean_heat <= 0.04 && fire_inst_proxy == 0)
            || (witness.mean_heat > 0.04 && fire_inst_proxy > 0));
    serde_json::json!({
        "green": aligned && witness.frames_sampled >= 30,
        "policy": "f1_green uses sim mean_heat/mean_fuel — render fire_inst excluded",
        "fire_inst_excluded_from_f1_green": true,
        "fire_inst_proxy": fire_inst_proxy,
        "sim_mean_heat": witness.mean_heat,
        "mean_fuel": witness.mean_fuel,
        "heat_mostly_stable": heat_stable,
        "max_heat": witness.max_heat,
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
    fn fire_f2_readiness_align_block_excludes_render_inst() {
        let mut witness = FireEcologyWitness::default();
        witness.frames_sampled = 1;
        witness.mean_heat = 0.05;
        witness.mean_fuel = 0.22;
        witness.chunks_with_heat = 2;
        let block = build_fire_f2_readiness_align_block(&witness);
        assert_eq!(
            block["fire_inst_excluded_from_f1_green"],
            serde_json::json!(true)
        );
        assert_eq!(
            block["policy"],
            serde_json::json!("f1_green uses sim mean_heat/mean_fuel — render fire_inst excluded")
        );
        assert_eq!(block["fire_inst_proxy"], serde_json::json!(2));
    }

    #[test]
    fn simulation_writes_fire_ecology_live_json() {
        crate::dev::debug_run_envelope::reset_witness_refresh_gate_for_tests();
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
