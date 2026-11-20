// Bevy 0.20 WESL (imports via `import`; material group 3).
// DESTROY-T2 — PixelWorld ground: lit StandardMaterial (bake albedo) + destruction mask.
// Extension of StandardMaterial: bindings 0..99 are the standard material's, ours start at 100.
// World XY tiles: mesh uses (x, h, -y); tile = (floor(x), floor(-z)).

import bevy_pbr::render::forward_io::{Vertex, VertexOutput, FragmentOutput, decompress_vertex};
import bevy_pbr::render::mesh_functions::{get_world_from_local, mesh_position_local_to_world, mesh_normal_local_to_world};
import bevy_pbr::render::view_transformations::position_world_to_clip;
import bevy_pbr::render::pbr_fragment::pbr_input_from_standard_material;
import bevy_pbr::render::pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing};

struct GroundMaskUniform {
    /// xy = world width/height in tiles; z = skirt Y for destroyed columns; w unused
    extent_skirt: vec4<f32>,
}

@group(3) @binding(100)
var<uniform> ground: GroundMaskUniform;
@group(3) @binding(101)
var destruction_mask: texture_2d<f32>;
@group(3) @binding(102)
var destruction_samp: sampler;
// GPU-B scorch authority: R = live fire heat per world tile (same field the 2D overlay shows).
@group(3) @binding(103)
var scorch_tex: texture_2d<f32>;
@group(3) @binding(104)
var scorch_samp: sampler;

// CPU packs state in .r as 0/1/2 (u8): intact / damaged (crater) / destroyed.
const MASK_DAMAGED: f32 = 0.5;
const MASK_DESTROYED: f32 = 1.5;
/// Crater ground (blast truth via the mask): charred soot.
const SCORCH: vec3<f32> = vec3<f32>(0.20, 0.17, 0.14);
/// Burning ground glow (fire heat from the scorch field).
const EMBER: vec3<f32> = vec3<f32>(1.0, 0.36, 0.08);

fn fire_heat(tile: vec2<i32>) -> f32 {
    let dims = vec2<f32>(textureDimensions(scorch_tex));
    if (dims.x < 2.0) {
        return 0.0;
    }
    let uv = (vec2<f32>(tile) + vec2<f32>(0.5)) / dims;
    return textureSampleLevel(scorch_tex, scorch_samp, uv, 0.0).r;
}

fn mask_state(tile: vec2<i32>) -> f32 {
    let extent = max(ground.extent_skirt.xy, vec2<f32>(1.0));
    let t = clamp(tile, vec2<i32>(0), vec2<i32>(extent) - vec2<i32>(1));
    let uv = (vec2<f32>(t) + vec2<f32>(0.5)) / extent;
    return textureSampleLevel(destruction_mask, destruction_samp, uv, 0.0).r * 255.0;
}

@vertex
fn vertex(vertex_in: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let v = decompress_vertex(vertex_in, vertex_in.instance_index);
    var local = v.position;
    // Mesh space: X = tile x, Z = -tile y
    let tile = vec2<i32>(i32(floor(local.x)), i32(floor(-local.z)));
    if (mask_state(tile) >= MASK_DESTROYED) {
        local.y = ground.extent_skirt.z;
    }
    let world_from_local = get_world_from_local(v.instance_index);
    out.world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(local, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
    out.world_normal = mesh_normal_local_to_world(v.normal, v.instance_index);
    @if(VERTEX_UVS_A) {
        out.uv = v.uv;
    }
    @if(VERTEX_COLORS) {
        out.color = v.color;
    }
    @if(VERTEX_OUTPUT_INSTANCE_INDEX) {
        out.instance_index = v.instance_index;
    }
    return out;
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color,
    );
    // Crater scorch, per pixel (merged top quads span many tiles, so not per vertex).
    let tile = vec2<i32>(i32(floor(in.world_position.x)), i32(floor(-in.world_position.z)));
    if (mask_state(tile) >= MASK_DAMAGED) {
        let c = pbr_input.material.base_color.rgb;
        pbr_input.material.base_color = vec4<f32>(mix(c, SCORCH, 0.8), pbr_input.material.base_color.a);
    }
    // Burning ground: darken toward char and glow with the fire's heat.
    let heat = fire_heat(tile);
    if (heat > 0.01) {
        let c = pbr_input.material.base_color.rgb;
        pbr_input.material.base_color = vec4<f32>(mix(c, SCORCH, 0.6 * heat), pbr_input.material.base_color.a);
    }
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = vec4<f32>(out.color.rgb + EMBER * heat * 0.9, out.color.a);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
