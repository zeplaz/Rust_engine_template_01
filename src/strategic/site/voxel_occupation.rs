//! Sparse voxel column occupancy — sim authority charter (PLAN-VOXEL-OCCUPATION-v1 · X2-P0–P3).
//!
//! **Dual-cite:** PCG-06 (grammar conditions preview candidates; commit only via construction funnel)
//! · **ECS-15 / V4-ECS15** (gea4v2 16.2.2): APS/Studio tool-side payloads carry `sim_occupation: false`;
//! massing is presentation-only. Studio / ParamFrame / `StudioMassingOverride` are **not** writers
//! of this book — only the construction commit funnel (flag on) or derived tile mirror may write.
//!
//! P0: [`VoxelOccupationBook`] is a **derived read-only mirror** of [`TileOccupationBook`] when
//! [`VoxelOccupationConfig::derived_mirror_enabled`] is true (default **off**).
//! P1: when [`VoxelOccupationConfig::commit_authority_enabled`] is true (default **off**), the
//! construction commit funnel writes this book and applies a corridor reserve mask.
//! P2: under `commit_authority_enabled`, this book is the **sole** occupation writer for
//! commit/demolish; [`TileOccupationBook`] is a derived cache refreshed via
//! [`TileOccupationBook::replace_from_voxel`]. Flag off preserves tile-only behavior.
//! P3: massing derive may **RO-read** this book for column `z_layers` intent when commit or
//! derived-mirror flags are on; dump emits `occupation.voxel_*` counts only.
//!
//! **CQ-07 / Review 2026-11-16:** both flags stay **default off** until the save overlay is
//! wired and OCC-P4/P5 re-close product default-on. Env `"1"|"true"|"on"|"yes"` opts in.

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::{IVec2, Resource};
use serde::{Deserialize, Serialize};

use super::resources::SiteId;
use super::tile_occupation::{TileOccupationBook, TILE_OCCUPATION_OVERLAP_EPSILON};

pub const VOXEL_OCCUPATION_MIRROR_ENV: &str = "RUST_ENGINE_VOXEL_OCCUPATION_MIRROR";
pub const VOXEL_OCCUPATION_COMMIT_ENV: &str = "RUST_ENGINE_VOXEL_OCCUPATION_COMMIT";
pub const VOXEL_OCCUPATION_SNAPSHOT_SCHEMA_V1: u32 = 1;

/// Cutover flags — default **off** until save hydrate is proven (CQ-07); OCC-P4/P5 re-open default-on.
#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct VoxelOccupationConfig {
    pub derived_mirror_enabled: bool,
    /// Commit funnel writes voxel cells (+ corridor reserve). Default **off** (CQ-07).
    /// When on, voxel is sole occupation writer; tile book is derived cache.
    pub commit_authority_enabled: bool,
}

impl Default for VoxelOccupationConfig {
    fn default() -> Self {
        Self {
            derived_mirror_enabled: false,
            commit_authority_enabled: false,
        }
    }
}

impl VoxelOccupationConfig {
    #[must_use]
    pub fn from_env_mirror_var() -> Self {
        Self {
            derived_mirror_enabled: env_flag_default_off(VOXEL_OCCUPATION_MIRROR_ENV),
            commit_authority_enabled: env_flag_default_off(VOXEL_OCCUPATION_COMMIT_ENV),
        }
    }
}

/// Default **off**; only `"1"|"true"|"on"|"yes"` enables (CQ-07).
#[must_use]
pub fn env_flag_default_off(key: &str) -> bool {
    match std::env::var(key) {
        Ok(v) if matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "on" | "yes") => true,
        _ => false,
    }
}

#[must_use]
pub fn env_mirror_enabled() -> bool {
    env_flag_default_off(VOXEL_OCCUPATION_MIRROR_ENV)
}

#[must_use]
pub fn env_commit_enabled() -> bool {
    env_flag_default_off(VOXEL_OCCUPATION_COMMIT_ENV)
}

/// Stacked occupancy at a world XY column (Z layers capped per product rule at P1+).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OccupancyLayerStack {
    pub weight: f32,
    /// Surface layer count — P0 mirror uses `1` when tile weight > ε.
    pub z_layers: u8,
}

/// One sparse cell in [`VoxelOccupationBook`].
#[derive(Clone, Debug, PartialEq)]
pub struct VoxelOccupationCell {
    pub xy: IVec2,
    pub layers: OccupancyLayerStack,
    pub site_id: Option<SiteId>,
    pub corridor_reserve: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoxelOccupationProvenance {
    #[default]
    Empty,
    DerivedMirror,
    HydratedSnapshot,
    /// Written by construction commit funnel (X2-P1, flag on).
    CommitAuthority,
}

/// Sparse world occupancy — sole writer under `commit_authority_enabled` (P2).
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct VoxelOccupationBook {
    /// Cell lookup by tile (`IVec2` is not `Ord` on this Bevy; capture sorts for save).
    pub cells: HashMap<IVec2, VoxelOccupationCell>,
    pub revision: u64,
    provenance: VoxelOccupationProvenance,
    /// Per-site weight contributions (commit/demolish delta). `BTreeMap` → lowest-id first.
    by_site: BTreeMap<SiteId, Vec<(IVec2, f32)>>,
}

impl VoxelOccupationBook {
    #[must_use]
    pub fn provenance(&self) -> VoxelOccupationProvenance {
        self.provenance
    }

    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    #[must_use]
    pub fn weight_at(&self, tile: IVec2) -> f32 {
        self.cells
            .get(&tile)
            .map(|c| c.layers.weight)
            .unwrap_or(0.0)
    }

    /// True when cutover flags match provenance (no surprise independent writes).
    #[must_use]
    pub fn dual_writer_guard_ok(&self, config: &VoxelOccupationConfig) -> bool {
        if config.commit_authority_enabled {
            matches!(
                self.provenance,
                VoxelOccupationProvenance::Empty
                    | VoxelOccupationProvenance::CommitAuthority
                    | VoxelOccupationProvenance::HydratedSnapshot
                    | VoxelOccupationProvenance::DerivedMirror
            )
        } else if config.derived_mirror_enabled {
            matches!(
                self.provenance,
                VoxelOccupationProvenance::Empty
                    | VoxelOccupationProvenance::DerivedMirror
                    | VoxelOccupationProvenance::HydratedSnapshot
            )
        } else {
            self.cells.is_empty()
                && matches!(
                    self.provenance,
                    VoxelOccupationProvenance::Empty | VoxelOccupationProvenance::HydratedSnapshot
                )
        }
    }

    /// Overlap + corridor reserve gate for commit candidates (P1).
    #[must_use]
    pub fn can_apply_weights(&self, weights: &[(IVec2, f32)]) -> bool {
        weights.iter().all(|(tile, w_new)| {
            if self
                .cells
                .get(tile)
                .is_some_and(|c| c.corridor_reserve)
            {
                return false;
            }
            self.weight_at(*tile) + w_new <= 1.0 + TILE_OCCUPATION_OVERLAP_EPSILON
        })
    }

    /// Construction commit funnel write (P1/P2). Preserves existing corridor_reserve bits.
    /// Re-commits **append** site contributions (do not replace the prior `by_site` row).
    pub fn apply_site_commit(&mut self, site_id: SiteId, weights: &[(IVec2, f32)]) {
        let mut stored: Vec<(IVec2, f32)> = Vec::with_capacity(weights.len());
        for (tile, w) in weights {
            if *w < TILE_OCCUPATION_OVERLAP_EPSILON {
                continue;
            }
            let reserve = self
                .cells
                .get(tile)
                .map(|c| c.corridor_reserve)
                .unwrap_or(false);
            let prev = self.weight_at(*tile);
            let owner = self
                .primary_site_at(*tile)
                .map(|existing| existing.min(site_id))
                .unwrap_or(site_id);
            self.cells.insert(
                *tile,
                VoxelOccupationCell {
                    xy: *tile,
                    layers: OccupancyLayerStack {
                        weight: prev + w,
                        z_layers: 1,
                    },
                    site_id: Some(owner),
                    corridor_reserve: reserve,
                },
            );
            stored.push((*tile, *w));
        }
        if !stored.is_empty() {
            self.by_site.entry(site_id).or_default().extend(stored);
        }
        self.revision = self.revision.wrapping_add(1);
        self.provenance = VoxelOccupationProvenance::CommitAuthority;
    }

    /// Demolish / collapse funnel remove (P2). Keeps corridor-only cells.
    pub fn remove_site_commit(&mut self, site_id: SiteId) -> Option<Vec<(IVec2, f32)>> {
        let weights = self.by_site.remove(&site_id)?;
        for (tile, w) in &weights {
            let Some(cell) = self.cells.get(tile) else {
                continue;
            };
            let new_w = (cell.layers.weight - w).max(0.0);
            let reserve = cell.corridor_reserve;
            if new_w < TILE_OCCUPATION_OVERLAP_EPSILON {
                if reserve {
                    if let Some(cell) = self.cells.get_mut(tile) {
                        cell.layers.weight = 0.0;
                        cell.site_id = None;
                    }
                } else {
                    self.cells.remove(tile);
                }
            } else {
                let next_owner = if cell.site_id == Some(site_id) {
                    self.primary_site_at(*tile)
                } else {
                    cell.site_id
                };
                if let Some(cell) = self.cells.get_mut(tile) {
                    cell.layers.weight = new_w;
                    cell.site_id = next_owner;
                }
            }
        }
        self.revision = self.revision.wrapping_add(1);
        self.provenance = VoxelOccupationProvenance::CommitAuthority;
        Some(weights)
    }

    /// Lowest-id site that still owns nonzero weight on `tile` (deterministic).
    #[must_use]
    pub fn primary_site_at(&self, tile: IVec2) -> Option<SiteId> {
        self.by_site.iter().find_map(|(site_id, weights)| {
            weights
                .iter()
                .any(|(t, w)| *t == tile && *w > TILE_OCCUPATION_OVERLAP_EPSILON)
                .then_some(*site_id)
        })
    }

    /// Per-site contributions for derived tile cache refresh.
    pub fn iter_by_site(&self) -> impl Iterator<Item = (&SiteId, &Vec<(IVec2, f32)>)> {
        self.by_site.iter()
    }

    /// Mark road/rail corridor cells as reserved (P1). Creates empty reserved cells when missing.
    pub fn apply_corridor_reserve_mask<'a>(
        &mut self,
        tiles: impl IntoIterator<Item = &'a IVec2>,
        reserved: bool,
    ) {
        let mut touched = false;
        for tile in tiles {
            match self.cells.get_mut(tile) {
                Some(cell) => {
                    if cell.corridor_reserve != reserved {
                        cell.corridor_reserve = reserved;
                        touched = true;
                    }
                }
                None if reserved => {
                    self.cells.insert(
                        *tile,
                        VoxelOccupationCell {
                            xy: *tile,
                            layers: OccupancyLayerStack::default(),
                            site_id: None,
                            corridor_reserve: true,
                        },
                    );
                    touched = true;
                }
                None => {}
            }
        }
        if touched {
            self.revision = self.revision.wrapping_add(1);
            if self.provenance == VoxelOccupationProvenance::Empty {
                self.provenance = VoxelOccupationProvenance::CommitAuthority;
            }
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.by_site.clear();
        self.revision = self.revision.wrapping_add(1);
        self.provenance = VoxelOccupationProvenance::Empty;
    }

    /// Sole P0 mutation path from live tile book (read-only source).
    pub fn replace_from_tile_mirror(&mut self, tile_book: &TileOccupationBook) {
        self.cells.clear();
        self.by_site.clear();
        for (tile, weight) in tile_book.iter_tile_weights() {
            if *weight < TILE_OCCUPATION_OVERLAP_EPSILON {
                continue;
            }
            self.cells.insert(
                *tile,
                VoxelOccupationCell {
                    xy: *tile,
                    layers: OccupancyLayerStack {
                        weight: *weight,
                        z_layers: 1,
                    },
                    site_id: tile_book.primary_site_at(*tile),
                    corridor_reserve: false,
                },
            );
        }
        for (site_id, weights) in tile_book.iter_by_site() {
            self.by_site.insert(*site_id, weights.clone());
        }
        self.revision = self.revision.wrapping_add(1);
        self.provenance = VoxelOccupationProvenance::DerivedMirror;
    }

    /// Hydrate loader entry (X2 save slice — stub consumer at P0).
    pub fn apply_snapshot(&mut self, snapshot: &VoxelOccupationSnapshot) {
        debug_assert_eq!(
            snapshot.schema_version,
            VOXEL_OCCUPATION_SNAPSHOT_SCHEMA_V1
        );
        self.cells.clear();
        self.by_site.clear();
        for row in &snapshot.cells {
            let xy = IVec2::new(row.x, row.y);
            self.cells.insert(
                xy,
                VoxelOccupationCell {
                    xy,
                    layers: OccupancyLayerStack {
                        weight: row.weight,
                        z_layers: row.z_layers.max(1),
                    },
                    site_id: row.site_id.map(SiteId),
                    corridor_reserve: row.corridor_reserve,
                },
            );
            if let Some(sid) = row.site_id {
                if row.weight >= TILE_OCCUPATION_OVERLAP_EPSILON {
                    self.by_site
                        .entry(SiteId(sid))
                        .or_default()
                        .push((xy, row.weight));
                }
            }
        }
        self.revision = self.revision.wrapping_add(1);
        self.provenance = VoxelOccupationProvenance::HydratedSnapshot;
    }

    /// Refresh mirror only when cutover flag is enabled; otherwise clears to empty (P0 default).
    pub fn sync_mirror_if_enabled(
        &mut self,
        config: &VoxelOccupationConfig,
        tile_book: &TileOccupationBook,
    ) {
        if config.derived_mirror_enabled {
            self.replace_from_tile_mirror(tile_book);
        } else if !self.cells.is_empty() {
            self.clear();
        }
    }
}

/// RON save slice — stable sort of cells (deterministic hydrate).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoxelOccupationSnapshot {
    pub schema_version: u32,
    pub cells: Vec<VoxelOccupationCellRecord>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoxelOccupationCellRecord {
    pub x: i32,
    pub y: i32,
    pub layer_index: u8,
    pub weight: f32,
    pub z_layers: u8,
    pub site_id: Option<u64>,
    pub corridor_reserve: bool,
}

#[must_use]
pub fn capture_voxel_occupation_snapshot(book: &VoxelOccupationBook) -> VoxelOccupationSnapshot {
    let mut cells: Vec<VoxelOccupationCellRecord> = book
        .cells
        .values()
        .map(|c| VoxelOccupationCellRecord {
            x: c.xy.x,
            y: c.xy.y,
            layer_index: 0,
            weight: c.layers.weight,
            z_layers: c.layers.z_layers,
            site_id: c.site_id.map(|s| s.0),
            corridor_reserve: c.corridor_reserve,
        })
        .collect();
    cells.sort_by_key(|r| (r.x, r.y, r.layer_index));
    VoxelOccupationSnapshot {
        schema_version: VOXEL_OCCUPATION_SNAPSHOT_SCHEMA_V1,
        cells,
    }
}

pub fn read_voxel_occupation_snapshot_ron(
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<VoxelOccupationSnapshot> {
    let body = std::fs::read_to_string(path)?;
    let snap: VoxelOccupationSnapshot = ron::from_str(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    if snap.schema_version != VOXEL_OCCUPATION_SNAPSHOT_SCHEMA_V1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "unsupported voxel occupation schema_version {} (expected {VOXEL_OCCUPATION_SNAPSHOT_SCHEMA_V1})",
                snap.schema_version
            ),
        ));
    }
    Ok(snap)
}

pub fn write_voxel_occupation_snapshot_ron(
    path: impl AsRef<std::path::Path>,
    snapshot: &VoxelOccupationSnapshot,
) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = ron::ser::to_string_pretty(snapshot, ron::ser::PrettyConfig::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(path, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voxel_occupation_mirror_disabled_leaves_book_empty() {
        let mut tile = TileOccupationBook::default();
        tile.apply_site(SiteId(1), &[(IVec2::new(1, 2), 0.5)]);

        let config = VoxelOccupationConfig {
            derived_mirror_enabled: false,
            commit_authority_enabled: false,
        };
        assert!(!config.derived_mirror_enabled);

        let mut voxel = VoxelOccupationBook::default();
        voxel.sync_mirror_if_enabled(&config, &tile);
        assert!(voxel.cells.is_empty());
        assert!(voxel.dual_writer_guard_ok(&config));
    }

    #[test]
    fn cq07_default_config_is_off_until_save_proven() {
        let config = VoxelOccupationConfig::default();
        assert!(!config.derived_mirror_enabled);
        assert!(!config.commit_authority_enabled);
    }

    #[test]
    fn voxel_occupation_derived_mirror_matches_tile_weights() {
        let mut tile = TileOccupationBook::default();
        tile.apply_site(SiteId(3), &[(IVec2::new(4, 5), 0.75)]);

        let config = VoxelOccupationConfig {
            derived_mirror_enabled: true,
            commit_authority_enabled: false,
        };
        let mut voxel = VoxelOccupationBook::default();
        voxel.sync_mirror_if_enabled(&config, &tile);

        assert_eq!(voxel.cell_count(), 1);
        assert!((voxel.weight_at(IVec2::new(4, 5)) - 0.75).abs() < 1e-5);
        assert_eq!(voxel.provenance(), VoxelOccupationProvenance::DerivedMirror);
        assert!(voxel.dual_writer_guard_ok(&config));
        assert_eq!(voxel.cells[&IVec2::new(4, 5)].site_id, Some(SiteId(3)));
    }

    #[test]
    fn voxel_occupation_dual_writer_guard_tile_mutation_without_mirror() {
        let mut tile = TileOccupationBook::default();
        tile.apply_site(SiteId(2), &[(IVec2::ZERO, 1.0)]);

        let config = VoxelOccupationConfig::default();
        let voxel = VoxelOccupationBook::default();
        assert!(voxel.dual_writer_guard_ok(&config));
        assert!(voxel.cells.is_empty());
        assert!((tile.weight_at(IVec2::ZERO) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn voxel_occupation_snapshot_ron_round_trip() {
        let mut voxel = VoxelOccupationBook::default();
        voxel.apply_snapshot(&VoxelOccupationSnapshot {
            schema_version: VOXEL_OCCUPATION_SNAPSHOT_SCHEMA_V1,
            cells: vec![VoxelOccupationCellRecord {
                x: 0,
                y: 1,
                layer_index: 0,
                weight: 0.4,
                z_layers: 1,
                site_id: Some(9),
                corridor_reserve: false,
            }],
        });

        let snap = capture_voxel_occupation_snapshot(&voxel);
        let dir = std::env::temp_dir().join("voxel_occ_ron_rt");
        let path = dir.join("occupation.ron");
        write_voxel_occupation_snapshot_ron(&path, &snap).expect("write");
        let loaded = read_voxel_occupation_snapshot_ron(&path).expect("read");
        assert_eq!(loaded, snap);

        let mut voxel2 = VoxelOccupationBook::default();
        voxel2.apply_snapshot(&loaded);
        assert_eq!(voxel2.weight_at(IVec2::new(0, 1)), 0.4);
        assert_eq!(
            voxel2.provenance(),
            VoxelOccupationProvenance::HydratedSnapshot
        );
    }

    #[test]
    fn apply_site_commit_appends_on_recommit_and_picks_lowest_site_id() {
        let mut voxel = VoxelOccupationBook::default();
        let tile = IVec2::new(1, 1);
        voxel.apply_site_commit(SiteId(20), &[(tile, 0.2)]);
        voxel.apply_site_commit(SiteId(5), &[(tile, 0.3)]);
        assert!((voxel.weight_at(tile) - 0.5).abs() < 1e-5);
        assert_eq!(voxel.primary_site_at(tile), Some(SiteId(5)));
        assert_eq!(voxel.by_site.get(&SiteId(20)).map(|v| v.len()), Some(1));
        assert_eq!(voxel.by_site.get(&SiteId(5)).map(|v| v.len()), Some(1));
        voxel.apply_site_commit(SiteId(20), &[(IVec2::new(2, 2), 0.1)]);
        assert_eq!(voxel.by_site.get(&SiteId(20)).map(|v| v.len()), Some(2));
    }

    #[test]
    fn voxel_occupation_commit_authority_writes_and_blocks_reserve() {
        let config = VoxelOccupationConfig {
            derived_mirror_enabled: false,
            commit_authority_enabled: true,
        };
        let mut voxel = VoxelOccupationBook::default();
        let road = IVec2::new(10, 10);
        voxel.apply_corridor_reserve_mask(&[road], true);
        assert!(voxel.cells[&road].corridor_reserve);
        assert!(!voxel.can_apply_weights(&[(road, 0.5)]));

        let site_tile = IVec2::new(11, 10);
        assert!(voxel.can_apply_weights(&[(site_tile, 0.6)]));
        voxel.apply_site_commit(SiteId(42), &[(site_tile, 0.6)]);
        assert!((voxel.weight_at(site_tile) - 0.6).abs() < 1e-5);
        assert_eq!(voxel.provenance(), VoxelOccupationProvenance::CommitAuthority);
        assert!(voxel.dual_writer_guard_ok(&config));
        assert!(!voxel.can_apply_weights(&[(site_tile, 0.5)]));
    }

    #[test]
    fn voxel_occupation_p2_sole_writer_tile_derived_cache() {
        let config = VoxelOccupationConfig {
            derived_mirror_enabled: false,
            commit_authority_enabled: true,
        };
        let mut voxel = VoxelOccupationBook::default();
        let tile_xy = IVec2::new(7, 8);
        voxel.apply_site_commit(SiteId(11), &[(tile_xy, 0.8)]);

        let mut tile = TileOccupationBook::default();
        // P2: no independent tile apply — refresh only.
        tile.replace_from_voxel(&voxel);
        assert!((tile.weight_at(tile_xy) - 0.8).abs() < 1e-5);

        voxel.remove_site_commit(SiteId(11));
        tile.replace_from_voxel(&voxel);
        assert!(tile.weight_at(tile_xy) < TILE_OCCUPATION_OVERLAP_EPSILON);
        assert!(voxel.cells.get(&tile_xy).is_none());
        assert!(voxel.dual_writer_guard_ok(&config));
        assert!(config.commit_authority_enabled);
    }
}