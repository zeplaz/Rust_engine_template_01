//! WSS-SLAB-PR-4 — `SubstratePersistBook` flush + `DynamicTerrainOverlay` → slab `dynamic` slice.

use std::collections::HashMap;

use bevy::prelude::*;
use crate::substrate::registry::{
    PersistChunkRecord, SubstratePersistBook, SubstratePersistSnapshot, WorldSubstrateRegistry,
};
use crate::substrate::slab::ChunkKey;
use crate::substrate::types::{DynamicOverlaySlice, WorldChunkState};
use crate::terrain::{ChunkCellKey, DynamicTerrainOverlay};

#[derive(Resource, Clone, Debug, Default)]
pub struct SubstratePr4Witness {
    pub substrate_persist_roundtrip_ok: bool,
    pub dynamic_overlay_migrated: bool,
}

impl SubstratePersistBook {
    /// Flush all dirty resident chunk keys into `slot` (sole persist writer for slab slices).
    pub fn flush_dirty_to_slot(
        &mut self,
        chunks: &crate::substrate::slab::ChunkSlab<WorldChunkState>,
        slot: usize,
    ) {
        while self.snapshots.len() <= slot {
            self.snapshots.push(SubstratePersistSnapshot::default());
        }
        let snapshot = &mut self.snapshots[slot];
        snapshot.chunks.clear();
        for &key in chunks.dirty.iter() {
            if !chunks.is_resident(key) {
                continue;
            }
            let Some(state) = chunks.get(key) else {
                continue;
            };
            snapshot.chunks.insert(
                key,
                PersistChunkRecord {
                    dynamic: state.dynamic.clone(),
                    version: state.version,
                },
            );
        }
        self.pending_slots = self.pending_slots.saturating_add(1);
        self.last_flush_tick = self.last_flush_tick.wrapping_add(1);
    }

    /// Restore `slot` dynamic slices into live slab rows (resident keys only).
    pub fn restore_slot_into_chunks(
        &self,
        chunks: &mut crate::substrate::slab::ChunkSlab<WorldChunkState>,
        slot: usize,
    ) -> bool {
        let Some(snapshot) = self.snapshots.get(slot) else {
            return false;
        };
        for (key, record) in &snapshot.chunks {
            let Some(state) = chunks.get_mut(*key) else {
                continue;
            };
            state.dynamic = record.dynamic.clone();
            state.version = record.version;
        }
        true
    }
}

#[inline]
fn write_overlay_cell(dynamic: &mut DynamicOverlaySlice, cell_index: u32, mud: f32, snow: f32, danger: f32, congestion: f32) {
    let i = cell_index as usize;
    if i >= dynamic.mud.len() {
        return;
    }
    if mud > 0.0 {
        dynamic.mud[i] = mud;
    }
    if snow > 0.0 {
        dynamic.snow_accum[i] = snow;
    }
    if danger > 0.0 {
        dynamic.danger[i] = danger;
    }
    if congestion > 0.0 {
        dynamic.congestion[i] = congestion;
    }
}

/// Copy sparse ECS overlay maps into per-chunk `WorldChunkState.dynamic` vectors.
pub fn migrate_dynamic_overlay_to_slab(
    registry: &mut WorldSubstrateRegistry,
    overlay: &DynamicTerrainOverlay,
) -> u32 {
    let mut migrated = 0_u32;
    for (key, value) in &overlay.mud {
        if apply_overlay_cell(registry, key, *value, 0.0, 0.0, 0.0) {
            migrated += 1;
        }
    }
    for (key, value) in &overlay.snow {
        if apply_overlay_cell(registry, key, 0.0, *value, 0.0, 0.0) {
            migrated += 1;
        }
    }
    for (key, value) in &overlay.danger {
        if apply_overlay_cell(registry, key, 0.0, 0.0, *value, 0.0) {
            migrated += 1;
        }
    }
    for (key, value) in &overlay.congestion {
        if apply_overlay_cell(registry, key, 0.0, 0.0, 0.0, *value) {
            migrated += 1;
        }
    }
    migrated
}

/// PR4-2: mirror one sparse overlay cell into resident slab (`WorldSubstrateRegistry` sole persist writer).
pub fn mirror_overlay_cell_to_slab(
    registry: &mut WorldSubstrateRegistry,
    cell: &ChunkCellKey,
    mud: f32,
    snow: f32,
    danger: f32,
    congestion: f32,
) {
    let _ = apply_overlay_cell(registry, cell, mud, snow, danger, congestion);
}

fn apply_overlay_cell(
    registry: &mut WorldSubstrateRegistry,
    cell: &ChunkCellKey,
    mud: f32,
    snow: f32,
    danger: f32,
    congestion: f32,
) -> bool {
    let key = ChunkKey::from(cell.chunk);
    let n = registry
        .chunks
        .get(key)
        .map(|s| s.dynamic.mud.len())
        .unwrap_or(0);
    if n == 0 {
        let state = WorldChunkState::new_empty(key, 4);
        registry.chunks.insert(key, state);
        registry.chunks.set_resident(key, true);
    }
    let Some(state) = registry.chunks.get_mut(key) else {
        return false;
    };
    write_overlay_cell(
        &mut state.dynamic,
        cell.cell_index,
        mud,
        snow,
        danger,
        congestion,
    );
    true
}

#[must_use]
pub fn dynamic_overlay_matches_slab(
    registry: &WorldSubstrateRegistry,
    overlay: &DynamicTerrainOverlay,
) -> bool {
    fn map_ok(
        registry: &WorldSubstrateRegistry,
        sparse: &HashMap<ChunkCellKey, f32>,
        read: impl Fn(&DynamicOverlaySlice, usize) -> f32,
    ) -> bool {
        for (cell_key, expected) in sparse {
            let slab_key = ChunkKey::from(cell_key.chunk);
            let Some(state) = registry.chunks.get(slab_key) else {
                return false;
            };
            let i = cell_key.cell_index as usize;
            if i >= state.dynamic.mud.len() {
                return false;
            }
            if (read(&state.dynamic, i) - *expected).abs() > 1e-5 {
                return false;
            }
        }
        true
    }
    map_ok(registry, &overlay.mud, |d, i| d.mud[i])
        && map_ok(registry, &overlay.snow, |d, i| d.snow_accum[i])
        && map_ok(registry, &overlay.danger, |d, i| d.danger[i])
        && map_ok(registry, &overlay.congestion, |d, i| d.congestion[i])
}

#[must_use]
pub fn persist_roundtrip_ok(registry: &mut WorldSubstrateRegistry) -> bool {
    let key = registry
        .chunks
        .chunks
        .keys()
        .next()
        .copied()
        .unwrap_or_else(|| ChunkKey::new(0, 0));
    if !registry.chunks.contains(key) {
        let state = WorldChunkState::new_empty(key, 4);
        registry.chunks.insert(key, state);
        registry.chunks.set_resident(key, true);
    }
    registry.chunks.mark_dirty(key);
    if let Some(state) = registry.chunks.get_mut(key) {
        state.dynamic.mud[0] = 0.31;
        state.version = 7;
    }

    registry
        .persist
        .flush_dirty_to_slot(&registry.chunks, 0);

    if let Some(state) = registry.chunks.get_mut(key) {
        state.dynamic.mud[0] = 0.0;
        state.version = 0;
    }

    let restored = registry
        .persist
        .restore_slot_into_chunks(&mut registry.chunks, 0);
    if !restored {
        return false;
    }

    let Some(state) = registry.chunks.get(key) else {
        return false;
    };
    (state.dynamic.mud[0] - 0.31).abs() < 1e-5 && state.version == 7
}

pub fn sync_substrate_persist_witness_system(
    mut registry: ResMut<WorldSubstrateRegistry>,
    mut pr4: ResMut<SubstratePr4Witness>,
    mut checked: Local<bool>,
) {
    if *checked {
        return;
    }
    *checked = true;
    pr4.substrate_persist_roundtrip_ok = persist_roundtrip_ok(&mut registry);
}

pub fn sync_dynamic_overlay_migrate_system(
    base: Res<State<crate::engine::states::BaseState>>,
    overlay: Option<Res<DynamicTerrainOverlay>>,
    dual: Res<crate::substrate::shim::DualWriteShimState>,
    mut registry: ResMut<WorldSubstrateRegistry>,
    mut pr4: ResMut<SubstratePr4Witness>,
) {
    if !matches!(base.get(), crate::engine::states::BaseState::Simulation) {
        return;
    }
    let Some(overlay) = overlay else {
        return;
    };
    let _ = migrate_dynamic_overlay_to_slab(&mut registry, overlay.as_ref());
    pr4.dynamic_overlay_migrated = crate::substrate::shim::dual_write_shim_green(&dual)
        && dynamic_overlay_matches_slab(registry.as_ref(), overlay.as_ref());
}

#[must_use]
pub fn dynamic_overlay_migrated_green(
    pr4: &SubstratePr4Witness,
    dual: &crate::substrate::shim::DualWriteShimState,
) -> bool {
    pr4.substrate_persist_roundtrip_ok
        && pr4.dynamic_overlay_migrated
        && crate::substrate::shim::dual_write_shim_green(dual)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::IVec2;

    #[test]
    fn substrate_persist_roundtrip_fixture() {
        let mut registry = WorldSubstrateRegistry::default();
        assert!(persist_roundtrip_ok(&mut registry));
    }

    #[test]
    fn dynamic_overlay_migrates_to_slab() {
        let mut registry = WorldSubstrateRegistry::default();
        let key = ChunkKey::from(IVec2::new(1, 2));
        registry
            .chunks
            .insert(key, WorldChunkState::new_empty(key, 4));
        registry.chunks.set_resident(key, true);

        let cell = ChunkCellKey::new(IVec2::new(1, 2), 1);
        let mut overlay = DynamicTerrainOverlay::default();
        overlay.mud.insert(cell, 0.55);
        overlay.congestion.insert(cell, 0.12);

        let n = migrate_dynamic_overlay_to_slab(&mut registry, &overlay);
        assert!(n >= 2);
        assert!(dynamic_overlay_matches_slab(&registry, &overlay));

        let state = registry.chunks.get(key).expect("chunk");
        assert!((state.dynamic.mud[1] - 0.55).abs() < 1e-5);
        assert!((state.dynamic.congestion[1] - 0.12).abs() < 1e-5);
    }
}
