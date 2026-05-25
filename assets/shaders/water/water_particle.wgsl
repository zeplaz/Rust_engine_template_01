// FX-WATER-PARTICLE-001 — expand water particle instances to billboard quads (fire spine clone).
// D-W05 A: ≤2px half-edge; D-W07 A: river streak elongation along flow_dir.

struct ParticleDrawUniforms {
    instance_count: u32,
    max_instances: u32,
    time_secs: f32,
    camera_zoom: f32,
    zoom_alpha: f32,
    _pad: f32,
}

struct GpuWaterParticleInstance {
    world_xyz_profile: vec4<f32>,
    flow_half_twinkle: vec4<f32>,
}

struct GpuWaterParticleQuadVertex {
    world_xy_profile_phase: vec4<f32>,
    uv_stretch_twinkle: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: ParticleDrawUniforms;
@group(1) @binding(0) var<storage, read> instances: array<GpuWaterParticleInstance>;
@group(2) @binding(0) var<storage, read_write> expanded: array<GpuWaterParticleQuadVertex>;

@compute @workgroup_size(64, 1, 1)
fn expand_instances(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= params.instance_count {
        return;
    }
    let row = instances[i];
    let profile = row.world_xyz_profile.z;
    let phase = row.world_xyz_profile.w;
    let flow = row.flow_half_twinkle.xy;
    let base_half = row.flow_half_twinkle.z;
    let stretch = max(row.flow_half_twinkle.w, 1.0);
    let z = max(params.camera_zoom, 0.06);
    let za = clamp(params.zoom_alpha, 0.0, 1.0);

    var half = base_half * (0.82 + za * 0.18);
    half = min(half, 2.0 / z);
    half = max(half, 0.25 / z);

    var dir = flow;
    if dot(dir, dir) < 1e-6 {
        dir = vec2<f32>(1.0, 0.0);
    } else {
        dir = normalize(dir);
    }
    let cross = vec2<f32>(-dir.y, dir.x);

    var half_along = half;
    var half_cross = half;
    if profile > 0.5 && profile < 1.5 {
        half_along = half * stretch;
        half_cross = half * 0.35;
    } else if profile > 1.5 && profile < 2.5 {
        half_along = half * 1.2;
        half_cross = half * 0.55;
    } else if profile > 2.5 {
        half_along = half * 0.9;
        half_cross = half * 0.45;
    }

    let world = row.world_xyz_profile.xy;
    let base = i * 4u;
    expanded[base + 0u] = GpuWaterParticleQuadVertex(
        vec4(world - cross * half_cross - dir * half_along, profile, phase),
        vec4(0.0, 0.0, stretch, row.flow_half_twinkle.w),
    );
    expanded[base + 1u] = GpuWaterParticleQuadVertex(
        vec4(world + cross * half_cross - dir * half_along, profile, phase),
        vec4(1.0, 0.0, stretch, row.flow_half_twinkle.w),
    );
    expanded[base + 2u] = GpuWaterParticleQuadVertex(
        vec4(world + cross * half_cross + dir * half_along, profile, phase),
        vec4(1.0, 1.0, stretch, row.flow_half_twinkle.w),
    );
    expanded[base + 3u] = GpuWaterParticleQuadVertex(
        vec4(world - cross * half_cross + dir * half_along, profile, phase),
        vec4(0.0, 1.0, stretch, row.flow_half_twinkle.w),
    );
}
