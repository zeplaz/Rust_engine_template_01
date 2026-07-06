// View globals — `view_proj` from ExtractedCameraMetrics (RTT-B5); raster sync uses metrics, not camera query.
struct Globals {
    view_proj: mat4x4<f32>,
    vertex_count: u32,
    time_secs: f32,
    zoom_alpha: f32,
    _pad: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;

struct GpuWaterParticleQuadVertex {
    world_xy_profile_phase: vec4<f32>,
    uv_stretch_twinkle: vec4<f32>,
}

@group(1) @binding(0) var<storage, read> expanded: array<GpuWaterParticleQuadVertex>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) profile: f32,
    @location(2) phase: f32,
    @location(3) world_xy: vec2<f32>,
    @location(4) stretch: f32,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let quad = vertex_index / 6u;
    let tri_corner = vertex_index % 6u;
    let corner_idx = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u)[tri_corner];
    let row = expanded[quad * 4u + corner_idx];
    let world = row.world_xy_profile_phase.xy;
    let uvs = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );
    var out: VertexOutput;
    out.clip_position = globals.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.uv = uvs[corner_idx];
    out.profile = row.world_xy_profile_phase.z;
    out.phase = row.world_xy_profile_phase.w;
    out.world_xy = world;
    out.stretch = row.uv_stretch_twinkle.z;
    return out;
}

const GLINT_CYAN: vec3<f32> = vec3<f32>(0.369, 0.878, 0.863);
const TEAL_EDGE: vec3<f32> = vec3<f32>(0.290, 0.471, 0.471);
const FOAM_ARCHIVAL: vec3<f32> = vec3<f32>(0.784, 0.722, 0.596);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv - vec2<f32>(0.5, 0.5);
    if in.profile > 0.5 && in.profile < 1.5 {
        uv.x *= in.stretch;
    }
    let d = length(uv);
    let core = 1.0 - smoothstep(0.015, 0.09, d);
    let pin = step(d, 0.07);

    let tw_x = sin(in.world_xy.x * 0.22 + globals.time_secs * 0.9) * 0.5 + 0.5;
    let tw_y = cos(in.world_xy.y * 0.19 + globals.time_secs * 0.7) * 0.5 + 0.5;
    let twinkle = tw_x * tw_y;

    var tint = GLINT_CYAN;
    var base_alpha = 0.28;
    if in.profile < 0.5 {
        tint = GLINT_CYAN;
        base_alpha = 0.20 + twinkle * 0.20;
    } else if in.profile < 1.5 {
        tint = TEAL_EDGE;
        base_alpha = 0.32 + twinkle * 0.12;
    } else if in.profile < 2.5 {
        tint = FOAM_ARCHIVAL;
        base_alpha = 0.50;
    } else {
        tint = FOAM_ARCHIVAL;
        base_alpha = 0.45;
    }

    let za = clamp(globals.zoom_alpha, 0.0, 1.0);
    let zoom_fade = za * za * (0.2 + 0.8 * za);

    let alpha = (core * base_alpha + pin * 0.12) * zoom_fade;
    let hot = select(0.0, 1.0, in.profile < 0.5);
    let rgb = tint * (1.0 + hot * pin * 0.8 + twinkle * 0.15);
    return vec4<f32>(rgb * alpha, alpha);
}
