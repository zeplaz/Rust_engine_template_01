//! BUILD-SET-MANIFEST-v1 — insured pilot sets (`_building_sets.ron` + ARCH-DNA F-axis coverage).

use std::fs;

use bevy::prelude::*;
use serde::Deserialize;

use super::pilot_catalog::{PilotCatalog, PilotKind, ARCH_DNA_EXAMPLES_DIR, repo_asset_path};
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
    load_preset_for_id(preset_id)
        .ok()
        .and_then(|p| p.arch_dna.get("F").and_then(|v| v.as_str().map(str::to_lowercase)))
}

/// BUILD-SET-001 — ≥2 grammar pilots + required F-axis coverage from manifest.
#[must_use]
pub fn building_set_coverage_witness_green() -> bool {
    building_set_coverage_self_check().is_ok()
}

fn building_set_coverage_self_check() -> Result<(), &'static str> {
    let manifest = BuildingSetManifest::load_from_disk();
    if !manifest.load_errors.is_empty() {
        return Err("manifest_load");
    }
    if manifest.sets.is_empty() {
        return Err("no_sets");
    }

    let catalog = PilotCatalog::load_from_disk();
    if !catalog.load_errors.is_empty() {
        return Err("catalog_load");
    }

    let grammar_count = catalog
        .pilots
        .iter()
        .filter(|p| p.pilot_kind == PilotKind::Grammar)
        .count();
    if grammar_count < 4 {
        return Err("min_grammar_pilots");
    }

    for set in &manifest.sets {
        let set_grammar = set
            .pilot_ids
            .iter()
            .filter(|id| {
                catalog
                    .by_id(id)
                    .is_some_and(|p| p.pilot_kind == PilotKind::Grammar)
            })
            .count();
        if set_grammar < set.min_grammar_pilots as usize {
            return Err("set_grammar_count");
        }
        for pilot_id in &set.pilot_ids {
            if catalog.by_id(pilot_id).is_none() {
                return Err("set_pilot_missing");
            }
        }
        let mut f_seen = std::collections::HashSet::new();
        for pilot_id in &set.pilot_ids {
            let Some(pilot) = catalog.by_id(pilot_id) else {
                continue;
            };
            let Some(preset) = pilot.arch_dna_preset.as_deref() else {
                continue;
            };
            if let Some(f) = arch_dna_f_function(preset) {
                f_seen.insert(f);
            }
        }
        for required in &set.required_f_functions {
            if !f_seen.contains(&required.to_lowercase()) {
                return Err("f_axis_missing");
            }
        }
    }

    let examples_dir = repo_asset_path(ARCH_DNA_EXAMPLES_DIR);
    let example_count = fs::read_dir(&examples_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter(|e| {
            e.as_ref().ok().and_then(|entry| {
                entry.path().file_name().and_then(|n| n.to_str()).map(|n| {
                    n.starts_with("arch_dna_") && n.ends_with(".json")
                })
            }) == Some(true)
        })
        .count();
    if example_count < 4 {
        return Err("min_arch_dna_examples");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_set_coverage_witness() {
        assert!(building_set_coverage_witness_green());
    }
}
