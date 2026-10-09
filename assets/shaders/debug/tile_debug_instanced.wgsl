// Tile LOD / fire debug: instanced quads from storage (`TileDebugInstance` rows) + small globals uniform.
// Vertex indices 0..5 expand two triangles; `instance_index` selects the row.

struct Globals {
    view_proj: mat4x4<f32>,
    instance_count: u32,
    view_tint: u32, // TileDebugViewId: 0=WorldMain, 1=Preview, 2=Minimap, 3=SimulationMap
    _pad0: u32,
    _pad1: u32,
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
    @location(1) suppress: f32,
}

fn debug_quad_suppressed(flags: u32) -> bool {
    // Construction-site and placement-footprint rows are not production paint.
    // Phase occupation is the egui overlay; the ghost is visual_authority.
    let footprint = ((flags >> 4u) & 7u) != 0u;
    let construction = ((flags >> 7u) & 1u) != 0u;
    return footprint || construction;
}

fn tile_instance_color(flags: u32, phase_lod: u32) -> vec4<f32> {
    let focus = (flags & 1u) != 0u;
    let fire = ((flags >> 1u) & 1u) != 0u;
    let terrain = ((flags >> 3u) & 1u) != 0u;
    if debug_quad_suppressed(flags) {
        return vec4<f32>(0.0, 0.0, 0.0, f32(phase_lod) * 0.0);
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

fn apply_view_tint(color: vec4<f32>, view_id: u32) -> vec4<f32> {
    if view_id == 1u {
        return vec4<f32>(color.r * 0.94, color.g * 0.90, color.b * 1.06, color.a);
    }
    if view_id == 2u {
        return vec4<f32>(color.r * 0.88, color.g * 0.95, color.b * 1.02, color.a);
    }
    if view_id == 3u {
        return vec4<f32>(color.r * 0.96, color.g * 1.02, color.b * 0.94, color.a);
    }
    return color;
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
    out.color = apply_view_tint(tile_instance_color(tile_row.flags, tile_row.lod), globals.view_tint);
    out.suppress = select(0.0, 1.0, debug_quad_suppressed(tile_row.flags));
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.suppress > 0.5) {
        discard;
    }
    return in.color;
}
