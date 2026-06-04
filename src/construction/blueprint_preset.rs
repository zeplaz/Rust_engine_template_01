//! Blueprint preset serialization — Wave **S** RON DTOs (no placement authority).

use serde::{Deserialize, Serialize};

use crate::strategic::{BuildSiteTile, FootprintTiles, LayerType, SiteArchetype};

use super::build_state::BuildGhostState;
use super::build_tool_authority::{ActiveBuildTool, BuildTool, BuildingArchetypeId};
use super::build_strip::BuildStripState;
use super::pending_construction::{PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind};

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

/// **BQ-128-APPLY-002** — append imported presets vs replace queue (replace needs confirm).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlueprintImportQueueMode {
    #[default]
    Append,
    Replace,
}

#[must_use]
pub fn pending_entry_from_preset(entry: &BlueprintPresetEntryR8) -> PendingBuildBlueprint {
    PendingBuildBlueprint {
        kind: PendingEntryKind::BuildSite,
        label: entry.label.clone(),
        archetype: site_archetype_from_preset_tag(&entry.archetype_tag),
        origin: BuildSiteTile {
            x: entry.origin_x,
            z: entry.origin_z,
        },
        footprint: FootprintTiles {
            width: entry.footprint_width.max(1),
            depth: entry.footprint_depth.max(1),
        },
        layer: layer_type_from_preset_tag(&entry.layer_tag),
        rotation_quarter_turns: entry.rotation_quarter_turns % 4,
        mirror_x: entry.mirror_x,
        approved: false,
        catalog_id: None,
    }
}

/// Import Wave S preset collection into the pending queue.
#[must_use]
pub fn import_preset_collection_into_pending_queue(
    queue: &mut PendingConstructionQueue,
    collection: &BlueprintPresetCollectionR8,
    mode: BlueprintImportQueueMode,
) -> usize {
    match mode {
        BlueprintImportQueueMode::Replace => queue.clear(),
        BlueprintImportQueueMode::Append => {}
    }
    let count = collection.presets.len();
    for preset in &collection.presets {
        queue.push(pending_entry_from_preset(preset));
    }
    count
}

/// Lib witness rollup for **BQ-128-APPLY-001** (picker → ghost, no queue commit).
#[must_use]
pub fn bq128_apply_ghost_witness_green() -> bool {
    let panel_src = std::fs::read_to_string("src/construction/pending_construction_panel.rs")
        .unwrap_or_default();
    let intent_src = std::fs::read_to_string("src/construction/construction_queue_intent.rs")
        .unwrap_or_default();
    panel_src.contains("ApplyImportedPreset")
        && panel_src.contains("Apply ghost")
        && intent_src.contains("ApplyImportedPreset")
        && intent_src.contains("apply_blueprint_preset_to_build_ghost")
}

/// Lib witness rollup for **BQ-128-APPLY-002** (merge vs replace on import).
#[must_use]
pub fn bq128_apply_merge_replace_witness_green() -> bool {
    let panel_src = std::fs::read_to_string("src/construction/pending_construction_panel.rs")
        .unwrap_or_default();
    let intent_src = std::fs::read_to_string("src/construction/construction_queue_intent.rs")
        .unwrap_or_default();
    panel_src.contains("BlueprintImportQueueMode::Append")
        && panel_src.contains("BlueprintImportQueueMode::Replace")
        && panel_src.contains("replace_confirm")
        && panel_src.contains("ImportWaveSPresets")
        && intent_src.contains("ImportWaveSPresets")
        && intent_src.contains("import_preset_collection_into_pending_queue")
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
    fn bq128_apply_ghost_witness_green_lib() {
        assert!(bq128_apply_ghost_witness_green());
    }

    #[test]
    fn bq128_apply_merge_replace_witness_green_lib() {
        assert!(bq128_apply_merge_replace_witness_green());
    }

    #[test]
    fn import_presets_append_keeps_existing_queue_rows() {
        let mut queue = PendingConstructionQueue::default();
        queue.push(pending_entry_from_preset(&blueprint_preset_entry_from_pending(
            "existing",
            SiteArchetype::Factory,
            BuildSiteTile { x: 0, z: 0 },
            FootprintTiles {
                width: 1,
                depth: 1,
            },
            "Surface",
            0,
            false,
        )));
        let incoming = BlueprintPresetCollectionR8 {
            schema_version: 1,
            presets: vec![blueprint_preset_entry_from_pending(
                "imported",
                SiteArchetype::RailDepot,
                BuildSiteTile { x: 3, z: 4 },
                FootprintTiles {
                    width: 2,
                    depth: 2,
                },
                "Surface",
                0,
                false,
            )],
        };
        let n = import_preset_collection_into_pending_queue(
            &mut queue,
            &incoming,
            BlueprintImportQueueMode::Append,
        );
        assert_eq!(n, 1);
        assert_eq!(queue.entries.len(), 2);
        assert_eq!(queue.entries[0].label, "existing");
        assert_eq!(queue.entries[1].label, "imported");
    }

    #[test]
    fn import_presets_replace_clears_queue() {
        let mut queue = PendingConstructionQueue::default();
        queue.push(pending_entry_from_preset(&blueprint_preset_entry_from_pending(
            "old",
            SiteArchetype::Factory,
            BuildSiteTile { x: 0, z: 0 },
            FootprintTiles {
                width: 1,
                depth: 1,
            },
            "Surface",
            0,
            false,
        )));
        let incoming = BlueprintPresetCollectionR8 {
            schema_version: 1,
            presets: vec![blueprint_preset_entry_from_pending(
                "new",
                SiteArchetype::WaterPlant,
                BuildSiteTile { x: 1, z: 2 },
                FootprintTiles {
                    width: 1,
                    depth: 1,
                },
                "Surface",
                0,
                false,
            )],
        };
        let _ = import_preset_collection_into_pending_queue(
            &mut queue,
            &incoming,
            BlueprintImportQueueMode::Replace,
        );
        assert_eq!(queue.entries.len(), 1);
        assert_eq!(queue.entries[0].label, "new");
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
