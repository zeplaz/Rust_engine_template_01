// World-anchored fire pinpoint sparks (Phase A — FX-FIRE-SPARK-001).
// Expanded quads from `fire_particle.wgsl`; sharp ≤2px read, legacy age/twinkle mix.

struct Globals {
    view_proj: mat4x4<f32>,
    vertex_count: u32,
    time_secs: f32,
    zoom_alpha: f32,
    _pad: f32,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct GpuParticleQuadVertex {
    world_xy_heat_ember: vec4<f32>,
}

@group(1) @binding(0)
var<storage, read> expanded: array<GpuParticleQuadVertex>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) heat: f32,
    @location(2) ember: f32,
    @location(3) world_xy: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let quad = vertex_index / 6u;
    let tri_corner = vertex_index % 6u;
    let corner_idx = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u)[tri_corner];
    let row = expanded[quad * 4u + corner_idx];
    let world = row.world_xy_heat_ember.xy;
    let heat = row.world_xy_heat_ember.z;
    let ember = row.world_xy_heat_ember.w;
    let uvs = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );
    var out: VertexOutput;
    out.clip_position = globals.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.uv = uvs[corner_idx];
    out.heat = heat;
    out.ember = ember;
    out.world_xy = world;
    return out;
}

// §6 color key — legacy elemental mapped to design tokens (D-F06 B).
const COLOR_ASH: vec3<f32> = vec3<f32>(0.112, 0.115, 0.12);
const COLOR_HOT: vec3<f32> = vec3<f32>(0.902, 0.27, 0.0);
const COLOR_DIRTY_AMBER: vec3<f32> = vec3<f32>(0.902, 0.515, 0.082);
const COLOR_GOLD: vec3<f32> = vec3<f32>(0.91, 0.75, 0.23);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // D-F01 A: point sprite / ≤2px sharp core — not smooth radial blob.
    let d = distance(in.uv, vec2<f32>(0.5, 0.5));
    let core = 1.0 - smoothstep(0.015, 0.09, d);
    let pin = step(d, 0.07);

    // D-F04 A: ash → orange age mix (heat + ember as lifetime intensity).
    let age_intensity = clamp(in.heat * 1.08 + in.ember * 0.32, 0.0, 1.0);
    let col_age = mix(COLOR_ASH, COLOR_HOT, age_intensity);

    // D-F05 A: legacy sin(pos.x) / cos(pos.y) twinkle personality.
    let sin_intzy = sin(in.world_xy.x * 0.31) * 0.5 + 0.5;
    let cos_intzy = cos(in.world_xy.y * 0.27) * 0.5 + 0.5;
    let twinkle = sin_intzy * cos_intzy;
    var col_pos = col_age;
    col_pos.r = mix(col_age.r, COLOR_DIRTY_AMBER.r, twinkle * 0.58);
    col_pos.g = mix(col_age.g, COLOR_DIRTY_AMBER.g, twinkle * 0.48);
    col_pos.b = mix(col_age.b, COLOR_DIRTY_AMBER.b, twinkle * 0.38);

    var col = mix(col_age, col_pos, 0.5);

    // D-F08 A: additive-leaning hot cores (gold peak on brightest pins).
    let hot_core = smoothstep(0.5, 0.92, age_intensity);
    col = mix(col, COLOR_GOLD, hot_core * pin * 0.62);

    // D-F07 A: many tiny points — low base α, not large soft blobs.
    let ember_alpha = (0.06 + in.ember * 0.12) * (0.28 + age_intensity * 0.42);

    // D-F09 A: fade sparks when zoomed out (strategic zoom → low zoom_alpha).
    let za = clamp(globals.zoom_alpha, 0.0, 1.0);
    let zoom_fade = za * za * (0.2 + 0.8 * za);

    let alpha = (core * ember_alpha + pin * hot_core * 0.18) * zoom_fade;
    let rgb = col * (1.0 + hot_core * pin * 1.6);

    return vec4<f32>(rgb * alpha, alpha);
}
