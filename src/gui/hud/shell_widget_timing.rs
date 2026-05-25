//! Per-widget shell render cost attribution.

use bevy::prelude::*;

use super::shell_framework::ProductShellWidgetId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WidgetRebuildReason {
    #[default]
    None,
    EguiWindow,
    TextureRebind,
    LayoutChange,
    DataRefresh,
    PolicySkip,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WidgetFrameCost {
    pub layout_us: u32,
    pub paint_us: u32,
    pub texture_uploads: u32,
    pub texture_rebinds: u32,
    pub rebuild_reason: WidgetRebuildReason,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShellWidgetTimingRow {
    pub layout_ms: f32,
    pub paint_ms: f32,
    pub texture_uploads: u32,
    pub rebuild_cause: &'static str,
}

#[derive(Clone, Copy, Debug, Default)]
struct WidgetFrameCostEma {
    layout_us: f32,
    paint_us: f32,
    uploads: f32,
    rebinds: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct ShellWidgetDiagnostics {
    pub rows: [WidgetFrameCost; ProductShellWidgetId::SLOT_COUNT],
    ema: [WidgetFrameCostEma; ProductShellWidgetId::SLOT_COUNT],
    pub worst_offender: Option<ProductShellWidgetId>,
    pub worst_offender_us: u32,
    pub frame_spike_markers: u64,
    spike_threshold_us: u32,
}

impl Default for ShellWidgetDiagnostics {
    fn default() -> Self {
        Self {
            rows: [WidgetFrameCost::default(); ProductShellWidgetId::SLOT_COUNT],
            ema: [WidgetFrameCostEma::default(); ProductShellWidgetId::SLOT_COUNT],
            worst_offender: None,
            worst_offender_us: 0,
            frame_spike_markers: 0,
            spike_threshold_us: 4_000,
        }
    }
}

impl ShellWidgetDiagnostics {
    pub fn record(
        &mut self,
        id: ProductShellWidgetId,
        layout_us: u32,
        paint_us: u32,
        texture_uploads: u32,
        texture_rebinds: u32,
        rebuild_reason: WidgetRebuildReason,
    ) {
        let row = &mut self.rows[id.index()];
        row.layout_us = layout_us;
        row.paint_us = paint_us;
        row.texture_uploads = texture_uploads;
        row.texture_rebinds = texture_rebinds;
        row.rebuild_reason = rebuild_reason;
        let alpha = 0.15;
        let ema = &mut self.ema[id.index()];
        ema.layout_us = ema.layout_us * (1.0 - alpha) + layout_us as f32 * alpha;
        ema.paint_us = ema.paint_us * (1.0 - alpha) + paint_us as f32 * alpha;
        ema.uploads = ema.uploads * (1.0 - alpha) + texture_uploads as f32 * alpha;
        ema.rebinds = ema.rebinds * (1.0 - alpha) + texture_rebinds as f32 * alpha;
        let total = layout_us.saturating_add(paint_us);
        if total >= self.spike_threshold_us {
            self.frame_spike_markers = self.frame_spike_markers.wrapping_add(1);
        }
        if total > self.worst_offender_us {
            self.worst_offender_us = total;
            self.worst_offender = Some(id);
        }
    }

    pub fn record_ms(
        &mut self,
        id: ProductShellWidgetId,
        layout_ms: f32,
        paint_ms: f32,
        texture_uploads: u32,
        rebuild_cause: &'static str,
    ) {
        let reason = match rebuild_cause {
            "texture_rebind" => WidgetRebuildReason::TextureRebind,
            "layout_change" => WidgetRebuildReason::LayoutChange,
            "data_refresh" => WidgetRebuildReason::DataRefresh,
            "policy_skip" => WidgetRebuildReason::PolicySkip,
            _ => WidgetRebuildReason::EguiWindow,
        };
        self.record(
            id,
            (layout_ms * 1000.0) as u32,
            (paint_ms * 1000.0) as u32,
            texture_uploads,
            0,
            reason,
        );
    }

    pub fn begin_frame(&mut self) {
        self.worst_offender = None;
        self.worst_offender_us = 0;
    }

    pub fn ema_layout_us(&self, id: ProductShellWidgetId) -> f32 {
        self.ema[id.index()].layout_us
    }

    pub fn ema_paint_us(&self, id: ProductShellWidgetId) -> f32 {
        self.ema[id.index()].paint_us
    }

    pub fn sorted_rows(&self) -> Vec<(ProductShellWidgetId, WidgetFrameCost)> {
        let mut ranked: Vec<_> = ProductShellWidgetId::ALL
            .iter()
            .map(|id| (*id, self.rows[id.index()]))
            .collect();
        ranked.sort_by(|left, right| {
            let left_total = left.1.layout_us.saturating_add(left.1.paint_us);
            let right_total = right.1.layout_us.saturating_add(right.1.paint_us);
            right_total.cmp(&left_total)
        });
        ranked
    }

    pub fn legacy_row(&self, id: ProductShellWidgetId) -> ShellWidgetTimingRow {
        let row = self.rows[id.index()];
        ShellWidgetTimingRow {
            layout_ms: row.layout_us as f32 / 1000.0,
            paint_ms: row.paint_us as f32 / 1000.0,
            texture_uploads: row.texture_uploads,
            rebuild_cause: match row.rebuild_reason {
                WidgetRebuildReason::None => "none",
                WidgetRebuildReason::EguiWindow => "egui_window",
                WidgetRebuildReason::TextureRebind => "texture_rebind",
                WidgetRebuildReason::LayoutChange => "layout_change",
                WidgetRebuildReason::DataRefresh => "data_refresh",
                WidgetRebuildReason::PolicySkip => "policy_skip",
            },
        }
    }
}
