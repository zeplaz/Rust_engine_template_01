//! FX-FIRE-SPARK-002 — GPU spark advection (legacy `compute_expanse` port).
//!
//! Attractors deduped from existing [`WorldFireParticleFrame`] rows — **no second fire extract**.

use std::borrow::Cow;
use std::collections::HashMap;

use bevy::math::Vec4;
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use bevy::render::{
    render_graph::{self, RenderGraph, RenderLabel},
    render_resource::{
        binding_types::uniform_buffer,
        *,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue},
    Render, RenderApp, RenderStartup, RenderSystems,
};

use bytemuck::{Pod, Zeroable};

use crate::render::fire_smoke_shader_handles::FIRE_SPARK_COMPUTE_WGSL;
use crate::render::gpu_weather_fire_field::prepare_fire_particle_gpu_storage;
use crate::render::gpu_bind_group_registry::{
    BindGroupBufferBinding, FIRE_SPARK_ATTRACTORS_BIND_GROUP, FIRE_SPARK_INSTANCES_BIND_GROUP,
    FIRE_SPARK_STATE_BIND_GROUP, GPUBindGroupRegistry,
};
use crate::render::gpu_buffer_registry::{
    BufferVisibility, FIRE_PARTICLE_INSTANCES_BUFFER, FIRE_SPARK_ATTRACTORS_BUFFER,
    FIRE_SPARK_STATE_BUFFER, GPUBufferRegistry, RegisteredBufferDescriptor,
};
use crate::render::gpu_particles::{fire_spark_compute_enabled, GpuParticleInstance, WorldFireParticleFrame, WorldFireParticleGpuStorage};

const SPARK_WORKGROUP: u32 = 64;
const MAX_ATTRACTORS: usize = 24;

/// Per-particle simulation state (WGSL `SparkSimState`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct SparkSimState {
    pub pos: Vec4,
    pub vel: Vec4,
}

#[derive(Resource, Clone, Copy, Default, ShaderType)]
pub struct FireSparkComputeUniforms {
    pub delta_time: f32,
    pub instance_count: u32,
    pub attractor_count: u32,
    pub lifetime_decay: f32,
    pub respawn_life: f32,
    pub _pad: f32,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct FireSparkAttractors {
    pub rows: Vec<Vec4>,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct WorldFireSparkComputeDispatch {
    pub dispatch_count: u32,
}

#[derive(Resource, Default)]
pub(crate) struct FireSparkComputeUniformGpu {
    uniform: UniformBuffer<FireSparkComputeUniforms>,
    bind_group: Option<BindGroup>,
}

impl FireSparkComputeUniformGpu {
    fn new() -> Self {
        Self {
            uniform: UniformBuffer::from(FireSparkComputeUniforms::default()),
            bind_group: None,
        }
    }
}

#[derive(Resource)]
pub(crate) struct FireSparkComputePipeline {
    layout: BindGroupLayoutDescriptor,
    instances_layout: BindGroupLayoutDescriptor,
    state_layout: BindGroupLayoutDescriptor,
    attractors_layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct FireSparkComputeLabel;

struct FireSparkComputeNode {
    ready: bool,
}

impl Default for FireSparkComputeNode {
    fn default() -> Self {
        Self { ready: false }
    }
}

#[must_use]
pub fn build_fire_spark_attractors(instances: &[GpuParticleInstance]) -> FireSparkAttractors {
    let mut by_cell: HashMap<(i32, i32), (Vec3, f32)> = HashMap::new();
    for row in instances {
        let heat = row.world_xyz_heat.w;
        if heat < 0.05 {
            continue;
        }
        let p = row.world_xyz_heat.truncate();
        let cell = ((p.x * 0.25).floor() as i32, (p.y * 0.25).floor() as i32);
        by_cell
            .entry(cell)
            .and_modify(|(_, h)| {
                if heat > *h {
                    *h = heat;
                }
            })
            .or_insert((p, heat));
    }
    let mut ranked: Vec<(Vec3, f32)> = by_cell.into_values().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(MAX_ATTRACTORS);
    FireSparkAttractors {
        rows: ranked
            .into_iter()
            .map(|(p, heat)| Vec4::new(p.x, p.y, p.z, heat.max(0.1)))
            .collect(),
    }
}

fn init_spark_states(instances: &[GpuParticleInstance]) -> Vec<SparkSimState> {
    instances
        .iter()
        .map(|row| {
            let origin = row.world_xyz_heat;
            SparkSimState {
                pos: Vec4::new(origin.x, origin.y, origin.z, 4.2),
                vel: Vec4::new(0.0, 12.0, 0.0, 0.0),
            }
        })
        .collect()
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct FireSparkComputePrepareSet;

pub fn register_fire_spark_compute(app: &mut App) {
    app.init_resource::<FireSparkAttractors>()
        .init_resource::<WorldFireSparkComputeDispatch>();

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app
        .init_resource::<FireSparkAttractors>()
        .insert_resource(FireSparkComputeUniformGpu::new())
        .init_resource::<WorldFireSparkComputeDispatch>()
        .add_systems(RenderStartup, init_fire_spark_compute_pipeline)
        .add_systems(
            Render,
            (
                prepare_fire_spark_attractors,
                prepare_fire_spark_sim_buffers.after(prepare_fire_particle_gpu_storage),
                prepare_fire_spark_compute_bind_groups.after(prepare_fire_spark_sim_buffers),
                record_fire_spark_compute_dispatch.after(prepare_fire_spark_compute_bind_groups),
            )
                .chain()
                .in_set(RenderSystems::PrepareBindGroups)
                .in_set(FireSparkComputePrepareSet),
        );

    let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
    graph.add_node(FireSparkComputeLabel, FireSparkComputeNode::default());
}

/// Wire spark advection before expand once both render-graph nodes exist.
pub(crate) fn link_spark_compute_before_particle_expand(graph: &mut RenderGraph) {
    graph.add_node_edge(
        FireSparkComputeLabel,
        super::gpu_particle_draw::WorldFireParticleDrawLabel,
    );
}

fn init_fire_spark_compute_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "FireSparkComputeUniforms",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (uniform_buffer::<FireSparkComputeUniforms>(false),),
        ),
    );
    let instances_layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("FireSparkInstances"),
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
    let state_layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("FireSparkSimState"),
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
    let attractors_layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("FireSparkAttractors"),
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
    let shader = asset_server.load(FIRE_SPARK_COMPUTE_WGSL);
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![
            layout.clone(),
            instances_layout.clone(),
            state_layout.clone(),
            attractors_layout.clone(),
        ],
        shader,
        entry_point: Some(Cow::from("advect_sparks")),
        ..default()
    });
    commands.insert_resource(FireSparkComputePipeline {
        layout,
        instances_layout,
        state_layout,
        attractors_layout,
        pipeline,
    });
}

fn prepare_fire_spark_attractors(extracted: Option<Res<WorldFireParticleFrame>>, mut out: ResMut<FireSparkAttractors>) {
    out.rows.clear();
    if !fire_spark_compute_enabled() {
        return;
    }
    let Some(frame) = extracted else {
        return;
    };
    *out = build_fire_spark_attractors(&frame.instances);
}

pub(crate) fn prepare_fire_spark_sim_buffers(
    extracted: Option<Res<WorldFireParticleFrame>>,
    storage: Option<Res<WorldFireParticleGpuStorage>>,
    mut registry: ResMut<GPUBufferRegistry>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut last_stamp: Local<u64>,
) {
    let (Some(frame), Some(storage)) = (extracted, storage) else {
        return;
    };
    if storage.instance_count == 0 {
        return;
    }
    let active = storage.instance_count as usize;
    let rows = &frame.instances[..active.min(frame.instances.len())];
    let stamp_changed = frame.snapshot_stamp != *last_stamp;
  let missing = registry.get(FIRE_SPARK_STATE_BUFFER).is_none();
    if !stamp_changed && !missing {
        return;
    }
    *last_stamp = frame.snapshot_stamp;
    let spark_rows = init_spark_states(rows);
    let stride = std::mem::size_of::<SparkSimState>() as u32;
    let _ = registry.upload_pod_slice(
        &render_device,
        &queue,
        RegisteredBufferDescriptor {
            id: FIRE_SPARK_STATE_BUFFER,
            size_bytes: (rows.len().max(1) * stride as usize) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            visibility: BufferVisibility::RenderAndCompute,
            stride,
        },
        rows.len().max(1),
        &spark_rows,
        frame.snapshot_stamp,
    );
}

fn prepare_fire_spark_compute_bind_groups(
    pipeline: Res<FireSparkComputePipeline>,
    mut uniform_gpu: ResMut<FireSparkComputeUniformGpu>,
    mut bind_registry: ResMut<GPUBindGroupRegistry>,
    attractors: Res<FireSparkAttractors>,
    storage: Option<Res<WorldFireParticleGpuStorage>>,
    extracted: Option<Res<WorldFireParticleFrame>>,
    mut registry: ResMut<GPUBufferRegistry>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    mut last_anim: Local<f32>,
) {
    if !fire_spark_compute_enabled() {
        uniform_gpu.bind_group = None;
        return;
    }
    let count = storage.as_ref().map(|s| s.instance_count).unwrap_or(0);
    if count == 0 {
        uniform_gpu.bind_group = None;
        bind_registry.invalidate(FIRE_SPARK_INSTANCES_BIND_GROUP);
        bind_registry.invalidate(FIRE_SPARK_STATE_BIND_GROUP);
        bind_registry.invalidate(FIRE_SPARK_ATTRACTORS_BIND_GROUP);
        return;
    }

    let att_rows = attractors.rows.len().max(1);
    let att_upload: Vec<Vec4> = if attractors.rows.is_empty() {
        vec![Vec4::ZERO; att_rows]
    } else {
        attractors.rows.clone()
    };
    let _ = registry.upload_pod_slice(
        &render_device,
        &queue,
        RegisteredBufferDescriptor {
            id: FIRE_SPARK_ATTRACTORS_BUFFER,
            size_bytes: (att_rows * std::mem::size_of::<Vec4>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            visibility: BufferVisibility::RenderAndCompute,
            stride: std::mem::size_of::<Vec4>() as u32,
        },
        att_rows,
        &att_upload,
        extracted.as_ref().map(|f| f.snapshot_stamp).unwrap_or(0),
    );

    let anim = extracted.as_ref().map(|f| f.anim_time_secs).unwrap_or(0.0);
    let dt = (anim - *last_anim).max(0.0).min(0.05);
    *last_anim = anim;

    let uniforms = FireSparkComputeUniforms {
        delta_time: if dt > 0.0 { dt } else { 1.0 / 60.0 },
        instance_count: count,
        attractor_count: attractors.rows.len().min(MAX_ATTRACTORS) as u32,
        lifetime_decay: 0.00058,
        respawn_life: 5.8,
        _pad: 0.0,
    };
    uniform_gpu.uniform.set(uniforms);
    uniform_gpu.uniform.write_buffer(&render_device, &queue);

    let instance_buf = registry.get(FIRE_PARTICLE_INSTANCES_BUFFER);
    let spark_buf = registry.get(FIRE_SPARK_STATE_BUFFER);
    let att_buf = registry.get(FIRE_SPARK_ATTRACTORS_BUFFER);
    let (Some(instance_buf), Some(spark_buf), Some(att_buf)) = (instance_buf, spark_buf, att_buf) else {
        uniform_gpu.bind_group = None;
        return;
    };

    if !bind_registry.is_valid(FIRE_SPARK_INSTANCES_BIND_GROUP, &registry) {
        let layout = pipeline_cache.get_bind_group_layout(&pipeline.instances_layout);
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
            FIRE_SPARK_INSTANCES_BIND_GROUP,
            bg,
            vec![BindGroupBufferBinding {
                buffer_id: FIRE_PARTICLE_INSTANCES_BUFFER,
                buffer_version: instance_buf.version,
            }],
        );
    }
    if !bind_registry.is_valid(FIRE_SPARK_STATE_BIND_GROUP, &registry) {
        let layout = pipeline_cache.get_bind_group_layout(&pipeline.state_layout);
        let bg = render_device.create_bind_group(
            None,
            &layout,
            &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &spark_buf.buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        );
        bind_registry.insert(
            FIRE_SPARK_STATE_BIND_GROUP,
            bg,
            vec![BindGroupBufferBinding {
                buffer_id: FIRE_SPARK_STATE_BUFFER,
                buffer_version: spark_buf.version,
            }],
        );
    }
    if !bind_registry.is_valid(FIRE_SPARK_ATTRACTORS_BIND_GROUP, &registry) {
        let layout = pipeline_cache.get_bind_group_layout(&pipeline.attractors_layout);
        let bg = render_device.create_bind_group(
            None,
            &layout,
            &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &att_buf.buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        );
        bind_registry.insert(
            FIRE_SPARK_ATTRACTORS_BIND_GROUP,
            bg,
            vec![BindGroupBufferBinding {
                buffer_id: FIRE_SPARK_ATTRACTORS_BUFFER,
                buffer_version: att_buf.version,
            }],
        );
    }

    if uniform_gpu.bind_group.is_none() {
        let uniform_layout = pipeline_cache.get_bind_group_layout(&pipeline.layout);
        uniform_gpu.bind_group = Some(render_device.create_bind_group(
            None,
            &uniform_layout,
            &BindGroupEntries::single(&uniform_gpu.uniform),
        ));
    }
}

fn record_fire_spark_compute_dispatch(
    storage: Option<Res<WorldFireParticleGpuStorage>>,
    mut dispatch: ResMut<WorldFireSparkComputeDispatch>,
) {
    if !fire_spark_compute_enabled() {
        dispatch.dispatch_count = 0;
        return;
    }
    let count = storage.as_ref().map(|s| s.instance_count).unwrap_or(0);
    dispatch.dispatch_count = if count > 0 {
        count.div_ceil(SPARK_WORKGROUP)
    } else {
        0
    };
}

impl render_graph::Node for FireSparkComputeNode {
    fn update(&mut self, world: &mut World) {
        if self.ready {
            return;
        }
        let pipeline = world.resource::<FireSparkComputePipeline>();
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
        if !self.ready || !fire_spark_compute_enabled() {
            return Ok(());
        }
        let dispatch = world.resource::<WorldFireSparkComputeDispatch>();
        if dispatch.dispatch_count == 0 {
            return Ok(());
        }
        let Some(uniform_gpu) = world.get_resource::<FireSparkComputeUniformGpu>() else {
            return Ok(());
        };
        let Some(uniform_bg) = uniform_gpu.bind_group.as_ref() else {
            return Ok(());
        };
        let bind_registry = world.resource::<GPUBindGroupRegistry>();
        let Some(instance_entry) = bind_registry.get(FIRE_SPARK_INSTANCES_BIND_GROUP) else {
            return Ok(());
        };
        let Some(spark_entry) = bind_registry.get(FIRE_SPARK_STATE_BIND_GROUP) else {
            return Ok(());
        };
        let Some(att_entry) = bind_registry.get(FIRE_SPARK_ATTRACTORS_BIND_GROUP) else {
            return Ok(());
        };
        let pipeline = world.resource::<FireSparkComputePipeline>();
        let cache = world.resource::<PipelineCache>();
        let pl = cache
            .get_compute_pipeline(pipeline.pipeline)
            .expect("fire spark compute pipeline");
        let mut pass = render_ctx
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_pipeline(pl);
        pass.set_bind_group(0, uniform_bg, &[]);
        pass.set_bind_group(1, &instance_entry.bind_group, &[]);
        pass.set_bind_group(2, &spark_entry.bind_group, &[]);
        pass.set_bind_group(3, &att_entry.bind_group, &[]);
        pass.dispatch_workgroups(dispatch.dispatch_count, 1, 1);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attractor_builder_caps_at_24_and_ranks_by_heat() {
        let mut instances = Vec::new();
        for i in 0..40 {
            instances.push(GpuParticleInstance {
                world_xyz_heat: Vec4::new(i as f32, 0.0, 0.0, i as f32 / 40.0),
                ..Default::default()
            });
        }
        let att = build_fire_spark_attractors(&instances);
        assert!(att.rows.len() <= MAX_ATTRACTORS);
        assert!(!att.rows.is_empty());
        let top = att.rows[0].w;
        assert!(top >= 0.9, "expected hottest attractor first, got {top}");
    }

    #[test]
    fn spark_compute_wgsl_exists_and_advects() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let src = std::fs::read_to_string(root.join("assets/shaders/fire/fire_spark_compute.wgsl"))
            .expect("fire_spark_compute.wgsl");
        assert!(src.contains("advect_sparks"));
        assert!(src.contains("MAX_ATTRACTORS"));
    }
}
