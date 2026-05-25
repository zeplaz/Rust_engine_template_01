// UX-E01 — minimap GPU compositor (terrain + fire + logistics + construction + ecology).
struct MinimapCompositeParams {
    fire_heat_enabled: u32,
    logistics_heat_enabled: u32,
    construction_heat_enabled: u32,
    ecology_heat_enabled: u32,
    overlay_revision: u32,
    logistics_rows: u32,
    construction_rows: u32,
    ecology_rows: u32,
};

@group(0) @binding(0) var terrain_tex: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(1) var fire_tex: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(2) var logistics_tex: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(3) var construction_tex: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(4) var ecology_tex: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(5) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(6) var<uniform> params: MinimapCompositeParams;

@compute @workgroup_size(8, 8)
fn composite(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(terrain_tex);
    if gid.x >= dims.x || gid.y >= dims.y {
        return;
    }
    var col = textureLoad(terrain_tex, gid.xy);
    if params.fire_heat_enabled != 0u {
        let heat = textureLoad(fire_tex, gid.xy).r;
        col = vec4<f32>(col.rgb + vec3<f32>(heat * 0.85, heat * 0.35, 0.0), col.a);
    }
    if params.logistics_heat_enabled != 0u {
        let flow = textureLoad(logistics_tex, gid.xy).g;
        col = vec4<f32>(col.rgb + vec3<f32>(0.0, flow * 0.45, flow * 0.75), col.a);
    }
    if params.construction_heat_enabled != 0u {
        let build = textureLoad(construction_tex, gid.xy).r;
        col = vec4<f32>(col.rgb + vec3<f32>(build * 0.55, build * 0.18, build * 0.62), col.a);
    }
    if params.ecology_heat_enabled != 0u {
        let eco = textureLoad(ecology_tex, gid.xy);
        let biomass = eco.g;
        let risk = eco.b;
        col = vec4<f32>(col.rgb + vec3<f32>(risk * 0.22, biomass * 0.38, biomass * 0.28), col.a);
    }
    textureStore(output_tex, gid.xy, col);
}
