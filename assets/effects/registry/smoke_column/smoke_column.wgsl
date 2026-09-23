// Vertical smoke column pass (stub).
// Asset path: `shaders/atmosphere/smoke_column.wgsl`.

fn _sink(_a: u32) {}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    _sink(gid.x);
}
