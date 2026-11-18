// GPU-MIPS — box-filter downsample mip N-1 → mip N on the world bake Image.
// Dirty-rect only: regenerating the full chain every tiny bake is wasteful (stub-kill / RND-08).
// Src is a single-mip sampled view; dst is a single-mip storage view.

struct TerrainBakeMipParams {
    dst_w: u32,
    dst_h: u32,
    // Inclusive-exclusive dirty rect in *destination* mip pixels.
    dirty_x0: u32,
    dirty_y0: u32,
    dirty_x1: u32,
    dirty_y1: u32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> params: TerrainBakeMipParams;

@compute @workgroup_size(8, 8)
fn downsample(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = params.dirty_x0 + gid.x;
    let y = params.dirty_y0 + gid.y;
    if x >= params.dirty_x1 || y >= params.dirty_y1 {
        return;
    }
    if x >= params.dst_w || y >= params.dst_h {
        return;
    }

    let sx = i32(x) * 2;
    let sy = i32(y) * 2;
    let src_dims = textureDimensions(src_tex);
    let x1 = min(sx + 1, i32(src_dims.x) - 1);
    let y1 = min(sy + 1, i32(src_dims.y) - 1);

    let c00 = textureLoad(src_tex, vec2<i32>(sx, sy), 0);
    let c10 = textureLoad(src_tex, vec2<i32>(x1, sy), 0);
    let c01 = textureLoad(src_tex, vec2<i32>(sx, y1), 0);
    let c11 = textureLoad(src_tex, vec2<i32>(x1, y1), 0);
    let color = (c00 + c10 + c01 + c11) * 0.25;

    textureStore(dst_tex, vec2<i32>(i32(x), i32(y)), color);
}
