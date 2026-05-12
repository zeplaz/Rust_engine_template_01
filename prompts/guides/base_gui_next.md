Recommended Direction

Yes — you should absolutely prepare the architecture now for:

real AssetServer shader loading
GPU-driven particles
indirect rendering
field extraction
future Hanabi/custom compute migration

But:

do NOT jump directly into fully GPU-driven simulation yet
first establish stable interfaces + ownership boundaries

The key is:

simulation backend API stability first
GPU implementation second
Stage Plan
Stage 1 — CPU Simulation + GPU Rendering

Best current target.

Simulation:

CPU chunk fields
ecology
smoke density
fire heat
fuel

Rendering:

GPU particles
WGSL materials
fullscreen fog
instanced sprites
indirect smoke quads later

This gives:

debuggability
determinism
easier editor tooling
easier save/load
easier replay systems
Stage 2 — Extraction Layer

You need this NOW.

Do not let render systems query simulation state directly forever.

Instead:

Simulation ECS
    ↓
Extraction Systems
    ↓
Render Resources
    ↓
GPU
Recommended Extraction Resources
#[derive(Resource, Default)]
pub struct FireVisualField {
    pub emitters: Vec<FireEmitterGpu>,
}

#[derive(Resource, Default)]
pub struct SmokeVisualField {
    pub cells: Vec<SmokeCellGpu>,
}

#[derive(Resource, Default)]
pub struct AtmosphereVisualField {
    pub fog_density: Vec<f32>,
}

Simulation publishes into these.

**Template repo:** `SimFireEmitterVisualExtract` / `SimChunkSmokeVisualExtract` in `src/render/sim_visual_extract.rs` are filled by `publish_sim_visual_extract` (`AtmospherePipelineSet::VisualExtract` in `src/systems/atmosphere/visual_extract.rs`). See also `base_fire2_smoke.md` §18–19.

Rendering ONLY reads these.

This mirrors Bevy render extraction architecture.

Stage 3 — Real Shader Asset Loading

YES.

Do not inline WGSL strings long-term.

Use:

pub const FIRE_SHADER_PATH: &str =
    "shaders/fire/fire_particle.wgsl";

pub const SMOKE_SHADER_PATH: &str =
    "shaders/fire/smoke_volume.wgsl";

pub const HEAT_DISTORTION_SHADER_PATH: &str =
    "shaders/post/heat_distortion.wgsl";

Then:

#[derive(Resource)]
pub struct FireShaders {
    pub fire_particle: Handle<Shader>,
    pub smoke_volume: Handle<Shader>,
}

Load:

fn load_fire_shaders(
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    commands.insert_resource(FireShaders {
        fire_particle: assets.load(FIRE_SHADER_PATH),
        smoke_volume: assets.load(SMOKE_SHADER_PATH),
    });
}
Recommended Asset Layout
assets/
└── shaders/
    ├── fire/
    │   ├── fire_particle.wgsl
    │   ├── ember_particle.wgsl
    │   ├── smoke_volume.wgsl
    │   ├── heat_distortion.wgsl
    │   └── fire_light.wgsl
    │
    ├── atmosphere/
    │   ├── volumetric_fog.wgsl
    │   ├── cloud_layer.wgsl
    │   └── aerial_scatter.wgsl
    │
    ├── terrain/
    │   ├── terrain_blend.wgsl
    │   ├── scorch_overlay.wgsl
    │   └── wetness_overlay.wgsl
    │
    └── post/
        ├── hdr_composite.wgsl
        ├── thermal_vision.wgsl
        └── smoke_occlusion.wgsl
Strong Recommendation — Build Your Own GPU Interface Layer

Even if using Hanabi later.

Do NOT tightly couple gameplay systems to Hanabi APIs.

Bad:

commands.spawn(HanabiEffectBundle { ... });

inside gameplay systems.

Good:

#[derive(Event)]
pub struct SpawnFireEmitterEvent {
    pub position: Vec3,
    pub intensity: f32,
    pub fuel_type: FuelType,
}

Then:

fn fire_particles_bridge()

converts:

events
→ Hanabi emitters
or
→ custom GPU buffers
or
→ indirect draws

This keeps render backend replaceable.

Very important long-term.

Hanabi Recommendation

Hanabi is excellent for:

prototyping
editor workflows
stylized particles
medium-scale effects
effect authoring

But eventually:

giant wildfire smoke
atmospheric fields
battlefield smoke
embers at scale

will likely exceed particle ECS approaches.

At that point:

indirect rendering
compute simulation
sparse field rendering

become better.

So:

Hanabi = transitional render layer

not final architecture.

Recommended Particle Architecture
ECS Owns
#[derive(Component)]
pub struct FireEmitter {
    pub intensity: f32,
    pub radius: f32,
    pub smoke_rate: f32,
    pub ember_rate: f32,
}
Extraction Builds GPU Buffers
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FireEmitterGpu {
    pub position: Vec4,
    pub params: Vec4,
}
Render Pass Consumes Buffer
Emitter Buffer
    ↓
Compute Update
    ↓
Particle Buffer
    ↓
Indirect Draw

This is the scalable architecture.

IMPORTANT — Smoke Rendering

Do NOT rely solely on particles.

You want:

Layered Smoke Representation
1. Smoke Field (simulation)
SmokeDensityGrid

used for:

LOS
AI
gameplay
IR masking
2. Volume Fog Rendering

fullscreen or clustered fog:

cheap
scalable
atmospheric
3. Local Particle Plumes

visual richness only:

embers
curls
sparks
turbulence

This combination scales MUCH better.

Recommended Render Pipeline Stack
Terrain Pass
    ↓
Decal/Scorch Pass
    ↓
Fire Light Injection
    ↓
Smoke Volume Pass
    ↓
Particle Pass
    ↓
Heat Distortion
    ↓
Post Processing
Very Important Future Optimization

DO NOT SPAWN/DESPAWN PARTICLES CONSTANTLY.

Use:

pools
persistent GPU buffers
emitter reuse
alive flags

Especially for:

embers
smoke
ash

Otherwise allocator churn becomes brutal.

Recommended ECS Boundary
Simulation ECS

Owns:

fire state
fuel state
smoke density
ecology
weather
Render ECS

Owns:

emitter visuals
sprite instances
fog visuals
scorch decals
lighting
GPU Backend

Owns:

particle buffers
compute pipelines
indirect draw commands
volumetric accumulation
Best Long-Term Direction For Your Engine

You are building something closer to:

simulation platform

than:

normal game

So eventually your architecture should resemble:

simulation backend
orchestration ECS
extraction pipelines
GPU rendering backend
streaming world system

rather than:

gameplay scripts controlling effects directly