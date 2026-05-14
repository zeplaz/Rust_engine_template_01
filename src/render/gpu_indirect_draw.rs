//! GPU-driven draw spine — compact post-LOD particle rows into indirect draw args.

use bevy::prelude::*;

use crate::gui::RepresentationResult;
use crate::render::gpu_particle_draw::WorldFireParticleDrawDispatch;
use crate::render::gpu_particles::WorldFireParticleFrame;

/// Expanded billboard vertices per instanced fire particle row.
pub const WORLD_FIRE_VERTICES_PER_INSTANCE: u32 = 4;

/// Indirect draw arguments for world-fire instancing (future multi-draw indirect).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorldFireIndirectDrawArgs {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

/// Render-world bookkeeping for GPU cull → compact → indirect dispatch.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct GpuIndirectDrawSpine {
    pub world_fire: WorldFireIndirectDrawArgs,
    pub dispatch_count: u32,
}

#[must_use]
pub fn compact_world_fire_indirect_draw(
    policy: &RepresentationResult,
    particles: &WorldFireParticleFrame,
    draw: &WorldFireParticleDrawDispatch,
) -> GpuIndirectDrawSpine {
    let capped = if policy.gpu_budget.particle_rows_cap == 0 || !policy.particle_policy.instanced_draw {
        0
    } else {
        draw.instance_count
            .min(particles.instances.len() as u32)
            .min(policy.gpu_budget.particle_rows_cap as u32)
    };
    let dispatch_count = if capped > 0 {
        capped.div_ceil(64)
    } else {
        0
    };
    GpuIndirectDrawSpine {
        world_fire: WorldFireIndirectDrawArgs {
            vertex_count: WORLD_FIRE_VERTICES_PER_INSTANCE,
            instance_count: capped,
            first_vertex: 0,
            first_instance: 0,
        },
        dispatch_count,
    }
}

pub fn sync_world_fire_indirect_draw(
    policy: Res<RepresentationResult>,
    particles: Res<WorldFireParticleFrame>,
    draw: Res<WorldFireParticleDrawDispatch>,
    mut spine: ResMut<GpuIndirectDrawSpine>,
) {
    *spine = compact_world_fire_indirect_draw(policy.as_ref(), particles.as_ref(), draw.as_ref());
}

pub struct GpuIndirectDrawSpinePlugin;

impl Plugin for GpuIndirectDrawSpinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpuIndirectDrawSpine>().add_systems(
            Update,
            sync_world_fire_indirect_draw
                .after(crate::render::sync_particle_draw_dispatch_from_policy)
                .after(crate::render::extraction::FireVisualFrameSet::ProjectGpu),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{GpuBudgetPolicy, RepresentationBand, RepresentationResult};

    #[test]
    fn zero_particle_cap_zeroes_indirect_args() {
        let mut policy = RepresentationResult::default();
        policy.active_band = RepresentationBand::Strategic;
        policy.gpu_budget = GpuBudgetPolicy {
            particle_rows_cap: 0,
            fire_instance_cap: 0,
            reserved_capacity: 0,
            active_capacity: 0,
        };
        policy.particle_policy.instanced_draw = false;
        policy.particle_policy.rows_cap = 0;
        let particles = WorldFireParticleFrame::default();
        let draw = WorldFireParticleDrawDispatch {
            instance_count: 8,
            dispatch_count: 1,
        };
        let spine = compact_world_fire_indirect_draw(&policy, &particles, &draw);
        assert_eq!(spine.world_fire.instance_count, 0);
        assert_eq!(spine.dispatch_count, 0);
    }

    #[test]
    fn indirect_args_cap_rows_and_emit_compute_dispatch_groups() {
        let mut policy = RepresentationResult::default();
        policy.gpu_budget = GpuBudgetPolicy {
            particle_rows_cap: 3,
            fire_instance_cap: 3,
            reserved_capacity: 3,
            active_capacity: 3,
        };
        let mut particles = WorldFireParticleFrame::default();
        particles.instances.resize(8, Default::default());
        let draw = WorldFireParticleDrawDispatch {
            instance_count: 8,
            dispatch_count: 1,
        };
        let spine = compact_world_fire_indirect_draw(&policy, &particles, &draw);
        assert_eq!(spine.world_fire.instance_count, 3);
        assert_eq!(spine.dispatch_count, 1);
        assert_eq!(spine.world_fire.vertex_count, WORLD_FIRE_VERTICES_PER_INSTANCE);
    }
}
