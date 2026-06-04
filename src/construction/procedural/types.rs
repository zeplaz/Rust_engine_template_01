//! Procedural building data model — StylePack, archetypes, assembly requests (PG-1).

use std::collections::HashMap;

use bevy::prelude::Resource;
use serde::Deserialize;

/// High-level zoning / usage hint for archetype ↔ StylePack matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingUsage {
    Residential,
    Commercial,
    Industrial,
    Office,
    Government,
    Military,
    Warehouse,
    Farm,
    Civic,
    Bunker,
    Factory,
}

impl BuildingUsage {
    #[must_use]
    pub fn parse_tag(tag: &str) -> Option<Self> {
        match tag {
            "residential" => Some(Self::Residential),
            "commercial" => Some(Self::Commercial),
            "industrial" => Some(Self::Industrial),
            "office" => Some(Self::Office),
            "government" => Some(Self::Government),
            "military" => Some(Self::Military),
            "warehouse" => Some(Self::Warehouse),
            "farm" => Some(Self::Farm),
            "civic" => Some(Self::Civic),
            "bunker" => Some(Self::Bunker),
            "factory" => Some(Self::Factory),
            _ => None,
        }
    }
}

/// Canonical StylePack identifier (`style_victorian`, …).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct StylePackId(pub String);

impl StylePackId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for StylePackId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Slot key in a StylePack RON file (maps to canonical `module_id`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StylePackSlotKey {
    Wall1u,
    Wall2u,
    DoorDefault,
    DoorWide,
    Window1u,
    Window2u,
    WindowIndustrial,
    RoofDefault,
    RoofFlat,
    RoofIndustrial,
    CornerOuter,
    CornerInner,
    PropClutter,
}

impl StylePackSlotKey {
    #[must_use]
    pub const fn ron_key(self) -> &'static str {
        match self {
            Self::Wall1u => "wall_1u",
            Self::Wall2u => "wall_2u",
            Self::DoorDefault => "door_default",
            Self::DoorWide => "door_wide",
            Self::Window1u => "window_1u",
            Self::Window2u => "window_2u",
            Self::WindowIndustrial => "window_industrial",
            Self::RoofDefault => "roof_default",
            Self::RoofFlat => "roof_flat",
            Self::RoofIndustrial => "roof_industrial",
            Self::CornerOuter => "corner_outer",
            Self::CornerInner => "corner_inner",
            Self::PropClutter => "prop_clutter",
        }
    }

    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        match key {
            "wall_1u" => Some(Self::Wall1u),
            "wall_2u" => Some(Self::Wall2u),
            "door_default" => Some(Self::DoorDefault),
            "door_wide" => Some(Self::DoorWide),
            "window_1u" => Some(Self::Window1u),
            "window_2u" => Some(Self::Window2u),
            "window_industrial" => Some(Self::WindowIndustrial),
            "roof_default" => Some(Self::RoofDefault),
            "roof_flat" => Some(Self::RoofFlat),
            "roof_industrial" => Some(Self::RoofIndustrial),
            "corner_outer" => Some(Self::CornerOuter),
            "corner_inner" => Some(Self::CornerInner),
            "prop_clutter" => Some(Self::PropClutter),
            _ => None,
        }
    }
}

/// PG-2 fallback when a slot module is missing from the index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackPolicy {
    HideSlot,
    PrimitiveFootprint,
}

/// One promoted style pack — slot map only; meshes resolve via `ProceduralModuleRegistry`.
#[derive(Debug, Clone)]
pub struct StylePack {
    pub schema_version: u32,
    pub id: StylePackId,
    pub label: String,
    pub usage_bias: Vec<String>,
    pub style_tags: Vec<String>,
    pub slots: HashMap<String, String>,
    pub fallback_policy: FallbackPolicy,
}

impl StylePack {
    #[must_use]
    pub fn resolve_slot(&self, key: StylePackSlotKey) -> Option<&str> {
        self.slots.get(key.ron_key()).map(|s| s.as_str())
    }

    #[must_use]
    pub fn resolve_slot_str(&self, key: &str) -> Option<&str> {
        self.slots.get(key).map(|s| s.as_str())
    }

    pub fn module_ids(&self) -> impl Iterator<Item = &str> {
        self.slots.values().map(|s| s.as_str())
    }
}

/// Loaded StylePack catalog from `assets/configs/buildings/style_packs/style_*.ron`.
#[derive(Resource, Debug, Default)]
pub struct StylePackRegistry {
    pub packs: HashMap<String, StylePack>,
    pub load_errors: Vec<String>,
}

impl StylePackRegistry {
    #[must_use]
    pub fn len(&self) -> usize {
        self.packs.len()
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&StylePack> {
        self.packs.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &StylePack> {
        self.packs.values()
    }

    /// Union of all slot `module_id` values referenced by loaded packs (for scene preload).
    pub fn referenced_module_ids(&self) -> impl Iterator<Item = &str> {
        self.packs
            .values()
            .flat_map(|pack| pack.module_ids())
    }
}

/// Parametric building archetype stub (grammar / footprint rules — PG-4 expands).
#[derive(Debug, Clone)]
pub struct BuildingArchetype {
    pub id: String,
    pub usage: BuildingUsage,
    pub min_width: u32,
    pub max_width: u32,
    pub min_depth: u32,
    pub max_depth: u32,
    pub floors: std::ops::RangeInclusive<u32>,
}

/// Request driving PG-2 assembly extract (derived at commit in PG-3).
#[derive(Debug, Clone, PartialEq)]
pub struct ProceduralBuildingRequest {
    pub archetype_id: String,
    pub width: u32,
    pub depth: u32,
    pub floors: u32,
    pub style: StylePackId,
    pub seed: u64,
}

/// Active assembly request resource (demo / PG-2 until commit hook lands).
#[derive(Resource, Debug, Clone)]
pub struct ProceduralAssemblyRequest(pub ProceduralBuildingRequest);

impl Default for ProceduralAssemblyRequest {
    fn default() -> Self {
        Self(ProceduralBuildingRequest {
            archetype_id: "rect_perimeter".into(),
            width: 4,
            depth: 2,
            floors: 2,
            style: StylePackId("style_victorian".into()),
            seed: 1,
        })
    }
}
