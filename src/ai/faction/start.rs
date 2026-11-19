//! Game start for every faction (player and AI), defined in data.
//!
//! `assets/configs/scenario/faction_start_v1.ron` lists, per faction slot: an HQ depot (placed on the
//! faction's best-supplied held cell, or on a fallback tile when the faction holds no territory yet),
//! its starter stockpile, its truck fleet and its starting formations (spawned through
//! [`spawn_formation`], so their `Zone{Control}` is what gives the faction territory and fronts). Units are metric: cargo in the cargo's own unit
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
use crate::combat::balance::{FrontBalance, UnitKind};
use crate::combat::formation::{spawn_formation, Formation};
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

/// One starting formation on a world tile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormationLine {
    pub kind: UnitKind,
    pub tile: (i32, i32),
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
    /// Logistics auto-supply for this faction at start (`None` = the logistics default: AI on,
    /// player off in a windowed game). Same switch as the orders-panel toggle.
    #[serde(default)]
    pub auto_supply: Option<bool>,
    pub hq: HqPlacement,
    pub stockpile: Vec<StockLine>,
    pub fleet: Vec<FleetLine>,
    /// Formations the faction starts the game with.
    #[serde(default)]
    pub formations: Vec<FormationLine>,
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
        Self::load_path(FACTION_START_PATH)
    }

    /// A start config at a repo-relative path (scenes may ship their own, same schema).
    #[must_use]
    pub fn load_path(rel: &str) -> Option<Self> {
        let path = std::env::var_os("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .map_or_else(|| PathBuf::from(rel), |root| root.join(rel));
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

/// Apply each faction's start `auto_supply` choice once (the orders-panel toggle, set by config).
pub fn apply_start_auto_supply_policy_system(
    config: Option<Res<FactionStartConfig>>,
    mut policy: Option<ResMut<crate::economy::logistics::orders::AutoSupplyPolicy>>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let (Some(config), Some(policy)) = (config, policy.as_mut()) else {
        return;
    };
    *done = true;
    for f in &config.factions {
        if let Some(on) = f.auto_supply {
            policy.set(crate::economy::logistics::orders::OwnerId(f.slot), on);
        }
    }
}

fn insert_default_faction_start_if_absent(
    mut commands: Commands,
    existing: Option<Res<FactionStartConfig>>,
) {
    if existing.is_none() {
        if let Some(cfg) = FactionStartConfig::load_default() {
            commands.insert_resource(cfg);
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

/// Spawn every configured faction's starting formations once per slot, after the strategic overlay
/// exists (formations project control onto it). A slot that already has formations (a loaded game,
/// a test) is left alone; formations lost in play are not respawned.
pub fn apply_faction_start_formations_system(
    mut commands: Commands,
    config: Option<Res<FactionStartConfig>>,
    balance: Option<Res<FrontBalance>>,
    formations: Query<&Formation>,
    overlays: Query<(), With<ChunkStrategicOverlay>>,
    mut seeded: Local<std::collections::BTreeSet<u8>>,
) {
    let (Some(config), Some(balance)) = (config, balance) else { return };
    if overlays.is_empty() {
        return;
    }
    for f in &config.factions {
        if seeded.contains(&f.slot) {
            continue;
        }
        seeded.insert(f.slot);
        if formations.iter().any(|x| x.faction_slot == f.slot) {
            continue;
        }
        for line in &f.formations {
            let tile = IVec2::new(line.tile.0, line.tile.1);
            spawn_formation(&mut commands, Formation::new(f.slot, line.kind, tile), &balance);
        }
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
        // Default config at Startup, only if a scene didn't insert its own in PreStartup
        // (plugin order independent).
        app.add_systems(Startup, insert_default_faction_start_if_absent);
        app.init_resource::<FactionAiRoster>().add_systems(
            Update,
            (
                apply_faction_start_system,
                apply_start_auto_supply_policy_system,
                apply_faction_start_formations_system,
                seed_ai_roster_from_start_config_system,
            )
                .before(seed_faction_ai_controllers_system),
        );
    }
}
