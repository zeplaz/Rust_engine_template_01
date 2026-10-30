//! Game start for every faction (player and AI), defined in data.
//!
//! `assets/configs/scenario/faction_start_v1.ron` lists, per faction slot: an HQ depot (placed on the
//! faction's best-supplied held cell, or on a fallback tile when the faction holds no territory yet),
//! its starter stockpile and its truck fleet. Units are metric: cargo in the cargo's own unit
//! (tonnes for bulk goods, m3 for liquids), capacity in tonnes (m3 for tankers), speed in m/s, and
//! positions in tiles (16 m).
//!
//! The HQ is an ordinary [`LogisticsOrderSite`] depot carrying [`DepotStartingStock`] and
//! [`DepotStartingFleet`]; the order adapter applies both the first time it registers the site, so
//! there is one code path for stock and vehicles. AI slots in the config become the
//! [`FactionAiRoster`], and the AI reuses its slot's HQ instead of founding an empty depot.

use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::economy::logistics::orders::{
    CargoClassMask, DepotStartingFleet, DepotStartingStock, FactionSlot, LogisticsOrderSite, OwnerId, SiteRole,
};
use crate::strategic::ChunkStrategicOverlay;

use super::decide::FactionDoctrine;
use super::perception::perceive_faction;
use super::{seed_faction_ai_controllers_system, FactionAiRoster};

/// Repo-relative path of the shipped start config.
pub const FACTION_START_PATH: &str = "assets/configs/scenario/faction_start_v1.ron";
pub const FACTION_START_SCHEMA: u32 = 1;
const METRES_PER_TILE: f32 = crate::economy::logistics::orders::snapshot::METRES_PER_TILE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FleetClass {
    /// Box truck: bulk, munitions, deployables.
    Freight,
    /// Liquids.
    Tanker,
    /// Personnel.
    Troop,
}

impl FleetClass {
    #[must_use]
    pub const fn mask(self) -> CargoClassMask {
        match self {
            Self::Freight => CargoClassMask::FREIGHT,
            Self::Tanker => CargoClassMask::TANKER,
            Self::Troop => CargoClassMask::TROOP,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StockLine {
    pub cargo: String,
    /// In the cargo's own unit (t, or m3 for liquids).
    pub amount: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FleetLine {
    pub class: FleetClass,
    pub count: u32,
    /// Tonnes (m3 for tankers, seats for troop transports).
    pub capacity: u32,
    pub speed_mps: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum HqPlacement {
    /// The faction's best-supplied held cell; `fallback_tile` when it holds none yet.
    Territory { fallback_tile: (i32, i32) },
    Tile { x: i32, z: i32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FactionStart {
    pub slot: u8,
    pub name: String,
    /// `Some(doctrine)` makes the slot an AI faction.
    #[serde(default)]
    pub ai: Option<FactionDoctrine>,
    pub hq: HqPlacement,
    pub stockpile: Vec<StockLine>,
    pub fleet: Vec<FleetLine>,
}

#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FactionStartConfig {
    pub schema_version: u32,
    /// Frames to wait for overlays to appear before falling back to the fallback tile.
    #[serde(default = "default_territory_wait")]
    pub territory_wait_frames: u32,
    pub factions: Vec<FactionStart>,
}

fn default_territory_wait() -> u32 {
    300
}

impl FactionStartConfig {
    pub fn from_ron(text: &str) -> Result<Self, String> {
        let cfg: Self = ron::de::from_str(text).map_err(|e| e.to_string())?;
        if cfg.schema_version != FACTION_START_SCHEMA {
            return Err(format!(
                "faction start schema {} (expected {FACTION_START_SCHEMA})",
                cfg.schema_version
            ));
        }
        Ok(cfg)
    }

    /// The shipped config, or `None` when the file is missing or malformed (logged).
    #[must_use]
    pub fn load_default() -> Option<Self> {
        let path = std::env::var_os("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .map_or_else(|| PathBuf::from(FACTION_START_PATH), |root| root.join(FACTION_START_PATH));
        let text = std::fs::read_to_string(&path).ok()?;
        match Self::from_ron(&text) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                bevy::log::warn!("faction start config {} unusable: {e}", path.display());
                None
            }
        }
    }
}

/// A faction's headquarters depot. One per slot; the AI's depot logic reuses it.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactionHq {
    pub slot: u8,
}

/// Spawn the HQ depot of every configured faction that has none (idempotent per slot).
pub fn apply_faction_start_system(
    mut commands: Commands,
    config: Option<Res<FactionStartConfig>>,
    hqs: Query<&FactionHq>,
    overlays: Query<&ChunkStrategicOverlay>,
    mut frames: Local<u32>,
) {
    let Some(config) = config else { return };
    *frames += 1;
    for f in &config.factions {
        if hqs.iter().any(|h| h.slot == f.slot) {
            continue;
        }
        let tile = match &f.hq {
            HqPlacement::Tile { x, z } => IVec2::new(*x, *z),
            HqPlacement::Territory { fallback_tile } => {
                match perceive_faction(f.slot, overlays.iter()).rear_anchor {
                    Some(t) => t,
                    // Overlays may not be materialized yet: give them time before falling back.
                    None if *frames < config.territory_wait_frames => continue,
                    None => IVec2::new(fallback_tile.0, fallback_tile.1),
                }
            }
        };
        let centre = (tile.as_vec2() + Vec2::splat(0.5)) * METRES_PER_TILE;
        commands.spawn((
            Name::new(f.name.clone()),
            FactionHq { slot: f.slot },
            FactionSlot(f.slot),
            Transform::from_xyz(centre.x, 0.0, centre.y),
            GlobalTransform::default(),
            LogisticsOrderSite {
                name: f.name.clone(),
                owner: OwnerId(f.slot),
                role: SiteRole::Depot,
                position_m: Some([centre.x, centre.y]),
            },
            DepotStartingStock(f.stockpile.iter().map(|l| (l.cargo.clone(), l.amount)).collect()),
            DepotStartingFleet(
                f.fleet
                    .iter()
                    .map(|l| (l.class.mask(), l.capacity, l.speed_mps, l.count))
                    .collect(),
            ),
        ));
    }
}

/// AI slots of the start config join the roster once; existing roster entries win.
pub fn seed_ai_roster_from_start_config_system(
    config: Option<Res<FactionStartConfig>>,
    mut roster: ResMut<FactionAiRoster>,
    mut done: Local<bool>,
) {
    let Some(config) = config else { return };
    if *done {
        return;
    }
    *done = true;
    for f in &config.factions {
        if let Some(doctrine) = f.ai {
            if !roster.slots.iter().any(|(s, _)| *s == f.slot) {
                roster.slots.push((f.slot, doctrine));
            }
        }
    }
}

/// Loads the shipped config (when present) and runs the start systems.
pub struct FactionStartPlugin;

impl Plugin for FactionStartPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<FactionStartConfig>() {
            if let Some(cfg) = FactionStartConfig::load_default() {
                app.insert_resource(cfg);
            }
        }
        app.init_resource::<FactionAiRoster>().add_systems(
            Update,
            (apply_faction_start_system, seed_ai_roster_from_start_config_system)
                .before(seed_faction_ai_controllers_system),
        );
    }
}
