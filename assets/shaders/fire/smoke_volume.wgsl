// Stub: future smoke volume / fog compute (`base_gui_next.md`).
fn _sink(_a: u32) {}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    _sink(gid.x);
}
