//! Runtime-rebuilt overlay/layer mesh upload (tactical vegetation, fronts, PixelWorld layers).
//!
//! Bevy's mesh allocator skips slab allocation for a mesh with **zero vertices** but still tries to
//! copy its (empty) vertex data, logging `slab_allocator: Use-after-free: attempted to copy element
//! data for an unallocated key` for every empty mesh added or modified. Layers that rebuild
//! geometry from sim/UI state are frequently empty (no fronts, no ghost, no paths, a culled
//! vegetation window), so they must never hand an empty mesh to `Assets<Mesh>`: keep the last
//! asset untouched and hide the entity instead.

use bevy::asset::Assets;
use bevy::mesh::Mesh;
use bevy::prelude::Handle;

/// Write `mesh` into the layer's `slot` (insert in place, or allocate on first use / stale handle).
///
/// Returns `true` when the layer now holds drawable geometry. An empty `mesh` leaves the asset (and
/// `slot`) untouched and returns `false` — the caller hides the layer entity.
pub(crate) fn upload_layer_mesh(
    meshes: &mut Assets<Mesh>,
    slot: &mut Option<Handle<Mesh>>,
    mesh: Mesh,
) -> bool {
    if mesh.count_vertices() == 0 {
        return false;
    }
    match slot {
        Some(handle) if meshes.contains(&*handle) => {
            // In-place replace keeps the handle the layer entity already draws.
            let _ = meshes.insert(&*handle, mesh);
        }
        // First upload, or a stale handle (asset storage reset): allocate a fresh asset.
        _ => *slot = Some(meshes.add(mesh)),
    }
    true
}
