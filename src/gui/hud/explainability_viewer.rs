//! Explainability viewer scaffolding — DTO feed widgets only.

use bevy::prelude::*;
use bevy_egui::egui;

use super::virtualized_list::draw_virtualized_rows;

use crate::strategic::BeliefSnapshotDto;
use crate::systems::sim_control::SimStepStamp;

#[derive(Clone, Debug)]
pub struct ExplainabilityFeedEvent {
    pub stamp: SimStepStamp,
    pub category: String,
    pub summary: String,
    pub confidence: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct ExplainabilityViewerState {
    pub filter: String,
    pub group_by_category: bool,
    pub scrub_position: f32,
    pub events: Vec<ExplainabilityFeedEvent>,
}

impl Default for ExplainabilityViewerState {
    fn default() -> Self {
        Self {
            filter: String::new(),
            group_by_category: true,
            scrub_position: 1.0,
            events: mock_explainability_events(),
        }
    }
}

#[must_use]
pub fn mock_explainability_events() -> Vec<ExplainabilityFeedEvent> {
    vec![
        ExplainabilityFeedEvent {
            stamp: SimStepStamp::new(8, 0),
            category: "logistics".into(),
            summary: "Depot throughput constrained by corridor EW".into(),
            confidence: 0.66,
        },
        ExplainabilityFeedEvent {
            stamp: SimStepStamp::new(12, 0),
            category: "threat".into(),
            summary: "Recon confidence decay on eastern flank".into(),
            confidence: 0.52,
        },
    ]
}

pub fn explainability_events_from_belief(rows: &[BeliefSnapshotDto]) -> Vec<ExplainabilityFeedEvent> {
    rows.iter()
        .map(|row| ExplainabilityFeedEvent {
            stamp: row.last_refresh,
            category: "belief".into(),
            summary: row.summary.clone(),
            confidence: row.confidence.scalar,
        })
        .collect()
}

pub fn draw_explainability_viewer(ui: &mut egui::Ui, state: &mut ExplainabilityViewerState) {
    ui.horizontal(|ui| {
        ui.label("Filter");
        ui.text_edit_singleline(&mut state.filter);
        ui.checkbox(&mut state.group_by_category, "Group");
    });
    ui.add(egui::Slider::new(&mut state.scrub_position, 0.0..=1.0).text("Timeline scrub"));
    ui.separator();
    let filtered: Vec<_> = state
        .events
        .iter()
        .filter(|event| {
            state.filter.is_empty()
                || event
                    .summary
                    .to_lowercase()
                    .contains(&state.filter.to_lowercase())
        })
        .collect();
    draw_virtualized_rows(ui, "explainability_feed", 18.0, 140.0, filtered.len(), |ui, row| {
        let event = filtered[row];
        ui.label(format!(
            "t{} · {} · {:.0}% — {}",
            event.stamp.tick,
            event.category,
            event.confidence * 100.0,
            event.summary
        ));
    });
    ui.label(
        egui::RichText::new("Replay viewer placeholder — authoritative replay spine **BQ-117**.")
            .small()
            .weak(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explainability_mock_feed_is_non_empty() {
        assert!(!mock_explainability_events().is_empty());
    }
}
