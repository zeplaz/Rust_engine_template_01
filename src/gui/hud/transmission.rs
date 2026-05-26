//! Transmission / briefing shell — queue + HUD chrome; playback stays off egui repaint loops.
//!
//! v1 ingests [`crate::strategic::NarrativeObservationBus`] lines; media decode uploads to GPU textures later (**BQ-124**).

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::gui::input_bindings::InputBindings;
use crate::gui::hud::{
    capture_shell_layout, draw_virtualized_rows, shell_widget_runs_egui_with_budget,
    show_product_shell_window, HudDockRegistry, HudLayoutStore, HudWidgetId, HudAsyncTask,
    HudAsyncTaskQueue, PendingHudLayoutCommit, ProductShellUpdateBudget, ProductShellWidgetId,
    RetainedWidgetCache, ShellWidgetDiagnostics, ShellWindowHost, draw_retained_lines_or_build,
};
use crate::gui::style::{severity_tone_color, SeverityTone, UiPalette};
use crate::gui::ui_gates::in_simulation_or_editor;
use crate::strategic::{DispatchEnvelope, NarrativeCategory, NarrativeObservationBus};
use crate::systems::sim_control::SimStepStamp;

/// Thematic channel bucket for UI filtering (maps loosely to comms planes later).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TransmissionChannelId {
    #[default]
    General,
    Logistics,
    Command,
    FieldReports,
    Emergency,
    Intercept,
}

impl TransmissionChannelId {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::General => "GENERAL",
            Self::Logistics => "LOGISTICS",
            Self::Command => "COMMAND",
            Self::FieldReports => "FIELD",
            Self::Emergency => "EMERGENCY",
            Self::Intercept => "INTERCEPT",
        }
    }

    #[must_use]
    pub fn from_narrative_category(category: NarrativeCategory) -> Self {
        match category {
            NarrativeCategory::Logistics => Self::Logistics,
            NarrativeCategory::Infrastructure => Self::FieldReports,
            NarrativeCategory::Weather => Self::General,
            NarrativeCategory::Faction => Self::Command,
            NarrativeCategory::General => Self::General,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransmissionSeverity {
    Routine,
    #[default]
    Advisory,
    Urgent,
    Emergency,
}

impl TransmissionSeverity {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Routine => "ROUTINE",
            Self::Advisory => "ADVISORY",
            Self::Urgent => "URGENT",
            Self::Emergency => "EMERGENCY",
        }
    }

    #[must_use]
    pub fn from_narrative_category(category: NarrativeCategory) -> Self {
        match category {
            NarrativeCategory::Faction => Self::Urgent,
            NarrativeCategory::Logistics => Self::Advisory,
            NarrativeCategory::Infrastructure => Self::Advisory,
            NarrativeCategory::Weather => Self::Routine,
            NarrativeCategory::General => Self::Routine,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransmissionEvent {
    pub channel: TransmissionChannelId,
    pub severity: TransmissionSeverity,
    pub title: String,
    pub body: String,
    pub queued_at: SimStepStamp,
}

#[derive(Clone, Debug)]
pub struct ActiveTransmission {
    pub event: TransmissionEvent,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TransmissionDegradation {
    pub static_noise: f32,
    pub interrupted: bool,
    pub corrupt_picture: f32,
}

#[derive(Clone, Debug, Default)]
pub enum TransmissionMediaFrame {
    #[default]
    StaticText,
    ImagePlaceholder {
        asset_label: String,
    },
    VideoPlaceholder {
        asset_label: String,
    },
}

/// Placeholder media provider — uploads to GPU textures later (**BQ-126**).
#[derive(Resource, Clone, Debug, Default)]
pub struct TransmissionMediaProvider {
    pub current: Option<TransmissionMediaFrame>,
}

/// HUD widget state — logic only; no video decode in egui.
#[derive(Resource, Clone, Debug)]
pub struct TransmissionShellState {
    pub active: bool,
    pub minimized: bool,
    pub panel_state: super::panel_state::HudPanelState,
    pub paused: bool,
    pub filter_channel: Option<TransmissionChannelId>,
    pub current_channel: TransmissionChannelId,
    pub queue: VecDeque<TransmissionEvent>,
    pub current: Option<ActiveTransmission>,
    pub signal_pulse: f32,
    pub degradation: TransmissionDegradation,
    pub mock_dispatch: Option<DispatchEnvelope>,
}

impl Default for TransmissionShellState {
    fn default() -> Self {
        Self {
            active: false,
            minimized: false,
            panel_state: super::panel_state::HudPanelState::Collapsed,
            paused: false,
            filter_channel: None,
            current_channel: TransmissionChannelId::General,
            queue: VecDeque::new(),
            current: None,
            signal_pulse: 0.0,
            degradation: TransmissionDegradation::default(),
            mock_dispatch: None,
        }
    }
}

impl TransmissionShellState {
    pub const QUEUE_CAP: usize = 32;

    pub fn enqueue(&mut self, event: TransmissionEvent) {
        if event.severity == TransmissionSeverity::Emergency {
            self.interrupt_current();
        }
        if self.queue.len() >= Self::QUEUE_CAP {
            self.queue.pop_front();
        }
        self.queue.push_back(event);
    }

    pub fn promote_next(&mut self, media: &mut TransmissionMediaProvider) {
        while let Some(event) = self.queue.pop_front() {
            if self
                .filter_channel
                .is_some_and(|filter| filter != event.channel)
            {
                continue;
            }
            self.current = Some(ActiveTransmission { event });
            break;
        }
        if let Some(active) = self.current.as_ref() {
            self.current_channel = active.event.channel;
            media.current = Some(fake_media_for_channel(active.event.channel));
            self.mock_dispatch = Some(mock_dispatch_for_event(&active.event));
            self.degradation.static_noise = self
                .mock_dispatch
                .as_ref()
                .map(|env| env.loss_probability)
                .unwrap_or(0.0);
            self.degradation.corrupt_picture = self
                .mock_dispatch
                .as_ref()
                .map(|env| env.corruption_hint)
                .unwrap_or(0.0);
        } else {
            self.current = None;
            media.current = None;
            self.mock_dispatch = None;
        }
    }

    pub fn interrupt_current(&mut self) {
        self.degradation.interrupted = true;
        self.current = None;
    }

    pub fn replay_current(&mut self) {
        if let Some(active) = self.current.take() {
            self.queue.push_front(active.event);
        }
    }
}

fn fake_media_for_channel(channel: TransmissionChannelId) -> TransmissionMediaFrame {
    match channel {
        TransmissionChannelId::Emergency | TransmissionChannelId::Intercept => {
            TransmissionMediaFrame::VideoPlaceholder {
                asset_label: "static_noise_loop".into(),
            }
        }
        TransmissionChannelId::FieldReports => TransmissionMediaFrame::ImagePlaceholder {
            asset_label: "still_frame_stub".into(),
        },
        _ => TransmissionMediaFrame::StaticText,
    }
}

fn mock_dispatch_for_event(event: &TransmissionEvent) -> DispatchEnvelope {
    use crate::strategic::{CommunicationPlane, DispatchMessage};

    DispatchEnvelope {
        message: DispatchMessage {
            plane: CommunicationPlane::TacticalLine,
            issued_at: event.queued_at,
            deliver_after: event.queued_at,
            command_id: 0,
            summary: event.title.clone(),
        },
        loss_probability: match event.severity {
            TransmissionSeverity::Emergency => 0.35,
            TransmissionSeverity::Urgent => 0.2,
            TransmissionSeverity::Advisory => 0.08,
            TransmissionSeverity::Routine => 0.02,
        },
        corruption_hint: match event.severity {
            TransmissionSeverity::Emergency => 0.45,
            TransmissionSeverity::Urgent => 0.2,
            _ => 0.05,
        },
    }
}

fn severity_color(palette: &UiPalette, severity: TransmissionSeverity) -> egui::Color32 {
    let tone = match severity {
        TransmissionSeverity::Routine => SeverityTone::Routine,
        TransmissionSeverity::Advisory => SeverityTone::Advisory,
        TransmissionSeverity::Urgent => SeverityTone::Urgent,
        TransmissionSeverity::Emergency => SeverityTone::Emergency,
    };
    severity_tone_color(palette, tone)
}

pub struct TransmissionShellPlugin;

impl Plugin for TransmissionShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransmissionShellState>()
            .init_resource::<TransmissionMediaProvider>()
            .init_resource::<super::transmission_media::TransmissionMediaProviderRegistry>()
            .add_systems(
                Update,
                (
                    transmission_ingest_narrative_bus_system,
                    transmission_signal_pulse_system,
                    transmission_shell_keyboard_toggle,
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            );
    }
}

fn transmission_ingest_narrative_bus_system(
    bus: Option<Res<NarrativeObservationBus>>,
    frame: Option<Res<crate::gui::WorldRepresentationFrame>>,
    mut shell: ResMut<TransmissionShellState>,
    mut media: ResMut<TransmissionMediaProvider>,
    mut last_pushed: Local<Option<String>>,
) {
    let Some(bus) = bus.as_ref() else {
        return;
    };
    let Some(latest) = bus.recent.back() else {
        return;
    };
    if last_pushed.as_deref() == Some(latest.generated_text.as_str()) {
        return;
    }
    *last_pushed = Some(latest.generated_text.clone());
    let stamp = frame
        .as_ref()
        .map(|f| f.sim_step_stamp)
        .unwrap_or_else(|| SimStepStamp::new(0, 0));
    shell.enqueue(TransmissionEvent {
        channel: TransmissionChannelId::from_narrative_category(latest.category),
        severity: TransmissionSeverity::from_narrative_category(latest.category),
        title: latest.category_label(),
        body: latest.generated_text.clone(),
        queued_at: stamp,
    });
    if shell.current.is_none() {
        shell.promote_next(&mut media);
    }
    if shell.degradation.interrupted {
        shell.degradation.interrupted = false;
    }
}

fn transmission_signal_pulse_system(time: Res<Time>, mut shell: ResMut<TransmissionShellState>) {
    if shell.current.is_some() || !shell.queue.is_empty() {
        shell.signal_pulse = (shell.signal_pulse + time.delta_secs() * 2.5).fract();
    } else {
        shell.signal_pulse = 0.0;
    }
}

fn transmission_shell_keyboard_toggle(
    policy: Res<crate::gui::hud::WidgetPresentationPolicy>,
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut shell: ResMut<TransmissionShellState>,
) {
    if !policy.widget_enabled(crate::gui::hud::HudWidgetId::Transmission) {
        shell.active = false;
        return;
    }
    if keys.just_pressed(bindings.toggle_transmission_widget) {
        shell.active = !shell.active;
        if shell.active {
            shell.minimized = false;
        }
    }
}

pub fn draw_transmission_shell_egui(
    ctx: &mut egui::Context,
    palette: &UiPalette,
    policy: &crate::gui::hud::WidgetPresentationPolicy,
    shell: &mut TransmissionShellState,
    media: &mut TransmissionMediaProvider,
    dock: &mut HudDockRegistry,
    layout_store: &mut HudLayoutStore,
    update_budget: &mut ProductShellUpdateBudget,
    now_secs: f32,
    widget_timing: Option<&mut ShellWidgetDiagnostics>,
    retained: &mut RetainedWidgetCache,
    async_queue: &mut HudAsyncTaskQueue,
    pending_layout: &mut PendingHudLayoutCommit,
) {
    if !policy.widget_enabled(crate::gui::hud::HudWidgetId::Transmission) {
        shell.active = false;
        dock.slot_mut(HudWidgetId::Transmission).visible = false;
        return;
    }
    if !shell.active {
        return;
    }

    dock.slot_mut(HudWidgetId::Transmission).visible = shell.active;
    dock.slot_mut(HudWidgetId::Transmission).minimized = shell.minimized;
    if !shell_widget_runs_egui_with_budget(
        dock,
        HudWidgetId::Transmission,
        shell.active,
        Some(update_budget),
        now_secs,
    ) {
        return;
    }

    let mut open = shell.active;
    if let Some(response) = show_product_shell_window(
        ctx,
        ShellWindowHost {
            id: HudWidgetId::Transmission,
            title: "Transmission",
            default_pos: egui::pos2(12.0, 72.0),
            default_size: [360.0, 220.0],
            min_size: [260.0, 140.0],
        },
        layout_store,
        dock,
        &mut open,
        |ui| {
            ui.horizontal(|ui| {
                ui.label("Channel");
                egui::ComboBox::from_id_salt("tx_channel")
                    .selected_text(shell.current_channel.label())
                    .show_ui(ui, |ui| {
                        for channel in [
                            TransmissionChannelId::General,
                            TransmissionChannelId::Logistics,
                            TransmissionChannelId::Command,
                            TransmissionChannelId::FieldReports,
                            TransmissionChannelId::Emergency,
                            TransmissionChannelId::Intercept,
                        ] {
                            ui.selectable_value(
                                &mut shell.current_channel,
                                channel,
                                channel.label(),
                            );
                        }
                    });
                let filter_label = shell
                    .filter_channel
                    .map(|c| c.label())
                    .unwrap_or("ALL");
                if ui.button(format!("Filter: {filter_label}")).clicked() {
                    shell.filter_channel = match shell.filter_channel {
                        None => Some(shell.current_channel),
                        Some(_) => None,
                    };
                }
                if ui.button("Pause").clicked() {
                    shell.paused = !shell.paused;
                }
                if ui.button("Replay").clicked() {
                    shell.replay_current();
                }
                if ui.button("Interrupt").clicked() {
                    shell.interrupt_current();
                }
            });
            ui.horizontal(|ui| {
                let mut channel_knob = shell.current_channel as u8 as f32;
                if ui
                    .add(egui::Slider::new(&mut channel_knob, 0.0..=5.0).text("Channel knob"))
                    .changed()
                {
                    shell.current_channel = match channel_knob.round() as u8 {
                        1 => TransmissionChannelId::Logistics,
                        2 => TransmissionChannelId::Command,
                        3 => TransmissionChannelId::FieldReports,
                        4 => TransmissionChannelId::Emergency,
                        5 => TransmissionChannelId::Intercept,
                        _ => TransmissionChannelId::General,
                    };
                }
            });
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut shell.degradation.static_noise, 0.0..=1.0).text("Noise"));
                ui.add(
                    egui::Slider::new(&mut shell.degradation.corrupt_picture, 0.0..=1.0)
                        .text("Corruption"),
                );
            });
            ui.separator();
            if let Some(active) = shell.current.as_ref() {
                if shell.degradation.interrupted {
                    ui.label(
                        egui::RichText::new("Signal interrupted.").color(palette.warn),
                    );
                }
                if shell.paused {
                    ui.label(egui::RichText::new("Playback paused.").weak());
                }
                if let Some(frame) = media.current.as_ref() {
                    ui.label(
                        egui::RichText::new(format!("Media provider: {frame:?}"))
                            .small()
                            .weak(),
                    );
                }
                ui.label(
                    egui::RichText::new(format!(
                        "[{} / {}] {}",
                        active.event.channel.label(),
                        active.event.severity.label(),
                        active.event.title
                    ))
                    .strong()
                    .color(severity_color(palette, active.event.severity)),
                );
                ui.label(&active.event.body);
                if let Some(envelope) = shell.mock_dispatch.as_ref() {
                    let content_revision = envelope.message.command_id as u64;
                    draw_retained_lines_or_build(
                        ui,
                        retained,
                        ProductShellWidgetId::Transmission,
                        content_revision,
                        shell.queue.len() as u64,
                        content_revision,
                        |ui| {
                            let _ = ui;
                            vec![format!(
                                "Dispatch DTO loss {:.0}% · corruption {:.0}%",
                                envelope.loss_probability * 100.0,
                                envelope.corruption_hint * 100.0
                            )]
                        },
                    );
                }
            } else {
                ui.label(egui::RichText::new("No active transmission.").weak());
            }
            ui.separator();
            ui.label(format!("Queued: {}", shell.queue.len()));
            let history: Vec<String> = shell
                .queue
                .iter()
                .map(|event| {
                    format!(
                        "[{} / {}] {}",
                        event.channel.label(),
                        event.severity.label(),
                        event.title
                    )
                })
                .collect();
            draw_virtualized_rows(
                ui,
                "transmission_history",
                18.0,
                72.0,
                history.len(),
                |ui, row| {
                    ui.label(&history[row]);
                },
            );
            if async_queue.cache.transmission_lines.is_empty() {
                if let Some(active) = shell.current.as_ref() {
                    async_queue.enqueue(HudAsyncTask::TransmissionFormat {
                        title: active.event.title.clone(),
                        body: active.event.body.clone(),
                    });
                }
            } else {
                draw_retained_lines_or_build(
                    ui,
                    retained,
                    ProductShellWidgetId::Transmission,
                    shell.queue.len() as u64,
                    shell.signal_pulse.to_bits() as u64,
                    shell.queue.len() as u64,
                    |ui| {
                        let _ = ui;
                        async_queue.cache.transmission_lines.clone()
                    },
                );
            }
            if ui.button("Next").clicked() && !shell.paused {
                shell.promote_next(media);
            }
        },
        widget_timing,
    ) {
        capture_shell_layout(layout_store, HudWidgetId::Transmission, &response, Some(pending_layout));
    }
    shell.active = open;
    shell.minimized = dock.slot(HudWidgetId::Transmission).minimized;
}

trait NarrativeCategoryLabel {
    fn category_label(&self) -> String;
}

impl NarrativeCategoryLabel for crate::strategic::NarrativeObservation {
    fn category_label(&self) -> String {
        match self.category {
            NarrativeCategory::Logistics => "Logistics".into(),
            NarrativeCategory::Infrastructure => "Infrastructure".into(),
            NarrativeCategory::Weather => "Weather".into(),
            NarrativeCategory::Faction => "Faction".into(),
            NarrativeCategory::General => "General".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transmission_queue_caps_and_promotes() {
        let mut shell = TransmissionShellState::default();
        for i in 0..(TransmissionShellState::QUEUE_CAP + 4) {
            shell.enqueue(TransmissionEvent {
                channel: TransmissionChannelId::General,
                severity: TransmissionSeverity::Routine,
                title: format!("t{i}"),
                body: format!("body{i}"),
                queued_at: SimStepStamp::new(i as u64, 0),
            });
        }
        let mut media = TransmissionMediaProvider::default();
        shell.promote_next(&mut media);
        assert_eq!(shell.queue.len(), TransmissionShellState::QUEUE_CAP - 1);
        assert!(shell.current.is_some());
    }
}
