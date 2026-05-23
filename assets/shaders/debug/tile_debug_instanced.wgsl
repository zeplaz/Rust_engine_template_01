// Tile LOD / fire debug: instanced quads from storage (`TileDebugInstance` rows) + small globals uniform.
// Vertex indices 0..5 expand two triangles; `instance_index` selects the row.

struct Globals {
    view_proj: mat4x4<f32>,
    instance_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct TileInstance {
    world_pos: vec2<f32>,
    size: f32,
    lod: u32,
    flags: u32,
}

@group(1) @binding(0)
var<storage, read> instances: array<TileInstance>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

fn tile_instance_color(flags: u32, phase_lod: u32) -> vec4<f32> {
    let focus = (flags & 1u) != 0u;
    let fire = ((flags >> 1u) & 1u) != 0u;
    let fp_invalid = ((flags >> 6u) & 1u) != 0u;
    let fp_risky = ((flags >> 5u) & 1u) != 0u;
    let fp_valid = ((flags >> 4u) & 1u) != 0u;
    let terrain = ((flags >> 3u) & 1u) != 0u;
    let construction = ((flags >> 7u) & 1u) != 0u;
    if construction {
        // `phase_lod` = construction phase index (see site_phase_tile_instances.rs).
        if phase_lod <= 1u {
            return vec4<f32>(0.39, 0.63, 0.86, 0.42);
        }
        if phase_lod <= 3u {
            return vec4<f32>(0.86, 0.67, 0.24, 0.45);
        }
        if phase_lod <= 5u {
            return vec4<f32>(0.86, 0.47, 0.16, 0.48);
        }
        if phase_lod == 6u {
            return vec4<f32>(0.27, 0.67, 0.37, 0.42);
        }
        return vec4<f32>(0.63, 0.27, 0.22, 0.44);
    }
    if fp_invalid {
        return vec4<f32>(0.86, 0.12, 0.12, 0.42);
    }
    if fp_risky {
        return vec4<f32>(1.0, 0.78, 0.2, 0.38);
    }
    if fp_valid {
        return vec4<f32>(0.1, 0.82, 0.22, 0.32);
    }
    if focus {
        return vec4<f32>(0.95, 0.85, 0.15, 0.92);
    }
    if fire {
        return vec4<f32>(1.0, 0.15, 0.12, 0.92);
    }
    if terrain {
        return vec4<f32>(0.2, 0.75, 0.25, 0.88);
    }
    return vec4<f32>(0.12, 0.12, 0.14, 0.85);
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
    let tile_row = instances[safe_id];
    let world = tile_row.world_pos + corner * tile_row.size;
    let clip = globals.view_proj * vec4<f32>(world, 0.0, 1.0);

    var out: VertexOutput;
    out.clip_position = clip;
    out.color = tile_instance_color(tile_row.flags, tile_row.lod);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
