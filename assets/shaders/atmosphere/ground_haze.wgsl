// Ground-haze / low-altitude smoke bank (stub). Wire bind groups later.
// Asset path: `shaders/atmosphere/ground_haze.wgsl` — see `systems::atmosphere::gpu_paths`.

fn _sink(_a: u32) {}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    _sink(gid.x);
}
