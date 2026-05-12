//! L5 simulation storytelling — semantic observations (not “-5 wood”).

use bevy::prelude::*;
use std::collections::VecDeque;

use crate::strategic::{LogisticsAiRuntime, OperationalTheaterSummary};

/// High-level bucket for UI filtering / coloring later.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NarrativeCategory {
    Logistics,
    Infrastructure,
    Weather,
    Faction,
    General,
}

#[derive(Clone, Debug)]
pub struct NarrativeObservation {
    pub category: NarrativeCategory,
    pub severity: f32,
    pub generated_text: String,
}

const BUS_CAP: usize = 48;

#[derive(Resource, Debug, Default)]
pub struct NarrativeObservationBus {
    pub recent: VecDeque<NarrativeObservation>,
}

impl NarrativeObservationBus {
    pub fn push(&mut self, obs: NarrativeObservation) {
        if self
            .recent
            .back()
            .is_some_and(|b| b.generated_text == obs.generated_text)
        {
            return;
        }
        if self.recent.len() >= BUS_CAP {
            self.recent.pop_front();
        }
        self.recent.push_back(obs);
    }

    /// Most recent first, capped for HUD.
    pub fn format_hud_tail(&self, n: usize) -> String {
        self.recent
            .iter()
            .rev()
            .take(n)
            .map(|o| o.generated_text.as_str())
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

/// Edge-triggered stubs from runtime telemetry — replace with field/analytics writers as sim deepens.
pub fn narrative_observations_from_runtime_system(
    logistics: Res<LogisticsAiRuntime>,
    theater: Res<OperationalTheaterSummary>,
    mut bus: ResMut<NarrativeObservationBus>,
    mut prev_cong: Local<Option<f32>>,
    mut prev_threat: Local<Option<f32>>,
) {
    let cong = logistics.congestion_proxy;
    if let Some(p) = *prev_cong {
        if cong >= 0.55 && p < 0.38 {
            bus.push(NarrativeObservation {
                category: NarrativeCategory::Logistics,
                severity: cong,
                generated_text:
                    "Routing stress is climbing — corridor dwell times are stretching.".into(),
            });
        }
    }
    *prev_cong = Some(cong);

    let threat = theater.mean_threat_by_slot[0];
    if let Some(p) = *prev_threat {
        if threat >= 0.45 && p < 0.28 {
            bus.push(NarrativeObservation {
                category: NarrativeCategory::Faction,
                severity: threat,
                generated_text: "Theater tension jumped — sustain and repair loads are rising.".into(),
            });
        }
    }
    *prev_threat = Some(threat);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bus_dedupes_identical_lines() {
        let mut b = NarrativeObservationBus::default();
        let o = NarrativeObservation {
            category: NarrativeCategory::General,
            severity: 0.5,
            generated_text: "dup".into(),
        };
        b.push(o.clone());
        b.push(o);
        assert_eq!(b.recent.len(), 1);
    }
}
