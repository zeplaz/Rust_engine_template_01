// DESTROY-T2 — PixelWorld ground: bake albedo + destruction mask collapse.
// World XY tiles: mesh uses (x, h, -y); tile = (floor(x), floor(-z)).

#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_pbr::mesh_view_bindings::view

struct GroundMaskUniform {
    /// xy = world width/height in tiles; z = skirt Y for destroyed columns; w unused
    extent_skirt: vec4<f32>,
}

@group(2) @binding(0)
var<uniform> material: GroundMaskUniform;
@group(2) @binding(1)
var albedo_tex: texture_2d<f32>;
@group(2) @binding(2)
var albedo_samp: sampler;
@group(2) @binding(3)
var destruction_mask: texture_2d<f32>;
@group(2) @binding(4)
var destruction_samp: sampler;

// CPU packs state in .r as 0/1/2 (u8). Destroyed threshold.
const MASK_DESTROYED: f32 = 1.5 / 255.0;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var local = vertex.position;
    let extent = max(material.extent_skirt.xy, vec2<f32>(1.0));
    // Mesh space: X = tile x, Z = -tile y
    let tile_x = i32(floor(local.x));
    let tile_y = i32(floor(-local.z));
    let tx = clamp(tile_x, 0, i32(extent.x) - 1);
    let ty = clamp(tile_y, 0, i32(extent.y) - 1);
    let mask_uv = (vec2<f32>(f32(tx), f32(ty)) + vec2<f32>(0.5)) / extent;
    let state = textureSampleLevel(destruction_mask, destruction_samp, mask_uv, 0.0).r;
    // state stored as u8/255; destroyed == 2/255
    if (state * 255.0 >= 1.5) {
        local.y = material.extent_skirt.z;
    }

    out.position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(local, 1.0),
    );
    out.world_position = get_world_from_local(vertex.instance_index) * vec4<f32>(local, 1.0);
    out.world_normal = vertex.normal;
    out.uv = vertex.uv;
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex.instance_index;
#endif
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(albedo_tex, albedo_samp, in.uv);
    // Simple unlit terrain read — bake already holds lighting/tint.
    return vec4<f32>(albedo.rgb, 1.0);
}
