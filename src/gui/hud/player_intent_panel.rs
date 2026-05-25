//! Stage-7 consumer intent panel — display vs player intent separation (no simulation).

use bevy::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct PlayerIntentDraft {
    pub label: String,
    pub summary: String,
}

/// UI-only staging area for future intent submission (**BQ-127**).
#[derive(Resource, Clone, Debug, Default)]
pub struct PlayerIntentPanelState {
    pub drafts: Vec<PlayerIntentDraft>,
}

impl PlayerIntentPanelState {
    pub fn stage_mock(&mut self, label: impl Into<String>, summary: impl Into<String>) {
        self.drafts.push(PlayerIntentDraft {
            label: label.into(),
            summary: summary.into(),
        });
        const CAP: usize = 16;
        if self.drafts.len() > CAP {
            let overflow = self.drafts.len() - CAP;
            self.drafts.drain(0..overflow);
        }
    }
}
