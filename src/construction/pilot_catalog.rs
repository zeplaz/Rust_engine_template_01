//! BUILD-READ-PILOT-001 — authoritative pilot registry (`_pilot_catalog.ron`).
//!
//! All shape QA + grammar pilots load from one RON file. Rust must not branch on individual pilot ids.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use bevy::prelude::*;
use serde::de::Deserializer;
use serde::Deserialize;

fn de_ron_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RonOptString {
        Some(String),
        Bare(String),
    }
    match RonOptString::deserialize(deserializer)? {
        RonOptString::Some(s) | RonOptString::Bare(s) => {
            if s.is_empty() {
                Ok(None)
            } else {
                Ok(Some(s))
            }
        }
    }
}

use crate::strategic::SiteArchetype;

use super::building_catalog::{BuildingFamily, FootprintMatrix};
use super::site_zone_grid::{load_site_zone_grid_from_path, SiteZoneGrid};

pub const PILOT_CATALOG_RON: &str = "assets/configs/buildings/_pilot_catalog.ron";
pub const MOCK_SHAPES_RON: &str = "assets/configs/buildings/_mock_shapes.ron";
pub const BUILDINGS_CONFIG_ROOT: &str = "assets/configs/buildings";
pub const ARCH_DNA_EXAMPLES_DIR: &str = "tools/mcp/schemas/examples";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PilotKind {
    #[default]
    ShapeQa,
    Grammar,
}

#[derive(Debug, Clone, Deserialize)]
struct MockShapesFile {
    shapes: Vec<MockShapeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct MockShapeEntry {
    id: String,
    label: String,
    width: u32,
    depth: u32,
    cells: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
struct PilotCatalogFile {
    pilots: Vec<PilotCatalogEntryRon>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PilotCatalogEntryRon {
    pub id: String,
    pub label: String,
    #[serde(default, deserialize_with = "de_ron_optional_string")]
    pub mock_shape_id: Option<String>,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub depth: u32,
    #[serde(default)]
    pub cells: Vec<u8>,
    #[serde(default, deserialize_with = "de_ron_optional_string")]
    pub arch_dna_preset: Option<String>,
    #[serde(default, deserialize_with = "de_ron_optional_string")]
    pub grammar_archetype_id: Option<String>,
    #[serde(default, deserialize_with = "de_ron_optional_string")]
    pub district_style: Option<String>,
    #[serde(default, deserialize_with = "de_ron_optional_string")]
    pub site_json_path: Option<String>,
    #[serde(default)]
    pub pilot_kind: Option<PilotKind>,
    #[serde(default, deserialize_with = "de_ron_optional_string")]
    pub hover_hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedPilotEntry {
    pub id: String,
    pub catalog_id: String,
    pub label: String,
    pub footprint: FootprintMatrix,
    pub arch_dna_preset: Option<String>,
    pub grammar_archetype_id: Option<String>,
    pub district_style: Option<String>,
    pub site_json_path: Option<String>,
    pub pilot_kind: PilotKind,
    pub hover_hint: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PilotCatalog {
    pub pilots: Vec<ResolvedPilotEntry>,
    pub load_errors: Vec<String>,
}

#[must_use]
pub fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

fn load_mock_shapes_index() -> HashMap<String, MockShapeEntry> {
    let path = repo_asset_path(MOCK_SHAPES_RON);
    let Ok(text) = fs::read_to_string(&path) else {
        return HashMap::new();
    };
    let Ok(file) = ron::from_str::<MockShapesFile>(&text) else {
        return HashMap::new();
    };
    file.shapes
        .into_iter()
        .map(|s| (s.id.clone(), s))
        .collect()
}

fn footprint_from_entry(
    entry: &PilotCatalogEntryRon,
    mock_index: &HashMap<String, MockShapeEntry>,
) -> Result<FootprintMatrix, &'static str> {
    if let Some(shape_id) = entry.mock_shape_id.as_deref() {
        let shape = mock_index.get(shape_id).ok_or("mock_shape_missing")?;
        let _ = &shape.label;
        let expected = (shape.width as usize) * (shape.depth as usize);
        if shape.cells.len() != expected {
            return Err("mock_shape_cells");
        }
        return Ok(FootprintMatrix {
            width: shape.width,
            depth: shape.depth,
            cells: shape.cells.clone(),
        });
    }
    let expected = (entry.width as usize) * (entry.depth as usize);
    if entry.width == 0 || entry.depth == 0 || entry.cells.len() != expected {
        return Err("inline_cells");
    }
    Ok(FootprintMatrix {
        width: entry.width,
        depth: entry.depth,
        cells: entry.cells.clone(),
    })
}

impl PilotCatalog {
    #[must_use]
    pub fn load_from_disk() -> Self {
        let mut catalog = Self::default();
        let path = repo_asset_path(PILOT_CATALOG_RON);
        let Ok(text) = fs::read_to_string(&path) else {
            catalog
                .load_errors
                .push(format!("pilot catalog missing: {}", path.display()));
            return catalog;
        };
        let file: PilotCatalogFile = match ron::from_str(&text) {
            Ok(file) => file,
            Err(err) => {
                catalog.load_errors.push(format!(
                    "pilot catalog parse failed: {} ({err})",
                    path.display()
                ));
                return catalog;
            }
        };
        let mock_index = load_mock_shapes_index();
        for entry in file.pilots {
            let footprint = match footprint_from_entry(&entry, &mock_index) {
                Ok(fp) => fp,
                Err(reason) => {
                    catalog.load_errors.push(format!(
                        "pilot {} footprint: {reason}",
                        entry.id
                    ));
                    continue;
                }
            };
            let pilot_kind = entry.pilot_kind.unwrap_or(if entry.arch_dna_preset.is_some() {
                PilotKind::Grammar
            } else {
                PilotKind::ShapeQa
            });
            let pilot_id = entry.id.clone();
            catalog.pilots.push(ResolvedPilotEntry {
                catalog_id: format!("pilot:{}", pilot_id),
                id: pilot_id.clone(),
                label: if entry.label.is_empty() {
                    entry
                        .mock_shape_id
                        .as_deref()
                        .and_then(|id| mock_index.get(id).map(|s| s.label.clone()))
                        .unwrap_or(pilot_id)
                } else {
                    entry.label
                },
                footprint,
                arch_dna_preset: entry.arch_dna_preset,
                grammar_archetype_id: entry.grammar_archetype_id,
                district_style: entry.district_style,
                site_json_path: entry.site_json_path,
                pilot_kind,
                hover_hint: entry.hover_hint,
            });
        }
        catalog
    }

    #[must_use]
    pub fn by_id(&self, pilot_id: &str) -> Option<&ResolvedPilotEntry> {
        self.pilots.iter().find(|p| p.id == pilot_id)
    }

    #[must_use]
    pub fn by_catalog_id(&self, catalog_id: &str) -> Option<&ResolvedPilotEntry> {
        self.pilots.iter().find(|p| p.catalog_id == catalog_id)
    }

    #[must_use]
    pub fn by_arch_dna_preset(&self, preset: &str) -> Option<&ResolvedPilotEntry> {
        self.pilots
            .iter()
            .find(|p| p.arch_dna_preset.as_deref() == Some(preset))
    }

    #[must_use]
    pub fn grammar_pilots(&self) -> impl Iterator<Item = &ResolvedPilotEntry> {
        self.pilots
            .iter()
            .filter(|p| p.pilot_kind == PilotKind::Grammar)
    }

    #[must_use]
    pub fn first_grammar_pilot(&self) -> Option<&ResolvedPilotEntry> {
        self.grammar_pilots().next()
    }

    /// First grammar pilot arch_dna preset id — witness/proof paths avoid hardcoded pilot needles.
    #[must_use]
    pub fn first_grammar_arch_dna_preset_id(&self) -> Option<String> {
        self.first_grammar_pilot()
            .and_then(|p| p.arch_dna_preset.clone())
    }

    #[must_use]
    pub fn non_rect_shape_pilot(&self) -> Option<&ResolvedPilotEntry> {
        self.pilots.iter().find(|p| {
            p.pilot_kind == PilotKind::ShapeQa && p.footprint.is_non_rectangular()
        })
    }
}

static SITE_GRID_CACHE: OnceLock<Mutex<HashMap<String, SiteZoneGrid>>> = OnceLock::new();

/// Load site zone grid by path relative to `assets/configs/buildings/`.
#[must_use]
pub fn load_site_zone_grid_cached(rel_path: &str) -> Option<SiteZoneGrid> {
    let cache = SITE_GRID_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let Ok(mut guard) = cache.lock() else {
        return None;
    };
    if let Some(grid) = guard.get(rel_path) {
        return Some(grid.clone());
    }
    let path = repo_asset_path(BUILDINGS_CONFIG_ROOT).join(rel_path);
    let grid = load_site_zone_grid_from_path(&path)?;
    guard.insert(rel_path.to_string(), grid.clone());
    Some(grid)
}

#[must_use]
pub fn site_zone_grid_for_arch_dna_preset(preset: &str) -> Option<SiteZoneGrid> {
    let catalog = PilotCatalog::load_from_disk();
    let entry = catalog.by_arch_dna_preset(preset)?;
    let rel = entry.site_json_path.as_deref()?;
    load_site_zone_grid_cached(rel)
}

/// Registration defaults for a resolved pilot row.
#[must_use]
pub fn pilot_building_registration(
    entry: &ResolvedPilotEntry,
) -> (
    String,
    u32,
    u32,
    f32,
    u32,
    SiteArchetype,
    BuildingFamily,
    bool,
) {
    match entry.pilot_kind {
        PilotKind::ShapeQa => (
            entry.label.clone(),
            1,
            1,
            0.0,
            0,
            SiteArchetype::Factory,
            BuildingFamily::Industry,
            false,
        ),
        PilotKind::Grammar => (
            entry.label.clone(),
            800,
            120,
            24.0,
            4,
            SiteArchetype::Factory,
            BuildingFamily::Industry,
            true,
        ),
    }
}

/// BUILD-READ-PILOT-001 — all catalog pilots register; shape + grammar edge cases covered.
#[must_use]
pub fn pilot_catalog_parity_witness_green() -> bool {
    pilot_catalog_parity_self_check().is_ok()
}

/// Alias for BUILD-READ-SHAPE-002 board row.
#[must_use]
pub fn build_read_shape_002_witness_green() -> bool {
    pilot_catalog_parity_witness_green()
}

fn pilot_catalog_parity_self_check() -> Result<(), &'static str> {
    let catalog = PilotCatalog::load_from_disk();
    if !catalog.load_errors.is_empty() {
        return Err("catalog_load_errors");
    }
    if catalog.pilots.len() < 8 {
        return Err("min_pilot_count");
    }

    let reg = super::building_definitions::load_building_definitions_from_dir(
        super::building_definitions::default_buildings_dir(),
    );

    let mut shape_qa = 0u32;
    let mut grammar = 0u32;

    for pilot in &catalog.pilots {
        let def = reg.get(&pilot.catalog_id).ok_or("registry_missing")?;
        if def.footprint.occupied_count() != pilot.footprint.occupied_count() {
            return Err("footprint_parity");
        }
        if def.display_name != pilot.label {
            return Err("label_parity");
        }
        match pilot.pilot_kind {
            PilotKind::ShapeQa => {
                shape_qa += 1;
                if pilot.id == "shape_rectangle_2x2" {
                    if def.footprint.is_non_rectangular() {
                        return Err("rect_should_be_full");
                    }
                    if def.footprint.occupied_count() != 4 {
                        return Err("rect_occupied");
                    }
                }
                if pilot.id == "shape_l_3x2" && !def.footprint.is_non_rectangular() {
                    return Err("l_non_rect");
                }
                if pilot.id == "shape_t_3x3" {
                    if def.footprint.cells.get(4).copied() != Some(1) {
                        return Err("t_stem_center");
                    }
                    if def.footprint.cells.get(3).copied() != Some(0)
                        || def.footprint.cells.get(5).copied() != Some(0)
                    {
                        return Err("t_side_void");
                    }
                }
                if pilot.id == "shape_o_3x3" {
                    if def.footprint.cells.get(4).copied() != Some(0) {
                        return Err("o_hollow_center");
                    }
                }
            }
            PilotKind::Grammar => {
                grammar += 1;
                if pilot.arch_dna_preset.is_none() {
                    return Err("grammar_missing_dna");
                }
                if pilot.grammar_archetype_id.is_none() {
                    return Err("grammar_missing_archetype");
                }
                if let Some(rel) = pilot.site_json_path.as_deref() {
                    let grid = load_site_zone_grid_cached(rel).ok_or("grammar_site_load")?;
                    if grid.width < 4 || grid.depth < 4 {
                        return Err("grammar_site_dims");
                    }
                }
            }
        }
    }

    if shape_qa < 4 {
        return Err("shape_qa_count");
    }
    if grammar < 4 {
        return Err("grammar_count");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pilot_catalog_loads_eight_pilots() {
        let catalog = PilotCatalog::load_from_disk();
        assert!(
            catalog.pilots.len() >= 8,
            "pilots={:?} errors={:?}",
            catalog.pilots.iter().map(|p| &p.id).collect::<Vec<_>>(),
            catalog.load_errors
        );
    }

    #[test]
    fn pilot_catalog_parity_witness() {
        match pilot_catalog_parity_self_check() {
            Ok(()) => {}
            Err(reason) => panic!("pilot_catalog_parity failed: {reason}"),
        }
    }
}
