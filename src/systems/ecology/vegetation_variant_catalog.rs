//! **VEG-CATALOG-LOADER-001** — landscape vegetation variant catalog (mirror PT-4 `VariantCatalog`).

use std::collections::BTreeSet;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::Deserialize;

pub const VEGETATION_VARIANT_CATALOG_RON: &str =
    "assets/configs/landscape/_vegetation_variant_catalog.ron";

/// Byte-parity `veg_*` keys emitted by `variant_key_for_burn_row`.
pub const ENGINE_VEG_RESOLVER_KEYS: &[&str] = &[
    "veg_clean_day",
    "veg_old_growth",
    "veg_damaged",
    "veg_regrowth_nuclei",
    "veg_regrowth_front",
    "veg_burn_00",
    "veg_burn_01",
    "veg_burn_02",
    "veg_burn_03",
    "veg_burn_04",
    "veg_burn_05",
    "veg_burn_06",
    "veg_burn_07",
];

#[derive(Debug, Clone, Deserialize)]
pub struct VegetationVariantCatalogEntry {
    pub variant_key: String,
}

#[derive(Resource, Debug, Clone, Deserialize)]
pub struct VegetationVariantCatalog {
    pub schema_version: u32,
    #[serde(default)]
    pub catalog_id: String,
    #[serde(default)]
    pub entries: Vec<VegetationVariantCatalogEntry>,
}

impl VegetationVariantCatalog {
    #[must_use]
    pub fn veg_keys(&self) -> BTreeSet<String> {
        self.entries
            .iter()
            .filter(|e| e.variant_key.starts_with("veg_"))
            .map(|e| e.variant_key.clone())
            .collect()
    }

    #[must_use]
    pub fn has_all_resolver_keys(&self) -> bool {
        let keys = self.veg_keys();
        ENGINE_VEG_RESOLVER_KEYS
            .iter()
            .all(|k| keys.contains(*k))
    }

    #[must_use]
    pub fn contains_variant_key(&self, key: &str) -> bool {
        self.entries.iter().any(|e| e.variant_key == key)
    }
}

/// All catalog variant keys (veg + topology).
#[must_use]
pub fn catalog_variant_key_set(catalog: &VegetationVariantCatalog) -> BTreeSet<String> {
    catalog
        .entries
        .iter()
        .map(|e| e.variant_key.clone())
        .collect()
}

/// Clamp extract / burn resolver output to a catalog row (PT-4 parallel).
#[must_use]
pub fn clamp_vegetation_variant_to_catalog(
    catalog: &VegetationVariantCatalog,
    variant_key: &str,
) -> String {
    if catalog.contains_variant_key(variant_key) {
        return variant_key.to_owned();
    }
    if variant_key.starts_with("veg_topo_") {
        let topo = variant_key.replacen("veg_topo_", "topology_", 1);
        if catalog.contains_variant_key(&topo) {
            return topo;
        }
    }
    catalog
        .entries
        .iter()
        .find(|e| e.variant_key.starts_with("veg_"))
        .map(|e| e.variant_key.clone())
        .unwrap_or_else(|| "veg_clean_day".to_owned())
}

/// **VEG-CATALOG-RESOLVE-001** — resolve extract row variant_key through catalog authority.
#[must_use]
pub fn resolve_vegetation_variant(
    catalog: &VegetationVariantCatalog,
    variant_key: &str,
) -> String {
    clamp_vegetation_variant_to_catalog(catalog, variant_key)
}

/// Lib witness predicate for catalog clamp (all engine veg keys + topo remap).
#[must_use]
pub fn catalog_clamp_witness_green() -> bool {
    let Some(catalog) = load_vegetation_variant_catalog() else {
        return false;
    };
    for key in ENGINE_VEG_RESOLVER_KEYS {
        if resolve_vegetation_variant(&catalog, key) != *key {
            return false;
        }
    }
    resolve_vegetation_variant(&catalog, "veg_topo_patch") == "topology_patch"
        && catalog.contains_variant_key(&resolve_vegetation_variant(&catalog, "veg_unknown"))
}

#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

#[must_use]
pub fn load_vegetation_variant_catalog() -> Option<VegetationVariantCatalog> {
    let path = repo_asset_path(VEGETATION_VARIANT_CATALOG_RON);
    let text = std::fs::read_to_string(&path).ok()?;
    match ron::from_str(&text) {
        Ok(catalog) => Some(catalog),
        #[cfg(test)]
        Err(err) => {
            eprintln!("VegetationVariantCatalog RON parse failed ({path:?}): {err}");
            None
        }
        #[cfg(not(test))]
        Err(_) => None,
    }
}

pub fn init_vegetation_variant_catalog(mut commands: Commands) {
    match load_vegetation_variant_catalog() {
        Some(catalog) => {
            commands.insert_resource(catalog);
        }
        None => {
            warn!(
                "VegetationVariantCatalog missing or invalid at {VEGETATION_VARIANT_CATALOG_RON}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vegetation_variant_catalog_loads_all_resolver_veg_keys() {
        let catalog = load_vegetation_variant_catalog().expect("catalog on disk");
        assert!(
            catalog.has_all_resolver_keys(),
            "missing keys: {:?}",
            ENGINE_VEG_RESOLVER_KEYS
                .iter()
                .filter(|k| !catalog.veg_keys().contains(**k))
                .collect::<Vec<_>>()
        );
        assert_eq!(catalog.veg_keys().len(), ENGINE_VEG_RESOLVER_KEYS.len());
    }

    #[test]
    fn resolve_vegetation_variant_clamps_topo_and_unknown() {
        let catalog = load_vegetation_variant_catalog().expect("catalog on disk");
        assert!(catalog_clamp_witness_green());
        assert_eq!(
            resolve_vegetation_variant(&catalog, "veg_topo_corridor"),
            "topology_corridor"
        );
    }
}
