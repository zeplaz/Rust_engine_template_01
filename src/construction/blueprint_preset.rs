//! Blueprint preset serialization — Wave **S** RON DTOs (no placement authority).

use serde::{Deserialize, Serialize};

use crate::strategic::{BuildSiteTile, FootprintTiles, SiteArchetype};

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

#[cfg(test)]
mod tests {
    use super::*;

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
