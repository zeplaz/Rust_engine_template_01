// Instanced terrain tiles: sample material atlas by index (P0-C′).

struct Globals {
    view_proj: mat4x4<f32>,
    instance_count: u32,
    atlas_cols: u32,
    atlas_rows: u32,
    cell_uv: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(0) @binding(1)
var terrain_atlas: texture_2d<f32>;
@group(0) @binding(2)
var terrain_sampler: sampler;

struct TileInstance {
    world_pos: vec2<f32>,
    material_index: u32,
    _pad: u32,
}

@group(1) @binding(0)
var<storage, read> instances: array<TileInstance>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn material_uv(material_index: u32) -> vec2<f32> {
    let cols = max(globals.atlas_cols, 1u);
    let col = material_index % cols;
    let row = material_index / cols;
    return vec2<f32>(
        (f32(col) + 0.5) * globals.cell_uv.x,
        (f32(row) + 0.5) * globals.cell_uv.y,
    );
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
    let world = tile_row.world_pos + corner;
    var out: VertexOutput;
    out.clip_position = globals.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.uv = material_uv(tile_row.material_index);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(terrain_atlas, terrain_sampler, in.uv);
}
