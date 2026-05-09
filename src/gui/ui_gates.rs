//! Shared `run_if` helpers so **player shell** (Bevy) and **dev tooling** (egui) stay separated.

use crate::engine::states::BaseState;
use bevy::prelude::*;

#[must_use]
pub fn in_simulation_or_editor(base: Res<State<BaseState>>) -> bool {
    matches!(base.get(), BaseState::Simulation | BaseState::Editor)
}
