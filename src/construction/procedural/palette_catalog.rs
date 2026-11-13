//! **CITY-G2-C5-001** — palette catalog load + deterministic variation resolver (CITY-C5).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use super::module_index::ProceduralModuleEntry;
use crate::strategic::settlement::mix_u64;

pub const PALETTE_CATALOG_INDEX_RON: &str = "assets/configs/buildings/_palette_catalog_index.ron";
#[derive(Debug, Clone, Deserialize)]
struct PaletteCatalogIndexFile {
    #[serde(default)]
    schema_version: u32,
    catalogs: Vec<PaletteCatalogRefRon>,
    district_style_defaults: Vec<DistrictStylePaletteRon>,
}

#[derive(Debug, Clone, Deserialize)]
struct PaletteCatalogRefRon {
    palette_id: String,
    path: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DistrictStylePaletteRon {
    district_style: String,
    palette_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PaletteCatalogFile {
    schema: String,
    palette_id: String,
    style_pack: String,
    label: String,
    variations: Vec<PaletteVariationFile>,
}

#[derive(Debug, Clone, Deserialize)]
struct PaletteVariationFile {
    variation_id: String,
    label: String,
    material_slots: MaterialSlotsRon,
    #[serde(default)]
    variant_tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct MaterialSlotsRon {
    #[serde(default)]
    wall_primary: String,
    #[serde(default)]
    wall_secondary: String,
    #[serde(default)]
    trim: String,
    #[serde(default)]
    roof: String,
    #[serde(default)]
    door: String,
    #[serde(default)]
    window: String,
    #[serde(default)]
    foundation: String,
}

impl MaterialSlotsRon {
    fn into_map(self) -> HashMap<String, String> {
        let mut out = HashMap::new();
        for (key, value) in [
            ("wall_primary", self.wall_primary),
            ("wall_secondary", self.wall_secondary),
            ("trim", self.trim),
            ("roof", self.roof),
            ("door", self.door),
            ("window", self.window),
            ("foundation", self.foundation),
        ] {
            if !value.is_empty() {
                out.insert(key.to_owned(), value);
            }
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct PaletteVariation {
    pub variation_id: String,
    pub label: String,
    pub material_slots: HashMap<String, String>,
    pub variant_tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PaletteCatalog {
    pub palette_id: String,
    pub style_pack: String,
    pub label: String,
    pub variations: Vec<PaletteVariation>,
}

#[derive(Resource, Debug, Default)]
pub struct PaletteCatalogRegistry {
    pub schema_version: u32,
    pub catalogs: HashMap<String, PaletteCatalog>,
    pub district_style_to_palette: HashMap<String, String>,
    pub load_errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPaletteVariation {
    pub palette_id: String,
    pub variation_id: String,
    pub visual_variant_id: String,
    pub seed_pick_index: u32,
    pub material_slots: HashMap<String, String>,
}

impl PaletteCatalogRegistry {
    #[must_use]
    pub fn get(&self, palette_id: &str) -> Option<&PaletteCatalog> {
        self.catalogs.get(palette_id)
    }

    #[must_use]
    pub fn palette_id_for_district_style(&self, district_style: &str) -> Option<&str> {
        self.district_style_to_palette
            .get(district_style)
            .map(|s| s.as_str())
    }
}

impl PaletteCatalog {
    #[must_use]
    pub fn variation_count(&self) -> u32 {
        u32::try_from(self.variations.len()).unwrap_or(1).max(1)
    }

    #[must_use]
    pub fn variation_at(&self, index: u32) -> Option<&PaletteVariation> {
        self.variations.get(index as usize)
    }

    #[must_use]
    pub fn variation_by_id(&self, variation_id: &str) -> Option<&PaletteVariation> {
        self.variations
            .iter()
            .find(|v| v.variation_id == variation_id)
    }
}

#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

fn load_palette_catalog_file(path: &Path) -> Result<PaletteCatalog, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("palette catalog read {}: {e}", path.display()))?;
    let file: PaletteCatalogFile = ron::from_str(&text)
        .map_err(|e| format!("palette catalog parse {}: {e}", path.display()))?;
    if file.schema != "palette_catalog_v1" {
        return Err(format!(
            "palette catalog {}: expected schema palette_catalog_v1, got {}",
            path.display(),
            file.schema
        ));
    }
    if file.variations.len() < 2 {
        return Err(format!(
            "palette catalog {}: need >=2 variations",
            path.display()
        ));
    }
    Ok(PaletteCatalog {
        palette_id: file.palette_id,
        style_pack: file.style_pack,
        label: file.label,
        variations: file
            .variations
            .into_iter()
            .map(|v| PaletteVariation {
                variation_id: v.variation_id,
                label: v.label,
                material_slots: v.material_slots.into_map(),
                variant_tags: v.variant_tags,
            })
            .collect(),
    })
}

#[must_use]
pub fn palette_variation_pick_index(
    lot_seed: u64,
    module_id: &str,
    palette_family: &str,
    variation_count: u32,
) -> u32 {
    let count = variation_count.max(1);
    let key = format!("{module_id}:{palette_family}");
    let mixed = mix_u64(lot_seed, "palette_variation", &key);
    (mixed % u64::from(count)) as u32
}

#[must_use]
pub fn visual_variant_id(module_id: &str, palette_id: &str, variation_id: &str) -> String {
    format!("{module_id}::{palette_id}::{variation_id}")
}

#[must_use]
pub fn resolve_palette_variation(
    lot_seed: u64,
    module_id: &str,
    palette_id: &str,
    catalog: &PaletteCatalog,
    module_entry: Option<&ProceduralModuleEntry>,
) -> Option<ResolvedPaletteVariation> {
    let count = catalog.variation_count();
    if let Some(entry) = module_entry {
        if !entry.palette_family.is_empty()
            && entry.palette_variation_count > 0
            && u32::from(entry.palette_variation_count) != count
        {
            warn!(
                target: "palette_catalog",
                module_id = %module_id,
                palette_id = %palette_id,
                index_count = entry.palette_variation_count,
                catalog_count = count,
                "palette_variation_count mismatch — using catalog len"
            );
        }
    }
    let pick = palette_variation_pick_index(lot_seed, module_id, palette_id, count);
    let variation = catalog.variation_at(pick)?;
    Some(ResolvedPaletteVariation {
        palette_id: palette_id.to_owned(),
        variation_id: variation.variation_id.clone(),
        visual_variant_id: visual_variant_id(module_id, palette_id, &variation.variation_id),
        seed_pick_index: pick,
        material_slots: variation.material_slots.clone(),
    })
}

#[must_use]
pub fn resolve_palette_variation_default(
    module_id: &str,
    palette_id: &str,
    catalog: &PaletteCatalog,
    module_entry: Option<&ProceduralModuleEntry>,
) -> Option<ResolvedPaletteVariation> {
    let default_id = module_entry
        .and_then(|e| {
            if e.default_variation_id.is_empty() {
                None
            } else {
                Some(e.default_variation_id.as_str())
            }
        })
        .or_else(|| catalog.variations.first().map(|v| v.variation_id.as_str()))?;
    let variation = catalog.variation_by_id(default_id)?;
    Some(ResolvedPaletteVariation {
        palette_id: palette_id.to_owned(),
        variation_id: variation.variation_id.clone(),
        visual_variant_id: visual_variant_id(module_id, palette_id, &variation.variation_id),
        seed_pick_index: 0,
        material_slots: variation.material_slots.clone(),
    })
}

#[must_use]
pub fn load_palette_catalog_registry_from_path(path: &Path) -> PaletteCatalogRegistry {
    let mut registry = PaletteCatalogRegistry::default();
    let Ok(text) = std::fs::read_to_string(path) else {
        registry.load_errors.push(format!(
            "palette catalog index not found: {}",
            path.display()
        ));
        return registry;
    };
    let index: PaletteCatalogIndexFile = match ron::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            registry
                .load_errors
                .push(format!("palette catalog index parse: {e}"));
            return registry;
        }
    };
    registry.schema_version = index.schema_version;
    for row in index.district_style_defaults {
        registry
            .district_style_to_palette
            .insert(row.district_style, row.palette_id);
    }
    for row in index.catalogs {
        let cat_path = repo_asset_path(&row.path.replace('\\', "/"));
        match load_palette_catalog_file(&cat_path) {
            Ok(catalog) => {
                if catalog.palette_id != row.palette_id {
                    registry.load_errors.push(format!(
                        "palette id mismatch index={} file={}",
                        row.palette_id, catalog.palette_id
                    ));
                    continue;
                }
                registry.catalogs.insert(row.palette_id, catalog);
            }
            Err(e) => registry.load_errors.push(e),
        }
    }
    registry
}

#[must_use]
pub fn load_palette_catalog_registry() -> PaletteCatalogRegistry {
    load_palette_catalog_registry_from_path(&repo_asset_path(PALETTE_CATALOG_INDEX_RON))
}

pub fn init_palette_catalog_registry(mut commands: Commands) {
    let registry = load_palette_catalog_registry();
    if !registry.load_errors.is_empty() {
        for err in &registry.load_errors {
            warn!(target: "palette_catalog", "{err}");
        }
    } else {
        info!(
            target: "palette_catalog",
            "PaletteCatalogRegistry: {} catalogs schema_v{}",
            registry.catalogs.len(),
            registry.schema_version
        );
    }
    commands.insert_resource(registry);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_catalog_registry_loads_three_v1_catalogs() {
        let reg = load_palette_catalog_registry();
        assert!(reg.load_errors.is_empty(), "{:?}", reg.load_errors);
        assert!(reg.get("palette_industrial_west_v1").is_some());
        assert!(reg.get("palette_colonial_res_v1").is_some());
        assert!(reg.get("palette_rowhouse_urban_v1").is_some());
    }

    #[test]
    fn resolve_palette_variation_is_stable_for_fixed_seed() {
        let reg = load_palette_catalog_registry();
        let catalog = reg
            .get("palette_industrial_west_v1")
            .expect("industrial catalog");
        let lot_seed = 0x59DCFEF41AF9F0F9;
        let a = resolve_palette_variation(
            lot_seed,
            "wall_concrete_2u",
            "palette_industrial_west_v1",
            catalog,
            None,
        )
        .expect("resolved");
        let b = resolve_palette_variation(
            lot_seed,
            "wall_concrete_2u",
            "palette_industrial_west_v1",
            catalog,
            None,
        )
        .expect("resolved");
        assert_eq!(a, b);
        assert!(a.visual_variant_id.contains("wall_concrete_2u::"));
    }

    #[test]
    fn district_style_maps_to_palette_id() {
        let reg = load_palette_catalog_registry();
        assert_eq!(
            reg.palette_id_for_district_style("industrial_west"),
            Some("palette_industrial_west_v1")
        );
    }

}
