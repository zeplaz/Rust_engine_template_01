// DF-3 — GPU surface-water field (depth + flow + foam). Samples sim-packed Rgba8.

struct FieldGlobals {
    view_proj: mat4x4<f32>,
    // xy = world origin (tiles), zw = world size (tiles)
    origin_extent: vec4<f32>,
    time_secs: f32,
    zoom_alpha: f32,
    wet_texels: u32,
    _pad: u32,
}

@group(0) @binding(0)
var<uniform> globals: FieldGlobals;

@group(0) @binding(1)
var field_tex: texture_2d<f32>;

@group(0) @binding(2)
var field_samp: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_xy: vec2<f32>,
}

// ONE water look (flat + voxel share this block verbatim; test water_look_ramps_match).
// Blue, semi-transparent: the bed shows through shallows, deep water reads darker blue.
const WATER_DEEP: vec3<f32> = vec3<f32>(0.040, 0.140, 0.360);
const WATER_MID: vec3<f32> = vec3<f32>(0.090, 0.270, 0.560);
const WATER_SHALLOW: vec3<f32> = vec3<f32>(0.140, 0.380, 0.680);
const FOAM: vec3<f32> = vec3<f32>(0.86, 0.92, 0.97);
const WATER_ALPHA_BASE: f32 = 0.52;
const WATER_ALPHA_DEPTH: f32 = 0.33;
const WATER_ALPHA_MAX: f32 = 0.86;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let corner = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u)[vertex_index % 6u];
    let uvs = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );
    let uv = uvs[corner];
    let origin = globals.origin_extent.xy;
    let extent = max(globals.origin_extent.zw, vec2<f32>(1.0, 1.0));
    let world = origin + uv * extent;
    var out: VertexOutput;
    out.clip_position = globals.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.uv = uv;
    out.world_xy = world;
    return out;
}

fn sample_field(uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(field_tex, field_samp, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)), 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = sample_field(in.uv);
    let depth01 = texel.r;
    if depth01 < 0.02 {
        return vec4<f32>(0.0);
    }

    // Flow encoded as signed in G/B (0.5 = still).
    let flow = (texel.gb - vec2<f32>(0.5)) * 2.0;
    let foam01 = texel.a;
    let speed = length(flow);

    // Flow-map scroll: offset UV along velocity for streaking.
    let scroll_uv = in.uv - flow * globals.time_secs * 0.035;
    let scrolled = sample_field(scroll_uv);
    let scroll_mix = sin(dot(in.world_xy, normalize(flow + vec2<f32>(0.001, 0.0))) * 0.55
        + globals.time_secs * (0.4 + speed * 1.2)) * 0.5 + 0.5;

    // Shoreline wetting from depth neighborhood gradient.
    let eps = vec2<f32>(1.5) / max(globals.origin_extent.zw, vec2<f32>(1.0));
    let d_e = sample_field(in.uv + vec2<f32>(eps.x, 0.0)).r;
    let d_w = sample_field(in.uv - vec2<f32>(eps.x, 0.0)).r;
    let d_n = sample_field(in.uv + vec2<f32>(0.0, eps.y)).r;
    let d_s = sample_field(in.uv - vec2<f32>(0.0, eps.y)).r;
    let grad = abs(d_e - d_w) + abs(d_n - d_s);
    let shoreline = smoothstep(0.02, 0.18, grad) * smoothstep(0.25, 0.02, depth01);

    var tint = mix(WATER_SHALLOW, WATER_MID, smoothstep(0.05, 0.35, depth01));
    tint = mix(tint, WATER_DEEP, smoothstep(0.35, 0.85, depth01));
    tint = mix(tint, WATER_SHALLOW, scroll_mix * 0.18 * clamp(speed, 0.0, 1.0));
    tint = mix(tint, FOAM, clamp(foam01 * 0.55 + shoreline * 0.65, 0.0, 0.75));

    var alpha = WATER_ALPHA_BASE + depth01 * WATER_ALPHA_DEPTH + foam01 * 0.12 + shoreline * 0.2;
    alpha *= clamp(globals.zoom_alpha * 0.55 + 0.45, 0.4, 1.0);
    // Soft edge so subsampled ROI does not brick-wall.
    let edge = smoothstep(0.0, 0.04, min(min(in.uv.x, 1.0 - in.uv.x), min(in.uv.y, 1.0 - in.uv.y)));
    alpha *= edge;
    return vec4<f32>(tint, clamp(alpha, 0.0, WATER_ALPHA_MAX));
}
