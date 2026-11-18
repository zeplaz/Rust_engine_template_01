//! RND-02 + RND-12 / GPU-CULL.1 — compute frustum cull → compacted instances + indirect args.
//!
//! Production caller of [`crate::render::culling::FrustumPlanes`]. Fire expand still reads the
//! source instance buffer; this pass owns the `DrawIndirectArgs` buffer consumed by
//! [`super::gpu_fire_particle_raster`].

use std::borrow::Cow;

use bevy::prelude::*;
use bevy::render::{
    extract_resource::ExtractResource,
    render_resource::*,
    renderer::{RenderContext, RenderDevice, RenderGraph, RenderGraphSystems, RenderQueue},
    Render, RenderApp, RenderStartup, RenderSystems,
};

use crate::render::culling::FrustumPlanes;
use crate::render::core::gpu_bind_group_registry::{
    BindGroupBufferBinding, BindGroupId, GPUBindGroupRegistry,
};
use crate::render::core::gpu_buffer_registry::{
    BufferVisibility, GPUBufferRegistry, RegisteredBufferDescriptor,
    FIRE_PARTICLE_CULLED_INSTANCES_BUFFER, FIRE_PARTICLE_CULL_COUNTER_BUFFER,
    FIRE_PARTICLE_INDIRECT_DRAW_BUFFER, FIRE_PARTICLE_INSTANCES_BUFFER,
};
use crate::render::core::gpu_packed_formats::{fire_particle_instance_format, packed_byte_size};
use crate::render::extraction::extracted_camera_metrics::ExtractedCameraMetrics;
use crate::render::fire_vfx::WorldFireParticleGpuStorage;
use crate::render::pipelines::gpu_indirect_draw::{
    FireFrustumCullStats, WorldFireIndirectDrawArgs, FIRE_DRAW_VERTS_PER_INSTANCE,
};
use crate::render::pipelines::gpu_particle_draw::PARTICLE_WORKGROUP;

pub const FRUSTUM_INSTANCE_CULL_WGSL: &str = "shaders/cull/frustum_instance_cull.wgsl";

const FIRE_CULL_BIND_GROUP: BindGroupId = BindGroupId(40);

#[derive(Clone, Copy, ShaderType)]
struct CullParamsGpu {
    plane0: Vec4,
    plane1: Vec4,
    plane2: Vec4,
    plane3: Vec4,
    plane4: Vec4,
    plane5: Vec4,
    instance_count: u32,
    verts_per_instance: u32,
}

impl Default for CullParamsGpu {
    fn default() -> Self {
        Self {
            plane0: Vec4::ZERO,
            plane1: Vec4::ZERO,
            plane2: Vec4::ZERO,
            plane3: Vec4::ZERO,
            plane4: Vec4::ZERO,
            plane5: Vec4::ZERO,
            instance_count: 0,
            verts_per_instance: FIRE_DRAW_VERTS_PER_INSTANCE,
        }
    }
}

impl CullParamsGpu {
    fn from_frustum(frustum: FrustumPlanes, instance_count: u32) -> Self {
        let p = |i: usize| {
            let pl = frustum.planes[i];
            Vec4::new(pl.n.x, pl.n.y, pl.n.z, pl.d)
        };
        Self {
            plane0: p(0),
            plane1: p(1),
            plane2: p(2),
            plane3: p(3),
            plane4: p(4),
            plane5: p(5),
            instance_count,
            verts_per_instance: FIRE_DRAW_VERTS_PER_INSTANCE,
        }
    }
}

/// Extracted planes + submitted count for the render-world cull pass.
#[derive(Resource, Clone, Copy, Default, ExtractResource)]
#[extract_app(RenderApp)]
pub struct FireFrustumCullFrame {
    pub frustum: FrustumPlanes,
    pub submitted: u32,
    pub view_ready: u32,
}

#[derive(Resource)]
struct FireFrustumCullPipeline {
    layout: BindGroupLayoutDescriptor,
    cull_pipeline: CachedComputePipelineId,
    finalize_pipeline: CachedComputePipelineId,
}

#[derive(Resource, Default)]
struct FireFrustumCullBindGpu {
    params: UniformBuffer<CullParamsGpu>,
    bind_group: Option<BindGroup>,
}

#[derive(Resource, Default)]
struct FireFrustumCullPassReady {
    ready: bool,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct FireFrustumCullPassSet;

pub fn register_fire_frustum_cull(app: &mut App) {
    app.init_resource::<FireFrustumCullFrame>()
        .init_resource::<FireFrustumCullStats>()
        .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
            FireFrustumCullFrame,
        >::default())
        .add_systems(
            PostUpdate,
            sync_fire_frustum_cull_frame
                .after(crate::render::ExtractedCameraMetricsSet::Sync)
                .after(crate::render::extraction::FireVisualFrameSet::EmitParticles)
                .run_if(crate::gui::in_simulation_or_editor_map),
        );

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_resource::<FireFrustumCullBindGpu>()
        .init_resource::<FireFrustumCullPassReady>()
        .init_resource::<FireFrustumCullFrame>()
        .add_systems(RenderStartup, init_fire_frustum_cull_pipeline)
        .add_systems(
            Render,
            prepare_fire_frustum_cull_bind_groups
                .in_set(RenderSystems::PrepareBindGroups)
                .after(crate::render::pipelines::gpu_weather_fire_field::prepare_fire_particle_gpu_storage),
        )
        .add_systems(
            RenderGraph,
            (
                ensure_fire_frustum_cull_ready,
                fire_frustum_cull_pass.after(ensure_fire_frustum_cull_ready),
            )
                .chain()
                .in_set(RenderGraphSystems::Render)
                .in_set(FireFrustumCullPassSet),
        );
}

pub(crate) fn sync_fire_frustum_cull_frame(
    metrics: Res<ExtractedCameraMetrics>,
    particles: Res<crate::render::fire_vfx::WorldFireParticleFrame>,
    policy: Res<crate::gui::RepresentationResult>,
    mut frame: ResMut<FireFrustumCullFrame>,
    mut stats: ResMut<FireFrustumCullStats>,
) {
    if metrics.view_proj == Mat4::IDENTITY {
        frame.view_ready = 0;
        frame.submitted = 0;
        stats.clear();
        return;
    }
    let frustum = FrustumPlanes::from_view_proj(metrics.view_proj);
    let cap = policy
        .gpu_budget
        .particle_rows_cap
        .min(particles.instances.len());
    // CPU mirror of the GPU sphere test — Accept counters without GPU readback.
    let mut visible = 0u32;
    for row in particles.instances.iter().take(cap) {
        let center = Vec3::new(
            row.world_xyz_heat.x,
            row.world_xyz_heat.y,
            row.world_xyz_heat.z,
        );
        let radius = row.ember_class_radius_smoke.z.max(0.5);
        if frustum.contains_sphere(center, radius) {
            visible = visible.saturating_add(1);
        }
    }
    let submitted = cap as u32;
    stats.submitted = submitted;
    stats.visible = visible;
    stats.culled = submitted.saturating_sub(visible);
    frame.frustum = frustum;
    frame.submitted = submitted;
    frame.view_ready = 1;
}

fn storage_entry(binding: u32, read_only: bool) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn init_fire_frustum_cull_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("fire_frustum_cull_layout"),
        entries: vec![
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(CullParamsGpu::min_size()),
                },
                count: None,
            },
            storage_entry(1, true),
            storage_entry(2, false),
            storage_entry(3, false),
            storage_entry(4, false),
        ],
    };
    let shader = asset_server.load(FRUSTUM_INSTANCE_CULL_WGSL);
    let cull_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("fire_frustum_cull_compact".into()),
        layout: vec![layout.clone()],
        shader: shader.clone(),
        shader_defs: vec![],
        entry_point: Some(Cow::Borrowed("cull_compact")),
        zero_initialize_workgroup_memory: false,
        ..default()
    });
    let finalize_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("fire_frustum_cull_finalize".into()),
        layout: vec![layout.clone()],
        shader,
        shader_defs: vec![],
        entry_point: Some(Cow::Borrowed("finalize_indirect")),
        zero_initialize_workgroup_memory: false,
        ..default()
    });
    commands.insert_resource(FireFrustumCullPipeline {
        layout,
        cull_pipeline,
        finalize_pipeline,
    });
}

fn prepare_fire_frustum_cull_bind_groups(
    pipeline: Option<Res<FireFrustumCullPipeline>>,
    frame: Res<FireFrustumCullFrame>,
    storage: Option<Res<WorldFireParticleGpuStorage>>,
    mut registry: ResMut<GPUBufferRegistry>,
    mut bind_registry: ResMut<GPUBindGroupRegistry>,
    mut bind_gpu: ResMut<FireFrustumCullBindGpu>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
) {
    let Some(pipeline) = pipeline else {
        return;
    };
    let count = storage.as_ref().map(|s| s.instance_count).unwrap_or(0);
    if frame.view_ready == 0 || count == 0 {
        bind_gpu.bind_group = None;
        bind_registry.invalidate(FIRE_CULL_BIND_GROUP);
        return;
    }

    let format = fire_particle_instance_format();
    let rows = count.max(1) as usize;
    let needed = packed_byte_size(format, rows);
    let _ = registry.ensure_capacity(
        &render_device,
        RegisteredBufferDescriptor {
            id: FIRE_PARTICLE_CULLED_INSTANCES_BUFFER,
            size_bytes: needed,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
            visibility: BufferVisibility::RenderAndCompute,
            stride: format.stride,
        },
    );

    let zero_indirect = WorldFireIndirectDrawArgs::default();
    let _ = registry.upload_pod_slice(
        &render_device,
        &queue,
        RegisteredBufferDescriptor {
            id: FIRE_PARTICLE_INDIRECT_DRAW_BUFFER,
            size_bytes: 256,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE | BufferUsages::INDIRECT,
            visibility: BufferVisibility::RenderAndCompute,
            stride: std::mem::size_of::<WorldFireIndirectDrawArgs>() as u32,
        },
        1,
        &[zero_indirect],
        0,
    );

    let zero_counter = 0u32;
    let _ = registry.upload_pod_slice(
        &render_device,
        &queue,
        RegisteredBufferDescriptor {
            id: FIRE_PARTICLE_CULL_COUNTER_BUFFER,
            size_bytes: 256,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
            visibility: BufferVisibility::ComputeOnly,
            stride: 4,
        },
        1,
        &[zero_counter],
        0,
    );

    let Some(src) = registry.get(FIRE_PARTICLE_INSTANCES_BUFFER) else {
        return;
    };
    let Some(dst) = registry.get(FIRE_PARTICLE_CULLED_INSTANCES_BUFFER) else {
        return;
    };
    let Some(indirect) = registry.get(FIRE_PARTICLE_INDIRECT_DRAW_BUFFER) else {
        return;
    };
    let Some(counter) = registry.get(FIRE_PARTICLE_CULL_COUNTER_BUFFER) else {
        return;
    };

    let params = CullParamsGpu::from_frustum(frame.frustum, count);
    bind_gpu.params.set(params);
    bind_gpu.params.write_buffer(&render_device, &queue);
    let Some(params_binding) = bind_gpu.params.binding() else {
        return;
    };

    let layout = pipeline_cache.get_bind_group_layout(&pipeline.layout);
    let bg = render_device.create_bind_group(
        Some("fire_frustum_cull_bg"),
        &layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: params_binding,
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &src.buffer,
                    offset: 0,
                    size: None,
                }),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &dst.buffer,
                    offset: 0,
                    size: None,
                }),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &indirect.buffer,
                    offset: 0,
                    size: None,
                }),
            },
            BindGroupEntry {
                binding: 4,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &counter.buffer,
                    offset: 0,
                    size: None,
                }),
            },
        ],
    );
    bind_registry.insert(
        FIRE_CULL_BIND_GROUP,
        bg.clone(),
        vec![
            BindGroupBufferBinding {
                buffer_id: FIRE_PARTICLE_INSTANCES_BUFFER,
                buffer_version: src.version,
            },
            BindGroupBufferBinding {
                buffer_id: FIRE_PARTICLE_CULLED_INSTANCES_BUFFER,
                buffer_version: dst.version,
            },
            BindGroupBufferBinding {
                buffer_id: FIRE_PARTICLE_INDIRECT_DRAW_BUFFER,
                buffer_version: indirect.version,
            },
            BindGroupBufferBinding {
                buffer_id: FIRE_PARTICLE_CULL_COUNTER_BUFFER,
                buffer_version: counter.version,
            },
        ],
    );
    bind_gpu.bind_group = Some(bg);
}

fn ensure_fire_frustum_cull_ready(
    pipeline: Option<Res<FireFrustumCullPipeline>>,
    cache: Res<PipelineCache>,
    mut ready: ResMut<FireFrustumCullPassReady>,
) {
    if ready.ready {
        return;
    }
    let Some(pipeline) = pipeline else {
        return;
    };
    let cull_ok = matches!(
        cache.get_compute_pipeline_state(pipeline.cull_pipeline),
        CachedPipelineState::Ok(_)
    );
    let fin_ok = matches!(
        cache.get_compute_pipeline_state(pipeline.finalize_pipeline),
        CachedPipelineState::Ok(_)
    );
    ready.ready = cull_ok && fin_ok;
}

fn fire_frustum_cull_pass(world: &World, mut ctx: RenderContext) {
    let ready = world.resource::<FireFrustumCullPassReady>();
    if !ready.ready {
        return;
    }
    let frame = world.resource::<FireFrustumCullFrame>();
    let storage = world.get_resource::<WorldFireParticleGpuStorage>();
    let count = storage.map(|s| s.instance_count).unwrap_or(0);
    if frame.view_ready == 0 || count == 0 {
        return;
    }
    let bind = world.resource::<FireFrustumCullBindGpu>();
    let Some(bg) = bind.bind_group.as_ref() else {
        return;
    };
    let pipeline = world.resource::<FireFrustumCullPipeline>();
    let cache = world.resource::<PipelineCache>();
    let Some(cull_pl) = cache.get_compute_pipeline(pipeline.cull_pipeline) else {
        return;
    };
    let Some(fin_pl) = cache.get_compute_pipeline(pipeline.finalize_pipeline) else {
        return;
    };

    {
        let mut pass = ctx.command_encoder().begin_compute_pass(&ComputePassDescriptor {
            label: Some("fire_frustum_cull"),
            timestamp_writes: None,
        });
        pass.set_bind_group(0, bg, &[]);
        pass.set_pipeline(cull_pl);
        pass.dispatch_workgroups(count.div_ceil(PARTICLE_WORKGROUP), 1, 1);
        pass.set_pipeline(fin_pl);
        pass.dispatch_workgroups(1, 1, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::culling::Plane4;
    use crate::render::fire_vfx::GpuParticleInstance;

    #[test]
    fn cpu_mirror_culls_outside_sphere() {
        let fr = FrustumPlanes {
            planes: [
                Plane4::from_normal_distance(Vec3::X, 0.0),
                Plane4::from_normal_distance(-Vec3::X, 10.0),
                Plane4::from_normal_distance(Vec3::Y, 0.0),
                Plane4::from_normal_distance(-Vec3::Y, 10.0),
                Plane4::from_normal_distance(Vec3::Z, 0.0),
                Plane4::from_normal_distance(-Vec3::Z, 10.0),
            ],
        };
        let mut inside = GpuParticleInstance::default();
        inside.world_xyz_heat = Vec4::new(5.0, 5.0, 5.0, 1.0);
        inside.ember_class_radius_smoke = Vec4::new(1.0, 0.0, 1.0, 0.0);
        let mut outside = inside;
        outside.world_xyz_heat = Vec4::new(-5.0, 5.0, 5.0, 1.0);
        assert!(fr.contains_sphere(inside.world_xyz_heat.truncate(), 1.0));
        assert!(!fr.contains_sphere(outside.world_xyz_heat.truncate(), 1.0));
    }

    #[test]
    fn cull_shader_exports_book_entry_points() {
        let src = include_str!("../../../assets/shaders/cull/frustum_instance_cull.wgsl");
        assert!(src.contains("fn cull_compact"));
        assert!(src.contains("fn finalize_indirect"));
        assert!(src.contains("atomicAdd"));
        assert!(src.contains("DrawIndirectArgs"));
    }

    #[test]
    fn cull_stats_report_non_zero_when_half_outside() {
        let fr = FrustumPlanes {
            planes: [
                Plane4::from_normal_distance(Vec3::X, 0.0),
                Plane4::from_normal_distance(-Vec3::X, 10.0),
                Plane4::from_normal_distance(Vec3::Y, 0.0),
                Plane4::from_normal_distance(-Vec3::Y, 10.0),
                Plane4::from_normal_distance(Vec3::Z, 0.0),
                Plane4::from_normal_distance(-Vec3::Z, 10.0),
            ],
        };
        let rows = [
            {
                let mut r = GpuParticleInstance::default();
                r.world_xyz_heat = Vec4::new(5.0, 5.0, 5.0, 1.0);
                r.ember_class_radius_smoke.z = 1.0;
                r
            },
            {
                let mut r = GpuParticleInstance::default();
                r.world_xyz_heat = Vec4::new(50.0, 5.0, 5.0, 1.0);
                r.ember_class_radius_smoke.z = 1.0;
                r
            },
        ];
        let visible = rows
            .iter()
            .filter(|r| {
                fr.contains_sphere(
                    Vec3::new(r.world_xyz_heat.x, r.world_xyz_heat.y, r.world_xyz_heat.z),
                    r.ember_class_radius_smoke.z.max(0.5),
                )
            })
            .count();
        assert_eq!(visible, 1);
        assert_eq!(rows.len() - visible, 1);
    }
}
