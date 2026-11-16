//! **CQ-50 / DH-02** — GPU-primary dense heat: RenderApp H→V compute + `ReadbackComplete` sole writer.
//!
//! Main world stages inject+upload only. CPU [`super::run_dense_heat_step`] is parity/fallback
//! (no RenderApp / `RUST_ENGINE_DENSE_HEAT_CPU=1`).

use std::borrow::Cow;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    binding_types::uniform_buffer, BindGroup, BindGroupEntries, BindGroupLayoutDescriptor,
    BindGroupLayoutEntries, BindingType, BufferBindingType, BufferUsages, CachedComputePipelineId,
    ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache, ShaderStages, ShaderType,
    UniformBuffer,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderGraph, RenderGraphSystems, RenderQueue};
use bevy::render::storage::{GpuShaderBuffer, ShaderBuffer};
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};

use crate::systems::sim_control::SimStepStamp;

use super::frame_snapshots::NavFieldFrame;
use super::heat_diffusion::{sync_nav_field_from_heat_diffusion, HeatDiffusionFieldBuffers};
// NavFieldFrame mut only in readback observer.
use super::heat_diffusion_dense::{
    dense_roi_from_fire, run_dense_heat_step, DenseHeatGrid, DENSE_ROI_MAX_EDGE, DENSE_SOURCE_BLEND,
};
use super::heat_diffusion_gpu::{
    pack_dense_to_cells, DenseHeatPlayField, HEAT_DIFFUSION_DENSE_WGSL,
};
use super::heat_weather::{apply_climate_to_dense_heat, apply_weather_to_dense_heat};
use super::ComputeContext;

/// Max interleaved f32 count (`width*height*2`) for fixed ShaderBuffers.
pub const DENSE_HEAT_GPU_MAX_F32: usize =
    (DENSE_ROI_MAX_EDGE as usize) * (DENSE_ROI_MAX_EDGE as usize) * 2;

/// Env: force CPU dense step (parity / headless).
pub const DENSE_HEAT_CPU_ENV: &str = "RUST_ENGINE_DENSE_HEAT_CPU";

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DenseHeatGpuPrimary {
    pub enabled: bool,
}

impl Default for DenseHeatGpuPrimary {
    fn default() -> Self {
        Self {
            enabled: std::env::var_os(DENSE_HEAT_CPU_ENV).is_none(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
struct DenseHeatParamsGpu {
    width: u32,
    height: u32,
    w_side: f32,
    w_center: f32,
    decay: f32,
    _pad: f32,
}

/// Staging job extracted to RenderApp (inject already applied on the CPU lattice).
#[derive(Resource, Clone, Debug, Default, ExtractResource)]
#[extract_app(RenderApp)]
pub struct DenseHeatGpuJob {
    pub active: bool,
    pub origin: IVec2,
    pub width: u32,
    pub height: u32,
    pub stamp: SimStepStamp,
    pub interleaved: Vec<f32>,
    /// Post-diffusion inject (chunk, heat, smoke).
    pub post_inject: Vec<(IVec2, f32, f32)>,
    pub wind_mps: Vec2,
    pub rain01: f32,
}

/// Ping-pong ShaderBuffers — `field_a` holds the post-V lattice (VFX RO bind target).
#[derive(Resource, Clone, ExtractResource)]
#[extract_app(RenderApp)]
pub struct DenseHeatGpuHandles {
    pub field_a: Handle<ShaderBuffer>,
    pub field_b: Handle<ShaderBuffer>,
}

/// Render-graph ordering: dense H/V before fire particle expand (same-frame VFX sample).
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct DenseHeatGpuComputePassSet;

#[derive(Resource)]
struct DenseHeatGpuPipeline {
    params_layout: BindGroupLayoutDescriptor,
    src_layout: BindGroupLayoutDescriptor,
    dst_layout: BindGroupLayoutDescriptor,
    pipeline_h: CachedComputePipelineId,
    pipeline_v: CachedComputePipelineId,
}

#[derive(Resource, Default)]
struct DenseHeatGpuBindGroups {
    params: Option<BindGroup>,
    h_src: Option<BindGroup>,
    h_dst: Option<BindGroup>,
    v_src: Option<BindGroup>,
    v_dst: Option<BindGroup>,
    dispatch_x: u32,
    dispatch_y: u32,
    ready: bool,
}

#[derive(Resource)]
struct DenseHeatParamsUniform {
    buffer: UniformBuffer<DenseHeatParamsGpu>,
}

impl DenseHeatParamsUniform {
    fn new() -> Self {
        Self {
            buffer: UniformBuffer::from(DenseHeatParamsGpu::default()),
        }
    }
}

#[derive(Component)]
struct DenseHeatReadbackEntity;

pub struct DenseHeatGpuPlugin;

impl Plugin for DenseHeatGpuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DenseHeatGpuPrimary>()
            .init_resource::<DenseHeatGpuJob>()
            .add_plugins((
                ExtractResourcePlugin::<DenseHeatGpuJob>::default(),
                ExtractResourcePlugin::<DenseHeatGpuHandles>::default(),
            ))
            .add_systems(Startup, setup_dense_heat_gpu_buffers);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            if let Some(mut primary) = app.world_mut().get_resource_mut::<DenseHeatGpuPrimary>() {
                primary.enabled = false;
            }
            return;
        };

        render_app
            .init_resource::<DenseHeatGpuJob>()
            .init_resource::<DenseHeatGpuBindGroups>()
            .insert_resource(DenseHeatParamsUniform::new())
            .add_systems(RenderStartup, init_dense_heat_gpu_pipeline)
            .add_systems(
                Render,
                (
                    upload_dense_heat_gpu_job,
                    prepare_dense_heat_gpu_bind_groups.after(upload_dense_heat_gpu_job),
                )
                    .chain()
                    .in_set(RenderSystems::PrepareBindGroups),
            )
            .configure_sets(
                RenderGraph,
                DenseHeatGpuComputePassSet
                    .before(crate::render::pipelines::gpu_particle_draw::WorldFireParticleDrawPassSet)
                    .in_set(RenderGraphSystems::Render),
            )
            .add_systems(
                RenderGraph,
                dense_heat_gpu_compute_pass
                    .in_set(RenderGraphSystems::Render)
                    .in_set(DenseHeatGpuComputePassSet),
            );
    }
}

fn setup_dense_heat_gpu_buffers(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    primary: Res<DenseHeatGpuPrimary>,
) {
    // Always allocate ShaderBuffers when RenderApp exists so fire expand can bind group 4.
    // CPU-primary (`!enabled`) skips compute dispatch + readback observer.
    let bytes = (DENSE_HEAT_GPU_MAX_F32 * std::mem::size_of::<f32>()) as u64;
    let mut make = |label: &'static str| {
        let mut buf = ShaderBuffer::with_size(bytes, RenderAssetUsages::RENDER_WORLD);
        buf.label = Cow::Borrowed(label);
        buf.buffer_usage = BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;
        buffers.add(buf)
    };
    let field_a = make("dense_heat_field_a");
    let field_b = make("dense_heat_field_b");
    commands.insert_resource(DenseHeatGpuHandles {
        field_a: field_a.clone(),
        field_b,
    });
    if primary.enabled {
        commands
            .spawn((
                Name::new("dense_heat_readback"),
                DenseHeatReadbackEntity,
                Readback::buffer(field_a),
            ))
            .observe(on_dense_heat_readback_complete);
    }
}

fn init_dense_heat_gpu_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let params_layout = BindGroupLayoutDescriptor::new(
        "DenseHeatParams",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (uniform_buffer::<DenseHeatParamsGpu>(false),),
        ),
    );
    let storage_ro = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("DenseHeatSrc"),
        entries: vec![bevy::render::render_resource::BindGroupLayoutEntry {
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
    let storage_rw = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("DenseHeatDst"),
        entries: vec![bevy::render::render_resource::BindGroupLayoutEntry {
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
    let shader = asset_server.load(HEAT_DIFFUSION_DENSE_WGSL);
    let layout = vec![
        params_layout.clone(),
        storage_ro.clone(),
        storage_rw.clone(),
    ];
    let pipeline_h = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("dense_heat_pass_h".into()),
        layout: layout.clone(),
        shader: shader.clone(),
        entry_point: Some(Cow::from("pass_horizontal")),
        ..default()
    });
    let pipeline_v = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("dense_heat_pass_v".into()),
        layout,
        shader,
        entry_point: Some(Cow::from("pass_vertical")),
        ..default()
    });
    commands.insert_resource(DenseHeatGpuPipeline {
        params_layout,
        src_layout: storage_ro,
        dst_layout: storage_rw,
        pipeline_h,
        pipeline_v,
    });
}

fn upload_dense_heat_gpu_job(
    job: Res<DenseHeatGpuJob>,
    handles: Option<Res<DenseHeatGpuHandles>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    let Some(handles) = handles else {
        return;
    };
    if !job.active || job.interleaved.is_empty() {
        return;
    }
    let Some(mut buf) = buffers.get_mut(&handles.field_a) else {
        return;
    };
    buf.clear();
    let mut pad = job.interleaved.clone();
    pad.resize(DENSE_HEAT_GPU_MAX_F32, 0.0);
    buf.extend_from_slice(&pad);
}

fn prepare_dense_heat_gpu_bind_groups(
    job: Res<DenseHeatGpuJob>,
    handles: Option<Res<DenseHeatGpuHandles>>,
    pipeline: Option<Res<DenseHeatGpuPipeline>>,
    mut params_u: ResMut<DenseHeatParamsUniform>,
    mut groups: ResMut<DenseHeatGpuBindGroups>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    gpu_buffers: Res<RenderAssets<GpuShaderBuffer>>,
) {
    groups.ready = false;
    let (Some(handles), Some(pipeline)) = (handles, pipeline) else {
        return;
    };
    if !job.active || job.width == 0 || job.height == 0 {
        return;
    }
    let Some(gpu_a) = gpu_buffers.get(&handles.field_a) else {
        return;
    };
    let Some(gpu_b) = gpu_buffers.get(&handles.field_b) else {
        return;
    };

    params_u.buffer.set(DenseHeatParamsGpu {
        width: job.width,
        height: job.height,
        w_side: 0.15,
        w_center: 0.70,
        decay: 0.02,
        _pad: 0.0,
    });
    params_u.buffer.write_buffer(&render_device, &queue);
    let Some(params_binding) = params_u.buffer.binding() else {
        return;
    };

    groups.params = Some(render_device.create_bind_group(
        Some("dense_heat_params_bg"),
        &pipeline_cache.get_bind_group_layout(&pipeline.params_layout),
        &BindGroupEntries::single(params_binding),
    ));
    groups.h_src = Some(render_device.create_bind_group(
        Some("dense_heat_h_src"),
        &pipeline_cache.get_bind_group_layout(&pipeline.src_layout),
        &BindGroupEntries::single(gpu_a.buffer.as_entire_buffer_binding()),
    ));
    groups.h_dst = Some(render_device.create_bind_group(
        Some("dense_heat_h_dst"),
        &pipeline_cache.get_bind_group_layout(&pipeline.dst_layout),
        &BindGroupEntries::single(gpu_b.buffer.as_entire_buffer_binding()),
    ));
    groups.v_src = Some(render_device.create_bind_group(
        Some("dense_heat_v_src"),
        &pipeline_cache.get_bind_group_layout(&pipeline.src_layout),
        &BindGroupEntries::single(gpu_b.buffer.as_entire_buffer_binding()),
    ));
    groups.v_dst = Some(render_device.create_bind_group(
        Some("dense_heat_v_dst"),
        &pipeline_cache.get_bind_group_layout(&pipeline.dst_layout),
        &BindGroupEntries::single(gpu_a.buffer.as_entire_buffer_binding()),
    ));
    groups.dispatch_x = job.width.div_ceil(8).max(1);
    groups.dispatch_y = job.height.div_ceil(8).max(1);
    groups.ready = true;
}

fn dense_heat_gpu_compute_pass(
    mut render_context: RenderContext,
    pipeline_cache: Res<PipelineCache>,
    pipeline: Option<Res<DenseHeatGpuPipeline>>,
    groups: Res<DenseHeatGpuBindGroups>,
) {
    if !groups.ready {
        return;
    }
    let Some(pipeline) = pipeline else {
        return;
    };
    let (Some(params), Some(h_src), Some(h_dst), Some(v_src), Some(v_dst)) = (
        groups.params.as_ref(),
        groups.h_src.as_ref(),
        groups.h_dst.as_ref(),
        groups.v_src.as_ref(),
        groups.v_dst.as_ref(),
    ) else {
        return;
    };
    let Some(pipe_h) = pipeline_cache.get_compute_pipeline(pipeline.pipeline_h) else {
        return;
    };
    let Some(pipe_v) = pipeline_cache.get_compute_pipeline(pipeline.pipeline_v) else {
        return;
    };

    {
        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("dense_heat_h"),
                ..default()
            });
        pass.set_bind_group(0, params, &[]);
        pass.set_bind_group(1, h_src, &[]);
        pass.set_bind_group(2, h_dst, &[]);
        pass.set_pipeline(pipe_h);
        pass.dispatch_workgroups(groups.dispatch_x, groups.dispatch_y, 1);
    }
    {
        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("dense_heat_v"),
                ..default()
            });
        pass.set_bind_group(0, params, &[]);
        pass.set_bind_group(1, v_src, &[]);
        pass.set_bind_group(2, v_dst, &[]);
        pass.set_pipeline(pipe_v);
        pass.dispatch_workgroups(groups.dispatch_x, groups.dispatch_y, 1);
    }
}

/// Sole main-world writer of [`DenseHeatPlayField`] when GPU-primary is on.
fn on_dense_heat_readback_complete(
    event: On<ReadbackComplete>,
    job: Res<DenseHeatGpuJob>,
    mut play: ResMut<DenseHeatPlayField>,
    mut field: ResMut<HeatDiffusionFieldBuffers>,
    mut navigation: ResMut<NavFieldFrame>,
) {
    if !job.active || job.width == 0 || job.height == 0 {
        return;
    }
    let need = (job.width as usize)
        .saturating_mul(job.height as usize)
        .saturating_mul(2)
        .saturating_mul(std::mem::size_of::<f32>());
    if event.data.len() < need {
        return;
    }
    let floats: &[f32] = bytemuck::cast_slice(&event.data[..need]);
    let mut grid = DenseHeatGrid::new(job.origin, job.width, job.height);
    for y in 0..job.height {
        for x in 0..job.width {
            let i = grid.idx(x, y);
            let o = i * 2;
            grid.heat[i] = floats[o];
            grid.smoke[i] = floats[o + 1];
        }
    }
    // Parity schedule: re-inject sources after blur/decay.
    for &(chunk, heat, smoke) in &job.post_inject {
        if let Some((lx, ly)) = grid.local_of(chunk) {
            let i = grid.idx(lx, ly);
            grid.heat[i] =
                grid.heat[i] * (1.0 - DENSE_SOURCE_BLEND) + heat * DENSE_SOURCE_BLEND;
            grid.smoke[i] =
                grid.smoke[i] * (1.0 - DENSE_SOURCE_BLEND) + smoke * DENSE_SOURCE_BLEND;
        }
    }
    apply_weather_to_dense_heat(&mut grid, job.wind_mps, job.rain01);

    let cells = pack_dense_to_cells(&grid);
    let write = field.write_cells_mut_pub();
    *write = cells;
    field.swap_pub();
    field.active_cell_count = field.read_cells().len();
    field.source_stamp = job.stamp;
    field.stamp = job.stamp;
    play.grid = Some(grid);
    play.stamp = job.stamp;
    play.generation = play.generation.wrapping_add(1);
    sync_nav_field_from_heat_diffusion(&mut navigation, &field);
}

fn pack_interleaved(grid: &DenseHeatGrid) -> Vec<f32> {
    let n = grid.len();
    let mut out = Vec::with_capacity(n * 2);
    for i in 0..n {
        out.push(grid.heat[i]);
        out.push(grid.smoke[i]);
    }
    out
}

/// Stage inject → GPU job (no CPU H/V). Fallback: full CPU dense step.
/// Caller syncs nav after CPU path; GPU path syncs in [`on_dense_heat_readback_complete`].
pub fn advance_dense_heat_play_gpu_or_cpu(
    play: &mut DenseHeatPlayField,
    field: &mut HeatDiffusionFieldBuffers,
    job: &mut DenseHeatGpuJob,
    ctx: &ComputeContext<'_>,
    influence_active: bool,
    gpu_primary: bool,
) {
    job.active = false;
    if !influence_active {
        field.active_cell_count = 0;
        return;
    }
    let fallback = ctx.lod.global_band();
    let mut eligible =
        super::heat_diffusion::eligible_fire_rows(ctx.fire, ctx.lod_map, fallback, ctx.policy);
    let budget = ctx.path_budget.max(1) as usize;
    if eligible.len() > budget {
        eligible.truncate(budget);
    }
    if eligible.is_empty() && play.grid.as_ref().is_none_or(|g| g.heat.iter().all(|h| *h <= 1e-4)) {
        field.active_cell_count = 0;
        field.source_stamp = ctx.fire.stamp;
        field.stamp = ctx.fire.stamp;
        return;
    }

    // Ensure ROI + preserve overlap (CQ-63).
    let Some(needed) = dense_roi_from_fire(&eligible, play.pad_chunks) else {
        return;
    };
    let rebuild = match &play.grid {
        None => true,
        Some(g) => {
            needed.origin.x < g.origin.x
                || needed.origin.y < g.origin.y
                || needed.origin.x + needed.width as i32 > g.origin.x + g.width as i32
                || needed.origin.y + needed.height as i32 > g.origin.y + g.height as i32
        }
    };
    if rebuild {
        let mut next = needed;
        if let Some(old) = play.grid.take() {
            for y in 0..old.height {
                for x in 0..old.width {
                    let chunk = old.origin + IVec2::new(x as i32, y as i32);
                    if let Some((lx, ly)) = next.local_of(chunk) {
                        let oi = old.idx(x, y);
                        let ni = next.idx(lx, ly);
                        next.heat[ni] = old.heat[oi];
                        next.smoke[ni] = old.smoke[oi];
                    }
                }
            }
        }
        play.grid = Some(next);
    }

    let Some(grid) = play.grid.as_mut() else {
        return;
    };

    if !gpu_primary {
        run_dense_heat_step(grid, &eligible);
        if let Some(wx) = ctx.atmosphere_wx {
            apply_climate_to_dense_heat(grid, wx);
        }
        let cells = pack_dense_to_cells(grid);
        let write = field.write_cells_mut_pub();
        *write = cells;
        field.swap_pub();
        field.active_cell_count = field.read_cells().len();
        field.gpu_row_capacity = crate::render::LodBandBufferPolicy::heat_diffusion_rows(
            ctx.policy.world_lod_band,
            field.active_cell_count,
        );
        field.source_stamp = ctx.fire.stamp;
        field.stamp = ctx.fire.stamp;
        play.stamp = ctx.fire.stamp;
        play.generation = play.generation.wrapping_add(1);
        return;
    }

    // GPU path: inject once into staging lattice, upload; H/V + post-inject happen GPU/readback.
    grid.inject_sources(&eligible);
    let post_inject: Vec<(IVec2, f32, f32)> = eligible
        .iter()
        .map(|r| (r.chunk, r.heat, r.smoke))
        .collect();
    let (wind, rain) = ctx
        .atmosphere_wx
        .map(|w| (w.wind_mps, w.rain01))
        .unwrap_or((Vec2::ZERO, 0.0));
    *job = DenseHeatGpuJob {
        active: true,
        origin: grid.origin,
        width: grid.width,
        height: grid.height,
        stamp: ctx.fire.stamp,
        interleaved: pack_interleaved(grid),
        post_inject,
        wind_mps: wind,
        rain01: rain,
    };
}
