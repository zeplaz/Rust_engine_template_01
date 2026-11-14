//! Rolling fire-ecology sim metrics (fuel-gated ignition, heat stability) consumed by the sim run dump.

use bevy::prelude::*;

/// Rolling sim metrics for fuel-gated ignition and heat stability (not render instance count).
#[derive(Resource, Clone, Debug, Default)]
pub struct FireEcologyStats {
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
    /// W2 ember spotting: cells queued this frame (emit → SimEffectQueue).
    pub ember_spots_emitted: u64,
    /// W2 ember spotting: cells that actually received heat (apply after drain).
    pub ember_spots_applied: u64,
    sum_fuel: f64,
    sum_old_growth: f64,
    sum_heat: f64,
    chunk_samples: u64,
}

impl FireEcologyStats {
    pub fn accumulate_chunk(
        &mut self,
        mean_heat: f32,
        mean_fuel: f32,
        old_growth: f32,
        fuel_ok: bool,
    ) {
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
        self.frames_sampled > 0 && (self.fuel_gated_spark_cells > 0 || self.chunks_fuel_gated > 0)
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
        self.frames_sampled >= 30 && self.heat_spike_frames <= self.frames_sampled as u32 / 8
    }
}

pub fn finalize_fire_ecology_witness_frame(mut witness: ResMut<FireEcologyStats>) {
    witness.finalize_frame_means();
}
