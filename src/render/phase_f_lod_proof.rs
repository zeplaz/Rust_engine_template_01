//! Phase F runtime LOD proof — measurable upload / row scaling per representation band.

use bevy::prelude::*;

use crate::gui::RepresentationBand;
use crate::render::GpuRepresentationMetrics;

/// Last observed GPU upload bytes per band (monotonic max per band).
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct PhaseFLodProofReport {
    pub full_upload_bytes: u64,
    pub tactical_upload_bytes: u64,
    pub strategic_upload_bytes: u64,
    pub overlay_upload_bytes: u64,
    pub dormant_upload_bytes: u64,
    pub full_particle_rows: u32,
    pub strategic_particle_rows: u32,
    pub overlay_particle_rows: u32,
    pub samples: u32,
    pub ordering_ok: bool,
}

impl PhaseFLodProofReport {
    pub fn record_sample(&mut self, band: RepresentationBand, metrics: &GpuRepresentationMetrics) {
        self.samples = self.samples.saturating_add(1);
        match band {
            RepresentationBand::Full => {
                self.full_upload_bytes = self.full_upload_bytes.max(metrics.upload_bytes);
                self.full_particle_rows = self.full_particle_rows.max(metrics.particle_rows);
            }
            RepresentationBand::Tactical => {
                self.tactical_upload_bytes = self.tactical_upload_bytes.max(metrics.upload_bytes);
            }
            RepresentationBand::Strategic => {
                self.strategic_upload_bytes = self.strategic_upload_bytes.max(metrics.upload_bytes);
                self.strategic_particle_rows =
                    self.strategic_particle_rows.max(metrics.particle_rows);
            }
            RepresentationBand::OverlayOnly => {
                self.overlay_upload_bytes = self.overlay_upload_bytes.max(metrics.upload_bytes);
                self.overlay_particle_rows = self.overlay_particle_rows.max(metrics.particle_rows);
            }
            RepresentationBand::Dormant => {
                self.dormant_upload_bytes = self.dormant_upload_bytes.max(metrics.upload_bytes);
            }
        }
        self.ordering_ok = self.evaluate_ordering();
    }

    #[must_use]
    pub fn evaluate_ordering(&self) -> bool {
        let overlay_zero = self.overlay_particle_rows == 0
            && self.overlay_upload_bytes == 0
            && self.dormant_upload_bytes == 0;
        let strategic_shrinks = self.strategic_upload_bytes == 0
            || self.full_upload_bytes == 0
            || self.strategic_upload_bytes <= self.full_upload_bytes;
        let particles_shrink = self.full_particle_rows == 0
            || self.strategic_particle_rows <= self.full_particle_rows;
        strategic_shrinks && particles_shrink && overlay_zero
    }
}

pub fn record_phase_f_lod_proof_sample(
    policy: Option<Res<crate::gui::RepresentationResult>>,
    metrics: Option<Res<GpuRepresentationMetrics>>,
    mut proof: ResMut<PhaseFLodProofReport>,
) {
    let (Some(policy), Some(metrics)) = (policy.as_deref(), metrics.as_deref()) else {
        return;
    };
    proof.record_sample(policy.active_band, metrics);
}

pub struct PhaseFLodProofPlugin;

impl Plugin for PhaseFLodProofPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhaseFLodProofReport>().add_systems(
            PostUpdate,
            record_phase_f_lod_proof_sample
                .after(crate::render::sync_particle_draw_dispatch_from_policy)
                .after(crate::render::extraction::FireVisualFrameSet::ProjectGpu),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_band_upload_exceeds_strategic_and_overlay_zeroes_particles() {
        let mut proof = PhaseFLodProofReport::default();
        let mut full = GpuRepresentationMetrics::default();
        full.record_fire_upload(RepresentationBand::Full, 512, 65_536, 1_024, 1_024, 131_072);
        full.record_particle_upload(256, 16_384, 32_768);
        proof.record_sample(RepresentationBand::Full, &full);

        let mut strategic = GpuRepresentationMetrics::default();
        strategic.record_fire_upload(RepresentationBand::Strategic, 64, 8_192, 1_024, 128, 131_072);
        strategic.record_particle_upload(32, 2_048, 4_096);
        proof.record_sample(RepresentationBand::Strategic, &strategic);

        let mut overlay = GpuRepresentationMetrics::default();
        overlay.record_fire_upload(RepresentationBand::OverlayOnly, 0, 0, 1_024, 0, 131_072);
        proof.record_sample(RepresentationBand::OverlayOnly, &overlay);

        assert!(proof.full_upload_bytes > proof.strategic_upload_bytes);
        assert_eq!(proof.overlay_particle_rows, 0);
        assert!(proof.evaluate_ordering());
    }
}
