//! Blueprint preset serialization — Wave **S** RON DTOs (no placement authority).

use serde::{Deserialize, Serialize};

use crate::strategic::{BuildSiteTile, FootprintTiles, LayerType, SiteArchetype};

use super::build_state::BuildGhostState;
use super::build_tool_authority::{ActiveBuildTool, BuildTool, BuildingArchetypeId};
use super::build_strip::BuildStripState;

/// One authored blueprint row for offline / save interchange.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintPresetEntryR8 {
    pub schema_version: u32,
    pub label: String,
    pub archetype_tag: String,
    pub origin_x: u32,
    pub origin_z: u32,
    pub footprint_width: u32,
    pub footprint_depth: u32,
    pub layer_tag: String,
    pub rotation_quarter_turns: u8,
    pub mirror_x: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintPresetCollectionR8 {
    pub schema_version: u32,
    pub presets: Vec<BlueprintPresetEntryR8>,
}

impl BlueprintPresetCollectionR8 {
    pub const CURRENT_SCHEMA: u32 = 1;

    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA,
            presets: Vec::new(),
        }
    }

    pub fn push(&mut self, entry: BlueprintPresetEntryR8) {
        self.presets.push(entry);
    }
}

#[must_use]
pub fn blueprint_collection_from_pending(
    queue: &super::pending_construction::PendingConstructionQueue,
) -> BlueprintPresetCollectionR8 {
    let mut collection = BlueprintPresetCollectionR8::new();
    for entry in &queue.entries {
        collection.push(blueprint_preset_entry_from_pending(
            &entry.label,
            entry.archetype,
            entry.origin,
            entry.footprint,
            format!("{:?}", entry.layer),
            entry.rotation_quarter_turns,
            entry.mirror_x,
        ));
    }
    collection
}

#[must_use]
pub fn blueprint_preset_entry_from_pending(
    label: impl Into<String>,
    archetype: SiteArchetype,
    origin: BuildSiteTile,
    footprint: FootprintTiles,
    layer_tag: impl Into<String>,
    rotation_quarter_turns: u8,
    mirror_x: bool,
) -> BlueprintPresetEntryR8 {
    BlueprintPresetEntryR8 {
        schema_version: BlueprintPresetCollectionR8::CURRENT_SCHEMA,
        label: label.into(),
        archetype_tag: format!("{archetype:?}"),
        origin_x: origin.x,
        origin_z: origin.z,
        footprint_width: footprint.width,
        footprint_depth: footprint.depth,
        layer_tag: layer_tag.into(),
        rotation_quarter_turns,
        mirror_x,
    }
}

/// Parse `archetype_tag` written by [`blueprint_preset_entry_from_pending`] (`format!("{archetype:?}")`).
#[must_use]
pub fn site_archetype_from_preset_tag(tag: &str) -> SiteArchetype {
    match tag.trim() {
        "CivilHousing" => SiteArchetype::CivilHousing,
        "Factory" => SiteArchetype::Factory,
        "RailDepot" => SiteArchetype::RailDepot,
        "FuelDepot" => SiteArchetype::FuelDepot,
        "WaterPlant" => SiteArchetype::WaterPlant,
        "PowerPlant" => SiteArchetype::PowerPlant,
        "MilitaryBase" => SiteArchetype::MilitaryBase,
        "RadarSite" => SiteArchetype::RadarSite,
        "SensorPost" => SiteArchetype::SensorPost,
        "BunkerComplex" => SiteArchetype::BunkerComplex,
        "TrenchLine" => SiteArchetype::TrenchLine,
        _ => SiteArchetype::Factory,
    }
}

#[must_use]
pub fn layer_type_from_preset_tag(tag: &str) -> LayerType {
    match tag.trim() {
        "Subsurface" => LayerType::Subsurface,
        "DeepSubsurface" => LayerType::DeepSubsurface,
        _ => LayerType::Surface,
    }
}

#[must_use]
pub fn building_archetype_id_for_site(archetype: SiteArchetype) -> BuildingArchetypeId {
    match archetype {
        SiteArchetype::CivilHousing => BuildingArchetypeId::Housing,
        SiteArchetype::WaterPlant => BuildingArchetypeId::WaterPlant,
        SiteArchetype::PowerPlant => BuildingArchetypeId::PowerPlant,
        SiteArchetype::RailDepot | SiteArchetype::FuelDepot => BuildingArchetypeId::Depot,
        _ => BuildingArchetypeId::Factory,
    }
}

/// **BQ-128-APPLY-001** — load preset onto ghost only (no queue commit).
pub fn apply_blueprint_preset_to_build_ghost(
    entry: &BlueprintPresetEntryR8,
    ghost: &mut BuildGhostState,
    tool: &mut ActiveBuildTool,
    strip: &mut BuildStripState,
) {
    let _layer = layer_type_from_preset_tag(&entry.layer_tag);
    ghost.origin = Some(BuildSiteTile {
        x: entry.origin_x,
        z: entry.origin_z,
    });
    ghost.footprint = FootprintTiles {
        width: entry.footprint_width.max(1),
        depth: entry.footprint_depth.max(1),
    };
    ghost.rotation_quarter_turns = entry.rotation_quarter_turns % 4;
    ghost.mirror_x = entry.mirror_x;
    ghost.drag_active = false;
    let archetype = site_archetype_from_preset_tag(&entry.archetype_tag);
    let building_id = building_archetype_id_for_site(archetype);
    tool.close_submenus();
    tool.clear_building_intent();
    tool.tool = BuildTool::Building(building_id);
    tool.apply_to_strip(strip);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_blueprint_preset_sets_ghost_origin_and_footprint() {
        use super::*;
        use crate::construction::build_state::BuildGhostState;
        use crate::construction::build_tool_authority::ActiveBuildTool;
        use crate::construction::build_strip::BuildStripState;

        let entry = blueprint_preset_entry_from_pending(
            "depot_a",
            SiteArchetype::RailDepot,
            BuildSiteTile { x: 4, z: 8 },
            FootprintTiles {
                width: 2,
                depth: 2,
            },
            "Surface",
            1,
            false,
        );
        let mut ghost = BuildGhostState::default();
        let mut tool = ActiveBuildTool::default();
        let mut strip = BuildStripState::default();
        apply_blueprint_preset_to_build_ghost(&entry, &mut ghost, &mut tool, &mut strip);
        assert_eq!(ghost.origin, Some(BuildSiteTile { x: 4, z: 8 }));
        assert_eq!(ghost.footprint.width, 2);
        assert_eq!(ghost.rotation_quarter_turns, 1);
        assert!(!ghost.mirror_x);
        assert!(matches!(tool.tool, BuildTool::Building(BuildingArchetypeId::Depot)));
    }

    #[test]
    fn blueprint_preset_collection_ron_roundtrip() {
        let mut collection = BlueprintPresetCollectionR8::new();
        collection.push(blueprint_preset_entry_from_pending(
            "depot_a",
            SiteArchetype::RailDepot,
            BuildSiteTile { x: 4, z: 8 },
            FootprintTiles {
                width: 2,
                depth: 2,
            },
            "Surface",
            1,
            false,
        ));
        let ron = ron::ser::to_string(&collection).expect("serialize");
        let back: BlueprintPresetCollectionR8 = ron::from_str(&ron).expect("deserialize");
        assert_eq!(collection, back);
    }
}
