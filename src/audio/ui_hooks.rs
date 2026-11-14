//! HUD / panel button presses → [`AudioOneShotQueue`] (presentation only).
//! Alert bark priority + exclusivity (AUD-08).

use bevy::prelude::*;
use bevy::ui_widgets::Button;

use super::mix_frame::{AudioEventClass, AudioMixRequest, AudioOneShotKind, AudioOneShotQueue};

/// Alert bark classes for warning one-shots (priority table below).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlertBarkKind {
    UiAck,
    LogisticsWarning,
    WeatherCritical,
    LaunchCritical,
}

/// Higher value = more important; equal values must not stack.
#[must_use]
pub const fn alert_bark_priority(kind: AlertBarkKind) -> u8 {
    match kind {
        AlertBarkKind::UiAck => 10,
        AlertBarkKind::LogisticsWarning => 40,
        AlertBarkKind::WeatherCritical => 70,
        AlertBarkKind::LaunchCritical => 90,
    }
}

/// Outcome when admitting a warning one-shot under exclusivity rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmitAlertBark {
    Enqueued,
    Interrupted { replaced: AlertBarkKind },
    SuppressedEqual,
    SuppressedLower,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct AlertBarkLane {
    active: Option<(AlertBarkKind, u8)>,
}

/// Sole writer of [`AlertBarkLane`] during warning collapse (AUD-08).
pub fn apply_alert_bark_priority_in_place(
    lane: &mut AlertBarkLane,
    pending: &mut Vec<AudioMixRequest>,
) {
    lane.active = None;
    let mut kept: Vec<AudioMixRequest> = Vec::with_capacity(pending.len());
    let mut warnings: Vec<AudioMixRequest> = Vec::new();
    for req in pending.drain(..) {
        if req.class == AudioEventClass::Warning {
            warnings.push(req);
        } else {
            kept.push(req);
        }
    }

    for req in warnings {
        let Some(kind) = req.kind else {
            kept.push(req);
            continue;
        };
        let Some(bark) = alert_bark_kind_for_oneshot(kind) else {
            kept.push(req);
            continue;
        };
        let priority = alert_bark_priority(bark);
        match &lane.active {
            None => {
                lane.active = Some((bark, priority));
                kept.push(req);
            }
            Some((_, active_pri)) if priority > *active_pri => {
                kept.retain(|r| r.class != AudioEventClass::Warning);
                lane.active = Some((bark, priority));
                kept.push(req);
            }
            Some((active_kind, active_pri)) if priority == *active_pri => {
                let _ = (active_kind, bark);
            }
            Some((_, active_pri)) if priority < *active_pri => {}
            Some(_) => {}
        }
    }
    *pending = kept;
}

#[must_use]
pub fn alert_bark_kind_for_oneshot(kind: AudioOneShotKind) -> Option<AlertBarkKind> {
    match kind {
        AudioOneShotKind::UiConfirm | AudioOneShotKind::UiDeny => Some(AlertBarkKind::UiAck),
        AudioOneShotKind::WeatherAlert => Some(AlertBarkKind::WeatherCritical),
        AudioOneShotKind::LaunchWarning => Some(AlertBarkKind::LaunchCritical),
        _ => None,
    }
}

/// Apply [`alert_bark_priority`] rules to pending warning requests (mutates `pending` in place).
pub fn apply_alert_bark_priority_to_pending(
    lane: &mut AlertBarkLane,
    pending: &mut Vec<AudioMixRequest>,
) {
    apply_alert_bark_priority_in_place(lane, pending);
}

/// Any Bevy UI `Button` press enqueues a UI click (play UI only — no sim mut).
pub fn enqueue_ui_button_clicks(
    mut audio: ResMut<AudioOneShotQueue>,
    q: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in &q {
        if *interaction == Interaction::Pressed {
            audio.push_ui(super::mix_frame::AudioOneShotKind::UiClick);
        }
    }
}

/// Collapse warning one-shots by bark priority before the mix is built.
pub fn apply_alert_bark_exclusivity(
    mut lane: ResMut<AlertBarkLane>,
    mut queue: ResMut<AudioOneShotQueue>,
) {
    apply_alert_bark_priority_to_pending(&mut lane, &mut queue.pending);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn warning_req(kind: AudioOneShotKind, tag: &'static str) -> AudioMixRequest {
        AudioMixRequest {
            class: AudioEventClass::Warning,
            kind: Some(kind),
            world_xz: None,
            gain: 1.0,
            tag,
        }
    }

    #[test]
    fn alert_bark_priority_table_ordering() {
        assert!(alert_bark_priority(AlertBarkKind::UiAck) < alert_bark_priority(AlertBarkKind::LogisticsWarning));
        assert!(alert_bark_priority(AlertBarkKind::LogisticsWarning) < alert_bark_priority(AlertBarkKind::WeatherCritical));
        assert!(alert_bark_priority(AlertBarkKind::WeatherCritical) < alert_bark_priority(AlertBarkKind::LaunchCritical));
    }

    #[test]
    fn equal_priority_does_not_stack() {
        let mut lane = AlertBarkLane::default();
        let mut pending = vec![
            warning_req(AudioOneShotKind::LaunchWarning, "warning.launch"),
            warning_req(AudioOneShotKind::LaunchWarning, "warning.launch"),
        ];
        apply_alert_bark_priority_to_pending(&mut lane, &mut pending);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].kind, Some(AudioOneShotKind::LaunchWarning));
    }

    #[test]
    fn higher_priority_interrupts_lower() {
        let mut lane = AlertBarkLane::default();
        let mut pending = vec![
            warning_req(AudioOneShotKind::WeatherAlert, "warning.weather"),
            warning_req(AudioOneShotKind::LaunchWarning, "warning.launch"),
        ];
        apply_alert_bark_priority_to_pending(&mut lane, &mut pending);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].kind, Some(AudioOneShotKind::LaunchWarning));
    }

    #[test]
    fn lower_priority_suppressed_while_higher_active() {
        let mut lane = AlertBarkLane::default();
        let mut pending = vec![warning_req(AudioOneShotKind::LaunchWarning, "warning.launch")];
        apply_alert_bark_priority_to_pending(&mut lane, &mut pending);
        pending.push(warning_req(AudioOneShotKind::WeatherAlert, "warning.weather"));
        apply_alert_bark_priority_to_pending(&mut lane, &mut pending);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].kind, Some(AudioOneShotKind::LaunchWarning));
    }
}