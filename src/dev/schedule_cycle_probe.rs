//! Update schedule cycle probe — bisect which plugin graph edge fails initialization.

use bevy::prelude::*;

/// Initialize `Update` and return a human-readable error (includes cycle details when available).
pub fn probe_update_schedule(world: &mut World) -> Result<(), String> {
    world
        .try_schedule_scope(Update, |world, schedule| {
            schedule
                .initialize(world)
                .map_err(|err| {
                    format!(
                        "Update schedule initialize failed.\nDebug: {:?}\nDisplay:\n{}",
                        err,
                        err.to_string(schedule.graph(), world)
                    )
                })
                .map(|_| ())
        })
        .map_err(|_| "Update schedule missing".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::InputPlugin;
    use bevy::state::app::StatesPlugin;
    use bevy::window::WindowPlugin;

    #[test]
    fn probe_update_schedule_on_minimal_app() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            StatesPlugin,
            WindowPlugin::default(),
            InputPlugin,
        ));
        app.finish();
        if let Err(msg) = probe_update_schedule(app.world_mut()) {
            panic!("minimal app Update schedule failed:\n{msg}");
        }
    }
}
