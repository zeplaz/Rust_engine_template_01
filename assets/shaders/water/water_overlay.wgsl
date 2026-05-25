// FX-WATER-SHADER-001 — instanced river ribbon + lake/ocean motion overlay (W1).

struct Globals {
    view_proj: mat4x4<f32>,
    instance_count: u32,
    time_secs: f32,
    zoom_alpha: f32,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct WaterOverlayGpuInstance {
    world_kind: vec4<f32>,
    flow_extent: vec4<f32>,
    segment_b: vec4<f32>,
}

@group(1) @binding(0)
var<storage, read> instances: array<WaterOverlayGpuInstance>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) kind: f32,
    @location(2) flow: vec2<f32>,
    @location(3) world_xy: vec2<f32>,
}

const WATER_RIVER_DEEP: vec3<f32> = vec3<f32>(0.118, 0.271, 0.267);
const WATER_TEAL_EDGE: vec3<f32> = vec3<f32>(0.290, 0.471, 0.471);
const WATER_TEAL: vec3<f32> = vec3<f32>(0.165, 0.353, 0.345);
const WATER_OCEAN_DEEP: vec3<f32> = vec3<f32>(0.059, 0.157, 0.157);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let inst = vertex_index / 6u;
    let tri = vertex_index % 6u;
    let corner = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u)[tri];
    let row = instances[inst];
    let start = row.world_kind.xy;
    let end = row.segment_b.xy;
    let kind = row.world_kind.z;
    let flow = row.flow_extent.xy;
    let half_w = row.flow_extent.z;
    let seg_len = max(row.flow_extent.w, 0.25);
    let dir = normalize(end - start + vec2<f32>(0.0001, 0.0));
    let cross = vec2<f32>(-dir.y, dir.x);
    let uvs = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );
    let uv = uvs[corner];
    let along = uv.x * seg_len;
    let across = (uv.y - 0.5) * half_w * 2.0;
    let world = start + dir * along + cross * across;
    var out: VertexOutput;
    out.clip_position = globals.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.uv = uv;
    out.kind = kind;
    out.flow = flow;
    out.world_xy = world;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = globals.time_secs;
    var alpha = 0.35;
    var tint = WATER_TEAL;
    const HAZE: vec3<f32> = vec3<f32>(0.024, 0.031, 0.031);

    let strategic = globals.zoom_alpha < 0.35;

    if in.kind > 1.5 && in.kind < 2.5 {
        let ribbon = 1.0 - abs(in.uv.y - 0.5) * 2.0;
        let along = dot(in.world_xy, normalize(in.flow + vec2<f32>(0.0001, 0.0)));
        let scroll_hz = select(0.35, 0.62, strategic);
        let scroll = sin(along * 0.35 * scroll_hz + t * scroll_hz) * 0.5 + 0.5;
        tint = mix(WATER_RIVER_DEEP, WATER_TEAL_EDGE, ribbon);
        alpha = 0.28 + ribbon * 0.32 + scroll * 0.18;
        if strategic {
            alpha = alpha * 1.28 + 0.14;
        }
    } else if in.kind > 0.5 && in.kind < 1.5 {
        let ripple = sin(t * 0.6 + in.world_xy.x * 1.1 + in.world_xy.y * 0.9) * 0.5 + 0.5;
        alpha = 0.12 + ripple * 0.12;
        if strategic {
            alpha *= 0.5;
        }
        tint = WATER_TEAL;
    } else if in.kind > 2.5 {
        let swell = sin(in.world_xy.x * 0.08 + t * 0.02) * 0.5
            + cos(in.world_xy.y * 0.06 + t * 0.018) * 0.5;
        let d = distance(in.uv, vec2<f32>(0.5, 0.5));
        let haze = smoothstep(0.6, 1.0, d);
        tint = mix(WATER_OCEAN_DEEP, HAZE, haze * 0.35);
        alpha = 0.18 + swell * 0.15;
    }
    alpha *= clamp(globals.zoom_alpha * 0.6 + 0.4, 0.35, 1.0);
    return vec4<f32>(tint, alpha);
}
