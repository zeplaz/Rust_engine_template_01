//! CargoKind registry (`assets/configs/economy/cargo_kinds_v1.ron`) — spine P0.

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::Deserialize;

use super::model::{CargoClass, CargoKind};
use crate::utils::StringId;

pub const CARGO_KINDS_RON: &str = "assets/configs/economy/cargo_kinds_v1.ron";

const EMBEDDED: &str = include_str!("../../../../assets/configs/economy/cargo_kinds_v1.ron");

#[derive(Debug, Clone, Deserialize)]
struct CargoKindsFile {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    table_id: String,
    kinds: Vec<CargoKindRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct CargoKindRow {
    tag: String,
    class: String,
    #[serde(default)]
    consumers: Vec<String>,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Clone)]
pub struct RegisteredCargoKind {
    pub tag: String,
    pub class: CargoClass,
    pub consumers: Vec<String>,
    pub notes: String,
}

/// Loaded cargo registry (sole writer: load on startup / Default).
///
/// Keyed by [`StringId`] (MEM-07: FNV-1a of the RON `tag`, deterministic across reloads); the
/// row keeps its `tag` so load can reject two distinct tags hashing to one id.
#[derive(Resource, Debug, Clone)]
pub struct CargoKindRegistry {
    pub by_id: HashMap<StringId, RegisteredCargoKind>,
    pub schema_version: u32,
    pub table_id: String,
}

impl Default for CargoKindRegistry {
    fn default() -> Self {
        load_cargo_kind_registry().0
    }
}

impl CargoKindRegistry {
    #[must_use]
    pub fn get(&self, tag: &str) -> Option<&RegisteredCargoKind> {
        self.get_by_id(StringId::new(tag))
    }

    /// Lookup by precomputed id (hash-stable catalog key).
    #[must_use]
    pub fn get_by_id(&self, id: StringId) -> Option<&RegisteredCargoKind> {
        self.by_id.get(&id)
    }

    #[must_use]
    pub fn contains(&self, tag: &str) -> bool {
        self.by_id.contains_key(&StringId::new(tag))
    }

    /// Registry class if known; else [`CargoKind::class`] heuristic.
    #[must_use]
    pub fn class_of(&self, cargo: &CargoKind) -> CargoClass {
        self.get(cargo.as_str())
            .map(|r| r.class)
            .unwrap_or_else(|| cargo.class())
    }
}

fn parse_class(s: &str) -> CargoClass {
    match s.trim().to_ascii_lowercase().as_str() {
        "bulk" => CargoClass::Bulk,
        "liquid" => CargoClass::Liquid,
        "personnel" => CargoClass::Personnel,
        "munitions" => CargoClass::Munitions,
        "deployable" => CargoClass::Deployable,
        _ => CargoClass::Bulk,
    }
}

fn parse_file(text: &str) -> Result<CargoKindRegistry, String> {
    let file: CargoKindsFile = ron::de::from_str(text).map_err(|e| e.to_string())?;
    let mut by_id: HashMap<StringId, RegisteredCargoKind> = HashMap::new();
    for row in file.kinds {
        let id = StringId::new(&row.tag);
        if let Some(prev) = by_id.get(&id) {
            if prev.tag != row.tag {
                return Err(format!("cargo tag hash collision {id}: `{}` vs `{}`", prev.tag, row.tag));
            }
        }
        by_id.insert(
            id,
            RegisteredCargoKind {
                tag: row.tag,
                class: parse_class(&row.class),
                consumers: row.consumers,
                notes: row.notes,
            },
        );
    }
    Ok(CargoKindRegistry {
        by_id,
        schema_version: file.schema_version,
        table_id: file.table_id,
    })
}

#[must_use]
pub fn load_cargo_kind_registry() -> (CargoKindRegistry, Option<String>) {
    let path = PathBuf::from(CARGO_KINDS_RON);
    if path.is_file() {
        match std::fs::read_to_string(&path) {
            Ok(text) => match parse_file(&text) {
                Ok(reg) => return (reg, None),
                Err(e) => {
                    return (
                        parse_file(EMBEDDED).expect("embedded cargo_kinds"),
                        Some(format!("disk parse failed ({e}); using embedded")),
                    );
                }
            },
            Err(e) => {
                return (
                    parse_file(EMBEDDED).expect("embedded cargo_kinds"),
                    Some(format!("disk read failed ({e}); using embedded")),
                );
            }
        }
    }
    (
        parse_file(EMBEDDED).expect("embedded cargo_kinds"),
        Some(format!("missing {}; using embedded", path.display())),
    )
}

pub fn load_cargo_kind_registry_on_startup(mut commands: Commands) {
    let (reg, err) = load_cargo_kind_registry();
    if let Some(err) = err {
        warn!("cargo_kinds registry: {err}");
    }
    commands.insert_resource(reg);
}

/// True when `tag` is registered or matches a known heuristic class prefix (legacy).
#[must_use]
pub fn cargo_tag_known(reg: &CargoKindRegistry, tag: &str) -> bool {
    if reg.contains(tag) {
        return true;
    }
    // Heuristic escape hatch for dynamic tags not yet migrated.
    let c = CargoKind::new(tag).class();
    !matches!(c, CargoClass::Bulk) || tag.chars().next().is_some_and(|ch| ch.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn embedded_has_drone_airframe_and_fuel() {
        let reg = parse_file(EMBEDDED).unwrap();
        assert!(reg.contains("drone_airframe"));
        assert!(reg.contains("Fuel"));
        assert!(reg.contains("personnel"));
        assert!(reg.contains("missile_body"));
        assert_eq!(reg.get("Oil").unwrap().class, CargoClass::Liquid);
        assert_eq!(reg.get("ammo").unwrap().class, CargoClass::Munitions);
        assert_eq!(reg.get("missile_body").unwrap().class, CargoClass::Munitions);
        assert_eq!(reg.table_id, "cargo_kinds_v1");
    }

    #[test]
    fn path_constant_exists_or_embedded() {
        let _ = Path::new(CARGO_KINDS_RON);
        let (reg, _) = load_cargo_kind_registry();
        assert!(!reg.by_id.is_empty());
    }

    /// MEM-07 tripwire: same string → same id; catalog lookup by hash is stable across reloads.
    #[test]
    fn mem07_cargo_hashed_ids_stable_across_reloads() {
        let a = parse_file(EMBEDDED).unwrap();
        let b = parse_file(EMBEDDED).unwrap();
        assert_eq!(a.by_id.len(), b.by_id.len());
        let mut ids: Vec<StringId> = a.by_id.keys().copied().collect();
        ids.sort();
        for id in &ids {
            let row_a = a.get_by_id(*id).expect("id in first load");
            let row_b = b.get_by_id(*id).expect("id in second load");
            assert_eq!(row_a.tag, row_b.tag);
            assert_eq!(*id, StringId::new(&row_a.tag), "key is the hash of the tag");
            assert!(std::ptr::eq(a.get(&row_a.tag).unwrap(), a.get_by_id(*id).unwrap()));
        }
        // Heap-distinct string with equal content resolves to the same row.
        let owned = String::from("Fuel");
        assert_eq!(a.get(&owned).unwrap().tag, "Fuel");
        assert_eq!(a.get_by_id(StringId::new("Fuel")).unwrap().class, a.get("Fuel").unwrap().class);
        assert!(a.get_by_id(StringId::new("no_such_cargo_tag")).is_none());
    }

    #[test]
    fn mem07_cargo_duplicate_tag_is_single_row() {
        // Same tag twice is a benign overwrite; distinct tags must never share an id.
        let dup = "(schema_version: 1, table_id: \"t\", kinds: [(tag: \"Fuel\", class: \"liquid\"), (tag: \"Fuel\", class: \"liquid\")])";
        assert_eq!(parse_file(dup).unwrap().by_id.len(), 1);
    }
    
}
