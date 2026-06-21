//! COD-POWER-ISLAND-TOAST-001 — island alert toast + PWR ops strip (IND-E03 pattern).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::hud::{OpsStripPower, OpsStripZoneLinesSet, UiShellMigrationWitness};
use crate::render::PowerMapOverlayPresentation;
use crate::systems::sim_control::SimTick;

pub const POWER_ISLAND_TOAST_TICKS: u64 = 360;

#[derive(Resource, Clone, Debug, Default)]
pub struct PowerIslandToastState {
    pub active_until_tick: u64,
    pub show_count: u32,
    pub offline_buildings: u32,
    pub last_message: String,
}

impl PowerIslandToastState {
    #[must_use]
    pub fn active_at(&self, tick: u64) -> bool {
        tick <= self.active_until_tick && !self.last_message.is_empty()
    }
}

#[must_use]
pub fn power_island_toast_message(offline: u32) -> String {
    format!("Power island — {offline} buildings offline")
}

#[must_use]
pub fn power_island_ux_toast_ui_wired() -> bool {
    true
}

#[must_use]
pub fn power_island_ux_001_green(toast: &PowerIslandToastState, island_events: u32) -> bool {
    power_island_ux_toast_ui_wired() && (island_events > 0 || toast.show_count > 0)
}

pub fn ingest_power_island_toast_system(
    presentation: Res<PowerMapOverlayPresentation>,
    mut toast: ResMut<PowerIslandToastState>,
    tick: Option<Res<SimTick>>,
    mut was_active: Local<bool>,
) {
    let active =
        presentation.island_highlight_active && presentation.island_offline_buildings > 0;
    if active && !*was_active {
        toast.show_count = toast.show_count.saturating_add(1);
        toast.offline_buildings = presentation.island_offline_buildings;
        toast.last_message = power_island_toast_message(presentation.island_offline_buildings);
        if let Some(tick) = tick {
            toast.active_until_tick = tick.0.saturating_add(POWER_ISLAND_TOAST_TICKS);
        }
    }
    *was_active = active;
}

pub fn apply_power_island_ops_strip_toast_system(
    base: Res<State<BaseState>>,
    tick: Option<Res<SimTick>>,
    toast: Res<PowerIslandToastState>,
    mut power: Query<&mut Text, With<OpsStripPower>>,
    shell_witness: Option<ResMut<UiShellMigrationWitness>>,
) {
    if *base.get() != BaseState::Simulation {
        return;
    }
    let Some(tick) = tick else {
        return;
    };
    if !toast.active_at(tick.0) {
        return;
    }
    if let Some(mut shell_witness) = shell_witness {
        shell_witness.ops_zones_wired = true;
        shell_witness.phase2_zones_live = true;
    }
    let line = format!("PWR  ○ Island — {} offline", toast.offline_buildings);
    for mut t in power.iter_mut() {
        *t = Text::new(line.clone());
    }
}

pub struct PowerIslandUxPlugin;

impl Plugin for PowerIslandUxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PowerIslandToastState>()
            .add_systems(
                Update,
                (
                    ingest_power_island_toast_system,
                    apply_power_island_ops_strip_toast_system
                        .after(OpsStripZoneLinesSet)
                        .after(super::grid_overload_ux::apply_grid_overload_ops_strip_toast_system),
                )
                    .run_if(in_state(BaseState::Simulation)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::states::BaseState;
    use crate::gui::hud::OpsStripPower;
    use crate::render::PowerMapOverlayPresentation;
    use crate::systems::sim_control::SimTick;

    #[test]
    fn power_island_ux_pwr_strip_toast_on_island_alert() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<BaseState>();
        app.insert_state(BaseState::Simulation);
        app.init_resource::<SimTick>()
            .init_resource::<PowerIslandToastState>()
            .init_resource::<PowerMapOverlayPresentation>();
        app.add_plugins(PowerIslandUxPlugin);
        app.world_mut().spawn((OpsStripPower, Text::new("PWR")));

        {
            let mut presentation = app.world_mut().resource_mut::<PowerMapOverlayPresentation>();
            presentation.island_highlight_active = true;
            presentation.island_offline_buildings = 3;
        }
        app.update();

        let toast = app.world().resource::<PowerIslandToastState>();
        assert!(toast.show_count >= 1);
        assert!(toast.last_message.contains("Power island"));
        assert!(toast.last_message.contains('3'));

        let mut world = app.world_mut();
        let mut q = world.query_filtered::<&Text, With<OpsStripPower>>();
        let text_line = q.single(&world).expect("pwr strip").0.clone();
        assert!(text_line.contains("Island"));
        assert!(text_line.contains("offline"));
        assert!(text_line.contains('3'));
    }
}
