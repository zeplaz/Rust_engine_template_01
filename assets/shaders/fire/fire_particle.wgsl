// World fire particle instancing — reads registry-backed `GpuParticleInstance` rows
// and expands each into four billboard vertices for `WorldFireFx` draw.
// Phase A (FX-FIRE-SPARK-001): tiny spark half-edges; twinkle in draw fragment shader.
// Asset path: `shaders/fire/fire_particle.wgsl` (`FIRE_PARTICLE_WGSL`).
//
// Rust backend: `GpuInstancedQuadInstance` / fire alias `GpuParticleInstance` — same 32-byte stride.

struct ParticleDrawUniforms {
    instance_count: u32,
    max_instances: u32,
    time_secs: f32,
    camera_zoom: f32,
    zoom_alpha: f32,
    // FireSparkDrawExtension — separate bind when a second consumer lands.
    spark_sim_enabled: f32,
}

struct SparkSimState {
    pos: vec4<f32>,
    vel: vec4<f32>,
}

struct GpuParticleInstance {
    world_xyz_heat: vec4<f32>,
    /// `.z` is **world half-edge base** for billboard expansion (not light falloff radius).
    ember_class_radius_smoke: vec4<f32>,
}

struct GpuParticleQuadVertex {
    world_xy_heat_ember: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: ParticleDrawUniforms;
@group(1) @binding(0) var<storage, read> instances: array<GpuParticleInstance>;
@group(2) @binding(0) var<storage, read_write> expanded: array<GpuParticleQuadVertex>;
@group(3) @binding(0) var<storage, read> spark_state: array<SparkSimState>;

@compute @workgroup_size(64, 1, 1)
fn expand_instances(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.instance_count) {
        return;
    }
    let row = instances[i];
    let heat = row.world_xyz_heat.w;
    let ember = row.ember_class_radius_smoke.x;
    let class_id = row.ember_class_radius_smoke.y;
    let base_half = row.ember_class_radius_smoke.z;
    let smoke = row.ember_class_radius_smoke.w;
    let za = clamp(params.zoom_alpha, 0.0, 1.0);
    var world = row.world_xyz_heat.xyz;
    if params.spark_sim_enabled > 0.5 {
        world = spark_state[i].pos.xyz;
    }
    var half = base_half * (0.82 + heat * 0.18);
    half = half * (0.32 + za * 0.68);
    half = clamp(half, 0.015, 1.5);
    if class_id > 0.5 {
        half = half * 0.72;
    }
    let ember_scale = ember * (0.45 + class_id * 0.35);
    let heat_scale = heat * (0.38 + smoke * 0.52);
    let base = i * 4u;
    expanded[base + 0u] = GpuParticleQuadVertex(vec4(world.x - half, world.y - half, heat_scale, ember_scale));
    expanded[base + 1u] = GpuParticleQuadVertex(vec4(world.x + half, world.y - half, heat_scale, ember_scale));
    expanded[base + 2u] = GpuParticleQuadVertex(vec4(world.x + half, world.y + half, heat_scale, ember_scale));
    expanded[base + 3u] = GpuParticleQuadVertex(vec4(world.x - half, world.y + half, heat_scale, ember_scale));
}
