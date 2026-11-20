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
const WATER_SHALLOW: vec3<f32> = vec3<f32>(0.090, 0.300, 0.600);
const FOAM: vec3<f32> = vec3<f32>(0.86, 0.92, 0.97);
const WATER_ALPHA_BASE: f32 = 0.70;
const WATER_ALPHA_DEPTH: f32 = 0.20;
const WATER_ALPHA_MAX: f32 = 0.90;

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
    // Fixed-direction phase (see water_field_3d.wesl): normalize(flow) x world coords = moire.
    let ph = dot(in.world_xy, vec2<f32>(0.31, 0.47));
    let ph_aa = clamp(1.0 - fwidth(ph) * 0.6, 0.0, 1.0);
    let scroll_mix = (sin(ph - globals.time_secs * (0.4 + speed * 1.2)) * 0.5 + 0.5) * ph_aa;

    // Shoreline wetting from depth neighborhood gradient.
    let eps = vec2<f32>(1.5) / max(globals.origin_extent.zw, vec2<f32>(1.0));
    let d_e = sample_field(in.uv + vec2<f32>(eps.x, 0.0)).r;
    let d_w = sample_field(in.uv - vec2<f32>(eps.x, 0.0)).r;
    let d_n = sample_field(in.uv + vec2<f32>(0.0, eps.y)).r;
    let d_s = sample_field(in.uv - vec2<f32>(0.0, eps.y)).r;
    let grad = abs(d_e - d_w) + abs(d_n - d_s);
    // Shoreline lip only at the wet edge (thin, shallow water is not 'shore' everywhere).
    let shoreline = smoothstep(0.04, 0.2, grad) * smoothstep(0.08, 0.0, depth01) * 0.6;

    var tint = mix(WATER_SHALLOW, WATER_MID, smoothstep(0.05, 0.35, depth01));
    tint = mix(tint, WATER_DEEP, smoothstep(0.35, 0.85, depth01));
    tint = mix(tint, WATER_SHALLOW, scroll_mix * 0.18 * clamp(speed, 0.0, 1.0));
    // Foam is a broken, moving highlight on blue water, not the water's colour: flow-driven
    // foam is cut into streaks along the flow and capped, so a fast flood reads blue + white
    // streaks instead of a white/grey sheet.
    let sp = dot(in.world_xy, vec2<f32>(0.9, 1.7)) * 1.3;
    // Stripes vanish well before they reach pixel size (they read as zigzag moire on a
    // zoomed-out flood), and foam only marks genuinely turbulent water, not every fast cell.
    let sp_aa = clamp(1.0 - fwidth(sp) * 1.5, 0.0, 1.0);
    let foam_w = smoothstep(0.45, 0.9, foam01);
    let streak = smoothstep(0.55, 0.95, sin(sp + scroll_mix * 3.1416) * 0.5 + 0.5) * sp_aa;
    tint = mix(tint, FOAM, clamp(foam_w * streak * 0.35 + shoreline * 0.4, 0.0, 0.20));

    var alpha = WATER_ALPHA_BASE + depth01 * WATER_ALPHA_DEPTH + foam_w * streak * 0.08 + shoreline * 0.1;
    alpha *= clamp(globals.zoom_alpha * 0.15 + 0.85, 0.85, 1.0); // zoom fade no longer greys the river
    // Soft edge so subsampled ROI does not brick-wall.
    let edge = smoothstep(0.0, 0.04, min(min(in.uv.x, 1.0 - in.uv.x), min(in.uv.y, 1.0 - in.uv.y)));
    alpha *= edge;
    return vec4<f32>(tint, clamp(alpha, 0.0, WATER_ALPHA_MAX));
}
