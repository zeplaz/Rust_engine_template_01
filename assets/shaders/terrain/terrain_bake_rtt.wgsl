// GPU-A / GS-01 + GPU-ARRAY.1 — control → world bake via Texture2DArray materials.
// Control rgba8unorm: R = material_id/255, G = road_mask/255, B/A reserved.
// Atlas: texture_2d_array (one layer per material). Bake: storage write rgba8unorm.

struct TerrainBakeRttParams {
    atlas_cols: u32,
    atlas_tile_px: u32,
    bake_w: u32,
    bake_h: u32,
    // Inclusive-exclusive dirty rect in bake pixels.
    dirty_x0: u32,
    dirty_y0: u32,
    dirty_x1: u32,
    dirty_y1: u32,
};

@group(0) @binding(0) var control_tex: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(1) var atlas_tex: texture_2d_array<f32>;
@group(0) @binding(2) var atlas_samp: sampler;
@group(0) @binding(3) var bake_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var<uniform> params: TerrainBakeRttParams;

const ROAD_TINT: vec3<f32> = vec3<f32>(0.188, 0.196, 0.227);

@compute @workgroup_size(8, 8)
fn bake(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = params.dirty_x0 + gid.x;
    let y = params.dirty_y0 + gid.y;
    if x >= params.dirty_x1 || y >= params.dirty_y1 {
        return;
    }
    if x >= params.bake_w || y >= params.bake_h {
        return;
    }

    let control = textureLoad(control_tex, vec2<i32>(i32(x), i32(y)));
    let material_id = u32(round(control.r * 255.0));
    let road_mask = u32(round(control.g * 255.0));

    let layers = max(u32(textureNumLayers(atlas_tex)), 1u);
    let layer = i32(min(material_id, layers - 1u));
    // Layer-local UV center — no 2D atlas neighbour bleed under mips.
    let atlas_uv = vec2<f32>(0.5, 0.5);
    var color = textureSampleLevel(atlas_tex, atlas_samp, atlas_uv, layer, 0.0);

    if road_mask > 0u {
        color = vec4<f32>(mix(color.rgb, ROAD_TINT, 0.5), color.a);
    }

    textureStore(bake_tex, vec2<i32>(i32(x), i32(y)), color);
}
