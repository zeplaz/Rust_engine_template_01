// World fire particle instancing — reads registry-backed `GpuParticleInstance` rows
// and expands each into four billboard vertices for `WorldFireFx` draw.
// Asset path: `shaders/fire/fire_particle.wgsl` (`FIRE_PARTICLE_WGSL`).

struct ParticleDrawUniforms {
    instance_count: u32,
    max_instances: u32,
    time_secs: f32,
    _pad: f32,
}

struct GpuParticleInstance {
    world_xyz_heat: vec4<f32>,
    ember_class_radius_smoke: vec4<f32>,
}

struct GpuParticleQuadVertex {
    world_xy_heat_ember: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: ParticleDrawUniforms;
@group(1) @binding(0) var<storage, read> instances: array<GpuParticleInstance>;
@group(2) @binding(0) var<storage, read_write> expanded: array<GpuParticleQuadVertex>;

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
    let radius = row.ember_class_radius_smoke.z;
    let smoke = row.ember_class_radius_smoke.w;
    let t = params.time_secs;
    let pulse = sin(t * 6.28318 + heat * 12.0) * 0.5 + 0.5;
    let world = row.world_xyz_heat.xyz;
    let half = max(radius * (0.25 + pulse * 0.15), 0.5);
    let ember_scale = ember * (0.5 + class_id * 0.5);
    let heat_scale = heat * (0.35 + smoke * 0.65);
    let base = i * 4u;
    expanded[base + 0u] = GpuParticleQuadVertex(vec4(world.x - half, world.y - half, heat_scale, ember_scale));
    expanded[base + 1u] = GpuParticleQuadVertex(vec4(world.x + half, world.y - half, heat_scale, ember_scale));
    expanded[base + 2u] = GpuParticleQuadVertex(vec4(world.x + half, world.y + half, heat_scale, ember_scale));
    expanded[base + 3u] = GpuParticleQuadVertex(vec4(world.x - half, world.y + half, heat_scale, ember_scale));
}
