//! Phase F — registry-backed world fire particle instancing (compute pass + metrics).

use std::borrow::Cow;

use bevy::prelude::*;
use bevy::render::{
    render_graph::{self, RenderGraph, RenderLabel},
    render_resource::{
        binding_types::uniform_buffer,
        *,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue},
    Render, RenderApp, RenderStartup, RenderSystems,
};

use crate::gui::RepresentationResult;
use crate::render::fire_smoke_shader_handles::FIRE_PARTICLE_WGSL;
use crate::render::gpu_bind_group_registry::{
    BindGroupBufferBinding, GPUBindGroupRegistry, WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP,
    WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP,
};
use crate::render::gpu_buffer_registry::{
    BufferId, FIRE_PARTICLE_EXPANDED_VERTICES_BUFFER, FIRE_PARTICLE_INSTANCES_BUFFER,
    GPUBufferRegistry, RegisteredBufferDescriptor, BufferVisibility,
};
use crate::render::gpu_packed_formats::{
    fire_particle_expanded_vertex_format, packed_byte_size,
};
use crate::render::gpu_particles::{GpuParticleQuadVertex, WorldFireParticleGpuStorage};
use crate::render::gpu_representation_metrics::GpuRepresentationMetrics;

const PARTICLE_WORKGROUP: u32 = 64;

#[derive(Resource, Clone, ShaderType)]
pub struct WorldFireParticleDrawUniforms {
    pub instance_count: u32,
    pub max_instances: u32,
    pub time_secs: f32,
    pub _pad: f32,
}

impl Default for WorldFireParticleDrawUniforms {
    fn default() -> Self {
        Self {
            instance_count: 0,
            max_instances: 0,
            time_secs: 0.0,
            _pad: 0.0,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct WorldFireParticleDrawDispatch {
    pub instance_count: u32,
    pub dispatch_count: u32,
}

#[derive(Resource)]
struct WorldFireParticleDrawPipeline {
    layout: BindGroupLayoutDescriptor,
    instance_layout: BindGroupLayoutDescriptor,
    expanded_layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
}

#[derive(Resource)]
struct WorldFireParticleDrawBindGroup(BindGroup);

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct WorldFireParticleDrawLabel;

struct WorldFireParticleDrawNode {
    ready: bool,
}

impl Default for WorldFireParticleDrawNode {
    fn default() -> Self {
        Self { ready: false }
    }
}

pub fn register_world_fire_particle_draw(app: &mut App) {
    app.init_resource::<WorldFireParticleDrawDispatch>();

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app
        .init_resource::<WorldFireParticleDrawUniforms>()
        .init_resource::<WorldFireParticleDrawDispatch>()
        .add_systems(RenderStartup, init_world_fire_particle_draw_pipeline)
        .add_systems(
            Render,
            (
                prepare_world_fire_particle_draw_uniforms,
                prepare_world_fire_particle_draw_bind_group
                    .after(prepare_world_fire_particle_draw_uniforms),
                record_world_fire_particle_draw_dispatch
                    .after(prepare_world_fire_particle_draw_bind_group),
            )
                .chain()
                .in_set(RenderSystems::PrepareBindGroups),
        );

    let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
    graph.add_node(WorldFireParticleDrawLabel, WorldFireParticleDrawNode::default());
    graph.add_node_edge(WorldFireParticleDrawLabel, bevy::render::graph::CameraDriverLabel);
}

fn init_world_fire_particle_draw_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "WorldFireParticleDrawUniforms",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (uniform_buffer::<WorldFireParticleDrawUniforms>(false),),
        ),
    );
    let instance_layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("WorldFireParticleInstances"),
        entries: vec![BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    };
    let expanded_layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("WorldFireParticleExpandedVertices"),
        entries: vec![BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    };
    let shader = asset_server.load(FIRE_PARTICLE_WGSL);
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![
            layout.clone(),
            instance_layout.clone(),
            expanded_layout.clone(),
        ],
        shader,
        entry_point: Some(Cow::from("expand_instances")),
        ..default()
    });
    commands.insert_resource(WorldFireParticleDrawPipeline {
        layout,
        instance_layout,
        expanded_layout,
        pipeline,
    });
}

fn prepare_world_fire_particle_draw_uniforms(
    storage: Option<Res<WorldFireParticleGpuStorage>>,
    extracted: Option<Res<crate::render::gpu_particles::WorldFireParticleFrame>>,
    mut uniforms: ResMut<WorldFireParticleDrawUniforms>,
) {
    let count = storage.as_ref().map(|s| s.instance_count).unwrap_or(0);
    let max_instances = extracted
        .as_ref()
        .map(|f| f.gpu_capacity.min(u32::MAX as usize) as u32)
        .unwrap_or(0);
    uniforms.instance_count = count;
    uniforms.max_instances = max_instances;
    uniforms.time_secs = extracted
        .as_ref()
        .map(|f| f.snapshot_stamp as f32 * 0.001)
        .unwrap_or(0.0);
}

fn prepare_world_fire_particle_draw_bind_group(
    mut commands: Commands,
    pipeline: Res<WorldFireParticleDrawPipeline>,
    mut registry: ResMut<GPUBufferRegistry>,
    mut bind_registry: ResMut<GPUBindGroupRegistry>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
    uniforms: Res<WorldFireParticleDrawUniforms>,
    storage: Option<Res<WorldFireParticleGpuStorage>>,
) {
    let Some(storage) = storage else {
        return;
    };
    if storage.instance_count == 0 {
        bind_registry.invalidate(WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP);
        bind_registry.invalidate(WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP);
        return;
    }

    let buffer_id = FIRE_PARTICLE_INSTANCES_BUFFER;
    let Some(instance_buf) = registry.get(buffer_id) else {
        return;
    };
    if !bind_registry.is_valid(WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP, &registry) {
        let instance_layout = pipeline_cache.get_bind_group_layout(&pipeline.instance_layout);
        let instance_bg = render_device.create_bind_group(
            None,
            &instance_layout,
            &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &instance_buf.buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        );
        bind_registry.insert(
            WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP,
            instance_bg,
            vec![BindGroupBufferBinding {
                buffer_id,
                buffer_version: instance_buf.version,
            }],
        );
    }

    let expanded_format = fire_particle_expanded_vertex_format();
    let expanded_rows = uniforms
        .max_instances
        .saturating_mul(4)
        .max(1) as usize;
    let expanded_needed = packed_byte_size(expanded_format, expanded_rows);
    let _ = registry.upload_pod_slice(
        &render_device,
        &queue,
        RegisteredBufferDescriptor {
            id: expanded_format.buffer_id,
            size_bytes: expanded_needed,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            visibility: BufferVisibility::RenderOnly,
            stride: expanded_format.stride,
        },
        expanded_rows,
        &[] as &[GpuParticleQuadVertex],
        uniforms.max_instances as u64,
    );
    let expanded_id = FIRE_PARTICLE_EXPANDED_VERTICES_BUFFER;
    let Some(expanded_buf) = registry.get(expanded_id) else {
        return;
    };
    if !bind_registry.is_valid(WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP, &registry) {
        let expanded_layout = pipeline_cache.get_bind_group_layout(&pipeline.expanded_layout);
        let expanded_bg = render_device.create_bind_group(
            None,
            &expanded_layout,
            &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &expanded_buf.buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        );
        bind_registry.insert(
            WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP,
            expanded_bg,
            vec![BindGroupBufferBinding {
                buffer_id: expanded_id,
                buffer_version: expanded_buf.version,
            }],
        );
    }

    let mut ub = UniformBuffer::from(uniforms.clone());
    ub.write_buffer(&render_device, &queue);
    let uniform_layout = pipeline_cache.get_bind_group_layout(&pipeline.layout);
    let uniform_bg = render_device.create_bind_group(
        None,
        &uniform_layout,
        &BindGroupEntries::single(&ub),
    );
    commands.insert_resource(WorldFireParticleDrawBindGroup(uniform_bg));
}

fn record_world_fire_particle_draw_dispatch(
    storage: Option<Res<WorldFireParticleGpuStorage>>,
    extracted: Option<Res<crate::render::gpu_particles::WorldFireParticleFrame>>,
    mut draw: ResMut<WorldFireParticleDrawDispatch>,
    mut metrics: ResMut<GpuRepresentationMetrics>,
) {
    let count = storage.as_ref().map(|s| s.instance_count).unwrap_or(0);
    let capped = extracted
        .as_ref()
        .map(|f| {
            if f.gpu_capacity == 0 {
                0
            } else {
                count
            }
        })
        .unwrap_or(count);
    draw.instance_count = capped;
    draw.dispatch_count = if capped > 0 {
        capped.div_ceil(PARTICLE_WORKGROUP)
    } else {
        0
    };
    metrics.record_draw_instances(capped);
    metrics.record_dispatch_count(draw.dispatch_count);
}

impl render_graph::Node for WorldFireParticleDrawNode {
    fn update(&mut self, world: &mut World) {
        if self.ready {
            return;
        }
        let pipeline = world.resource::<WorldFireParticleDrawPipeline>();
        let cache = world.resource::<PipelineCache>();
        if matches!(
            cache.get_compute_pipeline_state(pipeline.pipeline),
            CachedPipelineState::Ok(_)
        ) {
            self.ready = true;
        }
    }

    fn run(
        &self,
        _ctx: &mut render_graph::RenderGraphContext,
        render_ctx: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if !self.ready {
            return Ok(());
        }
        let draw = world.resource::<WorldFireParticleDrawDispatch>();
        if draw.dispatch_count == 0 {
            return Ok(());
        }
        let Some(uniform_bg) = world.get_resource::<WorldFireParticleDrawBindGroup>() else {
            return Ok(());
        };
        let bind_registry = world.resource::<GPUBindGroupRegistry>();
        let Some(instance_entry) = bind_registry.get(WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP) else {
            return Ok(());
        };
        let Some(expanded_entry) = bind_registry.get(WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP) else {
            return Ok(());
        };
        let pipeline = world.resource::<WorldFireParticleDrawPipeline>();
        let cache = world.resource::<PipelineCache>();
        let pl = cache
            .get_compute_pipeline(pipeline.pipeline)
            .expect("world fire particle draw pipeline must be ready");
        let mut pass = render_ctx
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_pipeline(pl);
        pass.set_bind_group(0, &uniform_bg.0, &[]);
        pass.set_bind_group(1, &instance_entry.bind_group, &[]);
        pass.set_bind_group(2, &expanded_entry.bind_group, &[]);
        pass.dispatch_workgroups(draw.dispatch_count, 1, 1);
        Ok(())
    }
}

pub fn sync_particle_draw_dispatch_from_policy(
    policy: Res<RepresentationResult>,
    mut draw: ResMut<WorldFireParticleDrawDispatch>,
    mut metrics: ResMut<GpuRepresentationMetrics>,
) {
    if policy.gpu_budget.particle_rows_cap == 0 || !policy.particle_policy.instanced_draw {
        draw.dispatch_count = 0;
        draw.instance_count = 0;
        metrics.record_draw_instances(0);
        metrics.record_dispatch_count(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{GpuBudgetPolicy, RepresentationBand, RepresentationResult};
    use crate::render::gpu_bind_group_registry::BindGroupId;

    #[test]
    fn zero_particle_cap_zeroes_draw_dispatch() {
        let mut app = App::new();
        app.init_resource::<WorldFireParticleDrawDispatch>();
        app.init_resource::<GpuRepresentationMetrics>();
        app.insert_resource(RepresentationResult {
            active_band: RepresentationBand::Strategic,
            gpu_budget: GpuBudgetPolicy {
                particle_rows_cap: 0,
                fire_instance_cap: 0,
                reserved_capacity: 0,
                active_capacity: 0,
            },
            ..Default::default()
        });
        app.add_systems(Update, sync_particle_draw_dispatch_from_policy);
        app.update();
        let draw = app.world().resource::<WorldFireParticleDrawDispatch>();
        assert_eq!(draw.dispatch_count, 0);
        assert_eq!(draw.instance_count, 0);
    }

    #[test]
    fn draw_dispatch_scales_with_instance_count() {
        let mut draw = WorldFireParticleDrawDispatch::default();
        draw.instance_count = 130;
        draw.dispatch_count = draw.instance_count.div_ceil(PARTICLE_WORKGROUP);
        assert_eq!(draw.dispatch_count, 3);
    }

    #[test]
    fn particle_draw_bind_group_id_is_stable() {
        assert_eq!(WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP, BindGroupId(2));
        assert_eq!(WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP, BindGroupId(3));
        assert_eq!(FIRE_PARTICLE_INSTANCES_BUFFER, BufferId(3));
        assert_eq!(FIRE_PARTICLE_EXPANDED_VERTICES_BUFFER, BufferId(6));
    }
}
