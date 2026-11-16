//! `occupation` dump block — voxel / tile occupation **counts only** (no verdicts).
//!
//! Plan: `plan_voxel_occupation_v1.md` X2-P3 · dual-cite **PCG-06** · **ECS-15**.
//! Reads [`VoxelOccupationBook`] / [`VoxelOccupationConfig`] RO — never writes authority.

use serde_json::{json, Value};

use crate::strategic::{VoxelOccupationBook, VoxelOccupationConfig};

/// `occupation` block of the dump (`samples: 0` when book and config are both absent).
#[must_use]
pub fn occupation_json(
    book: Option<&VoxelOccupationBook>,
    config: Option<&VoxelOccupationConfig>,
) -> Value {
    let samples = u64::from(book.is_some() || config.is_some());
    let voxel_cells = book.map(VoxelOccupationBook::cell_count).unwrap_or(0);
    let revision = book.map(|b| b.revision).unwrap_or(0);
    let corridor_reserve_cells = book
        .map(|b| b.cells.values().filter(|c| c.corridor_reserve).count())
        .unwrap_or(0);
    let commit_authority_enabled = config.is_some_and(|c| c.commit_authority_enabled);
    let derived_mirror_enabled = config.is_some_and(|c| c.derived_mirror_enabled);
    let dual_writer_guard_ok = match (book, config) {
        (Some(b), Some(c)) => b.dual_writer_guard_ok(c),
        _ => true,
    };

    json!({
        "samples": samples,
        "voxel_cells": voxel_cells,
        "voxel_revision": revision,
        "voxel_corridor_reserve_cells": corridor_reserve_cells,
        "commit_authority_enabled": u8::from(commit_authority_enabled),
        "derived_mirror_enabled": u8::from(derived_mirror_enabled),
        "dual_writer_guard_ok": u8::from(dual_writer_guard_ok),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::{OccupancyLayerStack, VoxelOccupationCell};
    use bevy::prelude::IVec2;

    #[test]
    fn occupation_block_has_no_green_verdict() {
        let v = occupation_json(None, None);
        assert_eq!(v["samples"], 0);
        assert_eq!(v["voxel_cells"], 0);
        assert!(v.get("green").is_none());
        assert!(v.as_object().is_some_and(|o| !o.keys().any(|k| k.ends_with("_green"))));
    }

    #[test]
    fn occupation_counts_voxel_cells_and_flags() {
        let mut book = VoxelOccupationBook::default();
        book.cells.insert(
            IVec2::ZERO,
            VoxelOccupationCell {
                xy: IVec2::ZERO,
                layers: OccupancyLayerStack {
                    weight: 1.0,
                    z_layers: 2,
                },
                site_id: None,
                corridor_reserve: true,
            },
        );
        let config = VoxelOccupationConfig {
            derived_mirror_enabled: false,
            commit_authority_enabled: true,
        };
        let v = occupation_json(Some(&book), Some(&config));
        assert_eq!(v["samples"], 1);
        assert_eq!(v["voxel_cells"], 1);
        assert_eq!(v["voxel_corridor_reserve_cells"], 1);
        assert_eq!(v["commit_authority_enabled"], 1);
        assert_eq!(v["derived_mirror_enabled"], 0);
    }
}
