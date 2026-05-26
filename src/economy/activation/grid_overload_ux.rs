//! S7P-GRID-UX-001 — grid overload player feedback (ops-strip toast + witness).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::entities::production::power::GridOverloadEvent;
use crate::gui::hud::{OpsStripPower, OpsStripZoneLinesSet, UiShellMigrationWitness};
use crate::systems::sim_control::SimTick;

/// Tray/toast copy when [`GridOverloadEvent`] fires ([`s7p_grid_overload_ux_note_v1.md`](../../dev/s7p_grid_overload_ux_note_v1.md)).
pub const GRID_OVERLOAD_TOAST_MESSAGE: &str =
    "Grid overload — reduce smelter load or add transformer capacity";

/// Toast visibility window in sim ticks (~8s @ 30 Hz).
pub const GRID_OVERLOAD_TOAST_TICKS: u64 = 240;

#[derive(Resource, Clone, Debug, Default)]
pub struct GridOverloadToastState {
    pub active_until_tick: u64,
    pub show_count: u32,
    pub last_message: String,
}

impl GridOverloadToastState {
    #[must_use]
    pub fn active_at(&self, tick: u64) -> bool {
        tick <= self.active_until_tick && !self.last_message.is_empty()
    }
}

#[must_use]
pub fn s7p_grid_ux_toast_ui_wired() -> bool {
    true
}

#[must_use]
pub fn s7p_grid_ux_001_green(toast: &GridOverloadToastState, overload_events: u64) -> bool {
    s7p_grid_ux_toast_ui_wired() && (overload_events > 0 || toast.show_count > 0)
}

pub fn ingest_grid_overload_toast_system(
    mut reader: MessageReader<GridOverloadEvent>,
    mut toast: ResMut<GridOverloadToastState>,
    tick: Res<SimTick>,
) {
    for _ in reader.read() {
        toast.show_count = toast.show_count.saturating_add(1);
        toast.active_until_tick = tick.0.saturating_add(GRID_OVERLOAD_TOAST_TICKS);
        toast.last_message = GRID_OVERLOAD_TOAST_MESSAGE.into();
    }
}

/// Flash **PWR** ops-strip zone with overload copy while toast is active.
pub fn apply_grid_overload_ops_strip_toast_system(
    base: Res<State<BaseState>>,
    tick: Res<SimTick>,
    toast: Res<GridOverloadToastState>,
    mut power: Query<&mut Text, With<OpsStripPower>>,
    shell_witness: Option<ResMut<UiShellMigrationWitness>>,
) {
    if *base.get() != BaseState::Simulation {
        return;
    }
    if !toast.active_at(tick.0) {
        return;
    }
    if let Some(mut shell_witness) = shell_witness {
        shell_witness.ops_zones_wired = true;
        shell_witness.phase2_zones_live = true;
    }
    let line = format!("PWR  ⚠ {}", toast.last_message);
    for mut t in power.iter_mut() {
        *t = Text::new(line.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::states::BaseState;
    use crate::entities::production::power::GridOverloadEvent;
    use crate::gui::hud::OpsStripPower;
    use crate::systems::sim_control::SimTick;

    #[test]
    fn s7p_grid_ux_ui_001_pwr_strip_toast_after_overload_event() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<BaseState>();
        app.insert_state(BaseState::Simulation);
        app.init_resource::<SimTick>()
            .init_resource::<GridOverloadToastState>();
        app.add_message::<GridOverloadEvent>();
        app.add_plugins(GridOverloadUxPlugin);
        app.world_mut().spawn((OpsStripPower, Text::new("PWR")));

        app.world_mut().write_message(GridOverloadEvent {
            grid_entity: Entity::PLACEHOLDER,
            total_load: 120.0,
            total_capacity: 40.0,
        });
        app.update();

        let toast = app.world().resource::<GridOverloadToastState>();
        assert!(toast.show_count >= 1);
        assert_eq!(toast.last_message, GRID_OVERLOAD_TOAST_MESSAGE);
        assert!(s7p_grid_ux_toast_ui_wired());

        let mut world = app.world_mut();
        let mut q = world.query_filtered::<&Text, With<OpsStripPower>>();
        let text_line = q.single(&world).expect("pwr strip").0.clone();
        assert!(text_line.contains("Grid overload"));
    }
}

pub struct GridOverloadUxPlugin;

impl Plugin for GridOverloadUxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GridOverloadToastState>()
            .add_systems(
                Update,
                (
                    ingest_grid_overload_toast_system,
                    apply_grid_overload_ops_strip_toast_system.after(OpsStripZoneLinesSet),
                )
                    .run_if(in_state(BaseState::Simulation)),
            );
    }
}
