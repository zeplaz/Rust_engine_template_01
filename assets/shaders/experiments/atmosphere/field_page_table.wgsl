// Virtual atmosphere field page table — maps world page coords to atlas texel origins.
// Asset path: `shaders/atmosphere/field_page_table.wgsl`.

struct AtmospherePageEntry {
    atlas_origin: vec2<u32>,
    valid: u32,
    _pad: u32,
}

struct PageTableParams {
    page_count: u32,
    dirty_only: u32,
    frame: u32,
    _pad: u32,
}

@group(0) @binding(0) var<uniform> params: PageTableParams;
@group(0) @binding(1) var<storage, read> pages: array<AtmospherePageEntry>;

@compute @workgroup_size(64, 1, 1)
fn mark_resident_pages(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.page_count) {
        return;
    }
    let row = pages[i];
    if (row.valid == 0u) {
        return;
    }
    let _ = row.atlas_origin.x + row.atlas_origin.y + params.frame;
}
