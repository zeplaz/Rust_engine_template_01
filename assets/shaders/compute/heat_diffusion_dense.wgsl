// TAC-03 / DH-02 — dense separable heat diffusion (GPU play authority).
// Parity with `src/compute/heat_diffusion_dense.rs` — same constants + schedule:
//   inject → horizontal → vertical → decay → inject
// NOT a CPU-row upload fake twin.

struct DenseHeatParams {
    width: u32,
    height: u32,
    w_side: f32,
    w_center: f32,
    decay: f32,
    _pad: f32,
}

// Interleaved heat/smoke pairs: even = heat, odd = smoke. Length = width*height*2.
@group(0) @binding(0) var<uniform> params: DenseHeatParams;
@group(1) @binding(0) var<storage, read> src_field: array<f32>;
@group(2) @binding(0) var<storage, read_write> dst_field: array<f32>;

const W_SIDE: f32 = 0.15;
const W_CENTER: f32 = 0.70;
const DECAY: f32 = 0.02;

fn clamp_i(v: i32, lo: i32, hi: i32) -> i32 {
    return max(lo, min(hi, v));
}

fn idx(x: u32, y: u32, width: u32) -> u32 {
    return y * width + x;
}

fn sample_heat(x: i32, y: i32, width: u32, height: u32) -> f32 {
    let cx = u32(clamp_i(x, 0, i32(width) - 1));
    let cy = u32(clamp_i(y, 0, i32(height) - 1));
    return src_field[idx(cx, cy, width) * 2u];
}

fn sample_smoke(x: i32, y: i32, width: u32, height: u32) -> f32 {
    let cx = u32(clamp_i(x, 0, i32(width) - 1));
    let cy = u32(clamp_i(y, 0, i32(height) - 1));
    return src_field[idx(cx, cy, width) * 2u + 1u];
}

@compute @workgroup_size(8, 8, 1)
fn pass_horizontal(@builtin(global_invocation_id) gid: vec3<u32>) {
    let width = params.width;
    let height = params.height;
    if gid.x >= width || gid.y >= height {
        return;
    }
    let x = i32(gid.x);
    let y = i32(gid.y);
    let h = W_SIDE * sample_heat(x - 1, y, width, height)
        + W_CENTER * sample_heat(x, y, width, height)
        + W_SIDE * sample_heat(x + 1, y, width, height);
    let s = W_SIDE * sample_smoke(x - 1, y, width, height)
        + W_CENTER * sample_smoke(x, y, width, height)
        + W_SIDE * sample_smoke(x + 1, y, width, height);
    let o = idx(gid.x, gid.y, width) * 2u;
    dst_field[o] = h;
    dst_field[o + 1u] = s;
}

@compute @workgroup_size(8, 8, 1)
fn pass_vertical(@builtin(global_invocation_id) gid: vec3<u32>) {
    let width = params.width;
    let height = params.height;
    if gid.x >= width || gid.y >= height {
        return;
    }
    let x = i32(gid.x);
    let y = i32(gid.y);
    // After H pass, src_field is the H output (ping-pong bind).
    let h = W_SIDE * sample_heat(x, y - 1, width, height)
        + W_CENTER * sample_heat(x, y, width, height)
        + W_SIDE * sample_heat(x, y + 1, width, height);
    let s = W_SIDE * sample_smoke(x, y - 1, width, height)
        + W_CENTER * sample_smoke(x, y, width, height)
        + W_SIDE * sample_smoke(x, y + 1, width, height);
    let o = idx(gid.x, gid.y, width) * 2u;
    dst_field[o] = clamp(h * (1.0 - DECAY), 0.0, 1.5);
    dst_field[o + 1u] = clamp(s * (1.0 - DECAY), 0.0, 1.0);
}
