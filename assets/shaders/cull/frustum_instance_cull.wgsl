// RND-02 / RND-12 — GPU frustum sphere cull for fire particle instances (GPU-CULL.1).
// Book: GEA 4e Vol II §11.5.1.2 — sphere vs six planes (`h_i >= -radius`).
// Writes compacted instances + DrawIndirectArgs for the fire raster (`draw_indirect`).

struct CullParams {
    /// Inward planes as (n.xyz, d) — matches CPU `Plane4`.
    plane0: vec4<f32>,
    plane1: vec4<f32>,
    plane2: vec4<f32>,
    plane3: vec4<f32>,
    plane4: vec4<f32>,
    plane5: vec4<f32>,
    instance_count: u32,
    /// Expanded billboard verts per instance (fire draw uses 6 = two tris).
    verts_per_instance: u32,
}

struct GpuParticleInstance {
    world_xyz_heat: vec4<f32>,
    ember_class_radius_smoke: vec4<f32>,
}

struct DrawIndirectArgs {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

@group(0) @binding(0) var<uniform> params: CullParams;
@group(0) @binding(1) var<storage, read> instances: array<GpuParticleInstance>;
@group(0) @binding(2) var<storage, read_write> compacted: array<GpuParticleInstance>;
@group(0) @binding(3) var<storage, read_write> indirect: DrawIndirectArgs;
@group(0) @binding(4) var<storage, read_write> visible_counter: atomic<u32>;

fn plane_h(plane: vec4<f32>, p: vec3<f32>) -> f32 {
    return dot(plane.xyz, p) + plane.w;
}

fn sphere_inside(center: vec3<f32>, radius: f32) -> bool {
    let r = max(radius, 0.0);
    if plane_h(params.plane0, center) < -r { return false; }
    if plane_h(params.plane1, center) < -r { return false; }
    if plane_h(params.plane2, center) < -r { return false; }
    if plane_h(params.plane3, center) < -r { return false; }
    if plane_h(params.plane4, center) < -r { return false; }
    if plane_h(params.plane5, center) < -r { return false; }
    return true;
}

@compute @workgroup_size(64, 1, 1)
fn cull_compact(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= params.instance_count {
        return;
    }
    let row = instances[i];
    let center = row.world_xyz_heat.xyz;
    // Half-edge is a conservative horizontal radius for the billboard.
    let radius = max(row.ember_class_radius_smoke.z, 0.5);
    if !sphere_inside(center, radius) {
        return;
    }
    let slot = atomicAdd(&visible_counter, 1u);
    compacted[slot] = row;
}

@compute @workgroup_size(1, 1, 1)
fn finalize_indirect() {
    let visible = atomicLoad(&visible_counter);
    indirect.vertex_count = visible * params.verts_per_instance;
    indirect.instance_count = 1u;
    indirect.first_vertex = 0u;
    indirect.first_instance = 0u;
}
