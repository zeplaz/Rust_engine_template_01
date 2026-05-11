//! **Emergence output log** — fracture + resolution events as **readouts** (not quest triggers).

use std::collections::VecDeque;

use bevy::prelude::*;

use super::behavior_fracture::FractureEvent;
use super::hybrid_brain::{HybridSimLastResolved, WorldEvent};

/// Ring buffer for designer HUD / Mission composer **Event log** tab.
#[derive(Resource, Clone, Debug)]
pub struct StrategicEmergenceLog {
    pub max_lines: usize,
    pub lines: VecDeque<String>,
}

impl Default for StrategicEmergenceLog {
    fn default() -> Self {
        Self {
            max_lines: 256,
            lines: VecDeque::new(),
        }
    }
}

impl StrategicEmergenceLog {
    pub fn push(&mut self, line: impl Into<String>) {
        let s = line.into();
        while self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }
        self.lines.push_back(s);
    }

    #[inline]
    pub fn tail_joined(&self, n: usize) -> String {
        let n = n.max(1);
        self.lines
            .iter()
            .rev()
            .take(n)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn format_fracture_log_line(ev: &FractureEvent) -> String {
    format!(
        "[FRACTURE] faction {:?} pressure {:.2} drivers {:?}",
        ev.faction, ev.pressure, ev.drivers
    )
}

/// Logs when [`HybridSimLastResolved::event`] changes (PostUpdate, after resolve).
pub fn strategic_emergence_log_hybrid_resolution_system(
    last: Res<HybridSimLastResolved>,
    mut log: ResMut<StrategicEmergenceLog>,
    mut prev: Local<Option<WorldEvent>>,
) {
    let Some(cur) = last.event else {
        *prev = None;
        return;
    };
    if Some(cur) != *prev {
        log.push(format!("[RESOLVE] {:?}", cur));
        *prev = Some(cur);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_drops_oldest() {
        let mut log = StrategicEmergenceLog {
            max_lines: 2,
            lines: VecDeque::new(),
        };
        log.push("a");
        log.push("b");
        log.push("c");
        assert_eq!(log.lines.len(), 2);
        assert_eq!(log.lines[0], "b");
    }
}
