//! Active tool session — keep tools alive across commits (Round 2).

use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlacementBrushMode {
    #[default]
    Single,
    Line,
    Rectangle,
    Paint,
}

#[derive(Resource, Debug, Clone)]
pub struct ActiveToolSession {
    pub started_at_secs: f64,
    pub actions_committed: usize,
    /// Road/rail: chain commits without clearing the path anchor.
    pub continuous_path: bool,
    /// Building/road/zone: do not reset `ActiveBuildTool` on Esc (only clear ghosts).
    pub keep_tool_after_commit: bool,
    /// Zone paint: queue painted tiles when LMB drag ends (no Shift required).
    pub zone_auto_commit_on_release: bool,
    pub brush_mode: PlacementBrushMode,
}

impl Default for ActiveToolSession {
    fn default() -> Self {
        Self {
            started_at_secs: 0.0,
            actions_committed: 0,
            continuous_path: true,
            keep_tool_after_commit: true,
            zone_auto_commit_on_release: true,
            brush_mode: PlacementBrushMode::Paint,
        }
    }
}

impl ActiveToolSession {
    pub fn record_commit(&mut self) {
        self.actions_committed = self.actions_committed.saturating_add(1);
    }
}

pub fn tick_tool_session_time(time: Res<Time>, mut session: ResMut<ActiveToolSession>) {
    if session.started_at_secs <= 0.0 {
        session.started_at_secs = time.elapsed_secs_f64();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_session_keeps_tool_and_continuous_path() {
        let s = ActiveToolSession::default();
        assert!(s.keep_tool_after_commit);
        assert!(s.continuous_path);
        assert!(s.zone_auto_commit_on_release);
    }
}
