//! Save/load of order state (plan LO-7) — RON overlay `overlays/logistics_orders.ron`.
//!
//! The sim is saved whole (orders, vehicles, stock, demands keep their ids). Site entities are
//! re-created by their own lanes after a load and get fresh entities, so each saved site also
//! stores its identity (name, owner, role, position); the adapter rebinds re-registering
//! endpoints to their saved [`SiteId`] by [`saved_site_matches`].

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::model::{OwnerId, SiteId};
use super::sim::LogisticsOrderSim;
use super::snapshot::{SiteMeta, SiteRole, METRES_PER_TILE};

pub const LOGISTICS_ORDERS_SAVE_SCHEMA: u32 = 1;
pub const LOGISTICS_ORDERS_OVERLAY_NAME: &str = "logistics_orders";
/// Bundle-relative artifact path.
pub const LOGISTICS_ORDERS_OVERLAY_REL_PATH: &str = "overlays/logistics_orders.ron";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedSite {
    pub id: SiteId,
    pub name: String,
    pub owner: OwnerId,
    pub role: SiteRole,
    pub world_xz: [f32; 2],
}

impl SavedSite {
    #[must_use]
    pub fn from_meta(id: SiteId, meta: &SiteMeta) -> Self {
        Self {
            id,
            name: meta.name.clone(),
            owner: meta.owner,
            role: meta.role,
            world_xz: meta.world_xz,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogisticsOrdersSave {
    pub schema_version: u32,
    pub sim: LogisticsOrderSim,
    pub sites: Vec<SavedSite>,
}

/// Same endpoint after a reload: same name, owner and role, within one tile.
#[must_use]
pub fn saved_site_matches(saved: &SavedSite, meta: &SiteMeta) -> bool {
    let dx = saved.world_xz[0] - meta.world_xz[0];
    let dz = saved.world_xz[1] - meta.world_xz[1];
    saved.name == meta.name
        && saved.owner == meta.owner
        && saved.role == meta.role
        && dx * dx + dz * dz <= METRES_PER_TILE * METRES_PER_TILE
}

pub fn logistics_orders_save_to_ron(save: &LogisticsOrdersSave) -> Result<String, String> {
    ron::ser::to_string_pretty(save, ron::ser::PrettyConfig::default()).map_err(|e| e.to_string())
}

pub fn logistics_orders_save_from_ron(text: &str) -> Result<LogisticsOrdersSave, String> {
    let save: LogisticsOrdersSave = ron::de::from_str(text).map_err(|e| e.to_string())?;
    if save.schema_version != LOGISTICS_ORDERS_SAVE_SCHEMA {
        return Err(format!(
            "logistics orders save schema {} (expected {LOGISTICS_ORDERS_SAVE_SCHEMA})",
            save.schema_version
        ));
    }
    Ok(save)
}

pub fn write_logistics_orders_overlay(bundle_dir: &Path, save: &LogisticsOrdersSave) -> std::io::Result<()> {
    let path = bundle_dir.join(LOGISTICS_ORDERS_OVERLAY_REL_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = logistics_orders_save_to_ron(save)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, body)
}

/// `Ok(None)` when the bundle has no orders overlay (older saves).
pub fn read_logistics_orders_overlay(bundle_dir: &Path) -> std::io::Result<Option<LogisticsOrdersSave>> {
    let path = bundle_dir.join(LOGISTICS_ORDERS_OVERLAY_REL_PATH);
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path)?;
    logistics_orders_save_from_ron(&text)
        .map(Some)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}
