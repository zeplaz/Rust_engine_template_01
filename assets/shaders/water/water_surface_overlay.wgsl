// FX-WATER-SHADER-001 (W1) — tile water motion overlay (D-W01…D-W04 A).
// CPU raster mirrors these constants in `water_surface_visual.rs`.

struct WaterSurfaceGlobals {
    time_secs: f32,
    scroll_speed: f32,
    lake_ripple_hz: f32,
    ocean_swell_hz: f32,
    _pad: f32,
}

// §6 palette tokens
const WATER_RIVER_DEEP: vec3<f32> = vec3<f32>(0.118, 0.271, 0.267);
const WATER_TEAL_EDGE: vec3<f32> = vec3<f32>(0.290, 0.471, 0.471);
const WATER_TEAL: vec3<f32> = vec3<f32>(0.165, 0.353, 0.345);
const WATER_OCEAN_DEEP: vec3<f32> = vec3<f32>(0.059, 0.157, 0.157);

fn river_flow_scroll(
    world_xy: vec2<f32>,
    flow_dir: vec2<f32>,
    time_secs: f32,
    scroll_speed: f32,
) -> f32 {
    let flow_n = normalize(flow_dir + vec2<f32>(0.0001, 0.0));
    let cross = vec2<f32>(-flow_n.y, flow_n.x);
    let along = dot(world_xy, flow_n);
    let across = dot(world_xy, cross);
    let ribbon = 1.0 - abs(fract(across) - 0.5) * 2.0;
    let scroll = sin(along * 0.35 - time_secs * scroll_speed) * 0.5 + 0.5;
    return clamp(ribbon * 0.65 + scroll * 0.35, 0.0, 1.0);
}

fn lake_ripple(world_xy: vec2<f32>, time_secs: f32, hz: f32) -> f32 {
    let t = time_secs * hz;
    let a = sin(world_xy.x * 0.21 + world_xy.y * 0.17 + t);
    let b = cos(world_xy.x * 0.13 - world_xy.y * 0.19 + t * 0.85);
    return clamp(a * b * 0.5 + 0.5, 0.0, 1.0);
}

fn ocean_swell(world_xy: vec2<f32>, time_secs: f32, hz: f32) -> f32 {
    let swell = sin(world_xy.x * 0.08 + time_secs * hz) * 0.5 + 0.5;
    let haze = cos(world_xy.y * 0.05 - time_secs * hz * 0.75) * 0.5 + 0.5;
    return clamp(swell * 0.55 + haze * 0.45, 0.0, 1.0);
}

fn water_surface_tint(
    base: vec3<f32>,
    kind: u32,
    world_xy: vec2<f32>,
    flow_dir: vec2<f32>,
    globals: WaterSurfaceGlobals,
) -> vec3<f32> {
    if kind == 2u {
        let w = river_flow_scroll(world_xy, flow_dir, globals.time_secs, globals.scroll_speed);
        let tint = mix(WATER_RIVER_DEEP, WATER_TEAL_EDGE, w);
        return mix(base, tint, 0.42 + w * 0.38);
    }
    if kind == 1u {
        let w = lake_ripple(world_xy, globals.time_secs, globals.lake_ripple_hz);
        return mix(base, WATER_TEAL, 0.12 + w * 0.18);
    }
    if kind == 3u {
        let w = ocean_swell(world_xy, globals.time_secs, globals.ocean_swell_hz);
        return mix(base, WATER_OCEAN_DEEP, 0.25 + w * 0.32);
    }
    return base;
}
