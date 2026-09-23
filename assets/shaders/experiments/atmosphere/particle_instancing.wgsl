// Instancing / Hanabi bridge (stub).
// Asset path: `shaders/atmosphere/particle_instancing.wgsl`.

fn _sink(_a: u32) {}

@compute @workgroup_size(64, 1, 1)
fn expand_instances(@builtin(global_invocation_id) gid: vec3<u32>) {
    _sink(gid.x);
}
