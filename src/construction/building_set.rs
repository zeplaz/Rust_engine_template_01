//! BUILD-SET-MANIFEST-v1 — insured pilot sets (`_building_sets.ron` + ARCH-DNA F-axis coverage).

use std::fs;

use bevy::prelude::*;
use serde::Deserialize;

use super::pilot_catalog::repo_asset_path;
use super::procedural::load_preset_for_id;

pub const BUILDING_SETS_RON: &str = "assets/configs/buildings/_building_sets.ron";

#[derive(Debug, Clone, Deserialize)]
struct BuildingSetsFile {
    sets: Vec<BuildingSetEntryRon>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuildingSetEntryRon {
    pub set_id: String,
    pub label: String,
    pub min_grammar_pilots: u32,
    pub pilot_ids: Vec<String>,
    pub required_f_functions: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct BuildingSetManifest {
    pub sets: Vec<BuildingSetEntryRon>,
    pub load_errors: Vec<String>,
}

impl BuildingSetManifest {
    #[must_use]
    pub fn load_from_disk() -> Self {
        let mut manifest = Self::default();
        let path = repo_asset_path(BUILDING_SETS_RON);
        let Ok(text) = fs::read_to_string(&path) else {
            manifest
                .load_errors
                .push(format!("building sets missing: {}", path.display()));
            return manifest;
        };
        let Ok(file) = ron::from_str::<BuildingSetsFile>(&text) else {
            manifest
                .load_errors
                .push(format!("building sets parse failed: {}", path.display()));
            return manifest;
        };
        manifest.sets = file.sets;
        manifest
    }
}

#[must_use]
pub fn arch_dna_f_function(preset_id: &str) -> Option<String> {
    load_preset_for_id(preset_id).ok().and_then(|p| {
        p.arch_dna
            .get("F")
            .and_then(|v| v.as_str().map(str::to_lowercase))
    })
}

#[cfg(test)]
mod tests {
    

}
