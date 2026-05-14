//! **Phase F** — world-anchored GPU particle instances from post-LOD [`RenderProjectionGraph`].
//!
//! One upload path: `FireVisualFrame` → projection → [`WorldFireParticleFrame`] → registry buffer.

use bevy::math::Vec4;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bytemuck::{Pod, Zeroable};

use crate::gui::RepresentationBand;
use crate::render::extraction::RenderProjectionGraph;
use crate::render::gpu_buffer_registry::{BufferId, FIRE_PARTICLE_INSTANCES_BUFFER};
use crate::render::sim_visual_extract::FireVisualGpuInstance;

/// Presentation class for instanced quads (`WorldFireFx` vs macro garnish).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParticleClass {
    WorldFireFx,
    AtmosphereFx,
}

impl ParticleClass {
    #[inline]
    const fn as_f32(self) -> f32 {
        match self {
            Self::WorldFireFx => 0.0,
            Self::AtmosphereFx => 1.0,
        }
    }
}

/// Packed instanced-quad row for the GPU particle buffer.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct GpuParticleInstance {
    /// World `xyz`, `w` = heat `[0,1]`.
    pub world_xyz_heat: Vec4,
    /// `x` ember rate, `y` class ordinal, `z` influence radius, `w` smoke density.
    pub ember_class_radius_smoke: Vec4,
}

impl GpuParticleInstance {
    #[must_use]
    pub fn from_fire_visual(row: &FireVisualGpuInstance, class: ParticleClass) -> Self {
        let world = row.world_xyz_radius;
        Self {
            world_xyz_heat: Vec4::new(world.x, world.y, world.z, row.heat()),
            ember_class_radius_smoke: Vec4::new(
                row.smoke_ember_vis_priority.y,
                class.as_f32(),
                world.w,
                row.smoke_ember_vis_priority.x,
            ),
        }
    }
}

/// One expanded billboard vertex for instanced world-fire quads (`WorldFireFx`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct GpuParticleQuadVertex {
    /// World `xy`, `z` = heat, `w` = ember.
    pub world_xy_heat_ember: Vec4,
}

impl GpuParticleQuadVertex {
    #[must_use]
    pub fn from_corner(world_x: f32, world_y: f32, heat: f32, ember: f32) -> Self {
        Self {
            world_xy_heat_ember: Vec4::new(world_x, world_y, heat, ember),
        }
    }
}

/// CPU-side particle snapshot for one committed sim step (LOD-shaped).
#[derive(Resource, Debug, Clone, ExtractResource)]
pub struct WorldFireParticleFrame {
    pub snapshot_stamp: u64,
    pub active_band: RepresentationBand,
    pub gpu_capacity: usize,
    pub instances: Vec<GpuParticleInstance>,
}

impl Default for WorldFireParticleFrame {
    fn default() -> Self {
        Self {
            snapshot_stamp: 0,
            active_band: RepresentationBand::Full,
            gpu_capacity: usize::MAX,
            instances: Vec::new(),
        }
    }
}

/// Render-world view of the latest particle upload (count only).
#[derive(Resource, Default)]
pub struct WorldFireParticleGpuStorage {
    pub instance_count: u32,
    pub expanded_vertex_count: u32,
}

pub fn emit_world_fire_particles_from_projection(
    graph: Res<RenderProjectionGraph>,
    mut frame: ResMut<WorldFireParticleFrame>,
) {
    update_world_fire_particles_from_projection(graph.as_ref(), frame.as_mut());
}

pub fn update_world_fire_particles_from_projection(
    graph: &RenderProjectionGraph,
    frame: &mut WorldFireParticleFrame,
) {
    frame.snapshot_stamp = graph.fire.snapshot_stamp;
    frame.active_band = crate::gui::representation_band_from_world_lod(graph.fire.lod);
    frame.gpu_capacity = graph.fire.gpu_instance_capacity;
    frame.instances.clear();
    let capacity = graph.fire.gpu_instance_capacity;
    frame.instances.reserve(
        graph
            .fire
            .instance_buffer
            .len()
            .min(if capacity == usize::MAX {
                graph.fire.instance_buffer.len()
            } else {
                capacity
            }),
    );
    for row in &graph.fire.instance_buffer {
        if row.heat() < 0.05 && row.smoke_ember_vis_priority.y < 0.02 {
            continue;
        }
        frame.instances
            .push(GpuParticleInstance::from_fire_visual(row, ParticleClass::WorldFireFx));
        if capacity != usize::MAX && frame.instances.len() >= capacity {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{
        build_representation_inputs, build_representation_result, resolution_for_band,
        LodZoneRegistry, VisualBudgetSettings,
        VisualCadence, WorldLodBand, WorldLodBands, WorldLodMap, WorldRepresentationFrame,
    };
    use crate::render::extraction::{
        ProjectionNodeTrait, RenderProjectionContext, RenderProjectionGraph,
    };
    use crate::render::sim_visual_extract::{ChunkFireHeat, FireVisualFrame};
    use crate::render::{EcologyVisualSnapshot, LogisticsVisualSnapshot};
    use crate::systems::sim_control::SimStepStamp;
    use bevy::math::IVec2;

    fn sample_fire_row(chunk: IVec2, heat: f32, ember: f32) -> FireVisualGpuInstance {
        let mut row = FireVisualGpuInstance::default();
        row.chunk_xy_heat_lum = Vec4::new(chunk.x as f32, chunk.y as f32, heat, 1.0);
        row.world_xyz_radius = Vec4::new(chunk.x as f32 * 64.0, chunk.y as f32 * 64.0, 0.0, 32.0);
        row.smoke_ember_vis_priority = Vec4::new(0.1, ember, 0.0, 1.0);
        row
    }

    #[test]
    fn macro_band_drops_world_fire_particle_rows() {
        let mut fire = FireVisualFrame::default();
        fire.stamp = SimStepStamp::new(1, 0);
        fire.instances.push(sample_fire_row(IVec2::ZERO, 0.9, 0.5));
        fire.chunk_heat.push(ChunkFireHeat {
            chunk: IVec2::ZERO,
            heat: 0.9,
            smoke: 0.1,
        });

        let mut lod = WorldRepresentationFrame::default();
        lod.bands = WorldLodBands {
            global: WorldLodBand::Macro,
            ..Default::default()
        };
        lod.resolution = resolution_for_band(WorldLodBand::Macro);
        let lod_map = WorldLodMap::default();

        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            fire.stamp,
        );
        let policy = build_representation_result(&lod, &policy_inputs);

        let mut graph = RenderProjectionGraph::default();
        let logistics = LogisticsVisualSnapshot::default();
        let ecology = EcologyVisualSnapshot::default();
        let ctx = RenderProjectionContext {
            policy: &policy,
            lod: &lod,
            lod_map: &lod_map,
            fire: &fire,
            logistics: &logistics,
            ecology: &ecology,
            committed_stamp: fire.stamp,
        };
        graph.evaluate(&ctx);
        assert!(graph.fire.instance_buffer.is_empty());

        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(&graph, &mut particles);
        assert!(particles.instances.is_empty());
    }

    #[test]
    fn tactical_projection_feeds_world_fire_particle_rows() {
        let mut fire = FireVisualFrame::default();
        fire.instances.push(sample_fire_row(IVec2::new(2, 3), 0.8, 0.4));

        let mut lod = WorldRepresentationFrame::default();
        lod.bands.global = WorldLodBand::LocalTactical;
        lod.resolution = resolution_for_band(WorldLodBand::LocalTactical);
        let lod_map = WorldLodMap::default();

        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            fire.stamp,
        );
        let policy = build_representation_result(&lod, &policy_inputs);

        let mut graph = RenderProjectionGraph::default();
        let logistics = LogisticsVisualSnapshot::default();
        let ecology = EcologyVisualSnapshot::default();
        let ctx = RenderProjectionContext {
            policy: &policy,
            lod: &lod,
            lod_map: &lod_map,
            fire: &fire,
            logistics: &logistics,
            ecology: &ecology,
            committed_stamp: fire.stamp,
        };
        graph.evaluate(&ctx);
        graph.fire.snapshot_stamp = 7;
        graph.fire.lod = WorldLodBand::LocalTactical;
        graph.fire.gpu_instance_capacity = lod.resolution.fire_instance_cap;

        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(&graph, &mut particles);
        assert_eq!(particles.instances.len(), 1);
        assert_eq!(particles.snapshot_stamp, 7);
        assert_eq!(
            particles.instances[0].ember_class_radius_smoke.y,
            ParticleClass::WorldFireFx.as_f32()
        );
    }

    #[test]
    fn particle_rows_respect_gpu_capacity_ceiling() {
        let mut fire = FireVisualFrame::default();
        for i in 0..8 {
            fire.instances
                .push(sample_fire_row(IVec2::new(i, 0), 0.8, 0.4));
        }

        let mut lod = WorldRepresentationFrame::default();
        lod.bands.global = WorldLodBand::LocalTactical;
        lod.resolution = resolution_for_band(WorldLodBand::LocalTactical);
        let lod_map = WorldLodMap::default();

        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            fire.stamp,
        );
        let policy = build_representation_result(&lod, &policy_inputs);

        let mut graph = RenderProjectionGraph::default();
        let logistics = LogisticsVisualSnapshot::default();
        let ecology = EcologyVisualSnapshot::default();
        let ctx = RenderProjectionContext {
            policy: &policy,
            lod: &lod,
            lod_map: &lod_map,
            fire: &fire,
            logistics: &logistics,
            ecology: &ecology,
            committed_stamp: fire.stamp,
        };
        graph.evaluate(&ctx);
        graph.fire.snapshot_stamp = 3;
        graph.fire.lod = WorldLodBand::LocalTactical;
        graph.fire.gpu_instance_capacity = 3;

        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(&graph, &mut particles);
        assert_eq!(particles.instances.len(), 3);
        assert_eq!(particles.gpu_capacity, 3);
    }

    #[test]
    fn particle_buffer_id_is_stable() {
        assert_eq!(FIRE_PARTICLE_INSTANCES_BUFFER, BufferId(3));
    }
}
