PUBufferRegistry + ProjectionGraph Architecture Spec
Stage-5 spine (fully typed, registry-owned, render + compute unified)

This is the concrete architecture target replacing:

ad-hoc GPU uploads
per-domain allocators
fake placeholder GPU paths
fire-only rendering assumptions
render-only GPU ownership

The GPU becomes a shared execution fabric for:

render
particles
atmosphere
AI
pathfinding assists
sensor fields
logistics
future compute systems
1. Global Pipeline
ECS Sim
    ->
Snapshot Extract
    ->
WorldRepresentationResolver
    ->
RepresentationResult
    ->
ProjectionGraph
    ->
GPUBufferRegistry
    ->
Render + Compute Consumers
2. Core Design Rules
RULE 1 — Projection owns GPU visibility

Simulation does NOT decide GPU rows.

ProjectionGraph decides:

what becomes GPU-visible
density
cadence
aggregation
precision
representation class
RULE 2 — Registry owns memory

No subsystem allocates GPU buffers directly.

Forbidden:

FireGpuStorage
AtmosphereGpuStorage
AiGpuStorage
ProjectileGpuStorage

Allowed:

GPUBufferRegistry
    -> typed slices/views
RULE 3 — Snapshots only

GPU upload NEVER reads gameplay ECS directly.

Only reads:

committed snapshots
projection outputs
RULE 4 — Compute and render share projections

Atmosphere compute and fire rendering may consume:

same snapshot
same projected field
same GPU slice

without duplicate uploads.

3. Module Layout
src/render/
    gpu/
        registry/
            mod.rs
            buffer_registry.rs
            allocation.rs
            slice.rs
            layout.rs
            upload.rs
            metrics.rs

        projection/
            mod.rs
            projection_graph.rs
            projection_node.rs
            projection_context.rs
            projection_outputs.rs

            fire_projection.rs
            atmosphere_projection.rs
            ai_projection.rs
            overlay_projection.rs
            particle_projection.rs

        snapshots/
            frame_snapshot.rs
            snapshot_registry.rs

        compute/
            dispatch_graph.rs
            compute_context.rs
            compute_jobs.rs

        lod/
            representation_resolver.rs
            representation_band.rs
            lod_zone.rs
4. Snapshot Layer
ExtractFrameSnapshot
pub trait ExtractFrameSnapshot: Send + Sync + 'static {
    type Source;

    fn extract(
        source: &Self::Source,
        stamp: SimStepStamp,
    ) -> Self;
}
Example — FireVisualFrame
pub struct FireVisualFrame {
    pub stamp: SimStepStamp,

    pub instances: Vec<FireVisualInstance>,

    pub chunk_heat: Vec<ChunkHeatEntry>,

    pub aggregate: FireAtmosphereAggregate,
}
Snapshot Registry
pub struct FrameSnapshotRegistry {
    snapshots: TypeIdMap<Box<dyn Any + Send + Sync>>,
}
5. Representation Resolver
RepresentationInputs
pub struct RepresentationInputs {
    pub camera: CameraVisualState,

    pub zones: LodZoneRegistry,

    pub budgets: VisualBudgetSettings,

    pub cadence: VisualCadence,

    pub stamp: SimStepStamp,
}
RepresentationBand

Generic, NOT fire-specific.

pub enum RepresentationBand {
    Full,
    Tactical,
    Strategic,
    OverlayOnly,
    Dormant,
}
RepresentationResult
pub struct RepresentationResult {
    pub active_band: RepresentationBand,

    pub extract_plan: WorldRepresentationExtractPlan,

    pub overlay_policy: OverlayPolicy,

    pub gpu_budget: GpuBudgetPolicy,

    pub compute_budget: ComputeBudgetPolicy,

    pub cadence_policy: CadencePolicy,
}
6. ProjectionGraph

ProjectionGraph converts snapshots into GPU-ready representations.

Core Projection Trait
pub trait ProjectionNode: Send + Sync + 'static {
    type Input;
    type Output;

    fn project(
        &mut self,
        input: &Self::Input,
        ctx: &ProjectionContext,
    ) -> Self::Output;
}
ProjectionContext
pub struct ProjectionContext<'a> {
    pub representation: &'a RepresentationResult,

    pub registry: &'a mut GPUBufferRegistry,

    pub metrics: &'a mut GpuRepresentationMetrics,

    pub stamp: SimStepStamp,
}
ProjectionGraph
pub struct ProjectionGraph {
    fire: FireProjectionNode,

    atmosphere: AtmosphereProjectionNode,

    overlays: OverlayProjectionNode,

    particles: ParticleProjectionNode,

    ai: AiProjectionNode,
}
7. GPUBufferRegistry

Single authoritative GPU allocator + uploader.

GPUBufferHandle
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GPUBufferHandle(pub u32);
GPUBufferClass
pub enum GPUBufferClass {
    Vertex,
    Instance,
    Uniform,
    Storage,
    Indirect,
    ComputeField,
}
Buffer Layout Descriptor
pub struct GPUBufferLayout {
    pub label: Cow<'static, str>,

    pub stride: u64,

    pub usage: BufferUsages,

    pub class: GPUBufferClass,
}
Registry Allocation
pub struct GPUBufferAllocation {
    pub handle: GPUBufferHandle,

    pub offset_bytes: u64,

    pub size_bytes: u64,

    pub reserved_capacity: u32,

    pub active_capacity: u32,
}
Registry
pub struct GPUBufferRegistry {
    buffers: HashMap<GPUBufferHandle, RegistryBuffer>,
}
RegistryBuffer
pub struct RegistryBuffer {
    pub layout: GPUBufferLayout,

    pub gpu_buffer: Buffer,

    pub reserved_bytes: u64,

    pub active_bytes: u64,

    pub high_watermark: u64,
}
8. Typed Buffer Views

Never expose raw offsets to gameplay systems.

Typed Slice
pub struct GPUSlice<T> {
    pub allocation: GPUBufferAllocation,

    marker: PhantomData<T>,
}
Example
pub type FireInstanceSlice = GPUSlice<FireGpuInstance>;

pub type AtmosphereFieldSlice = GPUSlice<AtmosphereGpuCell>;
9. Upload API
Upload Request
pub struct GPUUploadRequest<T> {
    pub target: GPUSlice<T>,

    pub data: Vec<T>,
}
Registry Upload
impl GPUBufferRegistry {
    pub fn upload<T: Pod>(
        &mut self,
        queue: &RenderQueue,
        request: GPUUploadRequest<T>,
    ) {
        // mapped/staging upload
    }
}
10. Projection Outputs

Projection outputs are GPU-ready slices.

FireProjectionOutput
pub struct FireProjectionOutput {
    pub instances: FireInstanceSlice,

    pub indirect_args: Option<IndirectDrawSlice>,

    pub dispatch_groups: u32,
}
OverlayProjectionOutput
pub struct OverlayProjectionOutput {
    pub heat_field: GPUSlice<OverlayHeatCell>,

    pub smoke_field: GPUSlice<OverlaySmokeCell>,
}
11. Compute Dispatch Graph

Render and compute unified.

Compute Job
pub trait ComputeJob {
    fn dispatch(
        &self,
        ctx: &mut ComputeContext,
    );
}
ComputeContext
pub struct ComputeContext<'a> {
    pub registry: &'a GPUBufferRegistry,

    pub device: &'a RenderDevice,

    pub queue: &'a RenderQueue,

    pub representation: &'a RepresentationResult,
}
Example Jobs
AtmosphereAdvectJob
AiInfluenceJob
PathfindingFieldJob
SensorVisibilityJob
ProjectilePredictionJob
12. LOD-aware Capacity

Critical for performance stability.

Required Pattern

DO:

reserved_capacity >= active_capacity

Avoid reallocating every frame.

Example
pub struct CapacityClass {
    pub reserved_rows: u32,

    pub active_rows: u32,

    pub high_watermark: u32,
}
13. Zones

Zones are representation influence volumes.

NOT camera-only.

LodZone
pub struct LodZone {
    pub class: LodZoneClass,

    pub center: Vec3,

    pub inner_radius: f32,

    pub outer_radius: f32,

    pub min_band: RepresentationBand,

    pub max_band: RepresentationBand,

    pub priority: u32,
}
LodZoneClass
pub enum LodZoneClass {
    Camera,
    Combat,
    Projectile,
    Mission,
    Sensor,
    Editor,
    JumpPoint,
}
14. Projection Example (Fire)
FireProjectionNode
impl ProjectionNode for FireProjectionNode {
    type Input = FireVisualFrame;

    type Output = FireProjectionOutput;

    fn project(
        &mut self,
        input: &Self::Input,
        ctx: &ProjectionContext,
    ) -> Self::Output {
        let band = ctx.representation.active_band;

        let filtered = match band {
            RepresentationBand::Full => {
                full_density(&input.instances)
            }

            RepresentationBand::Strategic => {
                clustered_density(&input.instances)
            }

            RepresentationBand::OverlayOnly => {
                Vec::new()
            }

            _ => Vec::new(),
        };

        let slice = ctx.registry.allocate_slice::<FireGpuInstance>(
            filtered.len() as u32,
        );

        ctx.registry.upload(
            ctx.queue,
            GPUUploadRequest {
                target: slice,
                data: filtered,
            },
        );

        FireProjectionOutput {
            instances: slice,
            indirect_args: None,
            dispatch_groups: 0,
        }
    }
}
15. Metrics
GpuRepresentationMetrics
pub struct GpuRepresentationMetrics {
    pub active_band: RepresentationBand,

    pub upload_bytes: u64,

    pub instance_rows: u32,

    pub dispatch_count: u32,

    pub active_allocations: u32,
}
HUD Example
REP: Strategic
Rows: 24k
GPU: 3.1 MB
Dispatch: 4
Alloc: 11
16. Forbidden Architecture
NEVER DO
Fire ECS -> FireGpuStorage
Atmosphere ECS -> AtmosphereGpuStorage
AI ECS -> AiGpuStorage
NEVER DO
Render systems querying gameplay ECS directly
NEVER DO
Per-domain allocators with independent resize logic
NEVER DO
"temporary" preview-only upload paths