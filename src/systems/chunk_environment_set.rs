//! Ordering for per-chunk **environment** systems: LOD refresh → **weather** → **ecology**
//! (vegetation, ecology tick, [`FireFuelField`](crate::systems::fire::FireFuelField) derivation) (**fire**
//! overlay / smoke / ember) — matches `base_fire2_smoke.md` “weather → ecology → fire” and ensures fire reads
//! fuel updated this frame.
//! Configured from [`crate::engine::EnginePlugin`](crate::engine::EnginePlugin).

use bevy::prelude::*;

use crate::systems::sim_control::SimControlSystemSet;

/// Fixed ordering after [`SimControlSystemSet::AdvanceSimTick`] for chunk fields that feed each other.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChunkEnvironmentSet {
    /// Refresh sim intensity tier from prior frame’s weather + fire (adaptive tick weight).
    Lod,
    Weather,
    Ecology,
    Fire,
}

/// Call from the root engine plugin after `SimControlPlugin`.
pub fn configure_chunk_environment_sets(app: &mut App) {
    app.configure_sets(
        Update,
        (
            ChunkEnvironmentSet::Lod.after(SimControlSystemSet::AdvanceSimTick),
            ChunkEnvironmentSet::Weather.after(ChunkEnvironmentSet::Lod),
            ChunkEnvironmentSet::Ecology.after(ChunkEnvironmentSet::Weather),
            ChunkEnvironmentSet::Fire.after(ChunkEnvironmentSet::Ecology),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::ChunkEnvironmentSet;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    use crate::gui::InputBindings;
    use crate::systems::chunk_environment_persist::ChunkEnvironmentPersistPlugin;
    use crate::systems::chunk_sim_lod::ChunkSimLodPlugin;
    use crate::systems::ecology::EcologyPlugin;
    use crate::systems::fire::{derive_fire_fuel_from_vegetation, FireFuelField, FirePlugin};
    use crate::systems::sim_control::SimControlPlugin;
    use crate::systems::weather::WeatherSimulationPlugin;
    use crate::terrain::generation::Chunk;

    #[derive(Resource, Default, Clone)]
    struct OrderLog(Vec<u32>);

    fn push_lod(mut log: ResMut<OrderLog>) {
        log.0.push(1);
    }
    fn push_weather(mut log: ResMut<OrderLog>) {
        log.0.push(2);
    }
    fn push_ecology(mut log: ResMut<OrderLog>) {
        log.0.push(3);
    }
    fn push_fire(mut log: ResMut<OrderLog>) {
        log.0.push(4);
    }

    #[test]
    fn chunk_environment_set_runs_lod_weather_ecology_fire_in_order() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);
        app.init_resource::<InputBindings>();
        app.add_plugins(SimControlPlugin);
        super::configure_chunk_environment_sets(&mut app);
        app.init_resource::<OrderLog>();
        app.add_systems(
            Update,
            (
                push_lod.in_set(ChunkEnvironmentSet::Lod),
                push_weather.in_set(ChunkEnvironmentSet::Weather),
                push_ecology.in_set(ChunkEnvironmentSet::Ecology),
                push_fire.in_set(ChunkEnvironmentSet::Fire),
            ),
        );
        app.update();
        assert_eq!(app.world().resource::<OrderLog>().0, vec![1, 2, 3, 4]);
    }

    #[test]
    fn fire_fuel_field_spawns_and_derives_before_fire_pass() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);
        app.init_resource::<InputBindings>();
        app.add_plugins(SimControlPlugin);
        super::configure_chunk_environment_sets(&mut app);
        app.add_plugins((
            ChunkEnvironmentPersistPlugin,
            ChunkSimLodPlugin,
            WeatherSimulationPlugin,
            EcologyPlugin,
            FirePlugin,
        ));

        app.world_mut().spawn(Chunk {
            coord: IVec2::ZERO,
        });
        app.update();

        let (veg, wx, eco, fuel) = {
            let world = app.world_mut();
            let mut q = world.query::<(
                &crate::systems::ecology::VegetationField,
                &crate::systems::weather::ChunkWeather,
                &crate::systems::ecology::ChunkEcology,
                &FireFuelField,
            )>();
            let Some((v, w, e, f)) = q.iter(world).next() else {
                panic!("expected one chunk with ecology + weather + fuel");
            };
            (*v, *w, *e, *f)
        };
        let expected = derive_fire_fuel_from_vegetation(&veg, &wx, &eco);
        assert!(
            (fuel.ember_spread_factor - expected.ember_spread_factor).abs() < 1e-5,
            "fuel tick should run in Ecology before Fire; ember_spread_factor should match derived {:?} vs {:?}",
            fuel.ember_spread_factor,
            expected.ember_spread_factor
        );
        assert!(
            (fuel.surface_fuel - expected.surface_fuel).abs() < 1e-5,
            "surface_fuel mismatch"
        );
    }
}
