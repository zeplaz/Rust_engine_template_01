//! GPU compute dispatch for minimap compositor (render world).

use std::borrow::Cow;
use std::sync::atomic::AtomicU64;

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{self, RenderGraph, RenderLabel};
use bevy::render::render_resource::{
    binding_types::{texture_storage_2d, uniform_buffer},
    *,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};
use bevy::shader::PipelineCacheError;

use super::composite::{
    MinimapCompositeDispatch, MinimapCompositeHeatTextures, MinimapCompositeParamsGpu,
    MINIMAP_COMPOSITE_SHADER,
};
use super::pass::minimap_gpu_compositor_env_enabled;
use super::diagnostics::{MINIMAP_GPU_DEDUP_SKIP_COUNT, MINIMAP_GPU_EXECUTE_COUNT};

const WORKGROUP: u32 = 8;
const STORAGE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

#[derive(Resource, Clone, Copy, ShaderType, Default)]
pub struct MinimapCompositeUniforms {
    pub fire_heat_enabled: u32,
    pub logistics_heat_enabled: u32,
    pub construction_heat_enabled: u32,
    pub ecology_heat_enabled: u32,
    pub fow_heat_enabled: u32,
    pub ew_heat_enabled: u32,
    pub overlay_revision: u32,
    pub logistics_rows: u32,
    pub construction_rows: u32,
    pub ecology_rows: u32,
    pub fow_rows: u32,
    pub ew_rows: u32,
}

impl From<MinimapCompositeParamsGpu> for MinimapCompositeUniforms {
    fn from(p: MinimapCompositeParamsGpu) -> Self {
        Self {
            fire_heat_enabled: p.fire_heat_enabled,
            logistics_heat_enabled: p.logistics_heat_enabled,
            construction_heat_enabled: p.construction_heat_enabled,
            ecology_heat_enabled: p.ecology_heat_enabled,
            fow_heat_enabled: p.fow_heat_enabled,
            ew_heat_enabled: p.ew_heat_enabled,
            overlay_revision: p.overlay_revision as u32,
            logistics_rows: p.logistics_rows,
            construction_rows: p.construction_rows,
            ecology_rows: p.ecology_rows,
            fow_rows: p.fow_rows,
            ew_rows: p.ew_rows,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct MinimapCompositeLabel;

#[derive(Resource)]
struct MinimapCompositePipeline {
    layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
}

#[derive(Resource, Default)]
struct MinimapCompositeBindGpu {
    bind_group: Option<BindGroup>,
    terrain_id: AssetId<Image>,
    fire_id: AssetId<Image>,
    logistics_id: AssetId<Image>,
    construction_id: AssetId<Image>,
    ecology_id: AssetId<Image>,
    fow_id: AssetId<Image>,
    ew_id: AssetId<Image>,
    output_id: AssetId<Image>,
    params_stamp: u64,
    fire_enabled: u32,
    logistics_enabled: u32,
    construction_enabled: u32,
    ecology_enabled: u32,
    fow_enabled: u32,
    ew_enabled: u32,
}

struct MinimapCompositeNode {
    pipeline_ready: bool,
    last_executed_commit_stamp: AtomicU64,
}

impl Default for MinimapCompositeNode {
    fn default() -> Self {
        Self {
            pipeline_ready: false,
            last_executed_commit_stamp: AtomicU64::new(0),
        }
    }
}

pub fn register_minimap_composite_gpu(app: &mut App) {
    if !minimap_gpu_compositor_env_enabled() {
        return;
    }

    app.add_plugins((
        ExtractResourcePlugin::<MinimapCompositeDispatch>::default(),
        ExtractResourcePlugin::<MinimapCompositeHeatTextures>::default(),
    ));

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_resource::<MinimapCompositeBindGpu>()
        .add_systems(RenderStartup, init_minimap_composite_pipeline)
        .add_systems(
            Render,
            prepare_minimap_composite_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );

    let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
    graph.add_node(MinimapCompositeLabel, MinimapCompositeNode::default());
    graph.add_node_edge(
        MinimapCompositeLabel,
        bevy::render::graph::CameraDriverLabel,
    );
}

fn init_minimap_composite_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "MinimapComposite",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                texture_storage_2d(STORAGE_FORMAT, StorageTextureAccess::ReadOnly),
                texture_storage_2d(STORAGE_FORMAT, StorageTextureAccess::ReadOnly),
                texture_storage_2d(STORAGE_FORMAT, StorageTextureAccess::ReadOnly),
                texture_storage_2d(STORAGE_FORMAT, StorageTextureAccess::ReadOnly),
                texture_storage_2d(STORAGE_FORMAT, StorageTextureAccess::ReadOnly),
                texture_storage_2d(STORAGE_FORMAT, StorageTextureAccess::ReadOnly),
                texture_storage_2d(STORAGE_FORMAT, StorageTextureAccess::ReadOnly),
                texture_storage_2d(STORAGE_FORMAT, StorageTextureAccess::WriteOnly),
                uniform_buffer::<MinimapCompositeUniforms>(false),
            ),
        ),
    );

    let shader = asset_server.load(MINIMAP_COMPOSITE_SHADER);
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![layout.clone()],
        shader,
        entry_point: Some(Cow::from("composite")),
        ..default()
    });

    commands.insert_resource(MinimapCompositePipeline { layout, pipeline });
}

fn prepare_minimap_composite_bind_groups(
    pipeline: Res<MinimapCompositePipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    dispatch: Res<MinimapCompositeDispatch>,
    heat: Res<MinimapCompositeHeatTextures>,
    mut bind_gpu: ResMut<MinimapCompositeBindGpu>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
) {
    if !dispatch.has_commit() {
        return;
    }

    let terrain_id = heat.terrain.id();
    let fire_id = heat.fire.id();
    let logistics_id = heat.logistics.id();
    let construction_id = heat.construction.id();
    let ecology_id = heat.ecology.id();
    let fow_id = heat.fow.id();
    let ew_id = heat.ew.id();
    let output_id = dispatch.output.id();

    let params_key = (
        dispatch.params.fire_heat_enabled,
        dispatch.params.logistics_heat_enabled,
        dispatch.params.construction_heat_enabled,
        dispatch.params.ecology_heat_enabled,
        dispatch.params.fow_heat_enabled,
        dispatch.params.ew_heat_enabled,
        dispatch.params.overlay_revision,
        dispatch.params.logistics_rows,
        dispatch.params.construction_rows,
        dispatch.params.ecology_rows,
        dispatch.params.fow_rows,
        dispatch.params.ew_rows,
    );
    let same_assets = bind_gpu.terrain_id == terrain_id
        && bind_gpu.fire_id == fire_id
        && bind_gpu.logistics_id == logistics_id
        && bind_gpu.construction_id == construction_id
        && bind_gpu.ecology_id == ecology_id
        && bind_gpu.fow_id == fow_id
        && bind_gpu.ew_id == ew_id
        && bind_gpu.output_id == output_id
        && bind_gpu.params_stamp == params_key.6
        && bind_gpu.fire_enabled == params_key.0
        && bind_gpu.logistics_enabled == params_key.1
        && bind_gpu.construction_enabled == params_key.2
        && bind_gpu.ecology_enabled == params_key.3
        && bind_gpu.fow_enabled == params_key.4
        && bind_gpu.ew_enabled == params_key.5;

    if same_assets && bind_gpu.bind_group.is_some() {
        return;
    }

    let Some(terrain) = gpu_images.get(&heat.terrain) else {
        return;
    };
    let Some(fire) = gpu_images.get(&heat.fire) else {
        return;
    };
    let Some(logistics) = gpu_images.get(&heat.logistics) else {
        return;
    };
    let Some(construction) = gpu_images.get(&heat.construction) else {
        return;
    };
    let Some(ecology) = gpu_images.get(&heat.ecology) else {
        return;
    };
    let Some(fow) = gpu_images.get(&heat.fow) else {
        return;
    };
    let Some(ew) = gpu_images.get(&heat.ew) else {
        return;
    };
    let Some(output) = gpu_images.get(&dispatch.output) else {
        return;
    };

    let ub_value: MinimapCompositeUniforms = dispatch.params.into();
    let mut ub = UniformBuffer::from(ub_value);
    ub.write_buffer(&render_device, &queue);

    let layout = pipeline_cache.get_bind_group_layout(&pipeline.layout);
    let bind_group = render_device.create_bind_group(
        None,
        &layout,
        &BindGroupEntries::sequential((
            &terrain.texture_view,
            &fire.texture_view,
            &logistics.texture_view,
            &construction.texture_view,
            &ecology.texture_view,
            &fow.texture_view,
            &ew.texture_view,
            &output.texture_view,
            &ub,
        )),
    );

    bind_gpu.bind_group = Some(bind_group);
    bind_gpu.terrain_id = terrain_id;
    bind_gpu.fire_id = fire_id;
    bind_gpu.logistics_id = logistics_id;
    bind_gpu.construction_id = construction_id;
    bind_gpu.ecology_id = ecology_id;
    bind_gpu.fow_id = fow_id;
    bind_gpu.ew_id = ew_id;
    bind_gpu.output_id = output_id;
    bind_gpu.params_stamp = dispatch.params.overlay_revision;
    bind_gpu.fire_enabled = dispatch.params.fire_heat_enabled;
    bind_gpu.logistics_enabled = dispatch.params.logistics_heat_enabled;
    bind_gpu.construction_enabled = dispatch.params.construction_heat_enabled;
    bind_gpu.ecology_enabled = dispatch.params.ecology_heat_enabled;
    bind_gpu.fow_enabled = dispatch.params.fow_heat_enabled;
    bind_gpu.ew_enabled = dispatch.params.ew_heat_enabled;
}

impl render_graph::Node for MinimapCompositeNode {
    fn update(&mut self, world: &mut World) {
        if self.pipeline_ready {
            return;
        }
        let pipeline = world.resource::<MinimapCompositePipeline>();
        let cache = world.resource::<PipelineCache>();
        match cache.get_compute_pipeline_state(pipeline.pipeline) {
            CachedPipelineState::Ok(_) => self.pipeline_ready = true,
            CachedPipelineState::Err(PipelineCacheError::ShaderNotLoaded(_)) => {}
            CachedPipelineState::Err(e) => {
                super::diagnostics::MINIMAP_GPU_SHADER_FAILED
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                bevy::log::error!(
                    target: "minimap_compositor",
                    "GPU minimap shader failed — falling back to CPU raster. \
                     Fix assets/{MINIMAP_COMPOSITE_SHADER}: {e}"
                );
            }
            _ => {}
        }
    }

    fn run(
        &self,
        _ctx: &mut render_graph::RenderGraphContext,
        render_ctx: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if !self.pipeline_ready {
            return Ok(());
        }

        let dispatch = world.resource::<MinimapCompositeDispatch>();
        if !dispatch.has_commit() {
            return Ok(());
        }

        if dispatch.commit_stamp
            == self
                .last_executed_commit_stamp
                .load(std::sync::atomic::Ordering::Relaxed)
        {
            MINIMAP_GPU_DEDUP_SKIP_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Ok(());
        }

        let bind_gpu = world.resource::<MinimapCompositeBindGpu>();
        let Some(bind_group) = bind_gpu.bind_group.as_ref() else {
            return Ok(());
        };

        let pipeline_res = world.resource::<MinimapCompositePipeline>();
        let cache = world.resource::<PipelineCache>();
        let Some(pl) = cache.get_compute_pipeline(pipeline_res.pipeline) else {
            return Ok(());
        };

        let heat = world.resource::<MinimapCompositeHeatTextures>();
        let Some(terrain_gpu) = world
            .resource::<RenderAssets<GpuImage>>()
            .get(&heat.terrain)
        else {
            return Ok(());
        };

        let w = terrain_gpu.size.width;
        let h = terrain_gpu.size.height;
        if w == 0 || h == 0 {
            return Ok(());
        }

        let dx = w.div_ceil(WORKGROUP);
        let dy = h.div_ceil(WORKGROUP);

        let mut pass = render_ctx
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_pipeline(pl);
        pass.set_bind_group(0, bind_group, &[]);
        pass.dispatch_workgroups(dx, dy, 1);
        self.last_executed_commit_stamp.store(
            dispatch.commit_stamp,
            std::sync::atomic::Ordering::Relaxed,
        );
        MINIMAP_GPU_EXECUTE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }
}
