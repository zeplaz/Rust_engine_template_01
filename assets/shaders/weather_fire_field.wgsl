// Ping-pong GPU field: blends toward CPU-uploaded weather + fire means with spatial variation.
// Channels: R = rain-weighted signal, G = snow, B = fire heat, A = fog. Visual-only; gameplay reads CPU state.

struct WeatherFireFieldUniforms {
    means: vec4<f32>, // rain, snow, fire_heat, fog
    time_secs: f32,
    blend_rate: f32,
    decay: f32,
    _pad: f32,
}

@group(0) @binding(0) var prev_field: texture_storage_2d<rgba32float, read>;
@group(0) @binding(1) var next_field: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var<uniform> params: WeatherFireFieldUniforms;

@compute @workgroup_size(8, 8)
fn update(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = vec2<u32>(textureDimensions(prev_field));
    if (gid.x >= dims.x || gid.y >= dims.y) {
        return;
    }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let uv = vec2<f32>(f32(gid.x) + 0.5, f32(gid.y) + 0.5) / vec2<f32>(f32(dims.x), f32(dims.y));
    let t = params.time_secs;

    let old = textureLoad(prev_field, coord, 0);

    let n1 = sin(uv.x * 6.28318 + t * 0.35) * cos(uv.y * 6.28318 - t * 0.22);
    let target = vec4<f32>(
        params.means.x * clamp(0.55 + 0.45 * n1, 0.0, 1.5),
        params.means.y * clamp(0.45 + 0.55 * cos(uv.x * 12.0 + t * 0.4), 0.0, 1.5),
        params.means.z * clamp(0.35 + 0.65 * sin(uv.y * 10.0 + t * 0.55), 0.0, 1.5),
        params.means.w,
    );

    let m = params.blend_rate;
    var blended = mix(old, target, m);
    blended = blended * (1.0 - params.decay) + target * params.decay * 0.15;
    textureStore(next_field, vec2<i32>(gid.xy), blended);
}
