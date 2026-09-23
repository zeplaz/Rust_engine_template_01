// EFFECTS-SYSTEM ES-5 — weather precip anisotropic streak draw (vertex-pull).
// Layout must match GpuInstancedQuadInstance (32 bytes).

struct Globals {
    view_proj: mat4x4<f32>,
    instance_count: u32,
    time_secs: f32,
    zoom_alpha: f32,
    /// Half of visible world height (for fall wrap + coverage feel).
    view_half_y: f32,
    /// px-per-tile — streak size scales as 1/camera_zoom (screen-stable).
    camera_zoom: f32,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct PrecipInstance {
    // xyz world position; w = rain
    position_xyz_custom0: vec4<f32>,
    // x = snow, y = class_id, z = half edge, w = fog
    custom1_custom2_custom3_custom4: vec4<f32>,
}

@group(1) @binding(0)
var<storage, read> instances: array<PrecipInstance>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) rain: f32,
    @location(1) snow: f32,
    @location(2) fog: f32,
}

@vertex
fn vs_main(
    @builtin(instance_index) instance_id: u32,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    let corners = array<vec2<f32>, 4>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5),
        vec2<f32>(0.5, 0.5),
        vec2<f32>(-0.5, 0.5),
    );
    let tri = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u);
    let corner = corners[tri[vertex_index]];
    let last = select(0u, globals.instance_count - 1u, globals.instance_count > 0u);
    let safe_id = min(instance_id, last);
    let row = instances[safe_id];

    let rain = row.position_xyz_custom0.w;
    let snow = row.custom1_custom2_custom3_custom4.x;
    let base_half = max(row.custom1_custom2_custom3_custom4.z, 0.12);
    // True screen-stable: world half ∝ 1/px_per_tile (ref ≈ operational play zoom).
    let z = max(globals.camera_zoom, 0.25);
    let zoom_size = clamp(4.0 / z, 0.08, 6.0);
    let half = base_half * zoom_size;
    // Rain: tall thin streak; snow: nearer square flake.
    let half_x = half * mix(0.28, 0.85, clamp(snow, 0.0, 1.0));
    let half_y = half * mix(3.6, 1.15, clamp(snow, 0.0, 1.0));

    // Fast vertical fall + light wind drift; wrap inside view band so rain fills the frame.
    let half_view = max(globals.view_half_y, 40.0);
    let fall_speed = mix(95.0, 280.0, clamp(rain, 0.0, 1.0));
    let phase = fract(
        globals.time_secs * (fall_speed / (half_view * 2.0))
            + f32(safe_id) * 0.0713
            + row.position_xyz_custom0.x * 0.0011
    );
    let fall_y = (0.5 - phase) * half_view * 2.0;
    let wind_x = globals.time_secs * (12.0 + rain * 28.0 + snow * 6.0)
        + f32(safe_id) * 0.37;

    let world = vec3<f32>(
        row.position_xyz_custom0.x + corner.x * half_x * 2.0 + sin(wind_x) * 2.5,
        row.position_xyz_custom0.y + corner.y * half_y * 2.0 + fall_y,
        row.position_xyz_custom0.z,
    );

    var out: VertexOutput;
    out.clip_position = globals.view_proj * vec4<f32>(world, 1.0);
    out.rain = rain;
    out.snow = snow;
    out.fog = row.custom1_custom2_custom3_custom4.w;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let rain_col = vec3<f32>(0.55, 0.65, 0.85);
    let snow_col = vec3<f32>(0.92, 0.94, 0.98);
    let rgb = mix(rain_col, snow_col, clamp(in.snow, 0.0, 1.0));
    let alpha = clamp(0.22 + in.rain * 0.5 + in.snow * 0.35 - in.fog * 0.08, 0.08, 0.8);
    return vec4<f32>(rgb, alpha);
}
