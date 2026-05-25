//! FX-WATER-PARTICLE-001 — registry-backed water particle expand compute (fire spine clone).

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

use crate::gui::GPU_FIRE_INSTANCE_BUDGET_CEILING;
use crate::render::gpu_bind_group_registry::{
    BindGroupBufferBinding, GPUBindGroupRegistry, WORLD_WATER_PARTICLE_DRAW_BIND_GROUP,
    WORLD_WATER_PARTICLE_EXPANDED_BIND_GROUP,
};
use crate::render::gpu_buffer_registry::{
    GPUBufferRegistry, RegisteredBufferDescriptor, BufferVisibility,
    WATER_PARTICLE_EXPANDED_VERTICES_BUFFER, WATER_PARTICLE_INSTANCES_BUFFER,
};
use crate::render::gpu_packed_formats::{
    packed_byte_size, water_particle_expanded_vertex_format, water_particle_instance_format,
};
use crate::render::gpu_water_particles::{GpuWaterParticleQuadVertex, WorldWaterParticleFrame};

pub const WATER_PARTICLE_WGSL: &str = "shaders/water/water_particle.wgsl";

const PARTICLE_WORKGROUP: u32 = 64;

#[derive(Resource, Clone, ShaderType)]
pub struct WorldWaterParticleDrawUniforms {
    pub instance_count: u32,
    pub max_instances: u32,
    pub time_secs: f32,
    pub camera_zoom: f32,
    pub zoom_alpha: f32,
    pub _pad: f32,
}

impl Default for WorldWaterParticleDrawUniforms {
    fn default() -> Self {
        Self {
            instance_count: 0,
            max_instances: 0,
            time_secs: 0.0,
            camera_zoom: 1.0,
            zoom_alpha: 0.5,
            _pad: 0.0,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct WorldWaterParticleDrawDispatch {
    pub instance_count: u32,
    pub dispatch_count: u32,
}

#[derive(Resource)]
pub struct WorldWaterParticleDrawUniformGpu {
    pub uniform: UniformBuffer<WorldWaterParticleDrawUniforms>,
    pub bind_group: Option<BindGroup>,
}

impl Default for WorldWaterParticleDrawUniformGpu {
    fn default() -> Self {
        Self {
            uniform: UniformBuffer::from(WorldWaterParticleDrawUniforms::default()),
            bind_group: None,
        }
    }
}

#[derive(Resource)]
struct WorldWaterParticleDrawPipeline {
    layout: BindGroupLayoutDescriptor,
    instance_layout: BindGroupLayoutDescriptor,
    expanded_layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
}

#[derive(Resource, Default)]
pub struct WorldWaterParticleGpuStorage {
    pub instance_count: u32,
    pub expanded_vertex_count: u32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct WorldWaterParticleDrawLabel;

struct WorldWaterParticleDrawNode {
    ready: bool,
}

impl Default for WorldWaterParticleDrawNode {
    fn default() -> Self {
        Self { ready: false }
    }
}

pub fn register_world_water_particle_draw(app: &mut App) {
    super::gpu_water_particle_raster::register_world_water_particle_raster(app);
    app.init_resource::<WorldWaterParticleDrawDispatch>();

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app
        .init_resource::<WorldWaterParticleDrawUniforms>()
        .init_resource::<WorldWaterParticleDrawUniformGpu>()
        .init_resource::<WorldWaterParticleDrawDispatch>()
        .init_resource::<WorldWaterParticleGpuStorage>()
        .add_systems(RenderStartup, init_world_water_particle_draw_pipeline)
        .add_systems(
            Render,
            (
                prepare_world_water_particle_draw_uniforms,
                prepare_world_water_particle_gpu_storage.after(prepare_world_water_particle_draw_uniforms),
                prepare_world_water_particle_draw_bind_group
                    .after(prepare_world_water_particle_gpu_storage),
                record_world_water_particle_draw_dispatch
                    .after(prepare_world_water_particle_draw_bind_group),
            )
                .chain()
                .in_set(RenderSystems::PrepareBindGroups),
        );

    let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
    graph.add_node(WorldWaterParticleDrawLabel, WorldWaterParticleDrawNode::default());
    graph.add_node_edge(
        WorldWaterParticleDrawLabel,
        bevy::render::graph::CameraDriverLabel,
    );
}

fn init_world_water_particle_draw_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "WorldWaterParticleDrawUniforms",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (uniform_buffer::<WorldWaterParticleDrawUniforms>(false),),
        ),
    );
    let instance_layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("WorldWaterParticleInstances"),
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
        label: Cow::Borrowed("WorldWaterParticleExpandedVertices"),
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
    let shader = asset_server.load(WATER_PARTICLE_WGSL);
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
    commands.insert_resource(WorldWaterParticleDrawPipeline {
        layout,
        instance_layout,
        expanded_layout,
        pipeline,
    });
}

fn prepare_world_water_particle_draw_uniforms(
    extracted: Option<Res<WorldWaterParticleFrame>>,
    cam: Option<Res<crate::render::gpu_particles::FireParticleCameraScale>>,
    mut uniforms: ResMut<WorldWaterParticleDrawUniforms>,
    storage: Option<Res<WorldWaterParticleGpuStorage>>,
) {
    let count = storage.as_ref().map(|s| s.instance_count).unwrap_or(0);
    uniforms.instance_count = count;
    uniforms.max_instances = extracted
        .as_ref()
        .map(|f| f.instances.len().min(GPU_FIRE_INSTANCE_BUDGET_CEILING) as u32)
        .unwrap_or(0);
    uniforms.time_secs = extracted.as_ref().map(|f| f.anim_time_secs).unwrap_or(0.0);
    if let Some(c) = cam.as_deref() {
        uniforms.camera_zoom = c.camera_zoom;
        uniforms.zoom_alpha = c.zoom_alpha;
    }
}

fn prepare_world_water_particle_gpu_storage(
    extracted: Option<Res<WorldWaterParticleFrame>>,
    mut storage: ResMut<WorldWaterParticleGpuStorage>,
    mut registry: ResMut<GPUBufferRegistry>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    storage.instance_count = 0;
    storage.expanded_vertex_count = 0;
    let Some(frame) = extracted else {
        return;
    };
    if frame.instances.is_empty() {
        return;
    }
    let format = water_particle_instance_format();
    let active = frame.instances.len().min(GPU_FIRE_INSTANCE_BUDGET_CEILING);
    let _ = registry.upload_pod_slice(
        &render_device,
        &queue,
        RegisteredBufferDescriptor {
            id: format.buffer_id,
            size_bytes: packed_byte_size(format, active),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            visibility: BufferVisibility::RenderOnly,
            stride: format.stride,
        },
        active,
        &frame.instances[..active],
        frame.anim_time_secs.to_bits() as u64,
    );
    storage.instance_count = active as u32;
    storage.expanded_vertex_count = (active as u32).saturating_mul(4);
}

fn prepare_world_water_particle_draw_bind_group(
    pipeline: Res<WorldWaterParticleDrawPipeline>,
    mut registry: ResMut<GPUBufferRegistry>,
    mut bind_registry: ResMut<GPUBindGroupRegistry>,
    mut uniform_gpu: ResMut<WorldWaterParticleDrawUniformGpu>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
    uniforms: Res<WorldWaterParticleDrawUniforms>,
    storage: Option<Res<WorldWaterParticleGpuStorage>>,
) {
    let Some(storage) = storage else {
        return;
    };
    if storage.instance_count == 0 {
        bind_registry.invalidate(WORLD_WATER_PARTICLE_DRAW_BIND_GROUP);
        bind_registry.invalidate(WORLD_WATER_PARTICLE_EXPANDED_BIND_GROUP);
        return;
    }

    let buffer_id = WATER_PARTICLE_INSTANCES_BUFFER;
    let Some(instance_buf) = registry.get(buffer_id) else {
        return;
    };
    if !bind_registry.is_valid(WORLD_WATER_PARTICLE_DRAW_BIND_GROUP, &registry) {
        let layout = pipeline_cache.get_bind_group_layout(&pipeline.instance_layout);
        let bg = render_device.create_bind_group(
            None,
            &layout,
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
            WORLD_WATER_PARTICLE_DRAW_BIND_GROUP,
            bg,
            vec![BindGroupBufferBinding {
                buffer_id,
                buffer_version: instance_buf.version,
            }],
        );
    }

    let expanded_format = water_particle_expanded_vertex_format();
    let max_expand = (uniforms.max_instances as usize).min(GPU_FIRE_INSTANCE_BUDGET_CEILING);
    if max_expand > 0 {
        let expanded_rows = max_expand.saturating_mul(4).max(4);
        let _ = registry.upload_pod_slice(
            &render_device,
            &queue,
            RegisteredBufferDescriptor {
                id: expanded_format.buffer_id,
                size_bytes: packed_byte_size(expanded_format, expanded_rows),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                visibility: BufferVisibility::RenderOnly,
                stride: expanded_format.stride,
            },
            expanded_rows,
            &[] as &[GpuWaterParticleQuadVertex],
            max_expand as u64,
        );
        let Some(expanded_buf) = registry.get(WATER_PARTICLE_EXPANDED_VERTICES_BUFFER) else {
            return;
        };
        if !bind_registry.is_valid(WORLD_WATER_PARTICLE_EXPANDED_BIND_GROUP, &registry) {
            let layout = pipeline_cache.get_bind_group_layout(&pipeline.expanded_layout);
            let bg = render_device.create_bind_group(
                None,
                &layout,
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
                WORLD_WATER_PARTICLE_EXPANDED_BIND_GROUP,
                bg,
                vec![BindGroupBufferBinding {
                    buffer_id: WATER_PARTICLE_EXPANDED_VERTICES_BUFFER,
                    buffer_version: expanded_buf.version,
                }],
            );
        }
    }

    uniform_gpu.uniform.set((*uniforms).clone());
    uniform_gpu.uniform.write_buffer(&render_device, &queue);
    if uniform_gpu.bind_group.is_none() {
        let layout = pipeline_cache.get_bind_group_layout(&pipeline.layout);
        uniform_gpu.bind_group = Some(render_device.create_bind_group(
            None,
            &layout,
            &BindGroupEntries::single(&uniform_gpu.uniform),
        ));
    }
}

fn record_world_water_particle_draw_dispatch(
    storage: Option<Res<WorldWaterParticleGpuStorage>>,
    mut draw: ResMut<WorldWaterParticleDrawDispatch>,
) {
    let count = storage.as_ref().map(|s| s.instance_count).unwrap_or(0);
    draw.instance_count = count;
    draw.dispatch_count = if count > 0 {
        count.div_ceil(PARTICLE_WORKGROUP)
    } else {
        0
    };
}

impl render_graph::Node for WorldWaterParticleDrawNode {
    fn update(&mut self, world: &mut World) {
        if self.ready {
            return;
        }
        let pipeline = world.resource::<WorldWaterParticleDrawPipeline>();
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
        let draw = world.resource::<WorldWaterParticleDrawDispatch>();
        if draw.dispatch_count == 0 {
            return Ok(());
        }
        let Some(uniform_gpu) = world.get_resource::<WorldWaterParticleDrawUniformGpu>() else {
            return Ok(());
        };
        let Some(uniform_bg) = uniform_gpu.bind_group.as_ref() else {
            return Ok(());
        };
        let bind_registry = world.resource::<GPUBindGroupRegistry>();
        let Some(instance_entry) = bind_registry.get(WORLD_WATER_PARTICLE_DRAW_BIND_GROUP) else {
            return Ok(());
        };
        let Some(expanded_entry) = bind_registry.get(WORLD_WATER_PARTICLE_EXPANDED_BIND_GROUP) else {
            return Ok(());
        };
        let pipeline = world.resource::<WorldWaterParticleDrawPipeline>();
        let cache = world.resource::<PipelineCache>();
        let pl = cache
            .get_compute_pipeline(pipeline.pipeline)
            .expect("water particle expand pipeline");
        let mut pass = render_ctx
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_pipeline(pl);
        pass.set_bind_group(0, uniform_bg, &[]);
        pass.set_bind_group(1, &instance_entry.bind_group, &[]);
        pass.set_bind_group(2, &expanded_entry.bind_group, &[]);
        pass.dispatch_workgroups(draw.dispatch_count, 1, 1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn water_particle_wgsl_exists() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let expand = std::fs::read_to_string(root.join("assets/shaders/water/water_particle.wgsl"))
            .expect("water_particle.wgsl");
        let draw = std::fs::read_to_string(root.join("assets/shaders/water/water_particle_draw.wgsl"))
            .expect("water_particle_draw.wgsl");
        assert!(expand.contains("@compute"));
        assert!(expand.contains("expand_instances"));
        assert!(draw.contains("fs_main"));
    }
}
