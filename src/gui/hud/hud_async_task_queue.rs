//! Deferred HUD work — formatting and transforms off the egui pass.

use bevy::prelude::*;

use crate::strategic::BeliefSnapshotDto;

use super::explainability_viewer::explainability_events_from_belief;
use super::stage6_consumer::ResidencyOverlayConsumerDto;

#[derive(Clone, Debug, Default)]
pub struct TelemetrySnapshot {
    pub frame_revision: u64,
    pub avg_frame_ms: f32,
    pub spike_count: u32,
    pub residency_summary: String,
    pub gpu_summary: String,
    pub shell_metrics_line: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudAsyncTaskKind {
    DtoFormatting,
    LogLine,
    MinimapLegend,
    ExplainabilityTransform,
    TelemetryAggregation,
    DispatchLogFormat,
    TransmissionFormat,
}

#[derive(Clone, Debug)]
pub enum HudAsyncTask {
    FormatResidencyDto {
        resident_chunks: u32,
        ghost_chunks: u32,
        atlas_pages: u32,
    },
    LogLine(String),
    MinimapLegend {
        zoom: f32,
        revision: u64,
    },
    ExplainabilityTransform {
        beliefs: Vec<BeliefSnapshotDto>,
    },
    TelemetryAggregation {
        frame_revision: u64,
    },
    DispatchLogFormat,
    TransmissionFormat {
        title: String,
        body: String,
    },
    ShellMetricsReduce {
        widget_count: u32,
        spike_markers: u64,
    },
    GpuStatsAggregation {
        upload_bytes: u64,
        texture_rebuilds: u32,
    },
    SpikeAnalysis {
        avg_frame_ms: f32,
        max_frame_ms: f32,
    },
}

#[derive(Resource, Clone, Debug, Default)]
pub struct HudAsyncResultCache {
    pub residency_dto: Option<ResidencyOverlayConsumerDto>,
    pub minimap_legend: Option<String>,
    pub log_line: Option<String>,
    pub explainability_lines: Vec<String>,
    pub telemetry_summary: Option<String>,
    pub dispatch_log_lines: Vec<String>,
    pub transmission_lines: Vec<String>,
    pub tooltip_line: Option<String>,
    pub telemetry_snapshot: Option<TelemetrySnapshot>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct HudAsyncTaskQueue {
    pub pending: Vec<HudAsyncTask>,
    pub completed: u64,
    pub dropped: u64,
    pub cache: HudAsyncResultCache,
}

impl HudAsyncTaskQueue {
    pub const MAX_PENDING: usize = 32;

    pub fn enqueue(&mut self, task: HudAsyncTask) {
        if self.pending.len() >= Self::MAX_PENDING {
            self.dropped = self.dropped.wrapping_add(1);
            return;
        }
        self.pending.push(task);
    }
}

pub fn drain_hud_async_task_queue(mut queue: ResMut<HudAsyncTaskQueue>) {
    let pending = std::mem::take(&mut queue.pending);
    for task in pending {
        match task {
            HudAsyncTask::FormatResidencyDto {
                resident_chunks,
                ghost_chunks,
                atlas_pages,
            } => {
                queue.cache.residency_dto = Some(ResidencyOverlayConsumerDto {
                    schema_version: ResidencyOverlayConsumerDto::CURRENT_SCHEMA,
                    resident_chunks,
                    ghost_chunks,
                    utility_channel_mask: 0b1011,
                    paged_atlas_pages: atlas_pages,
                    chunks: Vec::new(),
                });
            }
            HudAsyncTask::LogLine(line) => {
                queue.cache.log_line = Some(line);
            }
            HudAsyncTask::MinimapLegend { zoom, revision } => {
                queue.cache.minimap_legend = Some(format!(
                    "zoom {:.2}x · raster rev {revision} · legend deferred",
                    zoom,
                ));
            }
            HudAsyncTask::ExplainabilityTransform { beliefs } => {
                let events = explainability_events_from_belief(&beliefs);
                queue.cache.explainability_lines = events
                    .iter()
                    .map(|event| {
                        format!(
                            "[{}] {:.0}% · {}",
                            event.category,
                            event.confidence * 100.0,
                            event.summary
                        )
                    })
                    .collect();
            }
            HudAsyncTask::TelemetryAggregation { frame_revision } => {
                let residency_summary = queue
                    .cache
                    .residency_dto
                    .as_ref()
                    .map(|dto| {
                        format!(
                            "resident={} ghost={} atlas_pages={}",
                            dto.resident_chunks, dto.ghost_chunks, dto.paged_atlas_pages
                        )
                    })
                    .unwrap_or_else(|| "residency pending".into());
                queue.cache.telemetry_summary = Some(format!(
                    "telemetry snapshot rev {frame_revision} · deferred aggregation"
                ));
                queue.cache.telemetry_snapshot = Some(TelemetrySnapshot {
                    frame_revision,
                    avg_frame_ms: 0.0,
                    spike_count: 0,
                    residency_summary,
                    gpu_summary: "deferred gpu summary".into(),
                    shell_metrics_line: format!("shell metrics rev {frame_revision}"),
                });
            }
            HudAsyncTask::DispatchLogFormat => {
                queue.cache.dispatch_log_lines = super::stage7_ui_shell::mock_dispatch_envelopes()
                    .into_iter()
                    .map(|envelope| {
                        format!(
                            "{:?} · loss {:.0}% · {}",
                            envelope.message.plane,
                            envelope.loss_probability * 100.0,
                            envelope.message.summary
                        )
                    })
                    .collect();
            }
            HudAsyncTask::TransmissionFormat { title, body } => {
                queue.cache.transmission_lines = vec![format!("{title}: {body}")];
            }
            HudAsyncTask::ShellMetricsReduce {
                widget_count,
                spike_markers,
            } => {
                if let Some(snapshot) = queue.cache.telemetry_snapshot.as_mut() {
                    snapshot.shell_metrics_line = format!(
                        "widgets {widget_count} · spikes {spike_markers}"
                    );
                }
            }
            HudAsyncTask::GpuStatsAggregation {
                upload_bytes,
                texture_rebuilds,
            } => {
                if let Some(snapshot) = queue.cache.telemetry_snapshot.as_mut() {
                    snapshot.gpu_summary = format!(
                        "upload {upload_bytes} B · rebuilds {texture_rebuilds}"
                    );
                }
            }
            HudAsyncTask::SpikeAnalysis {
                avg_frame_ms,
                max_frame_ms,
            } => {
                if let Some(snapshot) = queue.cache.telemetry_snapshot.as_mut() {
                    snapshot.avg_frame_ms = avg_frame_ms;
                    snapshot.spike_count = if max_frame_ms > 33.0 { 1 } else { 0 };
                }
            }
        }
        queue.completed = queue.completed.wrapping_add(1);
    }
}
