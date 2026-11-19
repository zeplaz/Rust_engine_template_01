// Bevy 0.20 WESL (imports via `import`; material group 3).
// PixelWorld3D water — GPU-D: VS displace from DF-3 field + bed texture; FS owns look.
// DF-3 field: R=depth01, GB=flow, A=foam. Bed is a separate height01 texture.

import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_functions::{get_world_from_local, mesh_position_local_to_clip};
import bevy_pbr::render::mesh_view_bindings::view;

struct WaterField3dUniform {
    time_secs: f32,
    zoom_alpha: f32,
    height_scale: f32,
    lift: f32,
    // xy = origin tiles, zw = extent tiles
    origin_extent: vec4<f32>,
    // xy = bed_min_h, bed_range (world Y units)
    bed_range: vec4<f32>,
}

@group(3) @binding(0)
var<uniform> material: WaterField3dUniform;
@group(3) @binding(1)
var field_tex: texture_2d<f32>;
@group(3) @binding(2)
var field_samp: sampler;
@group(3) @binding(3)
var bed_tex: texture_2d<f32>;
@group(3) @binding(4)
var bed_samp: sampler;

const WATER_DEEP: vec3<f32> = vec3<f32>(0.059, 0.157, 0.180);
const WATER_MID: vec3<f32> = vec3<f32>(0.118, 0.271, 0.267);
const WATER_SHALLOW: vec3<f32> = vec3<f32>(0.290, 0.471, 0.471);
const FOAM: vec3<f32> = vec3<f32>(0.86, 0.92, 0.95);

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

fn sample_field(uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(field_tex, field_samp, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)), 0.0);
}

fn sample_bed_y(uv: vec2<f32>) -> f32 {
    let bed01 = textureSampleLevel(bed_tex, bed_samp, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)), 0.0).r;
    return material.bed_range.x + bed01 * max(material.bed_range.y, 0.0001);
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let uv = vertex.uv;
    let field = sample_field(uv);
    let depth01 = field.r;
    let bed_y = sample_bed_y(uv);
    var local = vertex.position;
    // DF-3 depth01 → metres via height_scale (CPU: depth_m/TILE * PIXEL_WORLD_HEIGHT_SCALE).
    local.y = bed_y + depth01 * material.height_scale + material.lift;

    var model = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(local, 1.0),
    );
    out.position = model;
    out.world_position = get_world_from_local(vertex.instance_index) * vec4<f32>(local, 1.0);
    out.world_normal = vec3<f32>(0.0, 1.0, 0.0);
    out.uv = uv;
    @if(VERTEX_OUTPUT_INSTANCE_INDEX) {
        out.instance_index = vertex.instance_index;
    }
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    let origin = material.origin_extent.xy;
    let extent = max(material.origin_extent.zw, vec2<f32>(1.0));
    let world_xy = vec2<f32>(in.world_position.x, -in.world_position.z);
    if max(uv.x, uv.y) > 1.001 || min(uv.x, uv.y) < -0.001 {
        uv = (world_xy - origin) / extent;
    }

    let texel = sample_field(uv);
    let depth01 = texel.r;
    if depth01 < 0.02 {
        discard;
    }

    let flow = (texel.gb - vec2<f32>(0.5)) * 2.0;
    let foam01 = texel.a;
    let speed = length(flow);
    let scroll_mix = sin(
        dot(world_xy, normalize(flow + vec2<f32>(0.001, 0.0))) * 0.55
            + material.time_secs * (0.4 + speed * 1.2)
    ) * 0.5 + 0.5;

    let eps = vec2<f32>(1.5) / extent;
    let grad = abs(sample_field(uv + vec2<f32>(eps.x, 0.0)).r - sample_field(uv - vec2<f32>(eps.x, 0.0)).r)
        + abs(sample_field(uv + vec2<f32>(0.0, eps.y)).r - sample_field(uv - vec2<f32>(0.0, eps.y)).r);
    let shoreline = smoothstep(0.02, 0.18, grad) * smoothstep(0.25, 0.02, depth01);

    var tint = mix(WATER_SHALLOW, WATER_MID, smoothstep(0.05, 0.35, depth01));
    tint = mix(tint, WATER_DEEP, smoothstep(0.35, 0.85, depth01));
    tint = mix(tint, WATER_SHALLOW, scroll_mix * 0.18 * clamp(speed, 0.0, 1.0));
    tint = mix(tint, FOAM, clamp(foam01 * 0.55 + shoreline * 0.65, 0.0, 0.75));

    var alpha = 0.22 + depth01 * 0.48 + foam01 * 0.12 + shoreline * 0.2;
    alpha *= clamp(material.zoom_alpha * 0.55 + 0.45, 0.4, 1.0);
    return vec4<f32>(tint, clamp(alpha, 0.0, 0.88));
}
