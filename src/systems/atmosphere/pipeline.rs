//! Schedule sets for **post-fire** atmosphere work (`base_fire2_smoke.md` §18).
//!
//! Chunk-scale weather / ecology / fire run in [`crate::systems::chunk_environment_set::ChunkEnvironmentSet`].
//! This pipeline fills the global-ish [`super::field::AtmosphereField`], advects, then emitters → particles →
//! coupling hooks → **VisualExtract** (sim→render snapshots) → render prep → diagnostics. **Transport** is ordered after this stack from the engine.

use bevy::prelude::*;

use crate::systems::chunk_environment_set::ChunkEnvironmentSet;

/// Runs **after** [`ChunkEnvironmentSet::Fire`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AtmospherePipelineSet {
    /// Push chunk signals into [`super::field::AtmosphereField`].
    FieldFill,
    /// Semi-Lagrangian drift of smoke / toxic / ash.
    WindAdvect,
    /// Low-count [`super::emitter_sync::FireEmitter`] sync.
    Emitters,
    /// Budget / controller for GPU-bound particles (stub until instancing lands).
    Particles,
    /// Fold atmosphere into gameplay samples (logistics, LOS helpers).
    Coupling,
    /// Sim → render-facing snapshots ([`crate::render::SimFireEmitterVisualExtract`], smoke extract).
    VisualExtract,
    /// Extraction hooks for render world (stubs until GPU layers land).
    RenderPrep,
    /// Frame metrics for egui / HUD.
    Diagnostics,
}

/// Register ordering relative to chunk environment. Call from root after [`crate::systems::configure_chunk_environment_sets`].
pub fn configure_atmosphere_pipeline_sets(app: &mut App) {
    app.configure_sets(
        Update,
        (
            AtmospherePipelineSet::FieldFill.after(ChunkEnvironmentSet::Fire),
            AtmospherePipelineSet::WindAdvect.after(AtmospherePipelineSet::FieldFill),
            AtmospherePipelineSet::Emitters.after(AtmospherePipelineSet::WindAdvect),
            AtmospherePipelineSet::Particles.after(AtmospherePipelineSet::Emitters),
            AtmospherePipelineSet::Coupling.after(AtmospherePipelineSet::Particles),
            AtmospherePipelineSet::VisualExtract
                .after(AtmospherePipelineSet::Coupling)
                .after(ChunkEnvironmentSet::Fire),
            AtmospherePipelineSet::RenderPrep.after(AtmospherePipelineSet::VisualExtract),
            AtmospherePipelineSet::Diagnostics.after(AtmospherePipelineSet::RenderPrep),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::AtmospherePipelineSet;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    use crate::gui::InputBindings;
    use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
    use crate::systems::sim_control::SimControlPlugin;
    use crate::systems::transport::TransportSchedule;

    #[derive(Resource, Default, Clone)]
    struct Order(Vec<u32>);

    macro_rules! push {
        ($n:expr) => {
            |mut o: ResMut<Order>| {
                o.0.push($n);
            }
        };
    }

    #[test]
    fn atmosphere_pipeline_runs_after_chunk_fire_in_order() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);
        app.init_resource::<InputBindings>();
        app.add_plugins(SimControlPlugin);
        crate::systems::chunk_environment_set::configure_chunk_environment_sets(&mut app);
        super::configure_atmosphere_pipeline_sets(&mut app);
        app.init_resource::<Order>();
        app.add_systems(
            Update,
            (
                push!(1).in_set(ChunkEnvironmentSet::Lod),
                push!(2).in_set(ChunkEnvironmentSet::Weather),
                push!(3).in_set(ChunkEnvironmentSet::Ecology),
                push!(4).in_set(ChunkEnvironmentSet::Fire),
                push!(5).in_set(AtmospherePipelineSet::FieldFill),
                push!(6).in_set(AtmospherePipelineSet::WindAdvect),
                push!(7).in_set(AtmospherePipelineSet::Emitters),
                push!(8).in_set(AtmospherePipelineSet::Particles),
                push!(9).in_set(AtmospherePipelineSet::Coupling),
                push!(10).in_set(AtmospherePipelineSet::VisualExtract),
                push!(11).in_set(AtmospherePipelineSet::RenderPrep),
                push!(12).in_set(AtmospherePipelineSet::Diagnostics),
            ),
        );
        app.update();
        assert_eq!(
            app.world().resource::<Order>().0,
            (1..=12).collect::<Vec<_>>()
        );
    }

    /// Mirrors `engine_with_worldgen`: atmosphere diagnostics before transport topology.
    #[test]
    fn atmosphere_diagnostics_runs_before_transport_topology_when_configured() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);
        app.init_resource::<InputBindings>();
        app.add_plugins(SimControlPlugin);
        crate::systems::chunk_environment_set::configure_chunk_environment_sets(&mut app);
        super::configure_atmosphere_pipeline_sets(&mut app);
        app.configure_sets(
            Update,
            AtmospherePipelineSet::Diagnostics.before(TransportSchedule::Topology),
        );
        app.init_resource::<Order>();
        app.add_systems(
            Update,
            (
                push!(1).in_set(AtmospherePipelineSet::Diagnostics),
                push!(2).in_set(TransportSchedule::Topology),
            ),
        );
        app.update();
        assert_eq!(app.world().resource::<Order>().0, vec![1, 2]);
    }
}
