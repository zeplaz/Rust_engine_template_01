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

// ONE water look (flat + voxel share this block verbatim; test water_look_ramps_match).
// Blue, semi-transparent: the bed shows through shallows, deep water reads darker blue.
const WATER_DEEP: vec3<f32> = vec3<f32>(0.040, 0.140, 0.360);
const WATER_MID: vec3<f32> = vec3<f32>(0.090, 0.270, 0.560);
const WATER_SHALLOW: vec3<f32> = vec3<f32>(0.090, 0.300, 0.600);
const FOAM: vec3<f32> = vec3<f32>(0.86, 0.92, 0.97);
const WATER_ALPHA_BASE: f32 = 0.70;
const WATER_ALPHA_DEPTH: f32 = 0.20;
const WATER_ALPHA_MAX: f32 = 0.90;

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
    // DF-3 depth01 → display units via height_scale.
    var y = bed_y + depth01 * material.height_scale + material.lift;
    // Dry vertex: take the highest wet neighbour's surface so the sheet stays flat to the
    // shoreline (the fragment discards dry pixels). Dropping dry vertices to the bed made every
    // wet/dry edge a slope — the V-wedges at the dam and the ROI edges (OP-2/OP-3).
    if depth01 < 0.02 {
        let texel = vec2<f32>(1.0) / vec2<f32>(textureDimensions(field_tex));
        var best = -1.0e9;
        for (var r = 1; r <= 3; r = r + 1) {
            for (var oy = -1; oy <= 1; oy = oy + 1) {
                for (var ox = -1; ox <= 1; ox = ox + 1) {
                    if ox == 0 && oy == 0 {
                        continue;
                    }
                    let nuv = uv + vec2<f32>(f32(ox * r), f32(oy * r)) * texel;
                    let nd = sample_field(nuv).r;
                    if nd >= 0.02 {
                        best = max(best, sample_bed_y(nuv) + nd * material.height_scale + material.lift);
                    }
                }
            }
        }
        // No wet neighbour: tuck the vertex under the ground so nothing draws there.
        y = select(bed_y - 1.0, best, best > -1.0e8);
    }
    local.y = y;

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
    // Fixed-direction phase: `dot(world_xy, normalize(flow))` with world coords in the hundreds
    // turned tiny per-pixel flow-direction changes into many radians of phase -> marbled moire
    // over every flood. Speed still drives the scroll rate; fade where the wave is sub-pixel.
    let ph = dot(world_xy, vec2<f32>(0.31, 0.47));
    let ph_aa = clamp(1.0 - fwidth(ph) * 0.6, 0.0, 1.0);
    let scroll_mix = (sin(ph - material.time_secs * (0.4 + speed * 1.2)) * 0.5 + 0.5) * ph_aa;

    let eps = vec2<f32>(1.5) / extent;
    let grad = abs(sample_field(uv + vec2<f32>(eps.x, 0.0)).r - sample_field(uv - vec2<f32>(eps.x, 0.0)).r)
        + abs(sample_field(uv + vec2<f32>(0.0, eps.y)).r - sample_field(uv - vec2<f32>(0.0, eps.y)).r);
    // Shoreline lip only at the wet edge (thin, shallow water is not 'shore' everywhere).
    let shoreline = smoothstep(0.04, 0.2, grad) * smoothstep(0.08, 0.0, depth01) * 0.6;

    var tint = mix(WATER_SHALLOW, WATER_MID, smoothstep(0.05, 0.35, depth01));
    tint = mix(tint, WATER_DEEP, smoothstep(0.35, 0.85, depth01));
    tint = mix(tint, WATER_SHALLOW, scroll_mix * 0.18 * clamp(speed, 0.0, 1.0));
    // Foam is a broken, moving highlight on blue water, not the water's colour: flow-driven
    // foam is cut into streaks along the flow and capped, so a fast flood reads blue + white
    // streaks instead of a white/grey sheet.
    let sp = dot(world_xy, vec2<f32>(0.9, 1.7)) * 1.3;
    // Stripes vanish well before they reach pixel size (they read as zigzag moire on a
    // zoomed-out flood), and foam only marks genuinely turbulent water, not every fast cell.
    let sp_aa = clamp(1.0 - fwidth(sp) * 1.5, 0.0, 1.0);
    let foam_w = smoothstep(0.45, 0.9, foam01);
    let streak = smoothstep(0.55, 0.95, sin(sp + scroll_mix * 3.1416) * 0.5 + 0.5) * sp_aa;
    tint = mix(tint, FOAM, clamp(foam_w * streak * 0.35 + shoreline * 0.4, 0.0, 0.20));

    var alpha = WATER_ALPHA_BASE + depth01 * WATER_ALPHA_DEPTH + foam_w * streak * 0.08 + shoreline * 0.1;
    alpha *= clamp(material.zoom_alpha * 0.15 + 0.85, 0.85, 1.0); // zoom fade no longer greys the river
    return vec4<f32>(tint, clamp(alpha, 0.0, WATER_ALPHA_MAX));
}
