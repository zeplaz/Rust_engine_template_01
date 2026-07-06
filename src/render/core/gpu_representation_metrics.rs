//! Per-frame GPU representation cost proof (LOD must change bytes/rows, not enum-only).

use bevy::prelude::*;

use crate::gui::RepresentationBand;

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct GpuRepresentationMetrics {
    pub active_band: RepresentationBand,
    pub active_rows: u32,
    pub instance_rows: u32,
    pub particle_rows: u32,
    pub upload_bytes: u64,
    pub reserved_bytes: u64,
    pub high_watermark_bytes: u64,
    pub reserved_capacity: u32,
    pub active_capacity: u32,
    pub dispatch_count: u32,
    pub draw_instances: u32,
    pub active_allocations: u32,
    /// Last single-buffer reservation request (e.g. particle / expanded scratch) in bytes.
    pub gpu_last_alloc_request_bytes: u64,
}

impl GpuRepresentationMetrics {
    pub fn record_fire_upload(
        &mut self,
        band: RepresentationBand,
        instance_rows: u32,
        upload_bytes: u64,
        reserved_capacity: u32,
        active_capacity: u32,
        reserved_bytes: u64,
    ) {
        self.active_band = band;
        self.instance_rows = instance_rows;
        self.active_rows = instance_rows;
        self.upload_bytes = upload_bytes;
        self.reserved_capacity = reserved_capacity;
        self.active_capacity = active_capacity;
        self.reserved_bytes = reserved_bytes;
        self.high_watermark_bytes = self.high_watermark_bytes.max(reserved_bytes);
        self.active_allocations = self.active_allocations.max(1);
    }

    pub fn record_particle_upload(
        &mut self,
        particle_rows: u32,
        upload_bytes: u64,
        reserved_bytes: u64,
    ) {
        self.particle_rows = particle_rows;
        self.upload_bytes = self.upload_bytes.saturating_add(upload_bytes);
        self.reserved_bytes = self.reserved_bytes.saturating_add(reserved_bytes);
        self.high_watermark_bytes = self.high_watermark_bytes.max(self.reserved_bytes);
        self.active_allocations = self.active_allocations.saturating_add(1);
    }

    pub fn record_domain_overlay_upload(&mut self, upload_bytes: u64, reserved_bytes: u64, active_rows: u32) {
        self.upload_bytes = self.upload_bytes.saturating_add(upload_bytes);
        self.reserved_bytes = self.reserved_bytes.saturating_add(reserved_bytes);
        self.high_watermark_bytes = self.high_watermark_bytes.max(self.reserved_bytes);
        self.active_rows = self.active_rows.saturating_add(active_rows);
        self.active_allocations = self.active_allocations.saturating_add(1);
    }

    pub fn record_dispatch_count(&mut self, count: u32) {
        self.dispatch_count = count;
    }

    pub fn record_draw_instances(&mut self, count: u32) {
        self.draw_instances = count;
    }

    /// Record a planned registry allocation size (debug / proof; does not replace reserved_bytes totals).
    pub fn record_gpu_alloc_request(&mut self, needed_bytes: u64) {
        self.gpu_last_alloc_request_bytes = needed_bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_change_moves_rows_and_dispatch_count() {
        let mut metrics = GpuRepresentationMetrics::default();
        metrics.record_fire_upload(RepresentationBand::Full, 1_024, 65_536, 2_048, 2_048, 131_072);
        metrics.record_dispatch_count(3);
        assert_eq!(metrics.active_band, RepresentationBand::Full);
        assert_eq!(metrics.instance_rows, 1_024);
        assert!(metrics.upload_bytes > 0);
        assert_eq!(metrics.dispatch_count, 3);

        metrics.record_fire_upload(RepresentationBand::Strategic, 0, 0, 2_048, 0, 131_072);
        metrics.record_dispatch_count(0);
        assert_eq!(metrics.active_band, RepresentationBand::Strategic);
        assert_eq!(metrics.instance_rows, 0);
        assert_eq!(metrics.upload_bytes, 0);
        assert_eq!(metrics.dispatch_count, 0);
    }

    #[test]
    fn record_gpu_alloc_request_tracks_last_bytes() {
        let mut m = GpuRepresentationMetrics::default();
        m.record_gpu_alloc_request(64 * 1024);
        assert_eq!(m.gpu_last_alloc_request_bytes, 64 * 1024);
    }
}
