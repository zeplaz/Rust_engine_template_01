//! Ordering for per-chunk **environment** systems: LOD refresh → **weather** → **ecology**
//! (vegetation, ecology tick, [`FireFuelField`](crate::systems::fire::FireFuelField) derivation) (**fire**
//! overlay / smoke / ember) — matches `base_fire2_smoke.md` “weather → ecology → fire” and ensures fire reads
//! fuel updated this frame.
//! Configured from [`crate::engine::EnginePlugin`](crate::engine::EnginePlugin).
//!
//! # LOOP-06 — dependency tiers (GEA 4e Vol II §17.6.3.2 Bucketed Updates)
//!
//! Cross-domain SystemSets are grouped by **dependency depth**, not ad-hoc `.after(system_fn)` chains:
//!
//! | Tier | Sets (anchors) | Role |
//! |:---|:---|:---|
//! | **Env** | [`ChunkEnvironmentSet`] Lod → Weather → Ecology → Fire | Chunk fields that feed each other |
//! | **Logistics** | [`StrategicFieldPipeline`](crate::strategic::StrategicFieldPipeline) through `NetworkFlow` | Graph sync → net inject → network flow |
//! | **Combat** | `StrategicFieldPipeline::ZoneAndReadModel` (combat systems) | Fronts / formations / resolve after logistics + weather |
//! | **AI** | [`FactionAiSet::Tick`](crate::ai::faction::FactionAiSet) | Perceive/decide after zone + combat read model |
//!
//! Production edges (also reasserted by [`configure_sim_dependency_tier_edges`]):
//! - `NetworkFlow` **after** `ChunkEnvironmentSet::Weather` (env → logistics)
//! - `ZoneAndReadModel` **after** `NetworkFlow` **and** `Weather` (logistics → combat)
//! - `FactionAiSet::Tick` **after** `ZoneAndReadModel` (combat → AI)
//!
//! Dual-cite: **LOOP-06** · LEGACY `game_object_update` (gea_v2_e4 p606).

use bevy::prelude::*;

use crate::ai::faction::FactionAiSet;
use crate::strategic::StrategicFieldPipeline;
use crate::systems::sim_attention::AttentionSet;
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
/// Lod runs **after** [`AttentionSet::Build`] when attention is configured (Slice A3).
pub fn configure_chunk_environment_sets(app: &mut App) {
    app.configure_sets(
        Update,
        (
            AttentionSet::Build.after(SimControlSystemSet::AdvanceSimTick),
            ChunkEnvironmentSet::Lod
                .after(SimControlSystemSet::AdvanceSimTick)
                .after(AttentionSet::Build),
            ChunkEnvironmentSet::Weather.after(ChunkEnvironmentSet::Lod),
            ChunkEnvironmentSet::Ecology.after(ChunkEnvironmentSet::Weather),
            ChunkEnvironmentSet::Fire.after(ChunkEnvironmentSet::Ecology),
        ),
    );
    configure_sim_dependency_tier_edges(app);
}

/// LOOP-06: harden env → logistics → combat → AI as named SystemSet edges.
///
/// Intra-set chains stay owned by their plugins (`StrategicFieldsPlugin`, `FactionAiPlugin`,
/// `CombatPlugin`). This only asserts the **cross-tier** dependency-depth edges so partners do not
/// invent parallel `.after` chains.
pub fn configure_sim_dependency_tier_edges(app: &mut App) {
    app.configure_sets(
        Update,
        (
            StrategicFieldPipeline::NetworkFlow.after(ChunkEnvironmentSet::Weather),
            StrategicFieldPipeline::ZoneAndReadModel
                .after(StrategicFieldPipeline::NetworkFlow)
                .after(ChunkEnvironmentSet::Weather),
            FactionAiSet::Tick.after(StrategicFieldPipeline::ZoneAndReadModel),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::{configure_chunk_environment_sets, configure_sim_dependency_tier_edges, ChunkEnvironmentSet};
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    use crate::ai::faction::FactionAiSet;
    use crate::gui::InputBindings;
    use crate::strategic::StrategicFieldPipeline;
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
        configure_chunk_environment_sets(&mut app);
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

    /// LOOP-06: dependency-depth tiers env → logistics → combat → AI.
    #[test]
    fn loop06_dependency_tiers_env_logistics_combat_ai_order() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);
        app.init_resource::<InputBindings>();
        app.add_plugins(SimControlPlugin);
        configure_chunk_environment_sets(&mut app);
        // Mirror StrategicFieldsPlugin intra-logistics spine (minimal) so NetworkFlow is reachable.
        app.configure_sets(
            Update,
            (
                StrategicFieldPipeline::EnsureOverlays
                    .after(crate::systems::sim_control::SimControlSystemSet::AdvanceSimTick),
                StrategicFieldPipeline::GraphSync.after(StrategicFieldPipeline::EnsureOverlays),
                StrategicFieldPipeline::InjectTransportScalars
                    .after(StrategicFieldPipeline::GraphSync),
                StrategicFieldPipeline::LogisticsNetInject
                    .after(StrategicFieldPipeline::InjectTransportScalars),
                StrategicFieldPipeline::NetworkFlow
                    .after(StrategicFieldPipeline::LogisticsNetInject),
            ),
        );
        configure_sim_dependency_tier_edges(&mut app);
        app.init_resource::<OrderLog>();
        app.add_systems(
            Update,
            (
                (|mut log: ResMut<OrderLog>| log.0.push(1)).in_set(ChunkEnvironmentSet::Weather),
                (|mut log: ResMut<OrderLog>| log.0.push(2))
                    .in_set(StrategicFieldPipeline::NetworkFlow),
                (|mut log: ResMut<OrderLog>| log.0.push(3))
                    .in_set(StrategicFieldPipeline::ZoneAndReadModel),
                (|mut log: ResMut<OrderLog>| log.0.push(4)).in_set(FactionAiSet::Tick),
            ),
        );
        app.update();
        assert_eq!(
            app.world().resource::<OrderLog>().0,
            vec![1, 2, 3, 4],
            "LOOP-06 tiers: Weather(env) → NetworkFlow(logistics) → ZoneAndReadModel(combat) → FactionAi(AI)"
        );
    }

    #[test]
    fn fire_fuel_field_spawns_and_derives_before_fire_pass() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<Image>();
        app.add_plugins(InputPlugin);
        app.init_resource::<InputBindings>();
        // Fire's ember-spot emitter enqueues through the sim-effect queue (owned by `SimEffectsPlugin`).
        app.init_resource::<crate::sim::effects::SimEffectQueue>();
        app.init_resource::<crate::render::extraction::RenderProjectionGraph>();
        app.init_resource::<crate::gui::OverlayFieldFrame>();
        app.add_plugins(SimControlPlugin);
        configure_chunk_environment_sets(&mut app);
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
